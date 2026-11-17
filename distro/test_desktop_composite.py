#!/usr/bin/env python3
from PIL import Image

wall_path = "/mnt/c/Users/Tashvi/.gemini/antigravity/brain/13b4aced-df88-405a-9721-3d119aeec783/restored_wall.png"
vitals_path = "/mnt/c/Users/Tashvi/.gemini/antigravity/brain/13b4aced-df88-405a-9721-3d119aeec783/cairo_vitals_direct.png"
out_path = "/mnt/c/Users/Tashvi/.gemini/antigravity/brain/13b4aced-df88-405a-9721-3d119aeec783/simulated_desktop.png"

wall = Image.open(wall_path).convert("RGBA")
vitals = Image.open(vitals_path).convert("RGBA")

# Place vitals at x=1540, y=60
wall.alpha_composite(vitals, (1540, 60))
wall.save(out_path)
print(f"Simulated desktop with circular vitals saved to {out_path}, size={wall.size}")
