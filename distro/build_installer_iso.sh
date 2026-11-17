#!/usr/bin/env bash
# Build the minimal network-assisted installer ISO for MICRO-OS:
# Strictly below 209,715,200 bytes (200 MB).
#
# Combines:
# 1. Minimal live installer environment with LAN/network support (< 150 MB).
# 2. Network-assisted installer script (microos-install).
# 3. Independent x86-64 Rust Micro-OS kernel with all 68 academic tests.
# 4. GRUB2 3-mode bootloader menu.
#
# Output: distro/out/micro-os-final.iso
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(dirname "$HERE")"
IMG="$HERE/out/microos-s1.img"
LUNARA_ISO="$HERE/out/lunara-os-final.iso"
WORK="$HERE/build/installer_iso"
ISO="${ISO:-$HERE/out/micro-os-final.iso}"
ELF_X86="$ROOT/kernel/target/x86_64-unknown-none/release/microos"

log() { echo "[micro-os-installer] $*"; }

[ -f "$IMG" ] || { echo "missing $IMG" >&2; exit 1; }
[ -f "$LUNARA_ISO" ] || { echo "missing $LUNARA_ISO" >&2; exit 1; }

# Build Rust x86-64 microkernel if needed
if [ ! -f "$ELF_X86" ]; then
    log "building Rust Micro-OS kernel (x86-64 release)"
    (cd "$ROOT/kernel" && cargo build --release --target x86_64-unknown-none)
fi

R="$WORK/rootfs"
MNT="$WORK/mnt"
cleanup() {
    for m in "$R/dev/pts" "$R/dev" "$R/proc" "$R/sys" "$R/run" "$MNT" "/tmp/lunaramnt"; do
        mountpoint -q "$m" && umount -l "$m" || true
    done
    [ -n "${LOOP:-}" ] && losetup -d "$LOOP" 2>/dev/null || true
}
trap cleanup EXIT
rm -rf "$WORK"
mkdir -p "$R" "$MNT" "$WORK/iso/live" "$WORK/iso/boot/grub"

log "mounting base system from $(basename "$IMG")"
start=$(sfdisk -J "$IMG" | python3 -c 'import json,sys; print(json.load(sys.stdin)["partitiontable"]["partitions"][1]["start"])')
LOOP=$(losetup -f --show -r -o $((start * 512)) "$IMG")
mount -o ro "$LOOP" "$MNT"
cp -a "$MNT/." "$R/"
umount "$MNT"
losetup -d "$LOOP"
LOOP=""

log "installing installer scripts and utilities"
mkdir -p "$R/usr/local/bin" "$R/usr/local/sbin"
cp "$HERE/installer/microos-install.sh" "$R/usr/local/bin/microos-install"
chmod 0755 "$R/usr/local/bin/microos-install"

# Also include hwcheck and verify tools
cp "$HERE/overlay/usr/local/sbin/microos-hwcheck" "$R/usr/local/bin/microos-hwcheck" 2>/dev/null || true
chmod 0755 "$R/usr/local/bin/microos-hwcheck" 2>/dev/null || true
cp "$HERE/overlay/usr/local/sbin/microos-verify" "$R/usr/local/sbin/microos-verify" 2>/dev/null || true
chmod 0755 "$R/usr/local/sbin/microos-verify" 2>/dev/null || true

# Extract live-boot enabled kernel and initrd from verified working LUNARA ISO
log "extracting live-boot enabled kernel and initrd"
mkdir -p /tmp/lunaramnt
mount -o loop,ro "$LUNARA_ISO" /tmp/lunaramnt
cp "/tmp/lunaramnt/live/vmlinuz" "$WORK/iso/live/vmlinuz"
cp "/tmp/lunaramnt/live/initrd.img" "$WORK/iso/live/initrd.img"
umount /tmp/lunaramnt

