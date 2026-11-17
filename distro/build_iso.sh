#!/usr/bin/env bash
# Build the unified bootable ISO of NEXUS OS:
# Combines:
# 1. The full NEXUS Linux live desktop (Rust Studio, NEXUS Lab, Task Manager,
#    networking, real hardware firmware, package management).
# 2. The bare-metal x86-64 Rust Micro-OS kernel (instant boot, 0.2s, direct framebuffer,
#    real ready queue, PIT timer, all 68 in-kernel checks passing).
#
# Output: distro/out/nexus-os.iso  (also mirrored to Downloads/nexus-os.iso)
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(dirname "$HERE")"
IMG="$HERE/out/microos-s6.img"
WORK="$HERE/build/iso"
ISO="${ISO:-$HERE/out/lunara-os-final.iso}"
ELF_RISCV="$ROOT/kernel/target/riscv64gc-unknown-none-elf/release/microos"
ELF_X86="$ROOT/kernel/target/x86_64-unknown-none/release/microos"
PROXY="http://127.0.0.1:3142"   # apt-cacher-ng
log() { echo "[lunara-iso] $*"; }

[ -f "$IMG" ] || { echo "missing $IMG (run build.sh 6)" >&2; exit 1; }

[ -d "$HOME/.cargo/bin" ] && export PATH="$HOME/.cargo/bin:$PATH"
[ -d "/root/.cargo/bin" ] && export PATH="/root/.cargo/bin:$PATH"

log "building Rust Micro-OS kernels (x86-64 and RISC-V release)"
(cd "$ROOT/kernel" && cargo build --release --target x86_64-unknown-none)
(cd "$ROOT/kernel" && cargo build --release --target riscv64gc-unknown-none-elf)

[ -f "$ELF_RISCV" ] || { echo "missing RISC-V kernel ELF" >&2; exit 1; }
[ -f "$ELF_X86" ] || { echo "missing x86-64 kernel ELF" >&2; exit 1; }

for t in mksquashfs grub-mkrescue xorriso mformat; do
    command -v "$t" >/dev/null || { echo "missing tool $t" >&2; exit 1; }
done

R="$WORK/rootfs"; MNT="$WORK/mnt"
cleanup() {
    for m in "$R/dev/pts" "$R/dev" "$R/proc" "$R/sys" "$R/run" "$MNT"; do mountpoint -q "$m" && umount -l "$m" || true; done
    [ -n "${LOOP:-}" ] && losetup -d "$LOOP" 2>/dev/null || true
}
trap cleanup EXIT
rm -rf "$WORK"; mkdir -p "$R" "$MNT" "$WORK/iso/live" "$WORK/iso/boot/grub"

log "copying S6 root file system from $(basename "$IMG")"
start=$(sfdisk -J "$IMG" | python3 -c 'import json,sys; print(json.load(sys.stdin)["partitiontable"]["partitions"][1]["start"])')
LOOP=$(losetup -f --show -r -o $((start * 512)) "$IMG")
mount -o ro "$LOOP" "$MNT"
cp -a "$MNT/." "$R/"
umount "$MNT"; losetup -d "$LOOP"; LOOP=""

log "configuring live-boot and QEMU inside rootfs"
mount --bind /dev "$R/dev"; mount --bind /dev/pts "$R/dev/pts"
mount -t proc proc "$R/proc"; mount -t sysfs sys "$R/sys"; mount -t tmpfs tmpfs "$R/run"
cp /etc/resolv.conf "$R/etc/resolv.conf" 2>/dev/null || true

chroot "$R" env DEBIAN_FRONTEND=noninteractive apt-get -o Acquire::http::Proxy="$PROXY" -qq update || true
chroot "$R" env DEBIAN_FRONTEND=noninteractive apt-get -o Acquire::http::Proxy="$PROXY" -qq -y \
    --no-install-recommends install live-boot live-boot-initramfs-tools qemu-system-misc opensbi cargo rustc >/dev/null 2>&1 || true
chroot "$R" apt-get -qq clean || true

log "adding the Rust Micro-OS kernels and source to /opt/micro-os"
mkdir -p "$R/opt/micro-os/src"
install -m 0644 "$ELF_RISCV" "$R/opt/micro-os/microos.elf"
install -m 0644 "$ELF_X86" "$R/opt/micro-os/microos_x86.elf"
git -C "$ROOT" archive HEAD -- . ':!results/screenshots' ':!traces' | tar -x -C "$R/opt/micro-os/src" 2>/dev/null || true

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

log "applying NEXUS theme, Studio, Lab, and Task Manager"
bash "$HERE/theme.sh" "$R"

