//! Drawing into an off-screen buffer, then copying finished frames to the
//! framebuffer in one pass (no flicker): anti-aliased rounded rectangles,
//! translucent panels, ring gauges, soft shadows, text, and a smooth large
//! clock face. Colours are 0x00RRGGBB; alpha is 0..=256.

use alloc::vec;
use alloc::vec::Vec;

use super::font::{Font, W};
use crate::boot::Framebuffer;

pub type Rgb = u32;

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

/// Blend `b` over `a` with weight t/256 (0 = a, 256 = b).
pub fn mix(a: Rgb, b: Rgb, t: u32) -> Rgb {
    let ch = |s: u32| {
        let (x, y) = ((a >> s) & 0xFF, (b >> s) & 0xFF);
        ((x * (256 - t) + y * t) >> 8) << s
    };
    ch(16) | ch(8) | ch(0)
}

pub fn isqrt(n: u32) -> u32 {
    if n < 2 {
        return n;
    }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}

/// Coverage 0..=256 of a pixel at distance `d16` (1/16 px) from a circle edge
/// of radius `r16`, inside the circle.
fn edge(r16: i32, d16: i32) -> u32 {
    ((r16 - d16 + 8).clamp(0, 16) * 16) as u32
}

fn sin_pi(x: f32) -> f32 {
    // Bhaskara approximation of sin on [0, pi]
    const PI: f32 = 3.141_592_7;
    let p = x * (PI - x);
    16.0 * p / (5.0 * PI * PI - 4.0 * p)
}

/// (sin, cos) of an angle in turns (0..=1), clockwise from the top.
fn sincos_turns(t: f32) -> (f32, f32) {
    const PI: f32 = 3.141_592_7;
    let a = t * 2.0 * PI;
    let sin = |a: f32| {
        let a = a % (2.0 * PI);
        if a <= PI {
            sin_pi(a)
        } else {
            -sin_pi(a - PI)
        }
    };
    (sin(a), sin(a + PI / 2.0))
}

/// Monotone "pseudo angle" in 0..4 going clockwise from straight up.
fn pseudo_angle(ux: f32, uy: f32) -> f32 {
    // ux right, uy up
    if ux >= 0.0 && uy > 0.0 {
        ux / (ux + uy)
    } else if ux > 0.0 && uy <= 0.0 {
        1.0 + (-uy) / (ux - uy)
    } else if ux <= 0.0 && uy < 0.0 {
        2.0 + (-ux) / (-ux - uy)
    } else {
        3.0 + uy / (-ux + uy).max(0.0001)
    }
}

/// A coverage mask (8 bits per pixel) that can be tinted and blended.
pub struct Mask {
    pub w: usize,
    pub h: usize,
    pub a: Vec<u8>,
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

    fn put(&mut self, x: i32, y: i32, c: Rgb, a: u32) {
        if a == 0 || x < 0 || y < 0 || x as usize >= self.w || y as usize >= self.h {
            return;
        }
        let p = &mut self.px[y as usize * self.w + x as usize];
        *p = if a >= 256 { c } else { mix(*p, c, a) };
    }

    pub fn fill(&mut self, r: Rect, c: Rgb) {
        if let Some((x0, y0, x1, y1)) = self.clip(r) {
            for y in y0..y1 {
                self.px[y * self.w + x0..y * self.w + x1].fill(c);
            }
        }
    }

    /// Blend `c` over the area with weight t/256 (shadows, glass panels).
    pub fn shade(&mut self, r: Rect, c: Rgb, t: u32) {
        if let Some((x0, y0, x1, y1)) = self.clip(r) {
            for y in y0..y1 {
                for p in &mut self.px[y * self.w + x0..y * self.w + x1] {
                    *p = mix(*p, c, t);
                }
            }
        }
    }

