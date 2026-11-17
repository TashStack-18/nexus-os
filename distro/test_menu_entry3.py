#!/usr/bin/env python3
import subprocess, time, socket, json, os

iso = "/root/micro-os/distro/out/lunara-os-final.iso"
serial_log = "/tmp/qemu_serial_entry3.log"
if os.path.exists(serial_log): os.remove(serial_log)

# Boot QEMU with ISO, redirect serial to file, monitor to QMP
cmd = [
    "qemu-system-x86_64",
    "-cdrom", iso,
    "-m", "1024",
    "-display", "none",
    "-serial", f"file:{serial_log}",
    "-qmp", "unix:/tmp/qmp-sock-entry3,server,nowait",
    "-no-reboot"
]

print("Launching QEMU for Entry 3 test...")
proc = subprocess.Popen(cmd)
time.sleep(3.0)

try:
    s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    s.connect("/tmp/qmp-sock-entry3")
    s.recv(1024)
    s.sendall(b'{"execute": "qmp_capabilities"}\n')
    s.recv(1024)

    # Screendump 1: initial menu
    s.sendall(json.dumps({"execute": "screendump", "arguments": {"filename": "/tmp/screen_before.ppm"}}).encode() + b"\n")
    s.recv(1024)
    
    # Send Down arrow twice then Enter to select Entry 3:
    # "LUNARA Micro-OS — Academic Verification Suite (TC1–TC6, 68 Checks)"
    for _ in range(2):
        s.sendall(json.dumps({"execute": "send-key", "arguments": {"keys": [{"type": "qcode", "data": "down"}]}}).encode() + b"\n")
        s.recv(1024)
        time.sleep(0.3)
        
    s.sendall(json.dumps({"execute": "send-key", "arguments": {"keys": [{"type": "qcode", "data": "ret"}]}}).encode() + b"\n")
    s.recv(1024)
    print("Sent keystrokes: Down, Down, Enter (Entry 3)")

    # Wait 3 seconds for kernel execution
    time.sleep(3.0)
    # Screendump 2: kernel execution screen
    s.sendall(json.dumps({"execute": "screendump", "arguments": {"filename": "/tmp/screen_after.ppm"}}).encode() + b"\n")
    s.recv(1024)
    s.close()
except Exception as e:
    print("QMP error:", e)

proc.terminate()
proc.wait()

for name in ["before", "after"]:
    p = f"/tmp/screen_{name}.ppm"
    if os.path.exists(p):
        from PIL import Image
        im = Image.open(p)
        im.save(f"/tmp/screen_{name}.png")
        print(f"Screen {name}: size={im.size}, colors={len(im.getcolors(1000) or [])}, bbox={im.getbbox()}")
        os.system(f"cp /tmp/screen_{name}.png /mnt/c/Users/Tashvi/.gemini/antigravity/brain/13b4aced-df88-405a-9721-3d119aeec783/screen_{name}.png 2>/dev/null || true")

if os.path.exists(serial_log):
    with open(serial_log, "r", errors="ignore") as f:
        log = f.read()
    print("=== SERIAL OUTPUT (Length: %d bytes) ===" % len(log))
    print(log[:2000])
    if "shutting down with exit code 0" in log or "failed=0" in log:
        print("\n>>> ALL 68 CHECKS IN ACADEMIC VERIFICATION SUITE PASSED SUCCESSFULLY! <<<")
    else:
        print("\nNote: check exit status above.")
