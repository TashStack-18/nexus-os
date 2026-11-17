#!/usr/bin/env python3
import os
import subprocess

iso = "/root/micro-os/distro/out/lunara-os-final.iso"
mnt = "/tmp/mnt_efi_inspect"
os.makedirs(mnt, exist_ok=True)
os.system(f"mount -o loop,ro '{iso}' '{mnt}' 2>/dev/null")
efi_img = os.path.join(mnt, "efi.img")
mnt2 = "/tmp/mnt_efi_fat"
os.makedirs(mnt2, exist_ok=True)
os.system(f"mount -o loop,ro '{efi_img}' '{mnt2}' 2>/dev/null")

efi_bin = os.path.join(mnt2, "efi/boot/bootx64.efi")
if os.path.exists(efi_bin):
    print("Found bootx64.efi, size:", os.path.getsize(efi_bin))
    out = subprocess.check_output(["strings", efi_bin]).decode("latin1")
    for line in out.splitlines():
        l = line.lower()
        if any(k in l for k in ["prefix", "timeout", "grub.cfg", "configfile", "search"]):
            if len(line) < 120:
                print("  EFI string:", line)

os.system(f"umount '{mnt2}' 2>/dev/null")
os.system(f"umount '{mnt}' 2>/dev/null")
