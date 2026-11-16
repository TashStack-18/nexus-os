//! Screen resolution. GRUB can only use the modes the PC's BIOS lists, and
//! VirtualBox's BIOS lists few (often 640 x 480). VirtualBox's graphics card
//! (like QEMU's) also implements the "Bochs VBE" register interface, which
//! sets any resolution that fits in video memory. If GRUB left us in a small
//! mode and that interface is present, switch to the largest mode up to
//! 1920 x 1080 so the desktop fills the screen. The framebuffer address does
//! not change; only the size and pitch do.

use crate::arch::{inw, outw};
use crate::boot::Framebuffer;

const INDEX: u16 = 0x1CE;
const DATA: u16 = 0x1CF;
const ID: u16 = 0;
const XRES: u16 = 1;
const YRES: u16 = 2;
const BPP: u16 = 3;
const ENABLE: u16 = 4;
const VIRT_WIDTH: u16 = 6;
const VRAM_64K: u16 = 0x0A;

fn read(reg: u16) -> u16 {
    // SAFETY: Bochs VBE index/data ports; only probed after the ID check, and
    // reading a register has no side effects.
    unsafe {
        outw(INDEX, reg);
        inw(DATA)
    }
}

fn write(reg: u16, v: u16) {
    // SAFETY: as above; the writes follow the documented mode-set sequence.
    unsafe {
        outw(INDEX, reg);
        outw(DATA, v);
    }
}

/// Candidate modes, largest first.
const MODES: [(u16, u16); 5] = [(1920, 1080), (1600, 900), (1366, 768), (1280, 720), (1024, 768)];

/// If the screen is small and the card can do better, switch mode. Returns
/// the framebuffer to use from now on.
pub fn upgrade(fb: Framebuffer) -> Framebuffer {
    if fb.width >= 1280 {
        return fb; // GRUB already gave us a big mode (VMware, real hardware)
    }
    let id = read(ID);
    if !(0xB0C0..=0xB0CF).contains(&id) || id < 0xB0C2 {
        kinfo!("modeset: no Bochs VBE interface (id {:#x}); staying at {}x{}", id, fb.width, fb.height);
        return fb; // no Bochs VBE interface (or too old for 32 bpp)
    }
    // Some cards do not report their video memory; every supported VM has at
    // least 16 MiB, which fits 1920 x 1080 x 4 bytes.
    let reported = read(VRAM_64K) as usize * 65536;
    let vram = if reported < (8 << 20) { 16 << 20 } else { reported };
    kinfo!("modeset: Bochs VBE id {:#x}, video memory {} MiB (reported {} MiB)", id, vram >> 20, reported >> 20);
    let Some(&(w, h)) = MODES.iter().find(|&&(w, h)| w as usize * h as usize * 4 <= vram && w as usize > fb.width) else {
        return fb;
    };
    write(ENABLE, 0);
    write(XRES, w);
    write(YRES, h);
    write(BPP, 32);
    write(ENABLE, 0x01 | 0x40); // enabled, linear framebuffer
    let (rw, rh, rbpp) = (read(XRES), read(YRES), read(BPP));
    kinfo!("modeset: asked for {}x{}, card says {}x{} at {} bpp", w, h, rw, rh, rbpp);
    if (rw, rh, rbpp) != (w, h, 32) {
        // The card refused: go back to what GRUB set up.
        write(ENABLE, 0);
        write(XRES, fb.width as u16);
        write(YRES, fb.height as u16);
        write(BPP, fb.bpp as u16);
        write(ENABLE, 0x01 | 0x40);
        return fb;
    }
    let vw = read(VIRT_WIDTH) as usize;
    Framebuffer { addr: fb.addr, pitch: vw.max(w as usize) * 4, width: w as usize, height: h as usize, bpp: 32 }
}
