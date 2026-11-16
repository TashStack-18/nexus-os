//! Power off the PC. Tries the ACPI shutdown ports of the common virtual
//! machines; if none responds (e.g. real hardware without ACPI support in
//! this kernel), the screen says it is safe to switch off and the CPU halts.
//! Port 0xF4 is QEMU's debug-exit device, used only by the developer's test
//! scripts; on VirtualBox/VMware nothing is connected there.

use core::fmt::Write;

use crate::arch;
use crate::kernel::console;

pub fn exit(code: u32) -> ! {
    arch::intr_off();
    let _ = writeln!(console::emergency_writer(), "\nMicro-OS halted (exit code {}).", code);
    if super::gui::active() {
        super::gui::final_screen(match code {
            0 => "Micro-OS has shut down. It is now safe to turn off the computer.",
            101 => "Micro-OS stopped after a kernel error (see the messages above).",
            _ => "Micro-OS has stopped. It is now safe to turn off the computer.",
        });
    }
    // SAFETY: these ports are the ACPI PM1a control registers of the
    // supported virtual machines and QEMU's debug-exit port; writing them
    // only requests a power-off.
    unsafe {
        arch::outb(0xF4, code.min(0x7f) as u8); // QEMU isa-debug-exit (developer tests)
        arch::outw(0x4004, 0x3400); // VirtualBox
        arch::outw(0x604, 0x2000); // QEMU (newer machine types)
        arch::outw(0xB004, 0x2000); // Bochs / older QEMU
    }
    let _ = writeln!(console::emergency_writer(), "It is now safe to turn off the computer.");
    loop {
        // SAFETY: interrupts are off; hlt simply stops the CPU.
        unsafe { core::arch::asm!("cli", "hlt") };
    }
}
