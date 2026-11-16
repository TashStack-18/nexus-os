//! A simple everyday desktop for the PC port, drawn by the kernel into the
//! framebuffer GRUB sets up: wallpaper, desktop icons, a taskbar with a
//! Start menu and a clock, and windows for the Terminal (the kernel shell),
//! Task Manager and System Info. It runs as an ordinary kernel task, so the
//! scheduler shares the CPU between it, the shell and any programs.
//!
//! Without a framebuffer (text-mode boot) none of this starts and the shell
//! uses the VGA text screen as before.

mod apps;
mod canvas;
mod desktop;
mod font;
mod input;
mod lab;
mod modeset;
mod rtc;

use alloc::format;
use core::sync::atomic::{AtomicBool, Ordering::Relaxed};

use crate::boot::Framebuffer;
use crate::scheduler::{self, SpawnOpts};
use crate::sync::IrqSpinLock;

use super::vga;

static mut FB: Option<Framebuffer> = None;
static mut FONTS: Option<font::Fonts> = None;
static ACTIVE: AtomicBool = AtomicBool::new(false);
static ROOMY: AtomicBool = AtomicBool::new(false);
/// True while no desktop task is drawing: console cells are then painted
/// straight onto the screen (boot messages, test runs, the final screen).
static DIRECT: AtomicBool = AtomicBool::new(false);

const CONSOLE_BG: u32 = 0x0B1220;
const CONSOLE_TOP: usize = 40;

/// Keys typed into the Terminal window, waiting for the shell.
struct KeyQueue {
    buf: [u8; 256],
    head: usize,
    len: usize,
}
static KEYS: IrqSpinLock<KeyQueue> = IrqSpinLock::new("keys", KeyQueue { buf: [0; 256], head: 0, len: 0 });

pub fn push_key(b: u8) {
    let mut q = KEYS.lock();
    if q.len < q.buf.len() {
        let i = (q.head + q.len) % q.buf.len();
        q.buf[i] = b;
        q.len += 1;
    }
}

pub fn push_str(s: &str) {
    for b in s.bytes() {
        push_key(b);
    }
}

/// Next key for the shell (graphics mode).
pub fn pop_key() -> Option<u8> {
    let mut q = KEYS.lock();
    if q.len == 0 {
        return None;
    }
    let b = q.buf[q.head];
    q.head = (q.head + 1) % q.buf.len();
    q.len -= 1;
    Some(b)
}

pub fn active() -> bool {
    ACTIVE.load(Relaxed)
}

fn fb() -> Option<Framebuffer> {
    // SAFETY: written once in `init` during boot, before any task runs.
    unsafe { *core::ptr::addr_of!(FB) }
}

/// Called at boot with the framebuffer GRUB set up (if any). From here on
/// console output goes to the in-memory terminal grid.
pub fn init(fb: Option<Framebuffer>, splash: bool) {
    let Some(fb) = fb else { return };
    // Switch to a bigger screen mode if the card allows it.
    let fb = modeset::upgrade(fb);
    if fb.bpp != 32 {
        return; // the desktop draws 32-bit pixels; stay on the text console
    }
    // SAFETY: single-threaded boot; nothing reads FB or FONTS before this.
    unsafe {
        *core::ptr::addr_of_mut!(FB) = Some(fb);
        *core::ptr::addr_of_mut!(FONTS) = Some(font::Fonts::load());
    }
    for y in 0..fb.height {
        // SAFETY: row y of the mode GRUB set up (checked in boot::framebuffer).
        unsafe { core::slice::from_raw_parts_mut((fb.addr + y * fb.pitch) as *mut u32, fb.width).fill(CONSOLE_BG) };
    }
    // The desktop needs room for its windows; on a smaller screen the shell
    // is shown directly instead.
    let roomy = fb.width >= 1024 && fb.height >= 700;
    if splash && roomy {
        draw_splash(&fb);
    } else {
        DIRECT.store(true, Relaxed);
    }
    ROOMY.store(roomy, Relaxed);
    vga::use_grid();
    input::init_mouse();
    ACTIVE.store(true, Relaxed);
    kinfo!("desktop: framebuffer {}x{} at {:#x}", fb.width, fb.height, fb.addr);
}

pub fn resolution() -> (usize, usize) {
    fb().map_or((0, 0), |f| (f.width, f.height))
}

/// Start the desktop task (shell mode). It ends when the shell exits.
pub fn start() {
    if !ROOMY.load(Relaxed) {
        return;
    }
    if let Some(fb) = fb() {
        let spawned = scheduler::spawn("desktop", SpawnOpts { prio: 0, burst_ticks: 1, ..SpawnOpts::default() }, move || {
            DIRECT.store(false, Relaxed);
            desktop::run(fb);
            DIRECT.store(true, Relaxed);
        });
        if spawned.is_err() {
            kwarn!("desktop: could not start; the console stays on screen");
            DIRECT.store(true, Relaxed);
        }
    }
}

