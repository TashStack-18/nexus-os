#!/usr/bin/env python3
import subprocess, os

iso = "/root/micro-os/distro/out/lunara-os-final.iso"
mnt = "/tmp/mnt_search_cfg"
os.makedirs(mnt, exist_ok=True)
os.system(f"mount -o loop,ro '{iso}' '{mnt}' 2>/dev/null")

print("=== ALL FILES IN ISO ===")
for root, dirs, files in os.walk(mnt):
    for f in files:
        p = os.path.join(root, f)
        rel = os.path.relpath(p, mnt)
        size = os.path.getsize(p)
        print(f"  {rel} ({size} bytes)")
        if f.endswith(".cfg") or f.endswith(".lst") or f.endswith(".conf"):
            print(f"    --> CONFIG FILE: {rel}")
            with open(p, errors="ignore") as fp:
                print("    " + fp.read().replace("\n", "\n    "))

# Also mount efi.img if present
efi_img = os.path.join(mnt, "efi.img")
if os.path.exists(efi_img):
    mnt_efi = "/tmp/mnt_search_efi"
    os.makedirs(mnt_efi, exist_ok=True)
    os.system(f"mount -o loop,ro '{efi_img}' '{mnt_efi}' 2>/dev/null")
    print("\n=== ALL FILES IN EFI.IMG ===")
    for root, dirs, files in os.walk(mnt_efi):
        for f in files:
            p = os.path.join(root, f)
            rel = os.path.relpath(p, mnt_efi)
            size = os.path.getsize(p)
            print(f"  {rel} ({size} bytes)")
            if f.endswith(".cfg") or f.endswith(".lst") or f.endswith(".conf"):
                print(f"    --> CONFIG FILE IN EFI: {rel}")
                with open(p, errors="ignore") as fp:
                    print("    " + fp.read().replace("\n", "\n    "))
    os.system(f"umount '{mnt_efi}' 2>/dev/null")

os.system(f"umount '{mnt}' 2>/dev/null")
