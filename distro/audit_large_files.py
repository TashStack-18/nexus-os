#!/usr/bin/env python3
import os
import glob

def get_files():
    search_paths = ["/root/micro-os", "/mnt/c/Users/Tashvi/Downloads"]
    results = []
    for sp in search_paths:
        if not os.path.exists(sp):
            continue
        for root, dirs, files in os.walk(sp):
            # skip .git objects if any to avoid noise
            if "/.git/" in root:
                continue
            for f in files:
                p = os.path.join(root, f)
                try:
                    sz = os.path.getsize(p)
                    if sz >= 5 * 1024 * 1024: # 5MB+
                        results.append((sz, p))
                except OSError:
                    pass
    results.sort(reverse=True)
    return results

files = get_files()
print(f"Found {len(files)} files >= 5MB:\n")
print(f"{'Size (MB)':>10}  {'Category':<15}  {'Path'}")
print("-" * 90)

for sz, p in files[:45]:
    mb = sz / (1024 * 1024)
    cat = "Other"
    ext = os.path.splitext(p)[1].lower()
    base = os.path.basename(p).lower()
    
    if ext == ".iso":
        cat = "ISO Image"
    elif ext in [".img", ".raw", ".qcow2"]:
        cat = "Disk Image"
    elif "squashfs" in base:
        cat = "SquashFS"
    elif "target" in p or "cargo" in p:
        cat = "Cargo Build"
    elif ext in [".elf", ".bin"] or "vmlinuz" in base:
        cat = "Kernel/Binary"
    elif "initrd" in base or "initramfs" in base:
        cat = "Initrd"
    elif "snap" in p:
        cat = "Snap Package"
    elif "var/cache" in p:
        cat = "APT/OS Cache"
    elif "firmware" in p:
        cat = "Firmware"
        
    print(f"{mb:>9.1f} MB  {cat:<15}  {p}")
