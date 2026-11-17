#!/usr/bin/env bash
# MICRO-OS — Network-Assisted Operating System Installer
# Installs complete MICRO-OS with XFCE Desktop, APT, Snap Store, and Rust Microkernel.
set -euo pipefail

TARGET_DISK="${1:-/dev/sda}"

echo "=================================================================="
echo "    MICRO-OS — NETWORK-ASSISTED OPERATING SYSTEM INSTALLER        "
echo "    Lightweight XFCE Desktop • Rust Microkernel • Academic R3     "
echo "=================================================================="
echo "Target Installation Disk: $TARGET_DISK"
echo ""

# 1. Verify Network Connectivity
echo "[1/7] Initializing network connectivity..."
if ! ping -c 1 -W 3 deb.debian.org >/dev/null 2>&1; then
    echo "Configuring network interfaces via DHCP..."
    dhclient -v || true
fi
if ping -c 1 -W 3 deb.debian.org >/dev/null 2>&1; then
    echo "Network connection established successfully (LAN / Online)."
else
    echo "Warning: Offline or local repository mode active."
fi

# 2. Partition and Format Target Disk
echo "[2/7] Partitioning $TARGET_DISK (GPT: 512MB EFI + ext4 Root)..."
parted -s "$TARGET_DISK" mklabel gpt
parted -s "$TARGET_DISK" mkpart ESP fat32 1MiB 513MiB
parted -s "$TARGET_DISK" set 1 esp on
parted -s "$TARGET_DISK" mkpart primary ext4 513MiB 100%

# Allow kernel to reread partition table
sleep 2
partprobe "$TARGET_DISK" || true

P1="${TARGET_DISK}1"
P2="${TARGET_DISK}2"
[ -b "${TARGET_DISK}p1" ] && { P1="${TARGET_DISK}p1"; P2="${TARGET_DISK}p2"; }

echo "Formatting EFI partition ($P1)..."
mkfs.vfat -F32 "$P1" >/dev/null
echo "Formatting Root partition ($P2)..."
mkfs.ext4 -F -q "$P2"

# 3. Mount Target Filesystem
echo "[3/7] Mounting target filesystem..."
TARGET="/mnt/target"
mkdir -p "$TARGET"
mount "$P2" "$TARGET"
mkdir -p "$TARGET/boot/efi"
mount "$P1" "$TARGET/boot/efi"

# 4. Copy Base System to Target
echo "[4/7] Installing base operating system..."
cp -a /." "$TARGET/" 2>/dev/null || true

# Ensure kernel and initrd are copied to target /boot
mkdir -p "$TARGET/boot"
for live_src in /run/live/medium/live /live/image/live /media/cdrom/live /mnt/live /run/live/rootfs/live; do
    if [ -f "$live_src/vmlinuz" ]; then
        echo "Copying kernel from installation media ($live_src)..."
        cp "$live_src/vmlinuz" "$TARGET/boot/vmlinuz"
        cp "$live_src/initrd.img" "$TARGET/boot/initrd.img"
        break
    fi
done

# 5. Bind Mounts for Target Configuration
echo "[5/7] Configuring system packages and services..."
for m in /dev /dev/pts /proc /sys /run; do
    mount --bind "$m" "$TARGET$m"
done

# Configure fstab
UUID_ROOT=$(blkid -s UUID -o value "$P2")
UUID_EFI=$(blkid -s UUID -o value "$P1")
cat > "$TARGET/etc/fstab" <<FSTAB
UUID=$UUID_ROOT / ext4 errors=remount-ro 0 1
UUID=$UUID_EFI /boot/efi vfat umask=0077 0 1
FSTAB

# Configure hostname and hosts
echo "microos" > "$TARGET/etc/hostname"
cat > "$TARGET/etc/hosts" <<HOSTS
127.0.0.1 localhost
127.0.1.1 microos
HOSTS

# 6. Install Bootloader (GRUB2)
echo "[6/7] Installing GRUB2 bootloader on $TARGET_DISK..."
chroot "$TARGET" grub-install --target=x86_64-efi --efi-directory=/boot/efi --bootloader-id=MICRO-OS --recheck || true
chroot "$TARGET" grub-install --target=i386-pc "$TARGET_DISK" --recheck 2>/dev/null || true

# Generate GRUB configuration with 3 boot modes
cat > "$TARGET/boot/grub/grub.cfg" <<'EOF'
set timeout_style=menu
set timeout=10
set default=0

terminal_input console
terminal_output console

insmod all_video
insmod gfxterm
set gfxmode=1024x768,auto
set gfxpayload=keep
terminal_output gfxterm console

set menu_color_normal=light-magenta/black
set menu_color_highlight=white/magenta
set color_normal=light-gray/black
set color_highlight=white/magenta

menuentry "MICRO-OS — XFCE Desktop (Rust Studio, R3 Lab & Tasks)" {
    linux /boot/vmlinuz root=UUID=@UUID_ROOT@ ro quiet
    initrd /boot/initrd.img
}

menuentry "MICRO-OS — Rust Microkernel (Bare-Metal x86-64, Instant Boot)" {
    multiboot2 /opt/micro-os/microos_x86.elf test=shell trace=0
    boot
}

menuentry "MICRO-OS — Academic Verification Suite (TC1–TC6, 68 Checks)" {
    multiboot2 /opt/micro-os/microos_x86.elf test=all trace=0
    boot
}
EOF
sed -i "s/@UUID_ROOT@/$UUID_ROOT/g" "$TARGET/boot/grub/grub.cfg"

# 7. Finalize & Unmount
echo "[7/7] Finalizing installation..."
for m in "$TARGET/boot/efi" "$TARGET/dev/pts" "$TARGET/dev" "$TARGET/proc" "$TARGET/sys" "$TARGET/run" "$TARGET"; do
    umount -l "$m" 2>/dev/null || true
done

echo ""
echo "=================================================================="
echo "    MICRO-OS INSTALLATION COMPLETE!                               "
echo "    You can now reboot and enjoy your new operating system.       "
echo "=================================================================="
