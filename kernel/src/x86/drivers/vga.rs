//! VGA text-mode console (80 x 25 cells at 0xB8000): what a PC's screen
//! shows in VirtualBox or VMware. Understands the small part of ANSI the
//! shell uses (colours, bold, clear screen, cursor home, clear line) and
//! UTF-8, mapping the shell's symbols to the PC's code page 437.

use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering::Relaxed};

use crate::arch::outb;

pub const COLS: usize = 80;
pub const ROWS: usize = 25;

/// Graphics mode: the same character grid is kept in memory (GRID) and the
/// desktop's Terminal window draws it with a font, instead of the VGA text
/// buffer. Each cell is `attr << 8 | code page 437 glyph`, as in VGA.
static GRAPHICS: AtomicBool = AtomicBool::new(false);
static DIRTY: AtomicBool = AtomicBool::new(true);
static CURSOR: AtomicUsize = AtomicUsize::new(0);
struct Grid(UnsafeCell<[u16; COLS * ROWS]>);
// SAFETY: written under the console lock; the desktop copies it (`snapshot`)
// and tolerates a frame showing a half-written line.
unsafe impl Sync for Grid {}
static GRID: Grid = Grid(UnsafeCell::new([0x0720; COLS * ROWS]));

/// Switch the console to the in-memory grid (called when the desktop starts).
pub fn use_grid() {
    GRAPHICS.store(true, Relaxed);
}

/// Make the next `snapshot` copy the grid even if nothing changed.
pub fn mark_dirty() {
    DIRTY.store(true, Relaxed);
}

/// Copy the grid if it changed since the last call; returns the cursor cell.
pub fn snapshot(out: &mut [u16; COLS * ROWS]) -> Option<usize> {
    if !DIRTY.swap(false, Relaxed) {
        return None;
    }
    // SAFETY: see `Grid`.
    out.copy_from_slice(unsafe { &*GRID.0.get() });
    Some(CURSOR.load(Relaxed))
}

/// Code page 437 glyph -> Unicode, for drawing the grid with a Unicode font.
pub fn cp437_to_char(g: u8) -> char {
    match g {
        0x20..=0x7E => g as char,
        0xC4 => '─',
        0xB3 => '│',
        0xFB => '√', // the font has no ✓; √ is the check mark of code page 437
        0xFA => '·',
        0xE6 => 'µ',
        0xFE => '■',
        0x1A => '→',
        0x1B => '←',
        0xF3 => '≤',
        0xF2 => '≥',
        0xF1 => '±',
        0xF8 => '°',
        _ => ' ',
    }
}
const BUF: usize = 0xB8000;
const DEFAULT_ATTR: u8 = 0x07; // light grey on black

#[derive(Clone, Copy, PartialEq)]
enum Esc {
    None,
    Escape,
    Csi,
}

struct Screen {
    row: usize,
    col: usize,
    fg: u8,
    bg: u8,
    bright: bool,
    esc: Esc,
    params: [u16; 4],
    nparams: usize,
    utf8: u32,
    utf8_left: u8,
}

struct State(UnsafeCell<Screen>);
// SAFETY: the kernel has one CPU; the console lock serialises writers, and
// the panic path accepts interleaving with a writer it interrupted.
unsafe impl Sync for State {}
static SCREEN: State = State(UnsafeCell::new(Screen {
    row: 0,
    col: 0,
    fg: 7,
    bg: 0,
    bright: false,
    esc: Esc::None,
    params: [0; 4],
    nparams: 0,
    utf8: 0,
    utf8_left: 0,
}));

/// ANSI colour number (0-7) -> VGA colour number.
const ANSI_TO_VGA: [u8; 8] = [0, 4, 2, 6, 1, 5, 3, 7];

fn cell(i: usize, v: u16) {
    if GRAPHICS.load(Relaxed) {
        // SAFETY: see `Grid`; i < COLS * ROWS.
        let changed = unsafe {
            let g = &mut *GRID.0.get();
            let changed = g[i] != v;
            g[i] = v;
            changed
        };
        DIRTY.store(true, Relaxed);
        if changed {
            super::gui::draw_cell_direct(i, v);
        }
        return;
    }
    // SAFETY: i < COLS * ROWS, inside the VGA text buffer, which is
    // identity-mapped device memory.
    unsafe { core::ptr::write_volatile((BUF as *mut u16).add(i), v) };
}

fn read_cell(i: usize) -> u16 {
    if GRAPHICS.load(Relaxed) {
        // SAFETY: see `Grid`.
        return unsafe { (*GRID.0.get())[i] };
    }
    // SAFETY: as for `cell`.
    unsafe { core::ptr::read_volatile((BUF as *const u16).add(i)) }
}

impl Screen {
    fn attr(&self) -> u8 {
        let fg = if self.bright { self.fg | 8 } else { self.fg };
        (self.bg << 4) | fg
    }

    fn clear(&mut self) {
        for i in 0..COLS * ROWS {
            cell(i, (DEFAULT_ATTR as u16) << 8 | b' ' as u16);
        }
        self.row = 0;
        self.col = 0;
    }

    fn newline(&mut self) {
        self.col = 0;
        if self.row + 1 < ROWS {
            self.row += 1;
            return;
        }
        for i in 0..COLS * (ROWS - 1) {
            cell(i, read_cell(i + COLS));
        }
        for c in 0..COLS {
            cell(COLS * (ROWS - 1) + c, (DEFAULT_ATTR as u16) << 8 | b' ' as u16);
        }
    }

