#!/usr/bin/env bash
set -e

rm -f /tmp/.X*-lock /tmp/.X11-unix/X* 2>/dev/null || true

# Run with xvfb-run with auto display and xfwm4 compositor
xvfb-run -a -s "-screen 0 1920x1080x24" bash -c '
xfwm4 --compositor=on &
sleep 1.0
python3 /root/micro-os/distro/theme/lunara-vitals &
PID=$!
sleep 2.0
import -window root /tmp/vitals_full.png
kill $PID 2>/dev/null || true
pkill xfwm4 2>/dev/null || true
'

python3 -c '
from PIL import Image
im = Image.open("/tmp/vitals_full.png")
print("Full desktop screenshot size:", im.size)
# The vitals card is at x=1920-340-40=1540, y=60, w=340, h=390
card = im.crop((1538, 58, 1882, 452))
card.save("/mnt/c/Users/Tashvi/.gemini/antigravity/brain/13b4aced-df88-405a-9721-3d119aeec783/vitals_card.png")
print("Cropped vitals card saved to artifacts/vitals_card.png!")
colors = card.getcolors(1000)
print("Color count in card:", len(colors) if colors else ">1000")
'
