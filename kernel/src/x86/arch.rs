//! x86-64 CPU access: interrupts, port I/O, timestamps. The x86 counterpart
//! of `src/arch.rs` (RISC-V); every inline-asm block of the x86 port outside
//! its `.S` files lives here, behind small safe functions.
//!
//! The interrupt helpers omit `nomem` so they also act as compiler barriers.

use core::arch::asm;

/// How this port describes itself in banners and `sysinfo`.
pub const NAME: &str = "x86_64, ring 0";
pub const MACHINE: &str = "PC (x86-64), 1 CPU, booted by GRUB (Multiboot2)";

const RFLAGS_IF: u64 = 1 << 9;

#[inline]
pub fn intr_on() {
    // SAFETY: setting IF only lets pending, unmasked interrupts in.
    unsafe { asm!("sti", options(nostack)) };
}

#[inline]
pub fn intr_off() {
    // SAFETY: clearing IF only masks maskable interrupts.
    unsafe { asm!("cli", options(nostack)) };
}

#[inline]
pub fn rflags() -> u64 {
    let f: u64;
    // SAFETY: pushes and pops one word on the current stack.
    unsafe { asm!("pushfq", "pop {}", out(reg) f, options(nomem, preserves_flags)) };
    f
}

#[inline]
pub fn intr_enabled() -> bool {
    rflags() & RFLAGS_IF != 0
}

/// Wait for an interrupt and handle it, then restore the previous interrupt
/// state. Unlike RISC-V `wfi`, x86 `hlt` does not wake for a masked
/// interrupt, so interrupts are enabled for the halt (`sti; hlt` is atomic:
/// no interrupt can slip in between them).
#[inline]
pub fn wfi() {
    // SAFETY: the interrupt taken during hlt runs the normal trap path; the
    // saved RFLAGS (and so the IF bit) is restored afterwards.
    unsafe { asm!("pushfq", "sti", "hlt", "popfq") };
}

/// Current frame pointer (rbp), for backtraces.
#[inline(always)]
pub fn read_fp() -> usize {
    let v: usize;
    // SAFETY: copies a register.
    unsafe { asm!("mov {}, rbp", out(reg) v, options(nomem, nostack, preserves_flags)) };
    v
}

/// Time-stamp counter.
#[inline]
pub fn rdtsc() -> u64 {
    let (lo, hi): (u32, u32);
    // SAFETY: rdtsc only reads the counter.
    unsafe { asm!("rdtsc", out("eax") lo, out("edx") hi, options(nomem, nostack, preserves_flags)) };
    ((hi as u64) << 32) | lo as u64
}

/// Faulting address of the last page fault.
pub fn read_cr2() -> usize {
    let v: usize;
    // SAFETY: plain control-register read in ring 0.
    unsafe { asm!("mov {}, cr2", out(reg) v, options(nomem, nostack, preserves_flags)) };
    v
}

/// # Safety
/// Writing an I/O port can have device side effects; `port` must belong to
/// a device the caller is entitled to program.
#[inline]
pub unsafe fn outb(port: u16, v: u8) {
    asm!("out dx, al", in("dx") port, in("al") v, options(nomem, nostack, preserves_flags));
}

/// # Safety
/// As for [`outb`].
#[inline]
pub unsafe fn outw(port: u16, v: u16) {
    asm!("out dx, ax", in("dx") port, in("ax") v, options(nomem, nostack, preserves_flags));
}

/// # Safety
/// As for [`outb`].
#[inline]
pub unsafe fn inw(port: u16) -> u16 {
    let v: u16;
    asm!("in ax, dx", in("dx") port, out("ax") v, options(nomem, nostack, preserves_flags));
    v
}

/// # Safety
/// Reading some ports has device side effects (e.g. consuming a byte).
#[inline]
pub unsafe fn inb(port: u16) -> u8 {
    let v: u8;
    asm!("in al, dx", in("dx") port, out("al") v, options(nomem, nostack, preserves_flags));
    v
}
