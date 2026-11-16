//! PS/2 keyboard, polled (scan code set 1, US layout). Produces the same
//! bytes a serial terminal sends, so the shell needs no changes: Enter is
//! '\r', Backspace is DEL (0x7f), Ctrl+C is 0x03.

use core::sync::atomic::{AtomicBool, Ordering::Relaxed};

use crate::arch::inb;

const DATA: u16 = 0x60;
const STATUS: u16 = 0x64;
const OUTPUT_FULL: u8 = 1 << 0;
const FROM_MOUSE: u8 = 1 << 5;

static SHIFT: AtomicBool = AtomicBool::new(false);
static CTRL: AtomicBool = AtomicBool::new(false);
static CAPS: AtomicBool = AtomicBool::new(false);
static EXTENDED: AtomicBool = AtomicBool::new(false);

const NORMAL: &[u8; 58] = b"\0\x1b1234567890-=\x7f\tqwertyuiop[]\r\0asdfghjkl;'`\0\\zxcvbnm,./\0*\0 ";
const SHIFTED: &[u8; 58] = b"\0\x1b!@#$%^&*()_+\x7f\tQWERTYUIOP{}\r\0ASDFGHJKL:\"~\0|ZXCVBNM<>?\0*\0 ";

/// Translate waiting scan codes; returns the first key that produces a byte.
pub fn try_read() -> Option<u8> {
    loop {
        // SAFETY: 0x64 is the PS/2 controller status port; reading it has
        // no side effects. 0x60 is read only when a byte is waiting, which
        // consumes exactly that byte.
        let sc = unsafe {
            let st = inb(STATUS);
            if st & OUTPUT_FULL == 0 {
                return None;
            }
            let b = inb(DATA);
            if st & FROM_MOUSE != 0 {
                continue; // mouse packets are not ours
            }
            b
        };
        if sc == 0xE0 {
            EXTENDED.store(true, Relaxed);
            continue;
        }
        if EXTENDED.swap(false, Relaxed) {
            continue; // arrows, keypad Enter etc.: not used by the shell
        }
        let release = sc & 0x80 != 0;
        let code = sc & 0x7F;
        match code {
            0x2A | 0x36 => {
                SHIFT.store(!release, Relaxed);
                continue;
            }
            0x1D => {
                CTRL.store(!release, Relaxed);
                continue;
            }
            0x3A => {
                if !release {
                    CAPS.store(!CAPS.load(Relaxed), Relaxed);
                }
                continue;
            }
            _ => {}
        }
        if release || code as usize >= NORMAL.len() {
            continue;
        }
        let mut c = if SHIFT.load(Relaxed) { SHIFTED[code as usize] } else { NORMAL[code as usize] };
        if CAPS.load(Relaxed) && c.is_ascii_alphabetic() {
            c ^= 0x20;
        }
        if c == 0 {
            continue;
        }
        if CTRL.load(Relaxed) && c.is_ascii_alphabetic() {
            c = c.to_ascii_lowercase() & 0x1F; // Ctrl+C = 0x03
        }
        return Some(c);
    }
}
