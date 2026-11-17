#!/usr/bin/env bash
set -e
ISO="/root/micro-os/distro/out/lunara-os-final.iso"
MNT=$(mktemp -d)
mount -o loop,ro "$ISO" "$MNT"
MNT_EFI=$(mktemp -d)
mount -o loop,ro "$MNT/efi.img" "$MNT_EFI"
echo "=== efi.img Contents ==="
find "$MNT_EFI"
echo "=== Check embedded grub.cfg in efi.img ==="
find "$MNT_EFI" -name "*.cfg" -exec cat {} + 2>/dev/null || echo "No cfg in efi.img"
umount "$MNT_EFI"
rmdir "$MNT_EFI"
umount "$MNT"
rmdir "$MNT"
