//! Layer 6 — periodic timer on a PC. The 8253/8254 PIT raises IRQ 0 every
//! tick through the 8259 PIC; time stamps come from the TSC, calibrated
//! against the PIT at boot. To share the scheduler and research code with
//! the RISC-V port unchanged, "raw" time is kept in the same unit there:
//! 100 ns (RISC-V's 10 MHz `time` CSR).

use core::sync::atomic::{AtomicU64, Ordering::Relaxed};

use crate::arch::{self, inb, outb};

/// Raw time units per second (100 ns each), as on RISC-V.
pub const TIMEBASE_HZ: u64 = 10_000_000;
const UNITS_PER_US: u64 = TIMEBASE_HZ / 1_000_000;
const PIT_HZ: u64 = 1_193_182;

static TICKS: AtomicU64 = AtomicU64::new(0);
static TICK_US: AtomicU64 = AtomicU64::new(1000);
static TSC_HZ: AtomicU64 = AtomicU64::new(0);
static TSC_BASE: AtomicU64 = AtomicU64::new(0);

/// Measure the TSC frequency over 10 ms of PIT channel 2 (no interrupts needed).
fn calibrate_tsc() -> u64 {
    let count: u16 = (PIT_HZ / 100) as u16; // 10 ms
    // SAFETY: port 0x61 gates PIT channel 2 (speaker output stays off);
    // ports 0x42/0x43 program channel 2, which nothing else uses.
    unsafe {
        outb(0x61, (inb(0x61) & 0xFD) | 0x01);
        outb(0x43, 0xB0); // channel 2, lo/hi byte, mode 0 (count down once)
        outb(0x42, (count & 0xFF) as u8);
        outb(0x42, (count >> 8) as u8);
        let t0 = arch::rdtsc();
        while inb(0x61) & 0x20 == 0 {
            core::hint::spin_loop();
        }
        (arch::rdtsc() - t0) * 100
    }
}

pub fn init(tick_us: u64) {
    TSC_BASE.store(arch::rdtsc(), Relaxed);
    TSC_HZ.store(calibrate_tsc().max(1_000_000), Relaxed);
    let tick_us = tick_us.clamp(100, 50_000);
    let divisor = (PIT_HZ * tick_us / 1_000_000).clamp(1, 0xFFFF);
    TICK_US.store(divisor * 1_000_000 / PIT_HZ, Relaxed);
    // SAFETY: PIT command/data ports: channel 0, lo/hi byte, mode 2 (rate
    // generator), then the divisor. IRQ 0 is unmasked in trap::init.
    unsafe {
        outb(0x43, 0x34);
        outb(0x40, (divisor & 0xFF) as u8);
        outb(0x40, (divisor >> 8) as u8);
    }
    crate::trap::unmask_timer();
}

/// Count a tick (the PIC was already acknowledged by the trap handler).
pub fn handle_tick() -> u64 {
    TICKS.fetch_add(1, Relaxed) + 1
}

pub fn ticks() -> u64 {
    TICKS.load(Relaxed)
}

/// Raw time (100 ns units) since the timer was initialised.
pub fn now_raw() -> u64 {
    let hz = TSC_HZ.load(Relaxed);
    if hz == 0 {
        return 0; // before calibration (early boot log lines)
    }
    let elapsed = arch::rdtsc().wrapping_sub(TSC_BASE.load(Relaxed));
    ((elapsed as u128 * TIMEBASE_HZ as u128) / hz as u128) as u64
}

pub fn now_us() -> u64 {
    now_raw() / UNITS_PER_US
}

/// Length of one timer tick in microseconds.
pub fn tick_us() -> u64 {
    TICK_US.load(Relaxed).max(1)
}

pub fn raw_to_ns(raw: u64) -> u64 {
    raw * (1_000_000_000 / TIMEBASE_HZ)
}
