#!/usr/bin/env bash
# Build the bootable PC ISO of the Rust Micro-OS kernel (x86-64).
#
#   scripts/build_x86_iso.sh                  -> micro-os-x86.iso
#   DEFAULT=1 scripts/build_x86_iso.sh        same, but the test-suite entry boots by default
#
# The ISO contains GRUB and the kernel ELF (kernel/target/x86_64-unknown-none/
# release/microos). GRUB loads the kernel through Multiboot2; the kernel then
# runs directly on the (virtual) PC and draws its own desktop (framebuffer,
# PS/2 keyboard and mouse, PIT timer).
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
set timeout=1
set default=${DEFAULT:-0}
insmod all_video
insmod gfxterm
set gfxmode=1920x1080x32,1600x900x32,1366x768x32,1280x720x32,1024x768x32,auto
set gfxpayload=keep
terminal_output gfxterm

menuentry "Micro-OS (Rust kernel) - desktop" {
    multiboot2 /boot/microos.elf test=shell trace=0
    boot
}
menuentry "Micro-OS (Rust kernel) - run all tests (TC1-TC6, sync, memory, system calls)" {
    multiboot2 /boot/microos.elf test=all trace=0
    boot
}
menuentry "Micro-OS (Rust kernel) - desktop, Shortest Job First scheduler" {
    multiboot2 /boot/microos.elf test=shell trace=0 policy=sjf
    boot
}
menuentry "Micro-OS (Rust kernel) - desktop, Multi-Level Feedback Queue" {
    multiboot2 /boot/microos.elf test=shell trace=0 policy=mlfq
    boot
}
EOF
grub-mkrescue -o "$OUT" "$WORK" -- -volid MICROOS_X86 >/dev/null 2>&1
echo "built $OUT ($(du -h "$OUT" | cut -f1))"
