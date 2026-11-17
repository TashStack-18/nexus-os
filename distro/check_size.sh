#!/bin/bash
set -e
start=1050624
LOOP=$(losetup -f --show -r -o $((start * 512)) /root/micro-os/distro/out/microos-s1.img)
mkdir -p /tmp/s1mnt
mount -o ro "$LOOP" /tmp/s1mnt
echo "=== drivers ==="
du -hd1 /tmp/s1mnt/usr/lib/modules/6.12.116+deb13-amd64/kernel/drivers | sort -hr | head -20
umount /tmp/s1mnt
losetup -d "$LOOP"