log "pre-populating example Rust projects for student"
H="$R/home/student"
mkdir -p "$H/nexus_rust_projects"
python3 -c "
import os
sys_path = '$HERE/theme/nexus-studio'
import importlib.util
spec = importlib.util.spec_from_file_location('studio', sys_path)
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)
for k, v in mod.EXAMPLES.items():
    p_dir = os.path.join('$H/nexus_rust_projects', k)
    os.makedirs(os.path.join(p_dir, 'src'), exist_ok=True)
    with open(os.path.join(p_dir, 'Cargo.toml'), 'w') as f: f.write(v['toml'])
    with open(os.path.join(p_dir, 'src', 'main.rs'), 'w') as f: f.write(v['main'])
" 2>/dev/null || true
chroot "$R" chown -R student:student /home/student/nexus_rust_projects 2>/dev/null || true
if [ -d "$R/home/tashvi" ]; then
    mkdir -p "$R/home/tashvi/nexus_rust_projects"
    cp -r "$H/nexus_rust_projects/." "$R/home/tashvi/nexus_rust_projects/"
    chroot "$R" chown -R tashvi:tashvi /home/tashvi/nexus_rust_projects 2>/dev/null || true
fi

log "finalizing live system overlay configuration"
printf '# Live system: root overlay in RAM\n' > "$R/etc/fstab"
: > "$R/etc/machine-id"
rm -f "$R/etc/resolv.conf"
chroot "$R" update-initramfs -u -k all >/dev/null 2>&1 || true
for m in "$R/dev/pts" "$R/dev" "$R/proc" "$R/sys" "$R/run"; do umount -l "$m"; done

KVER=$(ls "$R/boot" | sed -n 's/^vmlinuz-//p' | sort -V | tail -1)
cp "$R/boot/vmlinuz-$KVER" "$WORK/iso/live/vmlinuz"
cp "$R/boot/initrd.img-$KVER" "$WORK/iso/live/initrd.img"

log "copying bare-metal x86-64 Rust Micro-OS kernel to ISO boot"
cp "$ELF_X86" "$WORK/iso/boot/microos_x86.elf"

log "ensuring GRUB unicode font is present for graphical bootloader menu"
mkdir -p "$WORK/iso/boot/grub/fonts"
if [ -f /usr/share/grub/unicode.pf2 ]; then
    cp /usr/share/grub/unicode.pf2 "$WORK/iso/boot/grub/fonts/unicode.pf2"
fi

log "compressing root file system (squashfs, xz)"
mksquashfs "$R" "$WORK/iso/live/filesystem.squashfs" -comp xz -e boot/efi -noappend -quiet

log "generating Multiboot2 / GRUB configuration for 3-mode boot menu"
cat > "$WORK/iso/boot/grub/grub.cfg" <<'EOF'
# LUNARA OS — 3-Mode Boot Menu Configuration
set timeout_style=menu
set timeout=10
set default=0

# Load Unicode font to ensure graphical menu renders glyphs cleanly
if [ -s /boot/grub/fonts/unicode.pf2 ]; then
    loadfont /boot/grub/fonts/unicode.pf2
fi

insmod all_video
insmod gfxterm
set gfxmode=1920x1080x32,1600x900x32,1366x768x32,1280x720x32,1024x768x32,auto
set gfxpayload=keep

# Dual terminal output guarantees visibility across both VESA/GOP graphics and BIOS text console
terminal_input console
terminal_output gfxterm console

# LUNARA Plum/Pink Color Scheme
set menu_color_normal=light-magenta/black
set menu_color_highlight=white/magenta
set color_normal=light-gray/black
set color_highlight=white/magenta

menuentry "LUNARA OS — Live Desktop (Rust Studio, Lab & Tasks)" {
    linux /live/vmlinuz boot=live quiet video=1920x1080 console=ttyS0,115200 console=tty0
    initrd /live/initrd.img
}

menuentry "LUNARA Micro-OS — Rust Kernel (Bare-Metal x86-64, Instant Boot)" {
    multiboot2 /boot/microos_x86.elf test=shell trace=0
    boot
}

menuentry "LUNARA Micro-OS — Academic Verification Suite (TC1–TC6, 68 Checks)" {
    multiboot2 /boot/microos_x86.elf test=all trace=0
    boot
}
EOF

log "writing hybrid bootable ISO (BIOS + UEFI)"
grub-mkrescue -o "$ISO" "$WORK/iso" -- -volid LUNARA_OS >/dev/null 2>&1
rm -rf "$R"

log "done: $ISO ($(du -h "$ISO" | cut -f1))"
sha256sum "$ISO" | tee "$ISO.sha256"

# Mirror to Windows Downloads folder for convenient VirtualBox mounting
if [ -d "/mnt/c/Users/Tashvi/Downloads" ]; then
    log "mirroring to /mnt/c/Users/Tashvi/Downloads/$(basename "$ISO")"
    cp "$ISO" "/mnt/c/Users/Tashvi/Downloads/$(basename "$ISO")"
    cp "$ISO.sha256" "/mnt/c/Users/Tashvi/Downloads/$(basename "$ISO").sha256"
fi
