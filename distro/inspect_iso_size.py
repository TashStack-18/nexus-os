#!/usr/bin/env python3
import subprocess
import json

out = subprocess.check_output(["sfdisk", "-J", "/root/micro-os/distro/out/microos-s6.img"])
start = json.loads(out)["partitiontable"]["partitions"][1]["start"]
loop = subprocess.check_output(["losetup", "-f", "--show", "-r", "-o", str(start * 512), "/root/micro-os/distro/out/microos-s6.img"]).decode().strip()
subprocess.run(["mount", "-o", "ro", loop, "/mnt"])
try:
    print("=== TOP 25 LARGEST PACKAGES IN S6 ===")
    dpkg_out = subprocess.check_output(["chroot", "/mnt", "dpkg-query", "-W", "-f=${Installed-Size}\t${Package}\n"]).decode()
    pkgs = []
    for line in dpkg_out.strip().splitlines():
        parts = line.split("\t")
        if len(parts) == 2:
            pkgs.append((int(parts[0]), parts[1]))
    pkgs.sort(reverse=True)
    for size_kb, name in pkgs[:25]:
        print(f"{size_kb / 1024:.1f} MB : {name}")

    print("\n=== QEMU FILES / PACKAGES IN IMAGE ===")
    for size_kb, name in pkgs:
        if "qemu" in name or "sbi" in name or "kvm" in name:
            print(f"{size_kb / 1024:.1f} MB : {name}")

    print("\n=== TOP DIRECTORIES IN ROOTFS ===")
    res = subprocess.check_output(["du", "-hm", "--max-depth=2", "/mnt/usr", "/mnt/var", "/mnt/opt", "/mnt/snap", "/mnt/lib"], stderr=subprocess.DEVNULL).decode()
    dirs = []
    for l in res.strip().splitlines():
        parts = l.split()
        if len(parts) == 2:
            dirs.append((int(parts[0]), parts[1]))
    dirs.sort(reverse=True)
    for sz, d in dirs[:25]:
        print(f"{sz} MB : {d}")

finally:
    subprocess.run(["umount", "/mnt"])
    subprocess.run(["losetup", "-d", loop])