    /// Rounded rectangle with anti-aliased corners, blended with weight `alpha`.
    pub fn rrect(&mut self, r: Rect, rad: i32, c: Rgb, alpha: u32) {
        let rad = rad.min(r.w / 2).min(r.h / 2).max(0);
        if rad == 0 {
            if alpha >= 256 {
                self.fill(r, c)
            } else {
                self.shade(r, c, alpha)
            }
            return;
        }
        // straight middle band and the two side bands between the corners
        let band = |s: &mut Canvas, rect: Rect| {
            if alpha >= 256 {
                s.fill(rect, c)
            } else {
                s.shade(rect, c, alpha)
            }
        };
        band(self, Rect::new(r.x, r.y + rad, r.w, r.h - 2 * rad));
        band(self, Rect::new(r.x + rad, r.y, r.w - 2 * rad, rad));
        band(self, Rect::new(r.x + rad, r.y + r.h - rad, r.w - 2 * rad, rad));
        let r16 = rad * 16;
        for cy in 0..rad {
            for cx in 0..rad {
                // distance of the pixel centre from the corner circle's centre
                let dx = (rad - cx) * 16 - 8;
                let dy = (rad - cy) * 16 - 8;
                let d = isqrt((dx * dx + dy * dy) as u32) as i32;
                let cov = edge(r16, d) * alpha.min(256) / 256;
                if cov == 0 {
                    continue;
                }
                self.put(r.x + cx, r.y + cy, c, cov);
                self.put(r.x + r.w - 1 - cx, r.y + cy, c, cov);
                self.put(r.x + cx, r.y + r.h - 1 - cy, c, cov);
                self.put(r.x + r.w - 1 - cx, r.y + r.h - 1 - cy, c, cov);
            }
        }
    }

    /// Opaque rounded rectangle.
    pub fn round(&mut self, r: Rect, rad: i32, c: Rgb) {
        self.rrect(r, rad, c, 256);
    }

    /// One-pixel outline of a rounded rectangle (drawn as ring of 1 px).
    pub fn outline(&mut self, r: Rect, rad: i32, c: Rgb, alpha: u32) {
        let rad = rad.min(r.w / 2).min(r.h / 2).max(0);
        let seg = |s: &mut Canvas, rect: Rect| s.shade(rect, c, alpha);
        seg(self, Rect::new(r.x + rad, r.y, r.w - 2 * rad, 1));
        seg(self, Rect::new(r.x + rad, r.y + r.h - 1, r.w - 2 * rad, 1));
        seg(self, Rect::new(r.x, r.y + rad, 1, r.h - 2 * rad));
        seg(self, Rect::new(r.x + r.w - 1, r.y + rad, 1, r.h - 2 * rad));
        let r16 = rad * 16;
        for cy in 0..rad {
            for cx in 0..rad {
                let dx = (rad - cx) * 16 - 8;
                let dy = (rad - cy) * 16 - 8;
                let d = isqrt((dx * dx + dy * dy) as u32) as i32;
                // 1 px wide band just inside the edge
                let band = edge(r16, d).saturating_sub(edge(r16 - 16, d));
                let cov = band * alpha / 256;
                if cov == 0 {
                    continue;
                }
                self.put(r.x + cx, r.y + cy, c, cov);
                self.put(r.x + r.w - 1 - cx, r.y + cy, c, cov);
                self.put(r.x + cx, r.y + r.h - 1 - cy, c, cov);
                self.put(r.x + r.w - 1 - cx, r.y + r.h - 1 - cy, c, cov);
            }
        }
    }

    /// Frosted-glass style panel: translucent dark fill, light rim.
    pub fn glass(&mut self, r: Rect, rad: i32) {
        self.rrect(r, rad, 0x0A1022, 150);
        self.outline(r, rad, 0xFFFFFF, 34);
    }

    /// Soft drop shadow under a panel.
    pub fn shadow(&mut self, r: Rect, rad: i32) {
        for (i, t) in [(18, 8u32), (12, 12), (7, 18), (3, 26)] {
            self.rrect(Rect::new(r.x - i / 2, r.y + 6 - i / 2, r.w + i, r.h + i), rad + i / 2, 0x000000, t);
        }
    }

