//! Layer 1 — boot on x86-64: the Multiboot2 entry stub (boot.S) and reading
//! what GRUB passes in the boot information: the kernel command line and
//! the graphics framebuffer.

use microos_core::args::BootArgs;

core::arch::global_asm!(include_str!("boot.S"));

/// Magic value GRUB leaves in eax for a Multiboot2 kernel.
pub const MULTIBOOT2_MAGIC: usize = 0x36D7_6289;
const TAG_END: u32 = 0;
const TAG_CMDLINE: u32 = 1;
const TAG_FRAMEBUFFER: u32 = 8;

/// A linear framebuffer set up by GRUB (32 bits per pixel, 0x00RRGGBB).
#[derive(Clone, Copy, Debug)]
pub struct Framebuffer {
    pub addr: usize,
    pub pitch: usize,
    pub width: usize,
    pub height: usize,
}

/// Address and size of the first tag of type `ty`, if present.
fn find_tag(magic: usize, info: usize, ty: u32) -> Option<(usize, usize)> {
    if magic != MULTIBOOT2_MAGIC || info == 0 || info % 8 != 0 {
        return None;
    }
    // SAFETY: GRUB placed a Multiboot2 information structure at `info`
    // (checked by the magic above): a u32 total size, a reserved u32, then
    // 8-byte-aligned tags {u32 type, u32 size, data}. It lies in identity-
    // mapped low memory and is never written; every read is bounded by `total`.
    unsafe {
        let total = *(info as *const u32) as usize;
        let mut off = 8;
        while off + 8 <= total {
            let tag = info + off;
            let (t, size) = (*(tag as *const u32), *((tag + 4) as *const u32) as usize);
            if t == TAG_END || size < 8 {
                return None;
            }
            if t == ty {
                return Some((tag, size));
            }
            off += (size + 7) & !7;
        }
    }
    None
}

/// The kernel command line from the Multiboot2 information.
pub fn boot_args(magic: usize, info: usize) -> BootArgs<'static> {
    let Some((tag, size)) = find_tag(magic, info, TAG_CMDLINE) else { return BootArgs::new("") };
    // SAFETY: the tag holds a NUL-terminated string of `size - 8` bytes in
    // GRUB's read-only boot information (see find_tag).
    let bytes = unsafe { core::slice::from_raw_parts((tag + 8) as *const u8, size - 8) };
    let len = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    BootArgs::new(core::str::from_utf8(&bytes[..len]).unwrap_or(""))
}

/// The graphics framebuffer, if GRUB set one up in a 32-bit RGB mode.
pub fn framebuffer(magic: usize, info: usize) -> Option<Framebuffer> {
    let (tag, size) = find_tag(magic, info, TAG_FRAMEBUFFER)?;
    if size < 30 {
        return None;
    }
    // SAFETY: framebuffer tag layout from the Multiboot2 specification:
    // u64 address @8, u32 pitch @16, u32 width @20, u32 height @24,
    // u8 bpp @28, u8 type @29 (1 = direct RGB). Read-only boot information.
    let (addr, pitch, width, height, bpp, kind) = unsafe {
        (
            *((tag + 8) as *const u64) as usize,
            *((tag + 16) as *const u32) as usize,
            *((tag + 20) as *const u32) as usize,
            *((tag + 24) as *const u32) as usize,
            *((tag + 28) as *const u8),
            *((tag + 29) as *const u8),
        )
    };
    // The boot code maps the first 4 GiB; the screen must lie inside it.
    let end = addr.checked_add(pitch.checked_mul(height)?)?;
    (kind == 1 && bpp == 32 && width >= 640 && height >= 480 && end <= 0x1_0000_0000).then_some(Framebuffer { addr, pitch, width, height })
}
