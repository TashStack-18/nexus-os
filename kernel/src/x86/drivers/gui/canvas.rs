//! Drawing into an off-screen buffer, then copying finished frames to the
//! framebuffer in one pass (no flicker): rectangles, rounded panels,
//! translucent shading, text.

use alloc::vec;
use alloc::vec::Vec;

use super::font::{Font, W};
use crate::boot::Framebuffer;

pub type Rgb = u32; // 0x00RRGGBB

pub struct Canvas {
    pub w: usize,
    pub h: usize,
    pub px: Vec<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    pub const fn new(x: i32, y: i32, w: i32, h: i32) -> Rect {
        Rect { x, y, w, h }
    }
    pub fn contains(&self, px: i32, py: i32) -> bool {
        px >= self.x && py >= self.y && px < self.x + self.w && py < self.y + self.h
    }
}

pub fn mix(a: Rgb, b: Rgb, t: u32) -> Rgb {
    // t in 0..=256: 0 = a, 256 = b
    let ch = |s: u32| {
        let (x, y) = ((a >> s) & 0xFF, (b >> s) & 0xFF);
        ((x * (256 - t) + y * t) >> 8) << s
    };
    ch(16) | ch(8) | ch(0)
}

impl Canvas {
    pub fn new(w: usize, h: usize) -> Canvas {
        Canvas { w, h, px: vec![0; w * h] }
    }

    fn clip(&self, r: Rect) -> Option<(usize, usize, usize, usize)> {
        let x0 = r.x.max(0) as usize;
        let y0 = r.y.max(0) as usize;
        let x1 = ((r.x + r.w).max(0) as usize).min(self.w);
        let y1 = ((r.y + r.h).max(0) as usize).min(self.h);
        (x0 < x1 && y0 < y1).then_some((x0, y0, x1, y1))
    }

    pub fn fill(&mut self, r: Rect, c: Rgb) {
        if let Some((x0, y0, x1, y1)) = self.clip(r) {
            for y in y0..y1 {
                self.px[y * self.w + x0..y * self.w + x1].fill(c);
            }
        }
    }

    /// Blend `c` over the area with weight t/256 (shadows, highlights).
    pub fn shade(&mut self, r: Rect, c: Rgb, t: u32) {
        if let Some((x0, y0, x1, y1)) = self.clip(r) {
            for y in y0..y1 {
                for p in &mut self.px[y * self.w + x0..y * self.w + x1] {
                    *p = mix(*p, c, t);
                }
            }
        }
    }

    /// Filled rectangle with corners of radius `rad` cut round.
    pub fn round(&mut self, r: Rect, rad: i32, c: Rgb) {
        for dy in 0..r.h {
            let inset = corner_inset(dy, r.h, rad);
            self.fill(Rect::new(r.x + inset, r.y + dy, r.w - 2 * inset, 1), c);
        }
    }

    /// One-pixel outline with rounded corners.
    pub fn outline(&mut self, r: Rect, rad: i32, c: Rgb) {
        for dy in 0..r.h {
            let inset = corner_inset(dy, r.h, rad);
            if dy == 0 || dy == r.h - 1 {
                self.fill(Rect::new(r.x + inset, r.y + dy, r.w - 2 * inset, 1), c);
            } else {
                self.fill(Rect::new(r.x + inset, r.y + dy, 1, 1), c);
                self.fill(Rect::new(r.x + r.w - 1 - inset, r.y + dy, 1, 1), c);
            }
        }
    }

    /// Soft drop shadow under a panel.
    pub fn shadow(&mut self, r: Rect, rad: i32) {
        for (i, t) in [(10, 18u32), (6, 26), (3, 40)] {
            for dy in 0..r.h + i {
                let inset = corner_inset(dy, r.h + i, rad + i / 2);
                let row = Rect::new(r.x - i / 2 + inset, r.y + 4 - i / 2 + dy, r.w + i - 2 * inset, 1);
                self.shade(row, 0x000000, t);
            }
        }
    }

    pub fn glyph(&mut self, font: &Font, x: i32, y: i32, ch: char, c: Rgb) {
        let rows = font.glyph(ch);
        for (dy, &bits) in rows.iter().enumerate() {
            let py = y + dy as i32;
            if py < 0 || py as usize >= self.h || bits == 0 {
                continue;
            }
            for dx in 0..W as i32 {
                if bits & (0x80 >> dx) != 0 {
                    let px = x + dx;
                    if px >= 0 && (px as usize) < self.w {
                        self.px[py as usize * self.w + px as usize] = c;
                    }
                }
            }
        }
    }

    /// Draw `s`; returns the x after the last character.
    pub fn text(&mut self, font: &Font, x: i32, y: i32, s: &str, c: Rgb) -> i32 {
        let mut cx = x;
        for ch in s.chars() {
            self.glyph(font, cx, y, ch, c);
            cx += W as i32;
        }
        cx
    }

    /// Text clipped to `max_w` pixels, with an ellipsis if it does not fit.
    pub fn text_fit(&mut self, font: &Font, x: i32, y: i32, s: &str, c: Rgb, max_w: i32) {
        let max_chars = (max_w / W as i32).max(0) as usize;
        let n = s.chars().count();
        if n <= max_chars {
            self.text(font, x, y, s, c);
        } else if max_chars > 1 {
            let mut cx = x;
            for ch in s.chars().take(max_chars - 1) {
                self.glyph(font, cx, y, ch, c);
                cx += W as i32;
            }
            self.glyph(font, cx, y, '…', c);
        }
    }

    pub fn text_w(s: &str) -> i32 {
        (s.chars().count() * W) as i32
    }

    /// Copy the finished frame to the screen.
    pub fn present(&self, fb: &Framebuffer) {
        for y in 0..self.h.min(fb.height) {
            let src = &self.px[y * self.w..y * self.w + self.w.min(fb.width)];
            // SAFETY: the framebuffer row lies inside the mode GRUB set up
            // (checked in boot::framebuffer and identity-mapped by boot.S);
            // `src` is no longer than one row.
            unsafe {
                core::ptr::copy_nonoverlapping(src.as_ptr(), (fb.addr + y * fb.pitch) as *mut u32, src.len());
            }
        }
    }
}

fn corner_inset(dy: i32, h: i32, rad: i32) -> i32 {
    let rad = rad.min(h / 2).max(0);
    let d = if dy < rad { rad - dy } else if dy >= h - rad { dy - (h - rad) + 1 } else { 0 };
    if d == 0 {
        return 0;
    }
    // inset = rad - sqrt(rad^2 - d^2), integer
    let r2 = rad * rad;
    let mut s = 0;
    while (s + 1) * (s + 1) <= r2 - d * d {
        s += 1;
    }
    rad - s
}