/// The start-up screen shown while the kernel boots, until the desktop starts.
fn draw_splash(fb: &Framebuffer) {
    let (w, h) = (fb.width, fb.height);
    let mut c = canvas::Canvas::new(w, h);
    c.px.copy_from_slice(&desktop::wallpaper(w, h));
    let fonts = font::Fonts::load();
    let (cx, cy) = (w as i32 / 2, h as i32 / 2 - 30);
    c.shadow(canvas::Rect::new(cx - 36, cy - 36, 72, 72), 20);
    c.round(canvas::Rect::new(cx - 36, cy - 36, 72, 72), 20, desktop::ACCENT);
    for (x0, y0, x1, y1) in [(-12.0, 14.0, -12.0, -14.0), (-12.0, -14.0, 12.0, 14.0), (12.0, 14.0, 12.0, -14.0)] {
        c.line(cx as f32 + x0, cy as f32 + y0, cx as f32 + x1, cy as f32 + y1, 3.5, 0xFFFFFF);
    }
    let name = "NEXUS";
    c.text(&fonts.bold, cx - canvas::Canvas::text_w(name) / 2, cy + 56, name, 0xF6F8FC);
    let sub = "Starting the Rust kernel...";
    c.text(&fonts.regular, cx - canvas::Canvas::text_w(sub) / 2, cy + 82, sub, 0xA7B2C8);
    c.present(fb);
}

/// Paint one console cell straight onto the screen (only while no desktop
/// is drawing). Called by the console for every cell that changes.
pub fn draw_cell_direct(i: usize, cell: u16) {
    if !DIRECT.load(Relaxed) {
        return;
    }
    // SAFETY: FB and FONTS are written once in `init`, before DIRECT is set.
    let (Some(fb), Some(fonts)) = (fb(), unsafe { (*core::ptr::addr_of!(FONTS)).as_ref() }) else { return };
    let x0 = fb.width.saturating_sub(vga::COLS * 8) / 2 + (i % vga::COLS) * 8;
    let y0 = CONSOLE_TOP + (i / vga::COLS) * 16;
    let fg = desktop::term_color(((cell >> 8) & 0x0F) as u8);
    let bg = match (cell >> 12) & 0x07 {
        0 => CONSOLE_BG,
        b => desktop::term_color(b as u8),
    };
    let rows = fonts.regular.glyph(vga::cp437_to_char((cell & 0xFF) as u8));
    for (dy, &bits) in rows.iter().enumerate() {
        // SAFETY: the 8 pixels lie inside row y0+dy of the framebuffer: the
        // console area (640 x 400 from CONSOLE_TOP) fits the checked minimum
        // mode of 640 x 480.
        let px = unsafe { core::slice::from_raw_parts_mut((fb.addr + (y0 + dy) * fb.pitch + x0 * 4) as *mut u32, 8) };
        for (dx, p) in px.iter_mut().enumerate() {
            *p = if bits & (0x80 >> dx) != 0 { fg } else { bg };
        }
    }
}

/// Final screen at power-off or panic: the terminal's last lines and a
/// message, drawn without the desktop task (it may not be running).
pub fn final_screen(message: &str) {
    let Some(fb) = fb() else { return };
    let mut c = canvas::Canvas::new(fb.width, fb.height);
    let fonts = font::Fonts::load();
    DIRECT.store(false, Relaxed); // this frame is drawn whole, below
    c.fill(canvas::Rect::new(0, 0, fb.width as i32, fb.height as i32), CONSOLE_BG);
    let mut grid = [0u16; vga::COLS * vga::ROWS];
    vga::mark_dirty(); // copy even if nothing changed since the desktop's last frame
    let _ = vga::snapshot(&mut grid);
    let (x0, y0) = ((fb.width as i32 - 640) / 2, CONSOLE_TOP as i32);
    for (i, &cell) in grid.iter().enumerate() {
        let ch = vga::cp437_to_char((cell & 0xFF) as u8);
        if ch != ' ' {
            let (r, col) = ((i / vga::COLS) as i32, (i % vga::COLS) as i32);
            c.glyph(&fonts.regular, x0 + col * 8, y0 + r * 16, ch, desktop::term_color(((cell >> 8) & 0x0F) as u8));
        }
    }
    let y = y0 + 25 * 16 + 40;
    let w = canvas::Canvas::text_w(message);
    c.text(&fonts.bold, (fb.width as i32 - w) / 2, y, message, 0xFFFFFF);
    let up = format!("Micro-OS ran for {} s", crate::timer::now_us() / 1_000_000);
    c.text(&fonts.regular, (fb.width as i32 - canvas::Canvas::text_w(&up)) / 2, y + 24, &up, 0x9CA3AF);
    c.present(&fb);
}