    /// Ring gauge: `frac` in 0..=1000 (per mille) of the ring is `fg`, the rest `track`.
    pub fn ring(&mut self, cx: i32, cy: i32, r_out: i32, thick: i32, frac: u32, fg: Rgb, track: Rgb) {
        let frac = frac.min(1000);
        let (tsin, tcos) = sincos_turns(frac as f32 / 1000.0);
        let target = if frac >= 1000 { 4.1 } else { pseudo_angle(tsin, tcos) };
        let (ro16, ri16) = (r_out * 16, (r_out - thick) * 16);
        for y in cy - r_out - 1..=cy + r_out + 1 {
            for x in cx - r_out - 1..=cx + r_out + 1 {
                let (dx, dy) = (x - cx, y - cy);
                let d = isqrt(((dx * 16) * (dx * 16) + (dy * 16) * (dy * 16)) as u32) as i32;
                let inner = ((d - ri16 + 8).clamp(0, 16) * 16) as u32;
                let cov = edge(ro16, d).min(inner);
                if cov == 0 {
                    continue;
                }
                let on = frac > 0 && pseudo_angle(dx as f32, -(dy as f32)) <= target;
                self.put(x, y, if on { fg } else { track }, cov);
            }
        }
        // round the two ends of the arc
        if frac > 0 && frac < 1000 {
            let mid = (r_out as f32) - thick as f32 / 2.0;
            for (s, c) in [(0.0f32, 0.0f32), (tsin, tcos)] {
                let (ex, ey) = if s == 0.0 && c == 0.0 { (0.0, 1.0) } else { (s, c) };
                self.disc(cx + (ex * mid) as i32, cy - (ey * mid) as i32, thick / 2, fg);
            }
        }
    }

    /// Filled anti-aliased circle.
    pub fn disc(&mut self, cx: i32, cy: i32, r: i32, c: Rgb) {
        let r16 = r * 16;
        for y in cy - r - 1..=cy + r + 1 {
            for x in cx - r - 1..=cx + r + 1 {
                let (dx, dy) = ((x - cx) * 16, (y - cy) * 16);
                let d = isqrt((dx * dx + dy * dy) as u32) as i32;
                self.put(x, y, c, edge(r16, d));
            }
        }
    }

