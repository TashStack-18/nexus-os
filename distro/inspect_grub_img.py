#!/usr/bin/env python3
from PIL import Image
p = "/mnt/c/Users/Tashvi/.gemini/antigravity/brain/13b4aced-df88-405a-9721-3d119aeec783/screen_before.png"
im = Image.open(p)
print("Screen before properties:")
print("Size:", im.size)
print("Mode:", im.mode)
colors = im.getcolors(maxcolors=100)
print("Color count:", len(colors) if colors else ">100")
for count, col in sorted(colors, key=lambda x: -x[0])[:10]:
    print("  Count:", count, "Color:", col)
