#!/usr/bin/env python3
import os

isos = [
    "/root/micro-os/distro/out/nexus-os-backup.iso",
    "/root/micro-os/distro/out/nexus-os.iso",
    "/root/micro-os/distro/out/nexus-os-v2.iso",
    "/mnt/c/Users/Tashvi/Downloads/nexus-os-backup.iso",
    "/mnt/c/Users/Tashvi/Downloads/micro-os-live.iso"
]

for iso in isos:
    if os.path.exists(iso):
        mnt = "/tmp/mnt_iso_test"
        os.makedirs(mnt, exist_ok=True)
        os.system(f"mount -o loop,ro '{iso}' '{mnt}' 2>/dev/null")
        print(f"================ {iso} ================")
        os.system(f"ls -la '{mnt}'")
        if os.path.exists(f"{mnt}/boot/grub/grub.cfg"):
            print("--- GRUB.CFG ---")
            with open(f"{mnt}/boot/grub/grub.cfg") as f:
                print(f.read())
        if os.path.exists(f"{mnt}/isolinux"):
            print("--- ISOLINUX ---")
            os.system(f"ls -la '{mnt}/isolinux'")
        os.system(f"umount '{mnt}' 2>/dev/null")
