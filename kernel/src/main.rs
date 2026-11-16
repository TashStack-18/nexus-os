//! Micro-OS: an educational kernel in Rust (CA-6), for RISC-V and x86-64.
//!
//! Boot path, RISC-V: OpenSBI (M-mode) → `_start` (boot/entry.S) → `kmain` (S-mode).
//! Boot path, x86-64: GRUB (Multiboot2) → `_start` (x86/boot.S, 32-bit) →
//! long mode → `kmain` (ring 0). Only the hardware modules differ between
//! the two (`#[cfg_attr(target_arch = "x86_64", path = ...)]` below); the
//! scheduler, sync, memory, user mode, shell and tests are shared.
//! `kmain` initialises each layer in order, then runs the scenarios selected
//! on the kernel command line and exits QEMU with the number of failed checks.

#![no_std]
#![no_main]

extern crate alloc;

#[macro_use]
mod kernel;

#[cfg_attr(target_arch = "x86_64", path = "x86/arch.rs")]
mod arch;
#[cfg_attr(target_arch = "x86_64", path = "x86/boot.rs")]
mod boot;
#[cfg_attr(target_arch = "x86_64", path = "x86/drivers/mod.rs")]
mod drivers;
mod memory;
mod programs;
mod scenarios;
mod scheduler;
mod shell;
mod sync;
mod sys;
mod task;
#[cfg_attr(target_arch = "x86_64", path = "x86/timer.rs")]
mod timer;
#[cfg_attr(target_arch = "x86_64", path = "x86/trap.rs")]
mod trap;
mod user;

use drivers::{qemu_exit, sbi};
use kernel::console::{self, Level};

#[no_mangle]
pub extern "C" fn kmain(a0: usize, a1: usize) -> ! {
    println!();
    #[cfg(target_arch = "riscv64")]
    let (hartid, dtb) = (a0, a1);
    #[cfg(target_arch = "x86_64")]
    let (hartid, dtb) = (0usize, a1); // a0 = Multiboot2 magic, a1 = boot information
    kinfo!("Micro-OS v{} booting on hart {} ({})", env!("CARGO_PKG_VERSION"), hartid, arch::NAME);
    ktrace!("BOOT", "stage=entry hart={} dtb={:#x}", hartid, dtb);
    let (major, minor) = sbi::spec_version();
    kinfo!("firmware: {} (interface v{}.{})", sbi::impl_name(), major, minor);

    #[cfg(target_arch = "riscv64")]
    let args = boot::boot_args(dtb);
    #[cfg(target_arch = "x86_64")]
    let args = boot::boot_args(a0, a1);
    if let Some(level) = args.get("log").and_then(Level::parse) {
        console::set_level(level);
    }
    console::set_trace(args.flag("trace", true));
    kinfo!("command line: '{}'", args.raw());
    ktrace!("BOOT", "stage=args fw={} sbi={}.{}", sbi::impl_name(), major, minor);

    trap::init();
    kinfo!("trap vector installed");
    ktrace!("BOOT", "stage=trap");

    let (heap_start, heap_size) = memory::init();
    // PC: switch the console to the desktop if GRUB gave us a graphics mode.
    #[cfg(target_arch = "x86_64")]
    drivers::gui::init(boot::framebuffer(a0, a1), args.get("test") == Some("shell"));
    let (kstart, kend) = memory::kernel_image();
    kinfo!("kernel image {:#x}..{:#x} ({} KiB)", kstart, kend, (kend - kstart) / 1024);
    kinfo!("heap {:#x}..{:#x} ({} KiB, first-fit free list)", heap_start, heap_start + heap_size, heap_size / 1024);
    ktrace!("BOOT", "stage=heap start={:#x} size={} kimg={:#x}-{:#x}", heap_start, heap_size, kstart, kend);

    let tick_us = args.get_u64("tick_us").unwrap_or(1000);
    timer::init(tick_us);
    arch::intr_on();
    kinfo!("timer running: one tick every {} us", tick_us);
    ktrace!("BOOT", "stage=timer tick_us={}", tick_us);

    let test = args.get("test").unwrap_or("all");
    let failed = scenarios::run(test, &args);

    let h = memory::stats();
    kinfo!(
        "heap at shutdown: in_use={} B peak={} B allocs={} frees={} failed={} rejected_frees={} free_blocks={}",
        h.bytes_in_use, h.peak_bytes, h.allocs, h.frees, h.failed, h.rejected_frees, h.free_blocks
    );
    ktrace!(
        "HEAP",
        "in_use={} peak={} allocs={} frees={} failed={} rejected={} free_blocks={} largest={}",
        h.bytes_in_use, h.peak_bytes, h.allocs, h.frees, h.failed, h.rejected_frees, h.free_blocks, h.largest_free
    );
    kinfo!("ticks={} - shutting down with exit code {}", timer::ticks(), failed);
    ktrace!("SHUTDOWN", "failed={}", failed);
    qemu_exit::exit(failed)
}
