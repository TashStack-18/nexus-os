#!/usr/bin/env bash
set -e
ISO="/root/micro-os/distro/out/lunara-os-final.iso"
echo "Testing ISO boot in QEMU: $ISO"
timeout 8s qemu-system-x86_64 \
    -cdrom "$ISO" \
    -m 1024 \
    -display none \
    -serial stdio \
    -no-reboot || true
