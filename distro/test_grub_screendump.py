#!/usr/bin/env python3
import subprocess
import time
import os

ISO = "/root/micro-os/distro/out/lunara-os-final.iso"
ppm_path = "/tmp/qemu_grub_screen.ppm"
png_path = "/tmp/qemu_grub_screen.png"

# Start QEMU with monitor on a unix socket or stdio
cmd = [
    "qemu-system-x86_64",
    "-cdrom", ISO,
    "-m", "1024",
    "-display", "none",
    "-qmp", "unix:/tmp/qmp-sock,server,nowait",
    "-no-reboot"
]

print("Launching QEMU to capture GRUB screen...")
proc = subprocess.Popen(cmd)
time.sleep(2.0)

# Connect to QMP and issue screendump
import socket, json

try:
    s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    s.connect("/tmp/qmp-sock")
    data = s.recv(1024)
    s.sendall(b'{"execute": "qmp_capabilities"}\n')
    data = s.recv(1024)
    dump_cmd = json.dumps({"execute": "screendump", "arguments": {"filename": ppm_path}}) + "\n"
    s.sendall(dump_cmd.encode())
    data = s.recv(1024)
    print("Screendump response:", data.decode())
    s.close()
except Exception as e:
    print("QMP Error:", e)

proc.terminate()
proc.wait()

if os.path.exists(ppm_path):
    print("Screendump created successfully:", os.path.getsize(ppm_path), "bytes")
    # Convert to png using PIL if available
    try:
        from PIL import Image
        im = Image.open(ppm_path)
        im.save(png_path)
        print("Saved PNG to", png_path, "size:", im.size)
        colors = im.getcolors(maxcolors=1000)
        print("Unique colors count:", len(colors) if colors else ">1000")
        print("Extrema:", im.getextrema())
        bbox = im.getbbox()
        print("Non-zero bounding box:", bbox)
    except Exception as e:
        print("PIL conversion error:", e)
else:
    print("Screendump was not created.")
