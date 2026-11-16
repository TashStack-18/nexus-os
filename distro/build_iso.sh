#!/usr/bin/env bash
# Build a bootable live ISO of Micro-OS Linux (stage S6) that also carries
# the Rust Micro-OS kernel.
#
#   distro/build_iso.sh            -> distro/out/micro-os-live.iso
#
# The ISO boots (BIOS or UEFI, from a USB stick or a VM) into the full
# Micro-OS Linux desktop: LAN + Wi-Fi, laptop firmware, APT/aptitude and the
# Snap Store. Inside it, `micro-os-kernel` starts the Rust RISC-V kernel in
# QEMU, and /opt/micro-os/src holds its source code.
#
# Steps: copy S6's root file system from the tested disk image, add live-boot
# and qemu-system-riscv64 to it, compress it into a squashfs, and wrap that
# with the kernel, initramfs and GRUB in a hybrid ISO (grub-mkrescue/xorriso).
# Note: this GRUB is not Secure Boot signed; disable Secure Boot to boot the
# ISO on a PC (the .img images keep the signed shim chain).
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(dirname "$HERE")"
IMG="$HERE/out/microos-s6.img"
WORK="$HERE/build/iso"
ISO="$HERE/out/micro-os-live.iso"
ELF="$ROOT/kernel/target/riscv64gc-unknown-none-elf/release/microos"
PROXY="http://127.0.0.1:3142"   # apt-cacher-ng, as for the stage builds
log() { echo "[iso] $*"; }

[ -f "$IMG" ] || { echo "missing $IMG (run build.sh 6)" >&2; exit 1; }
[ -f "$ELF" ] || { echo "missing kernel ELF (cargo build --release in kernel/)" >&2; exit 1; }
for t in mksquashfs grub-mkrescue xorriso mformat; do
    command -v "$t" >/dev/null || { echo "missing tool $t (apt install squashfs-tools grub-pc-bin grub-efi-amd64-bin xorriso mtools)" >&2; exit 1; }
done

R="$WORK/rootfs"; MNT="$WORK/mnt"
cleanup() {
    for m in "$R/dev/pts" "$R/dev" "$R/proc" "$R/sys" "$R/run" "$MNT"; do mountpoint -q "$m" && umount -l "$m" || true; done
    [ -n "${LOOP:-}" ] && losetup -d "$LOOP" 2>/dev/null || true
}
trap cleanup EXIT
rm -rf "$WORK"; mkdir -p "$R" "$MNT" "$WORK/iso/live" "$WORK/iso/boot/grub"

log "copying the S6 root file system from $(basename "$IMG")"
start=$(sfdisk -J "$IMG" | python3 -c 'import json,sys; print(json.load(sys.stdin)["partitiontable"]["partitions"][1]["start"])')
LOOP=$(losetup -f --show -r -o $((start * 512)) "$IMG")
mount -o ro "$LOOP" "$MNT"
cp -a "$MNT/." "$R/"
umount "$MNT"; losetup -d "$LOOP"; LOOP=""

log "adding live-boot and QEMU (RISC-V) inside the system"
mount --bind /dev "$R/dev"; mount --bind /dev/pts "$R/dev/pts"
mount -t proc proc "$R/proc"; mount -t sysfs sys "$R/sys"; mount -t tmpfs tmpfs "$R/run"
cp /etc/resolv.conf "$R/etc/resolv.conf" 2>/dev/null || true
chroot "$R" env DEBIAN_FRONTEND=noninteractive apt-get -o Acquire::http::Proxy="$PROXY" -qq update
chroot "$R" env DEBIAN_FRONTEND=noninteractive apt-get -o Acquire::http::Proxy="$PROXY" -qq -y \
    --no-install-recommends install live-boot live-boot-initramfs-tools qemu-system-misc opensbi >/dev/null
chroot "$R" apt-get -qq clean

log "adding the Rust Micro-OS kernel and its source"
mkdir -p "$R/opt/micro-os/src"
install -m 0644 "$ELF" "$R/opt/micro-os/microos.elf"
git -C "$ROOT" archive HEAD -- . ':!results/screenshots' ':!traces' | tar -x -C "$R/opt/micro-os/src"
cat > "$R/usr/local/bin/micro-os-kernel" <<'EOF'
#!/bin/sh
# Boot the Rust Micro-OS kernel (RISC-V) in QEMU on this terminal.
#   micro-os-kernel            interactive kernel shell (try: help, ps, user priv)
#   micro-os-kernel all        run the full in-kernel test suite
#   micro-os-kernel tc2 policy=mlfq
# Leave QEMU with Ctrl-A, then X.
test="${1:-shell}"; [ $# -gt 0 ] && shift
echo "Starting the Micro-OS kernel (test=$test). Leave with Ctrl-A, then X."
exec qemu-system-riscv64 -machine virt -cpu rv64 -smp 1 -m 128M -bios default \
    -nographic -kernel /opt/micro-os/microos.elf -append "test=$test $*"
EOF
chmod 0755 "$R/usr/local/bin/micro-os-kernel"

log "applying the NEXUS theme (login screen, panel, dock, wallpaper)"
bash "$HERE/theme.sh" "$R"

log "turning it into a live system"
# The live root is an overlay in RAM: no partitions to mount.
printf '# Live system: the root is set up by live-boot.\n' > "$R/etc/fstab"
: > "$R/etc/machine-id"
rm -f "$R/etc/resolv.conf"
chroot "$R" update-initramfs -u -k all >/dev/null 2>&1
for m in "$R/dev/pts" "$R/dev" "$R/proc" "$R/sys" "$R/run"; do umount -l "$m"; done

KVER=$(ls "$R/boot" | sed -n 's/^vmlinuz-//p' | sort -V | tail -1)
cp "$R/boot/vmlinuz-$KVER" "$WORK/iso/live/vmlinuz"
cp "$R/boot/initrd.img-$KVER" "$WORK/iso/live/initrd.img"

log "compressing the root file system (squashfs, xz)"
mksquashfs "$R" "$WORK/iso/live/filesystem.squashfs" -comp xz -e boot/efi -noappend -quiet

cat > "$WORK/iso/boot/grub/grub.cfg" <<'EOF'
set timeout=3
set default=0
menuentry "Micro-OS Linux - NEXUS desktop (live)" {
    linux /live/vmlinuz boot=live quiet console=ttyS0,115200 console=tty0
    initrd /live/initrd.img
}
menuentry "Micro-OS Linux (live, text messages)" {
    linux /live/vmlinuz boot=live console=ttyS0,115200 console=tty0
    initrd /live/initrd.img
}
EOF

log "writing the hybrid ISO (BIOS + UEFI)"
grub-mkrescue -o "$ISO" "$WORK/iso" -- -volid MICROOS_LIVE >/dev/null 2>&1
rm -rf "$R"
log "done: $ISO ($(du -h "$ISO" | cut -f1))"
sha256sum "$ISO" | tee "$ISO.sha256"
