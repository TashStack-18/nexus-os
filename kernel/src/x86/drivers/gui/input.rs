//! Keyboard and mouse for the desktop: reads the PS/2 controller, decodes
//! keyboard scan codes and 3-byte mouse packets.

use crate::arch::{inb, outb};
use crate::drivers::keyboard::{self, DATA, FROM_MOUSE, OUTPUT_FULL, STATUS};

pub enum Event {
    Key(u8),
    /// Relative movement (x right, y down) and button state (bit 0 left, bit 1 right).
    Mouse { dx: i32, dy: i32, buttons: u8 },
}

fn wait_write() {
    // SAFETY: reading the PS/2 status port has no side effects.
    for _ in 0..100_000 {
        if unsafe { inb(STATUS) } & 0x02 == 0 {
            return;
        }
        core::hint::spin_loop();
    }
}

fn wait_read() -> bool {
    for _ in 0..100_000 {
        // SAFETY: as above.
        if unsafe { inb(STATUS) } & OUTPUT_FULL != 0 {
            return true;
        }
        core::hint::spin_loop();
    }
    false
}

fn mouse_cmd(cmd: u8) {
    wait_write();
    // SAFETY: 0xD4 sends the next data byte to the auxiliary (mouse) port.
    unsafe {
        outb(STATUS, 0xD4);
    }
    wait_write();
    // SAFETY: data port write: one mouse command byte.
    unsafe { outb(DATA, cmd) };
    if wait_read() {
        // SAFETY: consume the mouse's acknowledgement (0xFA).
        unsafe { inb(DATA) };
    }
}

/// Enable the PS/2 mouse (auxiliary device) in streaming mode. Interrupts
/// stay off for it: the desktop polls.
pub fn init_mouse() {
    // SAFETY: standard i8042 controller sequence: enable the auxiliary port,
    // read the configuration byte, enable the aux clock (clear bit 5) and
    // leave both device IRQs off (bits 0, 1), write it back.
    unsafe {
        wait_write();
        outb(STATUS, 0xA8);
        wait_write();
        outb(STATUS, 0x20);
        let cfg = if wait_read() { inb(DATA) } else { 0 };
        let cfg = (cfg & !0x23) & 0xFF;
        wait_write();
        outb(STATUS, 0x60);
        wait_write();
        outb(DATA, cfg);
    }
    mouse_cmd(0xF6); // defaults
    mouse_cmd(0xF4); // enable data reporting
}

pub struct Input {
    packet: [u8; 3],
    have: usize,
}

impl Input {
    pub const fn new() -> Input {
        Input { packet: [0; 3], have: 0 }
    }

    /// Next input event, if any byte is waiting.
    pub fn poll(&mut self) -> Option<Event> {
        loop {
            // SAFETY: read status (no side effects), then consume exactly the
            // waiting byte from the data port.
            let (st, b) = unsafe {
                let st = inb(STATUS);
                if st & OUTPUT_FULL == 0 {
                    return None;
                }
                (st, inb(DATA))
            };
            if st & FROM_MOUSE == 0 {
                if let Some(c) = keyboard::decode(b) {
                    return Some(Event::Key(c));
                }
                continue;
            }
            if self.have == 0 && b & 0x08 == 0 {
                continue; // out of sync: the first byte always has bit 3 set
            }
            self.packet[self.have] = b;
            self.have += 1;
            if self.have == 3 {
                self.have = 0;
                let [b0, b1, b2] = self.packet;
                if b0 & 0xC0 != 0 {
                    continue; // overflow: drop the packet
                }
                let dx = b1 as i32 - (((b0 as i32) << 4) & 0x100);
                let dy = b2 as i32 - (((b0 as i32) << 3) & 0x100);
                return Some(Event::Mouse { dx, dy: -dy, buttons: b0 & 0x03 });
            }
        }
    }
}
