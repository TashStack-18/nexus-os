#!/usr/bin/env bash
set -e
ELF="/root/micro-os/kernel/target/x86_64-unknown-none/release/microos"
echo "Running x86-64 Rust Microkernel in QEMU (test=all)..."
timeout 10s qemu-system-x86_64 \
    -kernel "$ELF" \
    -append "test=all" \
    -display none \
    -serial stdio \
    -no-reboot || true

