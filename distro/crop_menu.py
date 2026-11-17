#!/usr/bin/env python3
from PIL import Image

p = "/mnt/c/Users/Tashvi/.gemini/antigravity/brain/13b4aced-df88-405a-9721-3d119aeec783/screen_before.png"
out = "/mnt/c/Users/Tashvi/.gemini/antigravity/brain/13b4aced-df88-405a-9721-3d119aeec783/grub_menu_cropped.png"

im = Image.open(p)
# The GRUB box is roughly in top/middle
crop_box = (20, 20, 1200, 400)
cropped = im.crop(crop_box)
cropped.save(out)
print(f"Cropped GRUB menu saved to {out}, size={cropped.size}")
