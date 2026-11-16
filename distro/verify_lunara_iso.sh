#!/usr/bin/env bash
set -euo pipefail

ISO="/root/micro-os/distro/out/lunara-os-final.iso"
WIN_ISO="/mnt/c/Users/Tashvi/Downloads/lunara-os-final.iso"

echo "=== 1. LUNARA ISO EXISTENCE AND SIZE ==="
ls -lh "$ISO"
ls -lh "$WIN_ISO"

echo "=== 2. CHECKSUM VERIFICATION ==="
sha256sum "$ISO"
sha256sum "$WIN_ISO"

echo "=== 3. ALL PRESERVED ISOS IN DISTRO OUT ==="
ls -lh /root/micro-os/distro/out/*.iso

echo "=== 4. ALL PRESERVED ISOS IN DOWNLOADS ==="
ls -lh /mnt/c/Users/Tashvi/Downloads/*.iso 2>/dev/null || true

echo "=== 5. MOUNT AND INSPECT ISO BOOT CONFIG ==="
MNT_ISO=$(mktemp -d)
mount -o loop,ro "$ISO" "$MNT_ISO"
ls -la "$MNT_ISO"
ls -la "$MNT_ISO/boot"
ls -la "$MNT_ISO/live"
echo "--- GRUB CONFIG ---"
cat "$MNT_ISO/boot/grub/grub.cfg"

echo "=== 6. INSPECT SQUASHFS ROOTFS CONTENTS ==="
MNT_SQ=$(mktemp -d)
mount -t squashfs -o ro "$MNT_ISO/live/filesystem.squashfs" "$MNT_SQ"

echo "--- CORE APPLICATIONS ---"
ls -la "$MNT_SQ/usr/local/bin/lunara-studio"
ls -la "$MNT_SQ/usr/local/bin/nexus-lab"
ls -la "$MNT_SQ/usr/local/bin/lunara-tasks"
ls -la "$MNT_SQ/usr/local/bin/lunara-vitals"
ls -la "$MNT_SQ/usr/local/bin/lunara-display"

echo "--- DESKTOP SHORTCUTS (LEFT COLUMN) ---"
ls -la "$MNT_SQ/home/student/Desktop/"
for f in "$MNT_SQ/home/student/Desktop"/*.desktop; do
    echo ">> $(basename "$f")"
    grep -E "Name=|Exec=|Icon=" "$f"
done

echo "--- CONKY CLOCK CONFIG ---"
ls -la "$MNT_SQ/home/student/.config/conky/"
cat "$MNT_SQ/home/student/.config/conky/conky_clock.conf"

echo "--- AUTOSTART ENTRIES ---"
ls -la "$MNT_SQ/home/student/.config/autostart/"

echo "--- FONTCONFIG SUBPIXEL CONFIG ---"
cat "$MNT_SQ/etc/fonts/local.conf"

echo "--- XRESOURCES & XSETTINGS ---"
cat "$MNT_SQ/home/student/.Xresources"
grep -E "DPI|Antialias|Hinting" "$MNT_SQ/home/student/.config/xfce4/xfconf/xfce-perchannel-xml/xsettings.xml"

echo "--- XFWM4 SETTINGS ---"
grep "show_frame_shadow" "$MNT_SQ/home/student/.config/xfce4/xfconf/xfce-perchannel-xml/xfwm4.xml"

echo "--- LIGHTDM CONFIG AND CSS ---"
cat "$MNT_SQ/etc/lightdm/lightdm-gtk-greeter.conf"
cat "$MNT_SQ/etc/lightdm/lightdm-gtk-greeter.css"

echo "--- ACCOUNTS AND PASSWORDS ---"
grep -E "student|tashvi" "$MNT_SQ/etc/passwd"
grep -E "student|tashvi" "$MNT_SQ/etc/shadow" | cut -d: -f1,2
cat "$MNT_SQ/etc/sudoers.d/microos"

echo "--- WALLPAPERS AND AVATAR ---"
ls -lh "$MNT_SQ/usr/share/backgrounds"
ls -lh "$MNT_SQ/usr/share/pixmaps"

# Cleanup mounts
umount "$MNT_SQ"
rmdir "$MNT_SQ"
umount "$MNT_ISO"
rmdir "$MNT_ISO"

echo "=== LUNARA OS VERIFICATION COMPLETE: ALL CHECKS PASSED ==="
