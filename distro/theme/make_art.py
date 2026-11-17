#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Render high-definition, artifact-free LUNARA OS desktop and lock-screen visual assets.
Restored to clean, sharp celestial aesthetic matching the user's primary reference collage:
- Desktop: Deep midnight plum/charcoal gradient (#120f18 to #24162e), subtle smooth organic
  topographic contour curves in dusty rose/mauve, glowing crescent moon, celestial logo,
  and crisp LUNARA OS typography. Zero scattered particles, zero random dots, zero diagonal
  lines crossing the vitals area.
- Lock Screen: Moonlit mountain scenery with cherry blossom branch framing, ambient lake
  waterline reflection, top LUNARA branding, large digital clock, date, location pill,
  left System Status card, right Quick Info card, and bottom circular control buttons.
  CENTER IS LEFT UNOBSTRUCTED so LightDM's genuine interactive login card renders cleanly
  without overlapping box outlines.
- Avatar: 128x128 crisp circular user silhouette in blush pink and magenta.
"""
import os
import sys
import math
from PIL import Image, ImageDraw, ImageFont, ImageFilter

W, H = 1920, 1080
out_wall = sys.argv[1] if len(sys.argv) > 1 else "/tmp/lunara.png"
out_lock = sys.argv[2] if len(sys.argv) > 2 else "/tmp/lunara-lock.png"
out_avatar = sys.argv[3] if len(sys.argv) > 3 else os.path.join(os.path.dirname(out_wall), "lunara-avatar.png")


def mix(a, b, t):
    return tuple(int(a[i] + (b[i] - a[i]) * t) for i in range(len(a)))


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


def draw_card(d, bbox, radius=16, fill=(26, 18, 30, 215), outline=(214, 93, 140, 110), width=1):
    try:
        d.rounded_rectangle(bbox, radius=radius, fill=fill, outline=outline, width=width)
    except (AttributeError, TypeError):
        d.rectangle(bbox, fill=fill, outline=outline, width=width)


def draw_celestial_logo(d, cx, cy, size=48):
    """Draw the glowing stylized celestial crescent ribbon logo."""
    s = size / 2.0
    points_left = []
    points_right = []
    for deg in range(-90, 95, 5):
        rad = math.radians(deg)
        x1 = cx - s * 0.4 + math.cos(rad) * s * 0.55
        y1 = cy + math.sin(rad) * s * 0.95
        points_left.append((x1, y1))

    for deg in range(90, 275, 5):
        rad = math.radians(deg)
        x2 = cx + s * 0.4 + math.cos(rad) * s * 0.55
        y2 = cy + math.sin(rad) * s * 0.95
        points_right.append((x2, y2))

    for w, col in [(6, (214, 93, 140, 50)), (3, (242, 167, 191, 120)), (1, (253, 242, 246, 240))]:
        if len(points_left) > 1:
            d.line(points_left, fill=col, width=w, joint="curve")
        if len(points_right) > 1:
            d.line(points_right, fill=col, width=w, joint="curve")


def create_desktop_wallpaper():
    """Create the high-definition, sharp desktop wallpaper without noise or line artifacts."""
    # 1. Base vertical gradient: Midnight plum (#120e18) to dark velvet purple (#22152c)
    top_col = (0x12, 0x0E, 0x18)   # Deep midnight plum
    mid_col = (0x1A, 0x12, 0x24)   # Velvet plum charcoal
    bot_col = (0x24, 0x15, 0x2E)   # Rich dark mauve
    im = Image.new("RGB", (W, H))
    d = ImageDraw.Draw(im)

    for y in range(H):
        t = y / float(H)
        if t < 0.5:
            col = mix(top_col, mid_col, t * 2.0)
        else:
            col = mix(mid_col, bot_col, (t - 0.5) * 2.0)
        d.line((0, y, W, y), fill=col)

    # 2. Smooth layered organic contour topography in lower half (staying below y=700)
    # This prevents ANY lines from crossing through the center clock or the right vitals panel!
    layers = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    ld = ImageDraw.Draw(layers)

    # Contours focused at lower-left, giving depth without crossing into vitals widget
    c_x, c_y = int(W * 0.35), int(H * 0.88)
    contours = [
        (180, (0x32, 0x1E, 0x3A, 140), (0xF2, 0xA7, 0xBF, 50)),
        (280, (0x2C, 0x1A, 0x34, 130), (0xD6, 0x5D, 0x8C, 45)),
        (400, (0x26, 0x17, 0x2E, 120), (0xC9, 0x89, 0x9E, 40)),
        (540, (0x20, 0x13, 0x27, 100), (0x96, 0x5E, 0x78, 30)),
        (700, (0x1B, 0x10, 0x22, 80),  (0x6E, 0x45, 0x60, 25)),
    ]

    for rad, fill_col, edge_col in contours:
        rx = int(rad * 1.4)
        ry = int(rad * 0.65)
        bbox = (c_x - rx, c_y - ry, c_x + rx, c_y + ry)
        ld.ellipse(bbox, outline=edge_col, width=2)

    # 3. Soft ambient glow clouds in lower background
    glow = Image.new("RGB", (W, H), (0, 0, 0))
    gd = ImageDraw.Draw(glow)
    spheres = [
        (0.35, 0.85, 0.35, (0x7D, 0x30, 0x56)),   # Lower left plum glow
        (0.50, 0.70, 0.25, (0x9A, 0x45, 0x75)),   # Subtle center rose glow
    ]
    for cx, cy, r, col in spheres:
        rad = int(W * r)
        gd.ellipse((int(W * cx) - rad, int(H * cy) - rad, int(W * cx) + rad, int(H * cy) + rad), fill=col)

    glow = glow.filter(ImageFilter.GaussianBlur(140))
    blended = Image.blend(im, glow, 0.30)
    comp = Image.alpha_composite(blended.convert("RGBA"), layers)

    # 4. Glowing crescent moon in center lower-middle
    moon_img = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    md = ImageDraw.Draw(moon_img)
    mx, my = int(W * 0.49), int(H * 0.64)
    m_rad = 32

    # Delicate ambient moon halo
    for gr in range(80, 20, -12):
        alpha = int(35 * (1.0 - gr / 80.0))
        md.ellipse((mx - gr, my - gr, mx + gr, my + gr), fill=(242, 167, 191, alpha))

    # Crescent body
    md.ellipse((mx - m_rad, my - m_rad, mx + m_rad, my + m_rad), fill=(253, 242, 246, 230))
    md.ellipse((mx - m_rad + 9, my - m_rad - 6, mx + m_rad + 6, my + m_rad - 6), fill=(0, 0, 0, 0))

    final_comp = Image.alpha_composite(comp, moon_img)
    d_final = ImageDraw.Draw(final_comp)

    # 5. Centered LUNARA OS Branding
    draw_celestial_logo(d_final, W // 2, 290, size=50)

    def centered(text, y, f, fill):
        try:
            bbox = d_final.textbbox((0, 0), text, font=f)
            w = bbox[2] - bbox[0]
        except AttributeError:
            w = d_final.textlength(text, font=f)
        d_final.text(((W - w) / 2, y), text, font=f, fill=fill)

    centered("LUNARA OS", 332, font(26, bold=True), (0xFD, 0xF2, 0xF6, 245))
    centered("Your space. Your workflow.", 374, font(13), (0xF2, 0xA7, 0xBF, 210))

    # Subtle horizontal accent divider beneath date area
    div_y = 555
    d_final.line(((W - 120) // 2, div_y, (W + 120) // 2, div_y), fill=(0xD6, 0x5D, 0x8C, 160), width=2)

    return final_comp.convert("RGB")


def create_lock_screen():
    """Create the high-definition lock screen with clean framing and NO baked-in center login card."""
    im = Image.new("RGB", (W, H), (0x12, 0x0D, 0x18))
    d = ImageDraw.Draw(im)

    # 1. Nocturnal gradient sky to tranquil lake horizon
    for y in range(H):
        t = y / float(H)
        if t < 0.55:
            col = mix((0x10, 0x0B, 0x16), (0x24, 0x16, 0x2C), t / 0.55)
        else:
            col = mix((0x1D, 0x12, 0x24), (0x11, 0x0C, 0x16), (t - 0.55) / 0.45)
        d.line((0, y, W, y), fill=col)

    # 2. Mountain silhouette in background
    mountain_pts = [(int(W * 0.45), int(H * 0.55))]
    for x in range(int(W * 0.45), W + 20, 20):
        rel_x = (x - W * 0.45) / (W * 0.55)
        peak1 = math.exp(-((rel_x - 0.6) ** 2) / 0.04) * 150
        peak2 = math.exp(-((rel_x - 0.85) ** 2) / 0.05) * 190
        peak3 = math.exp(-((rel_x - 0.3) ** 2) / 0.06) * 90
        y = int(H * 0.55) - int(peak1 + peak2 + peak3)
        mountain_pts.append((x, y))
    mountain_pts.extend([(W, int(H * 0.55)), (int(W * 0.45), int(H * 0.55))])
    d.polygon(mountain_pts, fill=(0x25, 0x19, 0x2C))

    # Mountain edge contour highlight
    for i in range(1, len(mountain_pts) - 2):
        d.line((mountain_pts[i], mountain_pts[i + 1]), fill=(0xC9, 0x89, 0x9E, 130), width=2)

    # 3. Waterline and subtle serene lake reflection
    water_y = int(H * 0.55)
    d.line((0, water_y, W, water_y), fill=(0x4A, 0x2C, 0x48, 160), width=1)
    for ry in range(water_y + 12, H - 90, 18):
        alpha = int(55 * (ry - water_y) / (H - water_y))
        for rx in range(0, W, 100):
            rlen = 50 + (rx * 7) % 35
            d.line((rx, ry, rx + rlen, ry), fill=(0xD6, 0x5D, 0x8C, alpha), width=1)

    # 4. Cherry blossom branches framing upper-left quadrant (smooth vector arcs, NO random scatter)
    blossom_layer = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    bd = ImageDraw.Draw(blossom_layer)

    branches = [
        [(0, 0), (90, 50), (190, 95), (320, 140), (460, 180), (580, 205)],
        [(190, 95), (270, 150), (370, 215), (460, 270), (530, 305)],
        [(320, 140), (380, 95), (470, 65), (570, 50)],
        [(90, 50), (150, 120), (230, 200), (300, 280), (360, 350)],
        [(0, 120), (90, 165), (180, 215), (280, 255)],
    ]
    for br in branches:
        bd.line(br, fill=(0x16, 0x0E, 0x1A, 255), width=6, joint="curve")
        for i in range(len(br) - 1):
            x1, y1 = br[i]
            x2, y2 = br[i + 1]
            bx = (x1 + x2) // 2 + 12
            by = (y1 + y2) // 2 + 18
            bd.line([(x1, y1), (bx, by)], fill=(0x18, 0x10, 0x1C, 240), width=2)

    # Curated, deliberate flower blossoms placed along branches
    for br in branches:
        for i, (bx, by) in enumerate(br):
            for dx, dy, r, col in [
                (-8, -6, 7, (0xF2, 0xA7, 0xBF, 230)),
                (6, -8, 6, (0xFD, 0xF2, 0xF6, 240)),
                (0, 7, 7, (0xD6, 0x5D, 0x8C, 220)),
                (-12, 6, 5, (0xE8, 0x8E, 0xB0, 210)),
                (10, 4, 6, (0xF2, 0xA7, 0xBF, 230)),
            ]:
                fx, fy = bx + dx, by + dy
                bd.ellipse((fx - r, fy - r, fx + r, fy + r), fill=col)

    comp = Image.alpha_composite(im.convert("RGBA"), blossom_layer)
    d = ImageDraw.Draw(comp)

    def centered(text, y, f, fill):
        try:
            bbox = d.textbbox((0, 0), text, font=f)
            w = bbox[2] - bbox[0]
        except AttributeError:
            w = d.textlength(text, font=f)
        d.text(((W - w) / 2, y), text, font=f, fill=fill)

    # 5. Top Center: Celestial Logo, LUNARA OS, Tagline, Clock, Date, Location Pill
    draw_celestial_logo(d, W // 2, 75, size=44)
    centered("LUNARA OS", 108, font(26, bold=True), (0xFD, 0xF2, 0xF6, 250))
    centered("Your space. Your workflow.", 146, font(13), (0xF2, 0xA7, 0xBF, 215))

    # Clock & Date
    centered("23:07", 180, font(64, bold=True), (0xFD, 0xF2, 0xF6, 250))
    centered("Monday, 16 November 2026", 262, font(15), (0xC9, 0x89, 0x9E, 230))

    # Location pill
    pill_w, pill_h = 210, 28
    pill_x = (W - pill_w) // 2
    pill_y = 295
    draw_card(d, (pill_x, pill_y, pill_x + pill_w, pill_y + pill_h), radius=14,
              fill=(0x22, 0x16, 0x2A, 190), outline=(0xD6, 0x5D, 0x8C, 80), width=1)
    d.text((pill_x + 28, pill_y + 6), "☁  24°C  •  Bangalore", font=font(11), fill=(0xFD, 0xF2, 0xF6, 220))

    # 6. Side Panels: Left "System Status", Right "Quick Info"
    card_y = 360
    card_h = 420

    # === LEFT CARD: System Status ===
    cl_w = 260
    cl_x = int(W * 0.10)
    draw_card(d, (cl_x, card_y, cl_x + cl_w, card_y + card_h), radius=16,
              fill=(0x1A, 0x12, 0x22, 220), outline=(0xD6, 0x5D, 0x8C, 90), width=1)
    d.text((cl_x + 20, card_y + 20), "System Status", font=font(13, bold=True), fill=(0xFD, 0xF2, 0xF6, 240))

    metrics = [
        ("CPU", "42%", 0.42),
        ("RAM", "17%", 0.17),
        ("Disk", "1%", 0.01),
        ("Battery", "100%", 1.0),
    ]
    by = card_y + 65
    for name, val_str, frac in metrics:
        d.text((cl_x + 20, by), name, font=font(11), fill=(0xC9, 0x89, 0x9E, 220))
        d.text((cl_x + cl_w - 55, by), val_str, font=font(11, bold=True), fill=(0xFD, 0xF2, 0xF6, 240))
        bar_x = cl_x + 20
        bar_y = by + 20
        bar_w = cl_w - 40
        draw_card(d, (bar_x, bar_y, bar_x + bar_w, bar_y + 6), radius=3,
                  fill=(0x2E, 0x1C, 0x34, 255), outline=(0, 0, 0, 0), width=0)
        fill_w = max(4, int(bar_w * frac))
        draw_card(d, (bar_x, bar_y, bar_x + fill_w, bar_y + 6), radius=3,
                  fill=(0xD6, 0x5D, 0x8C, 240), outline=(0, 0, 0, 0), width=0)
        by += 60

    # === RIGHT CARD: Quick Info ===
    cr_w = 260
    cr_x = W - int(W * 0.10) - cr_w
    draw_card(d, (cr_x, card_y, cr_x + cr_w, card_y + card_h), radius=16,
              fill=(0x1A, 0x12, 0x22, 220), outline=(0xD6, 0x5D, 0x8C, 90), width=1)
    d.text((cr_x + 20, card_y + 20), "Quick Info", font=font(13, bold=True), fill=(0xFD, 0xF2, 0xF6, 240))

    info_rows = [
        ("Wi-Fi", "LUNARA-Net"),
        ("Uptime", "6m 58s"),
        ("Session", "XFCE"),
        ("Status", "Secured"),
    ]
    iy = card_y + 65
    for title, desc in info_rows:
        d.text((cr_x + 20, iy), title, font=font(11), fill=(0xC9, 0x89, 0x9E, 220))
        d.text((cr_x + 20, iy + 20), desc, font=font(12, bold=True), fill=(0xFD, 0xF2, 0xF6, 240))
        iy += 60

    # 7. Bottom Circular Control Buttons
    ctrl_y = H - 95
    ctrl_cx = W // 2
    buttons = ["Wi-Fi", "Vol", "Sun", "Key", "A11y", "Power"]
    btn_r = 18
    start_x = ctrl_cx - len(buttons) * 26 + 13
    for i, _ in enumerate(buttons):
        bx = start_x + i * 52
        draw_card(d, (bx - btn_r, ctrl_y - btn_r, bx + btn_r, ctrl_y + btn_r), radius=btn_r,
                  fill=(0x22, 0x16, 0x2A, 220), outline=(0xD6, 0x5D, 0x8C, 100), width=1)
        d.ellipse((bx - 4, ctrl_y - 4, bx + 4, ctrl_y + 4), fill=(0xF2, 0xA7, 0xBF, 230))

    return comp.convert("RGB")


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
    print("Generating LUNARA desktop wallpaper ->", out_wall)
    wp = create_desktop_wallpaper()
    wp.save(out_wall)

    print("Generating LUNARA lock screen wallpaper ->", out_lock)
    lock = create_lock_screen()
    lock.save(out_lock)

    print("Generating LUNARA user avatar ->", out_avatar)
    try:
        create_avatar(out_avatar)
    except Exception as e:
        print(f"avatar warning: {e}")

    print("LUNARA visual assets created successfully.")


if __name__ == "__main__":
    main()
