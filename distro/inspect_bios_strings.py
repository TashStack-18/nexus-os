#!/usr/bin/env python3
import subprocess

iso = "/root/micro-os/distro/out/lunara-os-final.iso"
with open(iso, "rb") as f:
    # First 1 MB of ISO contains MBR and GRUB core.img
    data = f.read(1024 * 1024)

# Find strings
s = ""
for b in data:
    if 32 <= b <= 126:
        s += chr(b)
    else:
        if len(s) > 8 and any(k in s.lower() for k in ["prefix", "search", "uuid", "disk", "grub"]):
            print("  BIOS core string:", s)
        s = ""
