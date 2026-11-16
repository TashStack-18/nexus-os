//! The PC's real-time clock (CMOS RTC), for the taskbar clock and date.

use crate::arch::{inb, outb};

#[derive(Clone, Copy, Default, PartialEq)]
pub struct DateTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

fn reg(r: u8) -> u8 {
    // SAFETY: CMOS index/data ports; 0x80 keeps NMIs disabled while selecting.
    unsafe {
        outb(0x70, 0x80 | r);
        inb(0x71)
    }
}

fn updating() -> bool {
    reg(0x0A) & 0x80 != 0
}

/// Current date and time as kept by the RTC (normally local time on PCs,
/// as VirtualBox and VMware set it from the host).
pub fn now() -> DateTime {
    let read = || {
        while updating() {
            core::hint::spin_loop();
        }
        [reg(0x00), reg(0x02), reg(0x04), reg(0x07), reg(0x08), reg(0x09), reg(0x32)]
    };
    // Read twice until stable, so an update in between cannot tear the value.
    let mut a = read();
    loop {
        let b = read();
        if a == b {
            break;
        }
        a = b;
    }
    let status_b = reg(0x0B);
    let bcd = status_b & 0x04 == 0;
    let conv = |v: u8| if bcd { (v & 0x0F) + (v >> 4) * 10 } else { v };
    let mut hour = a[2];
    let pm = hour & 0x80 != 0;
    hour = conv(hour & 0x7F);
    if status_b & 0x02 == 0 && pm {
        hour = (hour % 12) + 12; // 12-hour mode
    }
    let century = if a[6] != 0 && a[6] != 0xFF { conv(a[6]) as u16 } else { 20 };
    DateTime {
        second: conv(a[0]),
        minute: conv(a[1]),
        hour,
        day: conv(a[3]),
        month: conv(a[4]),
        year: century * 100 + conv(a[5]) as u16,
    }
}