log "optimizing live rootfs size (removing duplicate boot files, docs, unused locales & drivers)"
# 1. Remove duplicate boot binaries inside rootfs
rm -rf "$R/boot"/*
# 2. Prune documentation and manuals
rm -rf "$R/usr/share/doc"/* "$R/usr/share/man"/* "$R/usr/share/info"/*
# 3. Prune unused locale files (preserve English)
find "$R/usr/share/locale" -mindepth 1 -maxdepth 1 ! -name 'en*' ! -name 'locale.alias' -exec rm -rf {} + 2>/dev/null || true
# 4. Clean APT cache and temporary files
rm -rf "$R/var/cache/apt/archives"/* "$R/var/lib/apt/lists"/* "$R/tmp"/* "$R/var/tmp"/*
# 5. Clean Python bytecode and cache
find "$R" -name "__pycache__" -exec rm -rf {} + 2>/dev/null || true
find "$R" -name "*.pyc" -delete 2>/dev/null || true
# 6. Prune non-essential enterprise/media/sound kernel modules (keeping LAN, Wi-Fi, Storage, Input, Display)
MOD_DIR="$R/usr/lib/modules/6.12.116+deb13-amd64/kernel"
if [ -d "$MOD_DIR" ]; then
    rm -rf "$MOD_DIR/sound" 2>/dev/null || true
    rm -rf "$MOD_DIR/drivers/media" "$MOD_DIR/drivers/infiniband" "$MOD_DIR/drivers/comedi" \
           "$MOD_DIR/drivers/mtd" "$MOD_DIR/drivers/iio" "$MOD_DIR/drivers/staging" \
           "$MOD_DIR/drivers/isdn" "$MOD_DIR/drivers/firewire" "$MOD_DIR/drivers/bluetooth" 2>/dev/null || true
fi

log "configuring live system environment (fstab, autologin, banner)"
printf '# Live system: root overlay in RAM\n' > "$R/etc/fstab"
: > "$R/etc/machine-id"
mkdir -p "$R/etc/network" "$R/etc/systemd/system/getty@tty1.service.d"

# Configure tty1 autologin for instant access to installer
cat > "$R/etc/systemd/system/getty@tty1.service.d/autologin.conf" <<'AUTOLOGIN'
[Service]
ExecStart=
ExecStart=-/sbin/agetty --autologin root --noclear %I $TERM
AUTOLOGIN

# Setup interactive greeting banner in /root/.bashrc
cat >> "$R/root/.bashrc" <<'GREETING'
echo ""
echo "=================================================================="
echo "    MICRO-OS — NETWORK-ASSISTED OPERATING SYSTEM INSTALLER        "
echo "    Lightweight XFCE Desktop • Rust Microkernel • Academic R3     "
echo "=================================================================="
echo "Commands available:"
echo "  • microos-install  : Install complete desktop system to /dev/sda"
echo "  • microos-hwcheck  : Run hardware compatibility audit"
echo "  • microos-verify   : Run component verification suite"
echo "=================================================================="
echo ""
GREETING

log "compressing root filesystem with high-efficiency XZ"
mksquashfs "$R" "$WORK/iso/live/filesystem.squashfs" -comp xz -b 1048576 -Xbcj x86 -Xdict-size 100% -e boot/efi -noappend -quiet

log "copying bare-metal Rust microkernel to boot directory"
cp "$ELF_X86" "$WORK/iso/boot/microos_x86.elf"

log "generating GRUB2 3-mode boot configuration"
mkdir -p "$WORK/iso/boot/grub" "$WORK/iso/efi/boot"
cat > "$WORK/iso/boot/grub/grub.cfg" <<'EOF'
# MICRO-OS — Bootloader Configuration
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

menuentry "MICRO-OS — Network-Assisted Desktop Installer (Automated & Interactive)" {
    linux /live/vmlinuz boot=live quiet console=tty0
    initrd /live/initrd.img
}

menuentry "MICRO-OS — Rust Microkernel (Bare-Metal x86-64, Instant Boot)" {
    multiboot2 /boot/microos_x86.elf test=shell trace=0
    boot
}

menuentry "MICRO-OS — Academic Verification Suite (TC1–TC6, 68 Checks)" {
    multiboot2 /boot/microos_x86.elf test=all trace=0
    boot
}
EOF
cp "$WORK/iso/boot/grub/grub.cfg" "$WORK/iso/efi/boot/grub.cfg"

log "generating hybrid ISO with grub-mkrescue"
grub-mkrescue -o "$ISO" "$WORK/iso" -- -volid MICRO_OS >/dev/null 2>&1
rm -rf "$R"

ISO_BYTES=$(stat -c%s "$ISO")
ISO_MB=$(du -h "$ISO" | cut -f1)
log "built: $ISO ($ISO_MB, $ISO_BYTES bytes)"
sha256sum "$ISO" | tee "$ISO.sha256"

# Mirror to project root
cp "$ISO" "$ROOT/micro-os-final.iso"
cp "$ISO.sha256" "$ROOT/micro-os-final.iso.sha256"

# Verify size < 200 MB
MAX_BYTES=209715200
if [ "$ISO_BYTES" -lt "$MAX_BYTES" ]; then
    log "SUCCESS: ISO size is $ISO_BYTES bytes (< 209,715,200 bytes / 200 MB limit)."
else
    log "WARNING: ISO size $ISO_BYTES exceeds 200 MB!"
    exit 1
fi
