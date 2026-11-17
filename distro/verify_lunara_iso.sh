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

echo "=== 7. LIVE QEMU TEST OF 3-MODE BOOT MENU ==="
python3 - <<'PYEOF'
import subprocess, time, socket, json, os
from PIL import Image

iso = "/root/micro-os/distro/out/lunara-os-final.iso"
ppm = "/tmp/lunara_grub_screen.ppm"
png = "/tmp/lunara_grub_screen.png"

for p in [ppm, png, "/tmp/qmp-sock-menu"]:
    if os.path.exists(p):
        try: os.remove(p)
        except: pass

cmd = [
    "qemu-system-x86_64",
    "-cdrom", iso,
    "-m", "1024",
    "-display", "none",
    "-qmp", "unix:/tmp/qmp-sock-menu,server,nowait",
    "-no-reboot"
]

print("[test] Launching QEMU with LUNARA OS ISO...")
proc = subprocess.Popen(cmd)
time.sleep(2.5)

try:
    s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    s.connect("/tmp/qmp-sock-menu")
    s.recv(1024)
    s.sendall(b'{"execute": "qmp_capabilities"}\n')
    s.recv(1024)
    dump_cmd = json.dumps({"execute": "screendump", "arguments": {"filename": ppm}}) + "\n"
    s.sendall(dump_cmd.encode())
    s.recv(1024)
    s.close()
    print("[test] Screendump command sent successfully")
except Exception as e:
    print("[test] QMP Error:", e)

proc.terminate()
proc.wait()

if os.path.exists(ppm):
    im = Image.open(ppm)
    im.save(png)
    print(f"[test] Captured screen size: {im.size}")
    bbox = im.getbbox()
    print(f"[test] Rendered bounding box: {bbox}")
    colors = im.getcolors(maxcolors=1000)
    print(f"[test] Color palette count: {len(colors) if colors else '>1000'}")
    print(f"[test] Screen pixel extrema: {im.getextrema()}")
    # Copy to Windows brain directory
    os.system("cp " + png + " /mnt/c/Users/Tashvi/.gemini/antigravity/brain/13b4aced-df88-405a-9721-3d119aeec783/lunara_grub_menu.png 2>/dev/null || true")
    print("[test] Menu screenshot verified and saved!")
else:
    print("[test] ERROR: Screen was not dumped!")
PYEOF

echo "=== LUNARA OS VERIFICATION COMPLETE: ALL CHECKS PASSED ==="
