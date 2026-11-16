#!/usr/bin/env bash
# Build the bootable PC ISO of the Rust Micro-OS kernel (x86-64).
#
#   scripts/build_x86_iso.sh                  -> micro-os-x86.iso
#   DEFAULT=1 scripts/build_x86_iso.sh        same, but the test-suite entry boots by default
#
# The ISO contains GRUB and the kernel ELF (kernel/target/x86_64-unknown-none/
# release/microos). GRUB loads the kernel through Multiboot2; the kernel then
# runs directly on the (virtual) PC: VGA screen, PS/2 keyboard, PIT timer.
# Boots in VirtualBox / VMware (BIOS) or from a USB stick on a PC in legacy
# BIOS mode. No emulator is involved.
set -euo pipefail
[ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="${OUT:-$ROOT/micro-os-x86.iso}"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

(cd "$ROOT/kernel" && cargo build --release --target x86_64-unknown-none)
mkdir -p "$WORK/boot/grub"
cp "$ROOT/kernel/target/x86_64-unknown-none/release/microos" "$WORK/boot/microos.elf"
cat > "$WORK/boot/grub/grub.cfg" <<EOF
set timeout=5
set default=${DEFAULT:-0}
insmod all_video
set gfxpayload=text

menuentry "Micro-OS (Rust kernel) - interactive shell" {
    multiboot2 /boot/microos.elf test=shell trace=0
    boot
}
menuentry "Micro-OS (Rust kernel) - run all tests (TC1-TC6, sync, memory, system calls)" {
    multiboot2 /boot/microos.elf test=all trace=0
    boot
}
menuentry "Micro-OS (Rust kernel) - shell, Shortest Job First scheduler" {
    multiboot2 /boot/microos.elf test=shell trace=0 policy=sjf
    boot
}
menuentry "Micro-OS (Rust kernel) - shell, Multi-Level Feedback Queue" {
    multiboot2 /boot/microos.elf test=shell trace=0 policy=mlfq
    boot
}
EOF
grub-mkrescue -o "$OUT" "$WORK" -- -volid MICROOS_X86 >/dev/null 2>&1
echo "built $OUT ($(du -h "$OUT" | cut -f1))"
