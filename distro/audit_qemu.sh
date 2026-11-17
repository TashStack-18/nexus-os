#!/usr/bin/env bash
echo "=== 1. RUNNING QEMU PROCESSES ==="
ps aux | grep -i qemu | grep -v grep || echo "No running QEMU processes."

echo ""
echo "=== 2. ACTIVE LOOP DEVICES ==="
losetup -a || echo "No loop devices."

echo ""
echo "=== 3. ACTIVE MOUNTS ==="
mount | grep -E "/mnt|loop|iso" || echo "No custom mounts."

echo ""
echo "=== 4. DISK IMAGES (.img, .raw, .qcow2) ==="
find /root/micro-os -type f -name "*.img" -o -name "*.raw" -o -name "*.qcow2" 2>/dev/null | xargs -r ls -lh

echo ""
echo "=== 5. ISO FILES IN PROJECT & DOWNLOADS ==="
ls -lh /root/micro-os/distro/out/*.iso /mnt/c/Users/Tashvi/Downloads/*.iso 2>/dev/null || echo "None"

echo ""
echo "=== 6. KERNEL ELFS & BINS ==="
find /root/micro-os/kernel -type f -name "*.elf" -o -name "*.bin" 2>/dev/null | xargs -r ls -lh
find /root/micro-os/distro -type f -name "*.elf" -o -name "*.bin" 2>/dev/null | xargs -r ls -lh

echo ""
echo "=== 7. LOGS AND TEMPORARY ARTIFACTS ==="
du -sh /root/micro-os/distro/out/logs 2>/dev/null || echo "No logs dir"
du -sh /root/micro-os/distro/build 2>/dev/null || echo "No build dir"
