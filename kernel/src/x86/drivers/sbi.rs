//! Firmware description for the PC port. RISC-V talks to OpenSBI through the
//! SBI; on a PC the kernel is started by GRUB through Multiboot2 and needs no
//! firmware calls afterwards, so this module only describes the boot path.

/// Boot protocol version as (major, minor): Multiboot2.
pub fn spec_version() -> (usize, usize) {
    (2, 0)
}

pub fn impl_name() -> &'static str {
    "GRUB (Multiboot2)"
}
