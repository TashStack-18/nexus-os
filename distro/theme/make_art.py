#!/usr/bin/env python3
"""Render the NEXUS wallpaper and the login/lock-screen background (original art)."""
import sys
from PIL import Image, ImageDraw, ImageFont, ImageFilter

W, H = 1920, 1080
out_wall, out_lock = sys.argv[1], sys.argv[2]


def mix(a, b, t):
    return tuple(int(a[i] + (b[i] - a[i]) * t) for i in range(3))


def wallpaper():
    top, bottom = (0x0B, 0x12, 0x30), (0x1B, 0x2C, 0x5E)
    im = Image.new("RGB", (W, H))
    d = ImageDraw.Draw(im)
    for y in range(H):
        d.line((0, y, W, y), fill=mix(top, bottom, y / H))
    glow = Image.new("RGB", (W, H), (0, 0, 0))
    g = ImageDraw.Draw(glow)
    for cx, cy, r, col in [
        (0.20, 0.85, 0.38, (0x1F, 0xA3, 0xA3)),
        (0.82, 0.22, 0.34, (0x7C, 0x4D, 0xFF)),
        (0.58, 1.05, 0.30, (0xFF, 0x7A, 0x59)),
    ]:
        rad = int(W * r)
        g.ellipse((int(W * cx) - rad, int(H * cy) - rad, int(W * cx) + rad, int(H * cy) + rad), fill=col)
    glow = glow.filter(ImageFilter.GaussianBlur(170))
    return Image.blend(im, glow, 0.6)


def font(size, bold=False):
    name = "DejaVuSans-Bold.ttf" if bold else "DejaVuSans.ttf"
    for base in ("/usr/share/fonts/truetype/dejavu/", "/usr/share/fonts/dejavu/"):
        try:
            return ImageFont.truetype(base + name, size)
        except OSError:
            pass
    return ImageFont.load_default()


wp = wallpaper()
wp.save(out_wall)

lock = wp.copy()
d = ImageDraw.Draw(lock)


def centered(text, y, f, fill):
    w = d.textlength(text, font=f)
    d.text(((W - w) / 2, y), text, font=f, fill=fill)


centered("NEXUS", 70, font(54, True), (255, 255, 255))
centered("Micro-OS Linux", 140, font(24), (200, 212, 240))
centered("Live session  -  user: student   password: microos", H - 80, font(20), (190, 200, 225))
lock.save(out_lock)
print("art written")
