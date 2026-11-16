//! Interrupt and exception layer for x86-64: GDT (with user segments and a
//! TSS), IDT, the 8259 PIC, and the Rust side of trap handling. Mirrors the
//! RISC-V `trap` module: timer interrupts drive the scheduler, `int 0x80`
//! from ring 3 is a system call, and an exception in ring 3 kills only that
//! program.

use core::arch::asm;
use core::cell::UnsafeCell;

use crate::arch::{self, outb};
use crate::scheduler;

core::arch::global_asm!(include_str!("trap.S"));

extern "C" {
    static microos_isr_table: [usize; 48];
    static microos_isr_syscall: usize;
}

pub const KERNEL_CS: u16 = 0x08;
pub const KERNEL_DS: u16 = 0x10;
const TSS_SEL: u16 = 0x28;
const VECTOR_TIMER: usize = 32;
const VECTOR_SYSCALL: usize = 0x80;
const PIC1: u16 = 0x20;
const PIC2: u16 = 0xA0;

/// Register state saved by `trap.S`; layout must match the assembly.
#[repr(C)]
pub struct TrapFrame {
    pub r15: usize,
    pub r14: usize,
    pub r13: usize,
    pub r12: usize,
    pub r11: usize,
    pub r10: usize,
    pub r9: usize,
    pub r8: usize,
    pub rbp: usize,
    pub rdi: usize,
    pub rsi: usize,
    pub rdx: usize,
    pub rcx: usize,
    pub rbx: usize,
    pub rax: usize,
    pub vector: usize,
    pub error: usize,
    pub rip: usize,
    pub cs: usize,
    pub rflags: usize,
    pub rsp: usize,
    pub ss: usize,
}

impl TrapFrame {
    /// Did this trap interrupt ring-3 (user) code?
    pub fn from_user(&self) -> bool {
        self.cs & 3 == 3
    }
    /// System call number (rax) and arguments (rdi, rsi), as in Linux.
    pub fn syscall_nr(&self) -> usize {
        self.rax
    }
    pub fn arg0(&self) -> usize {
        self.rdi
    }
    pub fn arg1(&self) -> usize {
        self.rsi
    }
    pub fn set_ret(&mut self, v: usize) {
        self.rax = v;
    }
}

// ------------------------------------------------------------- GDT and TSS

#[repr(C, packed)]
pub struct Tss {
    _r0: u32,
    pub rsp0: u64,
    rsp1: u64,
    rsp2: u64,
    _r1: u64,
    ist: [u64; 7],
    _r2: u64,
    _r3: u16,
    iomap: u16,
}

/// The TSS; trap.S writes `rsp0` (offset 4) before returning to ring 3.
#[no_mangle]
static mut MICROOS_TSS: Tss = Tss {
    _r0: 0,
    rsp0: 0,
    rsp1: 0,
    rsp2: 0,
    _r1: 0,
    ist: [0; 7],
    _r2: 0,
    _r3: 0,
    iomap: core::mem::size_of::<Tss>() as u16, // no I/O permission bitmap
};

#[repr(C, align(16))]
struct Gdt(UnsafeCell<[u64; 7]>);
// SAFETY: written once in `init` before interrupts are enabled; only the CPU reads it afterwards.
unsafe impl Sync for Gdt {}
static GDT: Gdt = Gdt(UnsafeCell::new([
    0,
    0x00AF_9A00_0000_FFFF, // 0x08 kernel code (64-bit)
    0x00CF_9200_0000_FFFF, // 0x10 kernel data
    0x00CF_F200_0000_FFFF, // 0x18 user data, DPL 3
    0x00AF_FA00_0000_FFFF, // 0x20 user code (64-bit), DPL 3
    0,                     // 0x28 TSS (low), filled in by init
    0,                     // TSS (high)
]));

#[repr(C, packed)]
struct DescPtr {
    limit: u16,
    base: u64,
}

// ------------------------------------------------------------- IDT

#[derive(Clone, Copy)]
#[repr(C)]
struct Gate {
    off_lo: u16,
    sel: u16,
    ist: u8,
    attr: u8,
    off_mid: u16,
    off_hi: u32,
    _zero: u32,
}

impl Gate {
    const EMPTY: Gate = Gate { off_lo: 0, sel: 0, ist: 0, attr: 0, off_mid: 0, off_hi: 0, _zero: 0 };
    fn new(handler: usize, dpl: u8) -> Gate {
        Gate {
            off_lo: handler as u16,
            sel: KERNEL_CS,
            ist: 0,
            attr: 0x8E | (dpl << 5), // present, 64-bit interrupt gate (clears IF)
            off_mid: (handler >> 16) as u16,
            off_hi: (handler >> 32) as u32,
            _zero: 0,
        }
    }
}

#[repr(C, align(16))]
struct Idt(UnsafeCell<[Gate; 256]>);
// SAFETY: written once in `init` before interrupts are enabled.
unsafe impl Sync for Idt {}
static IDT: Idt = Idt(UnsafeCell::new([Gate::EMPTY; 256]));

