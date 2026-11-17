#!/usr/bin/env bash
set -e
ISO="/root/micro-os/distro/out/lunara-os-final.iso"
MNT=$(mktemp -d)
mount -o loop,ro "$ISO" "$MNT"
ls -la "$MNT/boot/grub/fonts" 2>/dev/null || echo "No fonts in /boot/grub"
umount "$MNT"
rmdir "$MNT"
