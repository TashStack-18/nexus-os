# Micro-OS on a PC: the x86-64 port (bootable ISO)

The kernel runs on two architectures from **one source tree**:

| | RISC-V (original) | x86-64 (PC, VirtualBox, VMware) |
|---|---|---|
| Firmware / loader | OpenSBI | GRUB, Multiboot2 |
| Entry | `boot/entry.S` (S-mode) | `x86/boot.S`: 32-bit stub → page tables → long mode → `kmain` |
| Console | NS16550 UART | VGA text screen (ANSI colours) + PS/2 keyboard, and COM1 |
| Timer | SBI timer, `time` CSR | 8254 PIT through the 8259 PIC (1 ms); TSC calibrated against the PIT |
| Traps | `stvec`, `trap.S` | IDT, `x86/trap.S` |
| Context switch | `task/switch.S` | `task/switch_x86.S` |
| User mode | U-mode, `ecall`, `sscratch` | ring 3, `int 0x80`, `TSS.rsp0` |
| Boot arguments | device tree `bootargs` | Multiboot2 command line |
| Power off | QEMU test device / SBI | ACPI ports (VirtualBox, QEMU, Bochs) |

Only these hardware modules differ; `main.rs` picks them with
`#[cfg_attr(target_arch = "x86_64", path = "x86/...")]`. The scheduler and
its 8 policies, synchronisation, the allocator, user-mode system calls, the
shell, the programs, the R3 research code and every test scenario are shared.
Time is kept in the same 100 ns unit on both ports.

## Build and run

```bash
scripts/build_x86_iso.sh          # -> micro-os-x86.iso (about 17 MB)
```

GRUB menu: interactive shell (default), run all tests, shell under SJF, shell under MLFQ.

VirtualBox: New VM, type *Other*, version *Other/Unknown (64-bit)*, 256 MB RAM,
no hard disk; Settings → Storage → optical drive → `micro-os-x86.iso`; Start.
VMware: guest *Other 64-bit*, BIOS firmware. A PC: write the ISO to USB and boot in legacy BIOS (CSM) mode.

## Verified

* x86-64: full in-kernel suite **68 of 68 checks pass**; shell on the VGA
  screen with the PS/2 keyboard (colour output, `user priv` killed by a
  general protection fault, `ps` with two tasks).
* RISC-V after the port: 34 unit tests, 68/68 checks, byte-identical logs, shell smoke test.

## Limits

* BIOS boot only (VGA text mode); UEFI-only machines would need a framebuffer console.
* Ring 3 is privilege separation, not memory isolation (identity-mapped, user-accessible pages).
* One CPU; the keyboard is polled.
