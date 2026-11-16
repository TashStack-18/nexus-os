//! Hardware abstraction for a PC (VirtualBox, VMware or real hardware):
//! a console made of the VGA text screen, the PS/2 keyboard and COM1, and
//! ACPI power-off. Module names match the RISC-V port so the rest of the
//! kernel is unchanged.

mod keyboard;
pub mod gui;
pub mod qemu_exit;
pub mod sbi;
pub mod uart;
mod vga;