    fn glyph(&mut self, g: u8) {
        if self.col >= COLS {
            self.newline();
        }
        cell(self.row * COLS + self.col, (self.attr() as u16) << 8 | g as u16);
        self.col += 1;
    }

    fn sgr(&mut self) {
        let n = self.nparams.max(1);
        for &p in &self.params[..n] {
            match p {
                0 => {
                    self.fg = 7;
                    self.bg = 0;
                    self.bright = false;
                }
                1 => self.bright = true,
                22 => self.bright = false,
                30..=37 => self.fg = ANSI_TO_VGA[(p - 30) as usize],
                39 => self.fg = 7,
                40..=47 => self.bg = ANSI_TO_VGA[(p - 40) as usize],
                49 => self.bg = 0,
                90..=97 => {
                    self.fg = ANSI_TO_VGA[(p - 90) as usize];
                    self.bright = true;
                }
                _ => {}
            }
        }
    }

    fn csi(&mut self, fin: u8) {
        match fin {
            b'm' => self.sgr(),
            b'J' => self.clear(),
            b'H' => {
                let r = self.params[0].max(1) as usize - 1;
                let c = if self.nparams > 1 { self.params[1].max(1) as usize - 1 } else { 0 };
                self.row = r.min(ROWS - 1);
                self.col = c.min(COLS - 1);
            }
            b'K' => {
                for c in self.col..COLS {
                    cell(self.row * COLS + c, (DEFAULT_ATTR as u16) << 8 | b' ' as u16);
                }
            }
            _ => {} // other sequences (cursor queries, modes) are ignored
        }
    }

    fn byte(&mut self, b: u8) {
        match self.esc {
            Esc::Escape => {
                self.esc = if b == b'[' { Esc::Csi } else { Esc::None };
                self.params = [0; 4];
                self.nparams = 0;
                return;
            }
            Esc::Csi => {
                match b {
                    b'0'..=b'9' => {
                        if self.nparams == 0 {
                            self.nparams = 1;
                        }
                        let i = self.nparams - 1;
                        if i < 4 {
                            self.params[i] = self.params[i].saturating_mul(10).saturating_add((b - b'0') as u16);
                        }
                    }
                    b';' => self.nparams = (self.nparams.max(1) + 1).min(4),
                    b'?' | b'=' | b'!' | b'>' => {}
                    0x40..=0x7e => {
                        self.esc = Esc::None;
                        self.csi(b);
                    }
                    _ => self.esc = Esc::None,
                }
                return;
            }
            Esc::None => {}
        }
        if self.utf8_left > 0 {
            if b & 0xC0 == 0x80 {
                self.utf8 = (self.utf8 << 6) | (b & 0x3F) as u32;
                self.utf8_left -= 1;
                if self.utf8_left == 0 {
                    let g = cp437(self.utf8);
                    self.glyph(g);
                }
                return;
            }
            self.utf8_left = 0;
        }
        match b {
            0x1B => self.esc = Esc::Escape,
            b'\r' => self.col = 0,
            b'\n' => self.newline(),
            0x08 => self.col = self.col.saturating_sub(1),
            b'\t' => {
                let next = (self.col / 8 + 1) * 8;
                while self.col < next.min(COLS) {
                    self.glyph(b' ');
                }
            }
            0x20..=0x7e => self.glyph(b),
            0xC0..=0xDF => {
                self.utf8 = (b & 0x1F) as u32;
                self.utf8_left = 1;
            }
            0xE0..=0xEF => {
                self.utf8 = (b & 0x0F) as u32;
                self.utf8_left = 2;
            }
            0xF0..=0xF7 => {
                self.utf8 = (b & 0x07) as u32;
                self.utf8_left = 3;
            }
            _ => {}
        }
    }

    fn cursor(&self) {
        let pos = (self.row * COLS + self.col.min(COLS - 1)) as u16;
        if GRAPHICS.load(Relaxed) {
            CURSOR.store(pos as usize, Relaxed);
            DIRTY.store(true, Relaxed);
            return;
        }
        // SAFETY: CRT controller index/data ports; only the cursor moves.
        unsafe {
            outb(0x3D4, 0x0F);
            outb(0x3D5, (pos & 0xFF) as u8);
            outb(0x3D4, 0x0E);
            outb(0x3D5, (pos >> 8) as u8);
        }
    }
}

/// Unicode -> code page 437 for the symbols the kernel prints.
fn cp437(c: u32) -> u8 {
    match c {
        0x2500 => 0xC4, // ─
        0x2502 => 0xB3, // │
        0x2713 | 0x221A => 0xFB, // ✓ √
        0x00B7 | 0x2022 => 0xFA, // · •
        0x00B5 | 0x03BC => 0xE6, // µ
        0x25A0 => 0xFE, // ■
        0x2192 => 0x1A, // →
        0x2190 => 0x1B, // ←
        0x2264 => 0xF3, // ≤
        0x2265 => 0xF2, // ≥
        0x00B1 => 0xF1, // ±
        0x00B0 => 0xF8, // °
        0x2026 => b'.', // …
        0x2014 | 0x2013 => b'-', // — –
        _ => b'?',
    }
}

pub fn put_byte(b: u8) {
    // SAFETY: see `State`: single CPU, serialised by the console lock.
    let s = unsafe { &mut *SCREEN.0.get() };
    s.byte(b);
    if b == b'\n' || b == 0x08 || (0x20..0x7f).contains(&b) {
        s.cursor();
    }
}
