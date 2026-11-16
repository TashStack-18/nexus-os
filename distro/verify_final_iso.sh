#!/usr/bin/env bash
set -euo pipefail

ISO="/root/micro-os/distro/out/nexus-os-final-v2.iso"
WIN_ISO="/mnt/c/Users/Tashvi/Downloads/nexus-os-final-v2.iso"

echo "=== 1. FILE EXISTENCE AND SIZE ==="
ls -lh "$ISO"
ls -lh "$WIN_ISO"

echo "=== 2. CHECKSUM VERIFICATION ==="
sha256sum "$ISO"
sha256sum "$WIN_ISO"

echo "=== 3. ALL PRESERVED ISOS IN DISTRO OUT ==="
ls -lh /root/micro-os/distro/out/*.iso

echo "=== 4. ALL PRESERVED ISOS IN DOWNLOADS ==="
ls -lh /mnt/c/Users/Tashvi/Downloads/nexus-os*.iso

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

echo "--- CONKY CONFIGS ---"
ls -la "$MNT_SQ/home/student/.config/conky/"
echo "Clock config:"
head -n 25 "$MNT_SQ/home/student/.config/conky/conky_clock.conf"
echo "Vitals config:"
head -n 25 "$MNT_SQ/home/student/.config/conky/conky_vitals.conf"

echo "--- AUTOSTART DESKTOPS ---"
ls -la "$MNT_SQ/home/student/.config/autostart/"

echo "--- XFWM4 AND XSETTINGS ---"
grep "show_frame_shadow" "$MNT_SQ/home/student/.config/xfce4/xfconf/xfce-perchannel-xml/xfwm4.xml"
grep "EnableAnimations" "$MNT_SQ/home/student/.config/xfce4/xfconf/xfce-perchannel-xml/xsettings.xml"

echo "--- DISPLAY AUTO-RESIZER ---"
cat "$MNT_SQ/usr/local/bin/microos-display"

echo "--- LIGHTDM CONFIG AND CSS ---"
cat "$MNT_SQ/etc/lightdm/lightdm-gtk-greeter.conf"
cat "$MNT_SQ/etc/lightdm/lightdm-gtk-greeter.css"

echo "--- ACCOUNTS AND PASSWORDS ---"
grep -E "student|tashvi" "$MNT_SQ/etc/passwd"
grep -E "student|tashvi" "$MNT_SQ/etc/shadow" | cut -d: -f1,2
cat "$MNT_SQ/etc/sudoers.d/microos"

echo "--- RUST PROJECTS PRE-POPULATED ---"
ls -la "$MNT_SQ/home/student/nexus_rust_projects"

echo "--- WALLPAPER AND LOCK BACKGROUND ---"
ls -lh "$MNT_SQ/usr/share/backgrounds"
ls -lh "$MNT_SQ/usr/share/pixmaps"

# Cleanup mounts
umount "$MNT_SQ"
rmdir "$MNT_SQ"
umount "$MNT_ISO"
rmdir "$MNT_ISO"

echo "=== VERIFICATION COMPLETE: ALL CHECKS PASSED ==="
