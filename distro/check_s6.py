#!/usr/bin/env python3
import subprocess, json, os

out = subprocess.check_output(['sfdisk', '-J', '/root/micro-os/distro/out/microos-s6.img'])
start = json.loads(out)['partitiontable']['partitions'][1]['start']
loop = subprocess.check_output(['losetup', '-f', '--show', '-r', '-o', str(start * 512), '/root/micro-os/distro/out/microos-s6.img']).decode().strip()
subprocess.run(['mkdir', '-p', '/tmp/chk_mnt'])
subprocess.run(['mount', '-o', 'ro', loop, '/tmp/chk_mnt'])
try:
    find_out = subprocess.check_output(['find', '/tmp/chk_mnt']).decode()
    print('Total files in s6 img:', len(find_out.splitlines()))
    if os.path.exists('/tmp/chk_mnt/usr/share/icons'):
        print('Icons dir:', os.listdir('/tmp/chk_mnt/usr/share/icons'))
        if os.path.exists('/tmp/chk_mnt/usr/share/icons/Papirus-Dark'):
            print('Papirus-Dark files:', len(os.listdir('/tmp/chk_mnt/usr/share/icons/Papirus-Dark')))
        else:
            print('NO Papirus-Dark!')
    else:
        print('NO /usr/share/icons in s6 img!')
finally:
    subprocess.run(['umount', '/tmp/chk_mnt'])
    subprocess.run(['losetup', '-d', loop])
