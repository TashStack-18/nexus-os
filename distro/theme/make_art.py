#!/usr/bin/env python3
"""Render the modern NEXUS desktop wallpaper and the lock-screen background (original art)."""
import os
import sys
import math
from PIL import Image, ImageDraw, ImageFont, ImageFilter

W, H = 1920, 1080
out_wall = sys.argv[1] if len(sys.argv) > 1 else "/tmp/nexus.png"
out_lock = sys.argv[2] if len(sys.argv) > 2 else "/tmp/nexus-lock.png"
out_avatar = sys.argv[3] if len(sys.argv) > 3 else os.path.join(os.path.dirname(out_wall), "nexus-avatar.png")


def mix(a, b, t):
    return tuple(int(a[i] + (b[i] - a[i]) * t) for i in range(3))


def font(size, bold=False):
    name = "DejaVuSans-Bold.ttf" if bold else "DejaVuSans.ttf"
    for base in (
        "/usr/share/fonts/truetype/dejavu/",
        "/usr/share/fonts/dejavu/",
        "C:/Windows/Fonts/",
    ):
        p = os.path.join(base, name)
        if os.path.exists(p):
            try:
                return ImageFont.truetype(p, size)
            except OSError:
                pass
        # Fallback for Windows if DejaVu not installed
        if "Windows" in base:
            for win_f in ["segoeui.ttf", "arial.ttf"]:
                wp = os.path.join(base, win_f)
                if os.path.exists(wp):
                    try:
                        return ImageFont.truetype(wp, size)
                    except OSError:
                        pass
    return ImageFont.load_default()


def create_wallpaper():
    # Premium deep obsidian & dark midnight gradient
    top_col = (0x08, 0x0C, 0x18)
    mid_col = (0x0E, 0x15, 0x2C)
    bot_col = (0x14, 0x1D, 0x38)

    im = Image.new("RGB", (W, H))
    d = ImageDraw.Draw(im)

    for y in range(H):
        t = y / float(H)
        if t < 0.5:
            col = mix(top_col, mid_col, t * 2.0)
        else:
            col = mix(mid_col, bot_col, (t - 0.5) * 2.0)
        d.line((0, y, W, y), fill=col)

    # Ambient organic glow nebulae
    glow = Image.new("RGB", (W, H), (0, 0, 0))
    g = ImageDraw.Draw(glow)

    # Soft glowing spheres representing kernel execution cores
    spheres = [
        (0.25, 0.70, 0.35, (0x1A, 0x7A, 0x8A)),   # Cyan glow
        (0.75, 0.35, 0.30, (0x4A, 0x35, 0x9A)),   # Deep Indigo/Purple
        (0.50, 0.90, 0.28, (0x1F, 0x48, 0x88)),   # Deep Sapphire
    ]
    for cx, cy, r, col in spheres:
        rad = int(W * r)
        x0 = int(W * cx) - rad
        y0 = int(H * cy) - rad
        x1 = int(W * cx) + rad
        y1 = int(H * cy) + rad
        g.ellipse((x0, y0, x1, y1), fill=col)

    glow = glow.filter(ImageFilter.GaussianBlur(150))
    blended = Image.blend(im, glow, 0.55)

    # Crisp, subtle geometric orbital rings in the background
    overlay = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    od = ImageDraw.Draw(overlay)
    center_x, center_y = int(W * 0.50), int(H * 0.48)

    for radius, alpha, width in [
        (260, 20, 1),
        (380, 14, 1),
        (520, 10, 1),
        (680, 8, 1),
    ]:
        od.ellipse(
            (center_x - radius, center_y - radius, center_x + radius, center_y + radius),
            outline=(200, 220, 255, alpha),
            width=width
        )

    # Subtle diagonal accent light
    od.line(
        (center_x - 350, center_y + 200, center_x + 350, center_y - 200),
        fill=(100, 150, 255, 12),
        width=1
    )

    result = Image.alpha_composite(blended.convert("RGBA"), overlay)
    return result.convert("RGB")


def create_lock_screen(base_wp):
    # Darken and frost the wallpaper for the lock screen
    darken = Image.new("RGBA", (W, H), (6, 9, 18, 130))
    lock = Image.alpha_composite(base_wp.convert("RGBA"), darken)
    d = ImageDraw.Draw(lock)

    def centered(text, y, f, fill):
        try:
            bbox = d.textbbox((0, 0), text, font=f)
            w = bbox[2] - bbox[0]
        except AttributeError:
            w = d.textlength(text, font=f)
        d.text(((W - w) / 2, y), text, font=f, fill=fill)

    # Professional branding at top of lock screen
    centered("NEXUS OS", 80, font(44, bold=True), (245, 248, 255, 230))
    centered("Educational Operating System & Microkernel", 136, font(16), (148, 163, 184, 180))

    # Clean credential footnote at bottom
    centered("Live Session  •  Username: student  •  Password: microos", H - 70, font(14), (140, 155, 180, 170))

    return lock.convert("RGB")


def create_avatar(out_path):
    size = 128
    av = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(av)

    # Outer circle gradient / solid accent
    d.ellipse((0, 0, size - 1, size - 1), fill=(0x22, 0x33, 0x58, 255), outline=(0x5B, 0x8C, 0xFF, 255), width=3)

    # User silhouette
    # Head
    head_rad = 22
    hx, hy = size // 2, 44
    d.ellipse((hx - head_rad, hy - head_rad, hx + head_rad, hy + head_rad), fill=(0xE8, 0xED, 0xF8, 255))

    # Shoulders
    sx, sy = size // 2, 102
    d.chord((sx - 38, sy - 28, sx + 38, sy + 44), 180, 360, fill=(0xE8, 0xED, 0xF8, 255))

    av.save(out_path)


def main():
    wp = create_wallpaper()
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
