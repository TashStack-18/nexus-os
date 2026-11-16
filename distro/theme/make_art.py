#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Render the sophisticated pink-themed NEXUS desktop wallpaper and lock-screen background.
Inspired by Reference 1 (organic contour wave topography) and Reference 2 (clean KDE layout).
Palette: Deep charcoal, dusty rose, blush pink, muted mauve, subtle magenta, soft pink highlights.
"""
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
        if "Windows" in base:
            for win_f in ["segoeui.ttf", "arial.ttf"]:
                wp = os.path.join(base, win_f)
                if os.path.exists(wp):
                    try:
                        return ImageFont.truetype(wp, size)
                    except OSError:
                        pass
    return ImageFont.load_default()


def create_topographic_wallpaper():
    # 1. Base gradient in deep charcoal and midnight mauve
    top_col = (0x12, 0x10, 0x18)    # Deep charcoal
    mid_col = (0x1A, 0x14, 0x24)    # Dark plum charcoal
    bot_col = (0x24, 0x19, 0x2E)    # Muted dark mauve

    im = Image.new("RGB", (W, H))
    d = ImageDraw.Draw(im)

    for y in range(H):
        t = y / float(H)
        if t < 0.5:
            col = mix(top_col, mid_col, t * 2.0)
        else:
            col = mix(mid_col, bot_col, (t - 0.5) * 2.0)
        d.line((0, y, W, y), fill=col)

    # 2. Layered organic topographic contour waves (inspired by Reference 1)
    layers = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    ld = ImageDraw.Draw(layers)

    # Focal centers for organic topographic rings
    c1_x, c1_y = int(W * 0.48), int(H * 0.46)
    c2_x, c2_y = int(W * 0.15), int(H * 0.85)
    c3_x, c3_y = int(W * 0.85), int(H * 0.25)

    # Draw series of layered topographic contours with subtle drop shadows
    contour_radii = [
        (180, (0x38, 0x26, 0x44, 180), (0xF2, 0xA7, 0xBF, 60)),
        (260, (0x30, 0x22, 0x3C, 160), (0xD6, 0x5D, 0x8C, 50)),
        (350, (0x2A, 0x1E, 0x35, 150), (0xC9, 0x89, 0x9E, 45)),
        (450, (0x24, 0x1B, 0x2F, 140), (0xB0, 0x72, 0x8A, 40)),
        (560, (0x20, 0x18, 0x2A, 130), (0x96, 0x5E, 0x78, 35)),
        (680, (0x1C, 0x15, 0x25, 120), (0x7D, 0x4D, 0x68, 30)),
        (820, (0x18, 0x13, 0x21, 100), (0x68, 0x3E, 0x58, 25)),
        (980, (0x15, 0x11, 0x1E, 90),  (0x56, 0x32, 0x4A, 20)),
        (1160, (0x12, 0x0F, 0x1A, 80), (0x46, 0x28, 0x3D, 18)),
    ]

    for rad, fill_col, edge_col in contour_radii:
        # Organic perturbed ellipse coordinates
        rx = int(rad * 1.25)
        ry = int(rad * 0.82)
        bbox = (c1_x - rx, c1_y - ry, c1_x + rx, c1_y + ry)
        ld.ellipse(bbox, outline=edge_col, width=2)

    # Secondary landscape contours
    for rad, edge_col in [
        (280, (0xF2, 0xA7, 0xBF, 35)),
        (420, (0xD6, 0x5D, 0x8C, 30)),
        (580, (0xC9, 0x89, 0x9E, 25)),
        (760, (0x96, 0x5E, 0x78, 20)),
    ]:
        bbox2 = (c2_x - rad, c2_y - int(rad * 0.7), c2_x + rad, c2_y + int(rad * 0.7))
        ld.ellipse(bbox2, outline=edge_col, width=2)
        bbox3 = (c3_x - rad, c3_y - int(rad * 0.7), c3_x + rad, c3_y + int(rad * 0.7))
        ld.ellipse(bbox3, outline=edge_col, width=2)

    # 3. Soft ambient glow clouds (dusty rose and blush pink nebulae)
    glow = Image.new("RGB", (W, H), (0, 0, 0))
    g = ImageDraw.Draw(glow)

    spheres = [
        (0.20, 0.80, 0.40, (0x8A, 0x35, 0x60)),   # Deep dusty rose
        (0.78, 0.30, 0.35, (0x9A, 0x45, 0x75)),   # Muted magenta
        (0.48, 0.50, 0.30, (0xB5, 0x5E, 0x86)),   # Blush pink core
    ]
    for cx, cy, r, col in spheres:
        rad = int(W * r)
        g.ellipse((int(W * cx) - rad, int(H * cy) - rad, int(W * cx) + rad, int(H * cy) + rad), fill=col)

    glow = glow.filter(ImageFilter.GaussianBlur(160))
    blended = Image.blend(im, glow, 0.42)

    # Composite layers
    result = Image.alpha_composite(blended.convert("RGBA"), layers)
    return result.convert("RGB")


def create_lock_screen(base_wp):
    # Darken and frost the wallpaper for the lock screen
    darken = Image.new("RGBA", (W, H), (14, 11, 18, 140))
    lock = Image.alpha_composite(base_wp.convert("RGBA"), darken)
    d = ImageDraw.Draw(lock)

    def centered(text, y, f, fill):
        try:
            bbox = d.textbbox((0, 0), text, font=f)
            w = bbox[2] - bbox[0]
        except AttributeError:
            w = d.textlength(text, font=f)
        d.text(((W - w) / 2, y), text, font=f, fill=fill)

    # Professional NEXUS branding at top of lock screen
    centered("NEXUS OS", 80, font(44, bold=True), (0xFD, 0xF2, 0xF6, 240))
    # Subtitle updated to: "Your space. Your workflow." as required!
    centered("Your space. Your workflow.", 136, font(16), (0xF2, 0xA7, 0xBF, 200))

    # Helper for rounded card or fallback to rectangle
    draw_card = getattr(d, "rounded_rectangle", d.rectangle)

    # Subtle floating information widget on left
    card_l = (80, H - 230, 430, H - 90)
    try:
        draw_card(card_l, radius=12, fill=(0x1A, 0x14, 0x22, 190), outline=(0xD6, 0x5D, 0x8C, 90), width=1)
    except TypeError:
        d.rectangle(card_l, fill=(0x1A, 0x14, 0x22, 190), outline=(0xD6, 0x5D, 0x8C, 90), width=1)
    d.text((105, H - 210), "SYSTEM STATUS", font=font(12, bold=True), fill=(0xD6, 0x5D, 0x8C, 240))
    d.text((105, H - 182), "• NEXUS Micro-OS 6.12 Kernel", font=font(11), fill=(0xFD, 0xF2, 0xF6, 210))
    d.text((105, H - 158), "• Secure Linux PAM Engine", font=font(11), fill=(0xFD, 0xF2, 0xF6, 210))
    d.text((105, H - 134), "• Status: Authenticated & Protected", font=font(11), fill=(0xC9, 0x89, 0x9E, 200))

    # Subtle quick info widget on right
    card_r = (W - 430, H - 230, W - 80, H - 90)
    try:
        draw_card(card_r, radius=12, fill=(0x1A, 0x14, 0x22, 190), outline=(0xD6, 0x5D, 0x8C, 90), width=1)
    except TypeError:
        d.rectangle(card_r, fill=(0x1A, 0x14, 0x22, 190), outline=(0xD6, 0x5D, 0x8C, 90), width=1)
    d.text((W - 405, H - 210), "SESSION CONTROLS", font=font(12, bold=True), fill=(0xD6, 0x5D, 0x8C, 240))
    d.text((W - 405, H - 182), "• Default User: student (pass: microos)", font=font(11), fill=(0xFD, 0xF2, 0xF6, 210))
    d.text((W - 405, H - 158), "• Secondary User: tashvi (pass: microos)", font=font(11), fill=(0xFD, 0xF2, 0xF6, 210))
    d.text((W - 405, H - 134), "• Shortcut: Super + L locks session", font=font(11), fill=(0xC9, 0x89, 0x9E, 200))

    # Clean credential footnote at bottom
    centered("Live Session  |  student:microos  |  tashvi:microos", H - 60, font(13), (0xC9, 0x89, 0x9E, 180))

    return lock.convert("RGB")


def create_avatar(out_path):
    size = 128
    av = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(av)

    # Outer circle in dusty rose / blush pink accent
    d.ellipse((0, 0, size - 1, size - 1), fill=(0x2A, 0x1E, 0x2E, 255), outline=(0xD6, 0x5D, 0x8C, 255), width=3)

    # User silhouette in soft pink
    head_rad = 22
    hx, hy = size // 2, 44
    d.ellipse((hx - head_rad, hy - head_rad, hx + head_rad, hy + head_rad), fill=(0xFD, 0xF2, 0xF6, 255))

    sx, sy = size // 2, 102
    d.chord((sx - 38, sy - 28, sx + 38, sy + 44), 180, 360, fill=(0xFD, 0xF2, 0xF6, 255))

    av.save(out_path)


def main():
    wp = create_topographic_wallpaper()
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
