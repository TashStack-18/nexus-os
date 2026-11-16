//! Bitmap fonts for the desktop: Terminus 8x16 (SIL Open Font License),
//! embedded as PSF1 files. The PSF1 Unicode table maps characters to
//! glyphs, so box-drawing and symbol characters show their real shapes.

use alloc::vec;
use alloc::vec::Vec;

static REGULAR: &[u8] = include_bytes!("../../../../assets/terminus16.psf");
static BOLD: &[u8] = include_bytes!("../../../../assets/terminus16b.psf");

pub const W: usize = 8;
const MAP_LIMIT: usize = 0x2600; // Unicode range we index (Latin, symbols, box drawing)

pub struct Font {
    data: &'static [u8],
    glyphs: usize,
    height: usize,
    /// Unicode code point (< MAP_LIMIT) -> glyph index + 1 (0 = missing).
    map: Vec<u16>,
}

impl Font {
    fn parse(data: &'static [u8]) -> Font {
        assert!(data.len() > 4 && data[0] == 0x36 && data[1] == 0x04, "not a PSF1 font");
        let mode = data[2];
        let height = data[3] as usize;
        let glyphs = if mode & 1 != 0 { 512 } else { 256 };
        let mut map = vec![0u16; MAP_LIMIT];
        let table = 4 + glyphs * height;
        if mode & 2 != 0 && data.len() > table {
            // Unicode table: per glyph, u16 code points, 0xFFFE starts a
            // multi-code-point sequence (ignored), 0xFFFF ends the glyph.
            let mut i = table;
            let mut g = 0usize;
            let mut in_seq = false;
            while i + 1 < data.len() && g < glyphs {
                let cp = u16::from_le_bytes([data[i], data[i + 1]]);
                i += 2;
                match cp {
                    0xFFFF => {
                        g += 1;
                        in_seq = false;
                    }
                    0xFFFE => in_seq = true,
                    c if !in_seq && (c as usize) < MAP_LIMIT && map[c as usize] == 0 => map[c as usize] = g as u16 + 1,
                    _ => {}
                }
            }
        } else {
            for c in 0..glyphs.min(256) {
                map[c] = c as u16 + 1;
            }
        }
        Font { data, glyphs, height, map }
    }

    /// Glyph bitmap rows (one byte per row, MSB = leftmost pixel) for `c`.
    pub fn glyph(&self, c: char) -> &[u8] {
        let cp = c as usize;
        let mut idx = if cp < MAP_LIMIT { self.map[cp] } else { 0 };
        if idx == 0 {
            idx = self.map[b'?' as usize];
        }
        let g = (idx.max(1) - 1) as usize % self.glyphs;
        &self.data[4 + g * self.height..4 + (g + 1) * self.height]
    }
}

pub struct Fonts {
    pub regular: Font,
    pub bold: Font,
}

impl Fonts {
    pub fn load() -> Fonts {
        Fonts { regular: Font::parse(REGULAR), bold: Font::parse(BOLD) }
    }
}