pub fn init() {
    // SAFETY: runs once on the boot CPU with interrupts off, before any
    // trap can occur. The tables are statics, so the addresses loaded with
    // lgdt/lidt/ltr stay valid for the life of the kernel. Reloading CS via
    // a far return and loading TR with the TSS selector follow the
    // architectural sequence.
    unsafe {
        let tss = core::ptr::addr_of!(MICROOS_TSS) as u64;
        let limit = (core::mem::size_of::<Tss>() - 1) as u64;
        let gdt = &mut *GDT.0.get();
        gdt[5] = (limit & 0xFFFF) | ((tss & 0xFF_FFFF) << 16) | (0x89 << 40) | (((limit >> 16) & 0xF) << 48) | (((tss >> 24) & 0xFF) << 56);
        gdt[6] = tss >> 32;
        let gp = DescPtr { limit: (core::mem::size_of::<[u64; 7]>() - 1) as u16, base: gdt.as_ptr() as u64 };
        asm!(
            "lgdt [{gp}]",
            "push {cs}",
            "lea {tmp}, [rip + 2f]",
            "push {tmp}",
            "retfq",
            "2:",
            "mov ds, {ds:x}",
            "mov es, {ds:x}",
            "mov ss, {ds:x}",
            "ltr {tss:x}",
            gp = in(reg) &gp,
            cs = in(reg) KERNEL_CS as u64,
            ds = in(reg) KERNEL_DS as u64,
            tss = in(reg) TSS_SEL as u64,
            tmp = out(reg) _,
        );

        let idt = &mut *IDT.0.get();
        for (v, &handler) in microos_isr_table.iter().enumerate() {
            idt[v] = Gate::new(handler, 0);
        }
        // int 0x80 may be issued from ring 3.
        idt[VECTOR_SYSCALL] = Gate::new(microos_isr_syscall, 3);
        let ip = DescPtr { limit: (core::mem::size_of::<[Gate; 256]>() - 1) as u16, base: idt.as_ptr() as u64 };
        asm!("lidt [{}]", in(reg) &ip);

        // Remap the PICs to vectors 32-47 and mask everything; the timer
        // unmasks IRQ 0. The keyboard is polled, so IRQ 1 stays masked.
        outb(PIC1, 0x11);
        outb(PIC2, 0x11);
        outb(PIC1 + 1, 32);
        outb(PIC2 + 1, 40);
        outb(PIC1 + 1, 4);
        outb(PIC2 + 1, 2);
        outb(PIC1 + 1, 1);
        outb(PIC2 + 1, 1);
        outb(PIC1 + 1, 0xFF);
        outb(PIC2 + 1, 0xFF);
    }
}

/// Let IRQ 0 (the PIT) through the PIC.
pub fn unmask_timer() {
    // SAFETY: PIC1 data port: clear only the IRQ 0 mask bit.
    unsafe { outb(PIC1 + 1, 0xFE) };
}

fn exception_name(v: usize) -> &'static str {
    match v {
        0 => "divide error",
        1 => "debug",
        2 => "non-maskable interrupt",
        3 => "breakpoint",
        4 => "overflow",
        5 => "bound range exceeded",
        6 => "invalid opcode",
        7 => "device not available",
        8 => "double fault",
        10 => "invalid TSS",
        11 => "segment not present",
        12 => "stack-segment fault",
        13 => "general protection fault",
        14 => "page fault",
        16 => "x87 floating-point error",
        17 => "alignment check",
        18 => "machine check",
        19 => "SIMD floating-point error",
        _ => "unknown exception",
    }
}

/// Called by `trap.S` with interrupts disabled (interrupt gates clear IF).
/// A timer interrupt may switch to another task inside
/// `on_timer_interrupt`; this function then returns only when the
/// interrupted task is scheduled again.
#[no_mangle]
extern "C" fn trap_handler(tf: &mut TrapFrame) {
    match tf.vector {
        VECTOR_TIMER => {
            // Acknowledge first: we may switch tasks before returning.
            // SAFETY: end-of-interrupt to the master PIC.
            unsafe { outb(PIC1, 0x20) };
            scheduler::on_timer_interrupt();
        }
        33..=47 => {
            // Spurious or unexpected PIC interrupt: acknowledge and ignore.
            // SAFETY: end-of-interrupt to the PICs.
            unsafe {
                if tf.vector >= 40 {
                    outb(PIC2, 0x20);
                }
                outb(PIC1, 0x20);
            }
        }
        VECTOR_SYSCALL => crate::user::syscall(tf),
        v if tf.from_user() => {
            // A user program did something it may not (e.g. a privileged
            // instruction): stop that program, keep the kernel running.
            let addr = if v == 14 { arch::read_cr2() } else { tf.error };
            crate::user::kill_current(exception_name(v), tf.rip, addr);
        }
        v => panic!(
            "unhandled exception: {} (vector {}, error {:#x}) at rip={:#x}, cr2={:#x}",
            exception_name(v),
            v,
            tf.error,
            tf.rip,
            arch::read_cr2()
        ),
    }
}
