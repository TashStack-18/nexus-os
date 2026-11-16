//! Layer 1 — boot on x86-64: the Multiboot2 entry stub (boot.S) and reading
//! the kernel command line GRUB passes in the boot information.

use microos_core::args::BootArgs;

core::arch::global_asm!(include_str!("boot.S"));

/// Magic value GRUB leaves in eax for a Multiboot2 kernel.
pub const MULTIBOOT2_MAGIC: usize = 0x36D7_6289;
const TAG_END: u32 = 0;
const TAG_CMDLINE: u32 = 1;

/// Find the command-line tag in the Multiboot2 information at `info`.
pub fn boot_args(magic: usize, info: usize) -> BootArgs<'static> {
    if magic != MULTIBOOT2_MAGIC || info == 0 || info % 8 != 0 {
        return BootArgs::new("");
    }
    // SAFETY: GRUB placed a Multiboot2 information structure at `info`
    // (checked by the magic above): a u32 total size, a reserved u32, then
    // 8-byte-aligned tags {u32 type, u32 size, data}. It lies in identity-
    // mapped low memory and is never written, so 'static shared reads are
    // sound. Every tag access is bounded by `total`.
    unsafe {
        let total = *(info as *const u32) as usize;
        let mut off = 8;
        while off + 8 <= total {
            let tag = info + off;
            let (ty, size) = (*(tag as *const u32), *((tag + 4) as *const u32) as usize);
            if ty == TAG_END || size < 8 {
                break;
            }
            if ty == TAG_CMDLINE {
                let bytes = core::slice::from_raw_parts((tag + 8) as *const u8, size - 8);
                let len = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
                return BootArgs::new(core::str::from_utf8(&bytes[..len]).unwrap_or(""));
            }
            off += (size + 7) & !7;
        }
    }
    BootArgs::new("")
}
