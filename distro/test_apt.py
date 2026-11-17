#!/usr/bin/env python3
import subprocess, json, os

out = subprocess.check_output(['sfdisk', '-J', '/root/micro-os/distro/out/microos-s6.img'])
start = json.loads(out)['partitiontable']['partitions'][1]['start']
loop = subprocess.check_output(['losetup', '-f', '--show', '-r', '-o', str(start * 512), '/root/micro-os/distro/out/microos-s6.img']).decode().strip()
subprocess.run(['mkdir', '-p', '/tmp/test_chroot'])
subprocess.run(['mount', '-o', 'ro', loop, '/tmp/test_chroot'])

# Create writable overlay to test apt-get in chroot
subprocess.run(['mkdir', '-p', '/tmp/ovl_upper', '/tmp/ovl_work', '/tmp/ovl_merged'])
subprocess.run(['mount', '-t', 'overlay', 'overlay', '-o', 'lowerdir=/tmp/test_chroot,upperdir=/tmp/ovl_upper,workdir=/tmp/ovl_work', '/tmp/ovl_merged'])
subprocess.run(['mount', '--bind', '/dev', '/tmp/ovl_merged/dev'])
subprocess.run(['mount', '-t', 'proc', 'proc', '/tmp/ovl_merged/proc'])
subprocess.run(['mount', '-t', 'sysfs', 'sys', '/tmp/ovl_merged/sys'])
subprocess.run(['cp', '/etc/resolv.conf', '/tmp/ovl_merged/etc/resolv.conf'])

# Test apt-get install WITHOUT virtualbox packages
cmd = [
    'chroot', '/tmp/ovl_merged', 'env', 'DEBIAN_FRONTEND=noninteractive',
    'apt-get', '-o', 'Acquire::http::Proxy=http://127.0.0.1:3142',
    'install', '--dry-run',
    'xfce4-screensaver', 'fonts-dejavu-core', 'papirus-icon-theme', 'librsvg2-common',
    'mupdf', 'python3', 'python3-gi', 'python3-gi-cairo', 'gir1.2-gtk-3.0',
    'firefox-esr', 'conky-all', 'xfce4-settings', 'xfce4-appfinder'
]
res = subprocess.run(cmd, capture_output=True, text=True)
print('INSTALL WITHOUT VBOX EXIT CODE:', res.returncode)
print('INSTALL STDOUT (first 5 lines):', '\n'.join(res.stdout.splitlines()[:5]))
print('INSTALL STDERR:', res.stderr)

subprocess.run(['umount', '-l', '/tmp/ovl_merged'])
subprocess.run(['umount', '/tmp/test_chroot'])
subprocess.run(['losetup', '-d', loop])
