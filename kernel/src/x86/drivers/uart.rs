//! The PC console device. Keeps the RISC-V module's name and interface
//! (`Uart`, `put_byte`, `try_read_byte`) so `kernel/console.rs` is shared:
//! output goes to the VGA text screen (what VirtualBox/VMware show) and to
//! COM1 if one is present; input comes from the PS/2 keyboard or COM1.

use core::fmt;
use core::sync::atomic::{AtomicU8, Ordering::Relaxed};

use super::{gui, keyboard, vga};
use crate::arch::{inb, outb};

pub const UART0_BASE: usize = 0x3F8; // COM1 I/O port base
const LSR: u16 = 5;
const LSR_DATA_READY: u8 = 1 << 0;
const LSR_THR_EMPTY: u8 = 1 << 5;

/// 0 = not probed yet, 1 = COM1 present, 2 = absent.
static COM1: AtomicU8 = AtomicU8::new(0);

pub struct Uart {
    base: u16,
}

impl Uart {
    /// # Safety
    /// `base` must be the I/O port base of a 16550-compatible UART (or of
    /// nothing: the device is probed before use).
    pub const unsafe fn new(base: usize) -> Self {
        Self { base: base as u16 }
    }

    fn com_present(&self) -> bool {
        match COM1.load(Relaxed) {
            1 => true,
            2 => false,
            _ => {
                // SAFETY: COM1's scratch register (base+7) has no side
                // effects; a missing device reads back 0xFF, not our value.
                let ok = unsafe {
                    outb(self.base + 7, 0xA5);
                    inb(self.base + 7) == 0xA5
                };
                if ok {
                    // SAFETY: standard 16550 setup: 115200 baud, 8N1, FIFO on.
                    unsafe {
                        outb(self.base + 1, 0x00);
                        outb(self.base + 3, 0x80);
                        outb(self.base, 0x01);
                        outb(self.base + 1, 0x00);
                        outb(self.base + 3, 0x03);
                        outb(self.base + 2, 0xC7);
                        outb(self.base + 4, 0x03);
                    }
                }
                COM1.store(if ok { 1 } else { 2 }, Relaxed);
                ok
            }
        }
    }

    pub fn put_byte(&mut self, byte: u8) {
        vga::put_byte(byte);
        if self.com_present() {
            // SAFETY: COM1 is present (probed); LSR and THR are its registers.
            unsafe {
                while inb(self.base + LSR) & LSR_THR_EMPTY == 0 {
                    core::hint::spin_loop();
                }
                outb(self.base, byte);
            }
        }
    }

    /// A key from the PS/2 keyboard or a byte from COM1, if one is waiting.
    pub fn try_read_byte(&mut self) -> Option<u8> {
        // With the desktop running, keys typed into the Terminal window
        // arrive through its queue; the desktop owns the PS/2 controller.
        let key = if gui::active() { gui::pop_key() } else { keyboard::try_read() };
        if key.is_some() {
            return key;
        }
        if self.com_present() {
            // SAFETY: COM1 is present; reading RBR consumes the waiting byte.
            unsafe {
                if inb(self.base + LSR) & LSR_DATA_READY != 0 {
                    return Some(inb(self.base));
                }
            }
        }
        None
    }
}

impl fmt::Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for b in s.bytes() {
            if b == b'\n' {
                self.put_byte(b'\r');
            }
            self.put_byte(b);
        }
        Ok(())
    }
}
