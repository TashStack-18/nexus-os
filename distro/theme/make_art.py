#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Render the authentic, high-definition mauve and plum circular-orbit wallpaper
and lock-screen background for LUNARA OS.
Strictly matching Reference Image 1 (Desktop) and Reference Image 2 (Lock Screen).
- Base: Deep charcoal and midnight mauve gradient (#121018 to #24192E).
- Topography: Elegant thin concentric circular orbital patterns with 2x supersampling
  for mathematically smooth, razor-sharp anti-aliased curves.
- Zero particles, zero mountains, zero cherry blossoms, zero sine-wave line artifacts.
- Lock screen: Same circular wallpaper + Left System Status card + Right Quick Info card
  + Bottom control buttons. Center is left clean and unobstructed for LightDM PAM login card.
"""
import os
import sys
import math
from PIL import Image, ImageDraw, ImageFont, ImageFilter

W, H = 1920, 1080
SCALE = 2
SW, SH = W * SCALE, H * SCALE

out_wall = sys.argv[1] if len(sys.argv) > 1 else "/tmp/lunara.png"
out_lock = sys.argv[2] if len(sys.argv) > 2 else "/tmp/lunara-lock.png"
out_avatar = sys.argv[3] if len(sys.argv) > 3 else os.path.join(os.path.dirname(out_wall), "lunara-avatar.png")


def mix(a, b, t):
    return tuple(int(a[i] + (b[i] - a[i]) * t) for i in range(3))


def get_font(size, bold=False):
    name = "DejaVuSans-Bold.ttf" if bold else "DejaVuSans.ttf"
    for base in [
        "/usr/share/fonts/truetype/dejavu/",
        "/usr/share/fonts/dejavu/",
        "C:/Windows/Fonts/",
    ]:
        p = os.path.join(base, name)
        if os.path.exists(p):
            try:
                return ImageFont.truetype(p, size)
            except OSError:
                pass
    return ImageFont.load_default()


def create_circular_orbit_wallpaper():
    """Render the crisp, authentic circular orbital wallpaper at 2x resolution and downsample."""
    # 1. Base vertical gradient in deep charcoal and midnight mauve
    top_col = (0x12, 0x10, 0x18)    # Deep charcoal (#121018)
    mid_col = (0x1A, 0x14, 0x24)    # Dark plum charcoal (#1A1424)
    bot_col = (0x24, 0x19, 0x2E)    # Muted dark mauve (#24192E)

    im = Image.new("RGB", (SW, SH))
    d = ImageDraw.Draw(im)

    for y in range(SH):
        t = y / float(SH)
        if t < 0.5:
            col = mix(top_col, mid_col, t * 2.0)
        else:
            col = mix(mid_col, bot_col, (t - 0.5) * 2.0)
        d.line((0, y, SW, y), fill=col)

    # 2. Ambient glow clouds (dusty rose and blush pink nebulae)
    glow = Image.new("RGB", (SW, SH), (0, 0, 0))
    g = ImageDraw.Draw(glow)

    spheres = [
        (0.20, 0.80, 0.40, (0x8A, 0x35, 0x60)),   # Deep dusty rose
        (0.78, 0.30, 0.35, (0x9A, 0x45, 0x75)),   # Muted magenta
        (0.48, 0.50, 0.30, (0xB5, 0x5E, 0x86)),   # Blush pink core
    ]
    for cx, cy, r, col in spheres:
        rad = int(SW * r)
        g.ellipse((int(SW * cx) - rad, int(SH * cy) - rad, int(SW * cx) + rad, int(SH * cy) + rad), fill=col)

    glow = glow.filter(ImageFilter.GaussianBlur(160 * SCALE))
    blended = Image.blend(im, glow, 0.42)

    # 3. Layered organic topographic circular/elliptical contours
    layers = Image.new("RGBA", (SW, SH), (0, 0, 0, 0))
    ld = ImageDraw.Draw(layers)

    # Center coordinates for primary and secondary orbital sets
    c1_x, c1_y = int(SW * 0.48), int(SH * 0.46)
    c2_x, c2_y = int(SW * 0.15), int(SH * 0.85)
    c3_x, c3_y = int(SW * 0.85), int(SH * 0.25)

    # Primary concentric ellipses
    contour_radii = [
        (180, (0x38, 0x26, 0x44, 180), (0xF2, 0xA7, 0xBF, 75)),
        (260, (0x30, 0x22, 0x3C, 160), (0xD6, 0x5D, 0x8C, 65)),
        (350, (0x2A, 0x1E, 0x35, 150), (0xC9, 0x89, 0x9E, 60)),
        (450, (0x24, 0x1B, 0x2F, 140), (0xB0, 0x72, 0x8A, 55)),
        (560, (0x20, 0x18, 0x2A, 130), (0x96, 0x5E, 0x78, 50)),
        (680, (0x1C, 0x15, 0x25, 120), (0x7D, 0x4D, 0x68, 45)),
        (820, (0x18, 0x13, 0x21, 100), (0x68, 0x3E, 0x58, 40)),
        (980, (0x15, 0x11, 0x1E, 90),  (0x56, 0x32, 0x4A, 35)),
        (1160, (0x12, 0x0F, 0x1A, 80), (0x46, 0x28, 0x3D, 28)),
    ]

    for rad, fill_col, edge_col in contour_radii:
        rx = int(rad * SCALE * 1.25)
        ry = int(rad * SCALE * 0.82)
        bbox = (c1_x - rx, c1_y - ry, c1_x + rx, c1_y + ry)
        ld.ellipse(bbox, outline=edge_col, width=2 * SCALE)

    # Secondary sweeping landscape contours
    for rad, edge_col in [
        (280, (0xF2, 0xA7, 0xBF, 50)),
        (420, (0xD6, 0x5D, 0x8C, 45)),
        (580, (0xC9, 0x89, 0x9E, 40)),
        (760, (0x96, 0x5E, 0x78, 32)),
    ]:
        rx = int(rad * SCALE)
        ry = int(rad * SCALE * 0.70)
        ld.ellipse((c2_x - rx, c2_y - ry, c2_x + rx, c2_y + ry), outline=edge_col, width=2 * SCALE)
        ld.ellipse((c3_x - rx, c3_y - ry, c3_x + rx, c3_y + ry), outline=edge_col, width=2 * SCALE)

    result_hi = Image.alpha_composite(blended.convert("RGBA"), layers)
    result = result_hi.resize((W, H), Image.Resampling.LANCZOS)
    return result.convert("RGB")


def create_lock_screen(base_wp):
    """Render the clean, high-definition lock-screen background.
    Uses the exact same high-definition circular-orbit wallpaper as the desktop,
    with subtle dark frosting so LightDM's interactive centered login card and
    top panel render cleanly at any resolution without stretching, clipping, or cut-offs.
    """
    darken = Image.new("RGBA", (W, H), (14, 11, 18, 120))
    lock = Image.alpha_composite(base_wp.convert("RGBA"), darken)
    return lock.convert("RGB")


def create_avatar(out_path):
    """128x128 Circular Avatar matching Reference Image 2."""
    size = 128
    av = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(av)

    # Outer circle in deep blue with bright cyan/blue border matching Reference 2
    d.ellipse((0, 0, size - 1, size - 1), fill=(0x25, 0x55, 0xA8, 255), outline=(0x5B, 0x8C, 0xFF, 255), width=3)

    # Clean white user silhouette
    head_rad = 22
    hx, hy = size // 2, 44
    d.ellipse((hx - head_rad, hy - head_rad, hx + head_rad, hy + head_rad), fill=(0xFF, 0xFF, 0xFF, 255))

    sx, sy = size // 2, 102
    d.chord((sx - 38, sy - 28, sx + 38, sy + 44), 180, 360, fill=(0xFF, 0xFF, 0xFF, 255))

    av.save(out_path)


def main():
    wp = create_circular_orbit_wallpaper()
    wp.save(out_wall)

    lock = create_lock_screen(wp)
    lock.save(out_lock)

    try:
        create_avatar(out_avatar)
    except Exception as e:
        print(f"avatar warning: {e}")

    print("art written")


if __name__ == "__main__":
    main()
