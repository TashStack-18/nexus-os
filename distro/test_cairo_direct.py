#!/usr/bin/env python3
import cairo, os
from importlib.machinery import SourceFileLoader

path = "/root/micro-os/distro/theme/lunara-vitals"
mod = SourceFileLoader("vitals", path).load_module()

win = mod.LunaraVitalsWindow()

# Create Cairo ARGB32 ImageSurface
surface = cairo.ImageSurface(cairo.FORMAT_ARGB32, win.card_w, win.card_h)
cr = cairo.Context(surface)

# Call drawing function
win._on_draw(win, cr)

# Write to PNG
out_png = "/mnt/c/Users/Tashvi/.gemini/antigravity/brain/13b4aced-df88-405a-9721-3d119aeec783/cairo_vitals_direct.png"
surface.write_to_png(out_png)
print(f"Direct Cairo surface written to {out_png}")

from PIL import Image
im = Image.open(out_png)
print("Cairo image size:", im.size)
colors = im.getcolors(10000)
print("Unique colors count:", len(colors) if colors else ">10000")
print("Pixel extrema:", im.getextrema())