    /// Thick anti-aliased line segment with round caps.
    pub fn line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, half: f32, c: Rgb) {
        let (minx, maxx) = (x0.min(x1) - half - 1.0, x0.max(x1) + half + 1.0);
        let (miny, maxy) = (y0.min(y1) - half - 1.0, y0.max(y1) + half + 1.0);
        for y in miny as i32..=maxy as i32 {
            for x in minx as i32..=maxx as i32 {
                let d = seg_dist(x as f32 + 0.5, y as f32 + 0.5, x0, y0, x1, y1);
                let cov = (half - d + 0.5).clamp(0.0, 1.0);
                self.put(x, y, c, (cov * 256.0) as u32);
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

    /// Blend a coverage mask tinted `c` with overall weight `alpha` (0..=256).
    pub fn blit_mask(&mut self, x: i32, y: i32, m: &Mask, c: Rgb, alpha: u32) {
        for my in 0..m.h {
            let py = y + my as i32;
            if py < 0 || py as usize >= self.h {
                continue;
            }
            for mx in 0..m.w {
                let a = m.a[my * m.w + mx] as u32;
                if a != 0 {
                    self.put(x + mx as i32, py, c, a * alpha / 256);
                }
            }
        }
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

fn seg_dist(px: f32, py: f32, x0: f32, y0: f32, x1: f32, y1: f32) -> f32 {
    let (dx, dy) = (x1 - x0, y1 - y0);
    let len2 = dx * dx + dy * dy;
    let t = if len2 == 0.0 { 0.0 } else { (((px - x0) * dx + (py - y0) * dy) / len2).clamp(0.0, 1.0) };
    let (qx, qy) = (x0 + t * dx - px, y0 + t * dy - py);
    // distance via integer sqrt (no libm): work in 1/16 px
    let d2 = ((qx * qx + qy * qy) * 256.0) as u32;
    isqrt(d2) as f32 / 16.0
}

/// Rasterise a digit (0-9) or ':' as a smooth rounded seven-segment shape
/// into a coverage mask of `w` x `h` pixels with stroke width `stroke`.
pub fn digit_mask(m: &mut Mask, ox: i32, ch: char, w: i32, h: i32, stroke: f32) {
    // segments a..g as (x0,y0,x1,y1) in 0..1 x 0..1
    const SEG: [(f32, f32, f32, f32); 7] = [
        (0.0, 0.0, 1.0, 0.0), // a top
        (1.0, 0.0, 1.0, 0.5), // b upper right
        (1.0, 0.5, 1.0, 1.0), // c lower right
        (0.0, 1.0, 1.0, 1.0), // d bottom
        (0.0, 0.5, 0.0, 1.0), // e lower left
        (0.0, 0.0, 0.0, 0.5), // f upper left
        (0.0, 0.5, 1.0, 0.5), // g middle
    ];
    const ON: [u8; 10] = [0b0111111, 0b0000110, 0b1011011, 0b1001111, 0b1100110, 0b1101101, 0b1111101, 0b0000111, 0b1111111, 0b1101111];
    let half = stroke / 2.0;
    let (pad, iw, ih) = (half + 1.0, w as f32 - stroke - 2.0, h as f32 - stroke - 2.0);
    let put = |m: &mut Mask, x: i32, y: i32, cov: f32| {
        if x >= 0 && y >= 0 && (x as usize) < m.w && (y as usize) < m.h {
            let p = &mut m.a[y as usize * m.w + x as usize];
            *p = (*p).max((cov.clamp(0.0, 1.0) * 255.0) as u8);
        }
    };
    if ch == ':' {
        for cy in [h as f32 * 0.32, h as f32 * 0.68] {
            let c = (ox as f32 + w as f32 / 2.0, cy);
            let r = half * 0.95;
            for y in (c.1 - r - 1.0) as i32..=(c.1 + r + 1.0) as i32 {
                for x in (c.0 - r - 1.0) as i32..=(c.0 + r + 1.0) as i32 {
                    let d = seg_dist(x as f32 + 0.5, y as f32 + 0.5, c.0, c.1, c.0, c.1);
                    put(m, x, y, r - d + 0.5);
                }
            }
        }
        return;
    }
    let Some(d) = ch.to_digit(10) else { return };
    let bits = ON[d as usize];
    for (i, &(x0, y0, x1, y1)) in SEG.iter().enumerate() {
        if bits & (1 << i) == 0 {
            continue;
        }
        // shorten each segment slightly so neighbours leave a small gap
        let (gx, gy) = ((x1 - x0) * 0.0, (y1 - y0) * 0.0);
        let (ax, ay) = (ox as f32 + pad + (x0 + gx) * iw, pad + (y0 + gy) * ih);
        let (bx, by) = (ox as f32 + pad + (x1 - gx) * iw, pad + (y1 - gy) * ih);
        let (minx, maxx) = (ax.min(bx) - half - 1.0, ax.max(bx) + half + 1.0);
        let (miny, maxy) = (ay.min(by) - half - 1.0, ay.max(by) + half + 1.0);
        for y in miny as i32..=maxy as i32 {
            for x in minx as i32..=maxx as i32 {
                let dd = seg_dist(x as f32 + 0.5, y as f32 + 0.5, ax, ay, bx, by);
                put(m, x, y, half - dd + 0.5);
            }
        }
    }
}

/// Render `text` (digits and ':') as a smooth large mask.
pub fn clock_mask(text: &str, digit_w: i32, digit_h: i32, stroke: f32, gap: i32) -> Mask {
    let colon_w = (stroke * 1.6) as i32;
    let total: i32 = text.chars().map(|c| if c == ':' { colon_w + gap } else { digit_w + gap }).sum::<i32>() - gap;
    let mut m = Mask { w: total.max(1) as usize, h: digit_h as usize, a: vec![0; total.max(1) as usize * digit_h as usize] };
    let mut x = 0;
    for ch in text.chars() {
        if ch == ':' {
            digit_mask(&mut m, x, ch, colon_w, digit_h, stroke * 0.8);
            x += colon_w + gap;
        } else {
            digit_mask(&mut m, x, ch, digit_w, digit_h, stroke);
            x += digit_w + gap;
        }
    }
    m
}
