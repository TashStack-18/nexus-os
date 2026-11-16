#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Render the sophisticated LUNARA OS desktop wallpaper and lock-screen background.
Directly matching the user's primary reference collage:
- Desktop: Deep plum night sky, silky flowing wave ribbons in dusty rose/magenta, glowing crescent moon,
  celestial ribbon logo, LUNARA OS branding, and tagline.
- Lock Screen: Cherry blossom branches framing upper-left, moonlit mountain ridge, tranquil lake reflection,
  top branding and clock, left 'System Status' card, center login card frame, right 'Quick Info' card,
  and bottom circular control buttons.
Palette: Deep plum (#16101c), dark mauve (#24162a), dusty rose (#c9899e), blush pink (#f2a7bf), vibrant magenta (#d65d8c).
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


def draw_card(d, bbox, radius=16, fill=(26, 18, 30, 210), outline=(214, 93, 140, 110), width=1):
    try:
        d.rounded_rectangle(bbox, radius=radius, fill=fill, outline=outline, width=width)
    except (AttributeError, TypeError):
        d.rectangle(bbox, fill=fill, outline=outline, width=width)


def draw_celestial_logo(d, cx, cy, size=48):
    """Draw the glowing stylized celestial / crescent ribbon logo from the reference collage."""
    s = size / 2.0
    # Two interlinked glowing curved petals forming a stylized crescent ribbon
    points_left = []
    points_right = []
    for deg in range(-90, 95, 5):
        rad = math.radians(deg)
        # Left crescent loop
        x1 = cx - s * 0.4 + math.cos(rad) * s * 0.55
        y1 = cy + math.sin(rad) * s * 0.95
        points_left.append((x1, y1))

    for deg in range(90, 275, 5):
        rad = math.radians(deg)
        # Right crescent loop
        x2 = cx + s * 0.4 + math.cos(rad) * s * 0.55
        y2 = cy + math.sin(rad) * s * 0.95
        points_right.append((x2, y2))

    # Glow strokes
    for w, col in [(8, (214, 93, 140, 40)), (4, (242, 167, 191, 100)), (2, (253, 242, 246, 230))]:
        if len(points_left) > 1:
            d.line(points_left, fill=col, width=w, joint="curve")
        if len(points_right) > 1:
            d.line(points_right, fill=col, width=w, joint="curve")


def create_desktop_wallpaper():
    """Create the atmospheric desktop wallpaper matching Panel 1 of the reference collage."""
    # 1. Base vertical gradient: Midnight plum (#16101c) to dark purple/mauve (#26152e)
    top_col = (0x15, 0x0F, 0x1B)   # Deep midnight plum
    mid_col = (0x1F, 0x13, 0x27)   # Dark velvet purple
    bot_col = (0x28, 0x16, 0x33)   # Rich dark mauve

    im = Image.new("RGB", (W, H))
    d = ImageDraw.Draw(im)

    for y in range(H):
        t = y / float(H)
        if t < 0.5:
            col = mix(top_col, mid_col, t * 2.0)
        else:
            col = mix(mid_col, bot_col, (t - 0.5) * 2.0)
        d.line((0, y, W, y), fill=col)

    # 2. Glowing crescent moon dipping behind the silky waves in lower-middle right
    moon_img = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    md = ImageDraw.Draw(moon_img)
    mx, my = int(W * 0.49), int(H * 0.65)
    m_rad = 34

    # Ambient moon glow
    for gr in range(120, 20, -10):
        alpha = int(40 * (1.0 - gr / 120.0))
        md.ellipse((mx - gr, my - gr, mx + gr, my + gr), fill=(242, 167, 191, alpha))

    # Crescent moon body (slanted crescent tilted downwards as in reference)
    md.ellipse((mx - m_rad, my - m_rad, mx + m_rad, my + m_rad), fill=(253, 242, 246, 220))
    # Cutout ellipse to form delicate crescent
    md.ellipse((mx - m_rad + 10, my - m_rad - 6, mx + m_rad + 6, my + m_rad - 6), fill=(0, 0, 0, 0))

    # 3. Silky flowing wave ribbons across lower half (dusty rose and magenta)
    ribbons = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    rd = ImageDraw.Draw(ribbons)

    # Generate multi-layered silky undulating waves
    wave_specs = [
        # (base_y, amplitude, freq, phase, fill_rgba, edge_rgba)
        (int(H * 0.58), 90, 0.0022, 1.2, (0x44, 0x1E, 0x38, 120), (0xF2, 0xA7, 0xBF, 60)),
        (int(H * 0.66), 110, 0.0028, 2.5, (0x3C, 0x1A, 0x32, 140), (0xD6, 0x5D, 0x8C, 90)),
        (int(H * 0.74), 130, 0.0020, 0.4, (0x32, 0x15, 0x2A, 170), (0xF2, 0xA7, 0xBF, 110)),
        (int(H * 0.82), 115, 0.0025, 3.1, (0x28, 0x12, 0x24, 210), (0xC9, 0x89, 0x9E, 130)),
        (int(H * 0.90), 80, 0.0032, 1.8, (0x20, 0x0E, 0x1C, 240), (0xD6, 0x5D, 0x8C, 150)),
    ]

    for base_y, amp, freq, phase, fill_col, edge_col in wave_specs:
        poly_pts = [(0, H)]
        top_curve = []
        for x in range(0, W + 10, 10):
            # Compound harmonic wave for silky fabric look
            y = base_y + math.sin(x * freq + phase) * amp + math.cos(x * freq * 1.8 + phase * 0.7) * (amp * 0.35)
            poly_pts.append((x, int(y)))
            top_curve.append((x, int(y)))
        poly_pts.append((W, H))

        rd.polygon(poly_pts, fill=fill_col)
        # Highlight edge
        rd.line(top_curve, fill=edge_col, width=2, joint="curve")

    # 4. Soft atmospheric glow clouds
    glow = Image.new("RGB", (W, H), (0, 0, 0))
    gd = ImageDraw.Draw(glow)
    spheres = [
        (0.50, 0.70, 0.35, (0xB5, 0x5E, 0x86)),   # Core wave glow
        (0.20, 0.85, 0.30, (0x8A, 0x35, 0x60)),   # Left wave plum glow
        (0.80, 0.55, 0.35, (0x9A, 0x45, 0x75)),   # Right ambient rose
    ]
    for cx, cy, r, col in spheres:
        rad = int(W * r)
        gd.ellipse((int(W * cx) - rad, int(H * cy) - rad, int(W * cx) + rad, int(H * cy) + rad), fill=col)

    glow = glow.filter(ImageFilter.GaussianBlur(130))
    blended = Image.blend(im, glow, 0.38)

    # Composite moon and ribbons
    comp1 = Image.alpha_composite(blended.convert("RGBA"), moon_img)
    comp2 = Image.alpha_composite(comp1, ribbons)

    # 5. Add centered LUNARA OS branding and subtle accent divider
    # (Conky renders the dynamic digital clock and date in the center space!)
    d_final = ImageDraw.Draw(comp2)
    draw_celestial_logo(d_final, W // 2, 290, size=52)

    def centered(text, y, f, fill):
        try:
            bbox = d_final.textbbox((0, 0), text, font=f)
            w = bbox[2] - bbox[0]
        except AttributeError:
            w = d_final.textlength(text, font=f)
        d_final.text(((W - w) / 2, y), text, font=f, fill=fill)

    # "LUNARA OS" in clean spaced capital font
    centered("LUNARA OS", 335, font(26, bold=True), (0xFD, 0xF2, 0xF6, 245))
    # Subtitle / Tagline
    centered("Your space. Your workflow.", 378, font(13), (0xF2, 0xA7, 0xBF, 210))

    # Horizontal accent divider beneath date area
    div_y = 575
    d_final.line(((W - 140) // 2, div_y, (W + 140) // 2, div_y), fill=(0xD6, 0x5D, 0x8C, 180), width=2)

    return comp2.convert("RGB")


def create_lock_screen():
    """Create the cherry-blossom moonlit mountain & lake lock screen matching Panel 2 of collage."""
    im = Image.new("RGB", (W, H), (0x12, 0x0D, 0x18))
    d = ImageDraw.Draw(im)

    # 1. Nocturnal gradient sky: Dark plum (#130c18) to misty mauve horizon (#2b1a30)
    for y in range(H):
        t = y / float(H)
        if t < 0.55:
            col = mix((0x10, 0x0B, 0x16), (0x24, 0x16, 0x2C), t / 0.55)
        else:
            # Lake reflection gradient
            col = mix((0x1E, 0x13, 0x24), (0x11, 0x0C, 0x16), (t - 0.55) / 0.45)
        d.line((0, y, W, y), fill=col)

    # 2. Moonlit mountain ridge silhouettes across horizon
    mountain_pts = [(W * 0.45, int(H * 0.54))]
    for x in range(int(W * 0.45), W + 20, 20):
        rel_x = (x - W * 0.45) / (W * 0.55)
        # Jagged alpine peaks on right side as in reference
        peak1 = math.exp(-((rel_x - 0.6) ** 2) / 0.04) * 160
        peak2 = math.exp(-((rel_x - 0.85) ** 2) / 0.05) * 210
        peak3 = math.exp(-((rel_x - 0.3) ** 2) / 0.06) * 110
        y = int(H * 0.54) - int(peak1 + peak2 + peak3)
        mountain_pts.append((x, y))
    mountain_pts.extend([(W, int(H * 0.55)), (int(W * 0.45), int(H * 0.55))])
    d.polygon(mountain_pts, fill=(0x26, 0x1A, 0x2D))

    # Mountain edge highlight (moonlit snow/rock contour)
    for i in range(1, len(mountain_pts) - 2):
        d.line((mountain_pts[i], mountain_pts[i + 1]), fill=(0xC9, 0x89, 0x9E, 140), width=2)

    # 3. Waterline and serene lake ripples
    water_y = int(H * 0.55)
    d.line((0, water_y, W, water_y), fill=(0x4A, 0x2C, 0x48, 180), width=1)
    for ry in range(water_y + 10, H, 14):
        alpha = int(70 * (ry - water_y) / (H - water_y))
        for rx in range(0, W, 80):
            rlen = 45 + (rx * 7) % 35
            d.line((rx, ry, rx + rlen, ry), fill=(0xD6, 0x5D, 0x8C, alpha), width=1)

    # 4. Cherry blossom tree branches framing upper-left quadrant
    blossom_layer = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    bd = ImageDraw.Draw(blossom_layer)

    # Dark branch silhouettes
    branches = [
        # Main trunk/branch arcs
        [(0, 0), (80, 50), (180, 90), (320, 140), (480, 180), (620, 210)],
        [(180, 90), (260, 150), (360, 220), (460, 280), (540, 320)],
        [(320, 140), (380, 90), (480, 60), (600, 45)],
        [(80, 50), (140, 120), (220, 210), (300, 310), (380, 400)],
        [(0, 120), (90, 170), (200, 230), (310, 280)],
    ]
    for br in branches:
        bd.line(br, fill=(0x16, 0x0E, 0x1A, 255), width=7, joint="curve")
        # Fine secondary branches
        for i in range(len(br) - 1):
            x1, y1 = br[i]
            x2, y2 = br[i + 1]
            bx = (x1 + x2) // 2 + 15
            by = (y1 + y2) // 2 + 25
            bd.line([(x1, y1), (bx, by)], fill=(0x18, 0x10, 0x1C, 240), width=3)

    # Abundant glowing pink cherry blossom clusters
    import random
    rng = random.Random(42)  # Deterministic seed for aesthetic perfection
    flower_colors = [
        (0xF2, 0xA7, 0xBF, 230),   # Blush pink
        (0xFD, 0xF2, 0xF6, 240),   # Pale petal white
        (0xD6, 0x5D, 0x8C, 220),   # Magenta petal core
        (0xE8, 0x8E, 0xB0, 210),   # Rose pink
    ]

    for br in branches:
        for x, y in br:
            for _ in range(14):
                fx = x + rng.randint(-55, 65)
                fy = y + rng.randint(-40, 60)
                if fx < 0 or fx > int(W * 0.42) or fy < 0 or fy > int(H * 0.48):
                    continue
                col = rng.choice(flower_colors)
                frad = rng.randint(4, 9)
                bd.ellipse((fx - frad, fy - frad, fx + frad, fy + frad), fill=col)
                # Delicate petal outline
                bd.ellipse((fx - frad + 2, fy - frad + 2, fx + frad - 2, fy + frad - 2), fill=(0xFF, 0xFF, 0xFF, 120))

    # Composite blossoms onto scenery
    im_with_blossoms = Image.alpha_composite(im.convert("RGBA"), blossom_layer)
    d = ImageDraw.Draw(im_with_blossoms)

    def centered(text, y, f, fill):
        try:
            bbox = d.textbbox((0, 0), text, font=f)
            w = bbox[2] - bbox[0]
        except AttributeError:
            w = d.textlength(text, font=f)
        d.text(((W - w) / 2, y), text, font=f, fill=fill)

    # 5. Top Center Branding, Clock, Date, and Location Pill (matching Reference Collage!)
    draw_celestial_logo(d, W // 2, 85, size=46)
    centered("LUNARA OS", 120, font(28, bold=True), (0xFD, 0xF2, 0xF6, 250))
    centered("Your space. Your workflow.", 160, font(13), (0xF2, 0xA7, 0xBF, 215))

    # Large Clock
    centered("23:07", 195, font(74, bold=True), (0xFD, 0xF2, 0xF6, 250))
    # Full Date
    centered("Monday, 16 November 2026", 285, font(15), (0xC9, 0x89, 0x9E, 230))

    # Location / Status Pill: ☁ 24°C • Bangalore
    pill_w, pill_h = 210, 30
    pill_x = (W - pill_w) // 2
    pill_y = 320
    draw_card(d, (pill_x, pill_y, pill_x + pill_w, pill_y + pill_h), radius=15,
              fill=(0x22, 0x16, 0x2A, 190), outline=(0xD6, 0x5D, 0x8C, 90), width=1)
    d.text((pill_x + 28, pill_y + 6), "☁  24°C  •  Bangalore", font=font(11), fill=(0xFD, 0xF2, 0xF6, 220))

    # 6. Center Row: Three Floating Cards
    card_y = 390
    card_h = 430

    # === LEFT CARD: System Status ===
    cl_w = 270
    cl_x = int(W * 0.12)
    draw_card(d, (cl_x, card_y, cl_x + cl_w, card_y + card_h), radius=18,
              fill=(0x1A, 0x12, 0x22, 215), outline=(0xD6, 0x5D, 0x8C, 110), width=1)
    d.text((cl_x + 22, card_y + 22), "System Status", font=font(13, bold=True), fill=(0xFD, 0xF2, 0xF6, 240))

    # Bars for CPU, RAM, Disk, Battery
    vitals_data = [
        ("CPU", "42%", 0.42, "▣"),
        ("RAM", "17%", 0.17, "▤"),
        ("Disk", "1%", 0.01, "💿"),
        ("Battery", "100%", 1.00, "🔋"),
    ]
    bar_y = card_y + 65
    for label, val_str, frac, icon in vitals_data:
        d.text((cl_x + 22, bar_y), f"{icon}  {label}", font=font(11, bold=True), fill=(0xF2, 0xA7, 0xBF, 230))
        d.text((cl_x + cl_w - 60, bar_y), val_str, font=font(11), fill=(0xFD, 0xF2, 0xF6, 220))
        # Background bar
        bg_bar = (cl_x + 22, bar_y + 24, cl_x + cl_w - 22, bar_y + 30)
        draw_card(d, bg_bar, radius=3, fill=(0x38, 0x22, 0x3E, 200), outline=(0, 0, 0, 0))
        # Fill bar
        fill_w = max(4, int((cl_w - 44) * frac))
        fg_bar = (cl_x + 22, bar_y + 24, cl_x + 22 + fill_w, bar_y + 30)
        draw_card(d, fg_bar, radius=3, fill=(0xD6, 0x5D, 0x8C, 240), outline=(0, 0, 0, 0))
        bar_y += 55

    # Mini live footer in System Status
    d.text((cl_x + 22, card_y + card_h - 45), "• NEXUS Micro-OS 6.12", font=font(10), fill=(0xC9, 0x89, 0x9E, 190))
    d.text((cl_x + 22, card_y + card_h - 26), "• All 68 checks passed", font=font(10), fill=(0xC9, 0x89, 0x9E, 190))

    # === RIGHT CARD: Quick Info ===
    cr_w = 270
    cr_x = W - int(W * 0.12) - cr_w
    draw_card(d, (cr_x, card_y, cr_x + cr_w, card_y + card_h), radius=18,
              fill=(0x1A, 0x12, 0x22, 215), outline=(0xD6, 0x5D, 0x8C, 110), width=1)
    d.text((cr_x + 22, card_y + 22), "Quick Info", font=font(13, bold=True), fill=(0xFD, 0xF2, 0xF6, 240))

    info_data = [
        ("Wi-Fi", "LUNARA-Net", "📶"),
        ("Uptime", "6m 58s", "⏱"),
        ("Session", "XFCE", "🖥"),
        ("Status", "Secured", "🛡"),
    ]
    info_y = card_y + 65
    for label, val_str, icon in info_data:
        d.text((cr_x + 22, info_y), f"{icon}  {label}", font=font(11, bold=True), fill=(0xF2, 0xA7, 0xBF, 230))
        d.text((cr_x + 22, info_y + 20), val_str, font=font(11), fill=(0xFD, 0xF2, 0xF6, 220))
        info_y += 55

    d.text((cr_x + 22, card_y + card_h - 45), "• Secure Linux PAM Auth", font=font(10), fill=(0xC9, 0x89, 0x9E, 190))
    d.text((cr_x + 22, card_y + card_h - 26), "• Super + L to lock", font=font(10), fill=(0xC9, 0x89, 0x9E, 190))

    # === CENTER CARD: (LightDM GTK Greeter overlays here) ===
    # Draw soft backdrop border guide so the screen looks fully coherent even when loading
    cc_w = 380
    cc_x = (W - cc_w) // 2
    draw_card(d, (cc_x, card_y, cc_x + cc_w, card_y + card_h), radius=22,
              fill=(0x1C, 0x13, 0x24, 230), outline=(0xD6, 0x5D, 0x8C, 120), width=1)

    # 7. Bottom Quick Controls: Circular pill buttons matching Reference Collage!
    ctrl_icons = ["📶", "🔊", "🔆", "⌨", "♿", "⏻"]
    btn_rad = 22
    total_btns = len(ctrl_icons)
    spacing = 56
    start_bx = (W - (total_btns - 1) * spacing) // 2
    btn_y = H - 85

    for idx, icon in enumerate(ctrl_icons):
        bx = start_bx + idx * spacing
        draw_card(d, (bx - btn_rad, btn_y - btn_rad, bx + btn_rad, btn_y + btn_rad), radius=btn_rad,
                  fill=(0x22, 0x16, 0x2A, 210), outline=(0xD6, 0x5D, 0x8C, 90), width=1)
        d.text((bx - 8, btn_y - 9), icon, font=font(12), fill=(0xFD, 0xF2, 0xF6, 230))

    return im_with_blossoms.convert("RGB")


def create_avatar(out_path):
    size = 128
    av = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(av)

    # Outer circle in dusty rose / blush pink accent
    d.ellipse((0, 0, size - 1, size - 1), fill=(0x24, 0x17, 0x28, 255), outline=(0xD6, 0x5D, 0x8C, 255), width=3)

    # User silhouette in soft pink
    head_rad = 22
    hx, hy = size // 2, 44
    d.ellipse((hx - head_rad, hy - head_rad, hx + head_rad, hy + head_rad), fill=(0xFD, 0xF2, 0xF6, 255))

    sx, sy = size // 2, 102
    d.chord((sx - 38, sy - 28, sx + 38, sy + 44), 180, 360, fill=(0xFD, 0xF2, 0xF6, 255))

    av.save(out_path)


def main():
    print(f"Generating LUNARA desktop wallpaper -> {out_wall}")
    wp = create_desktop_wallpaper()
    wp.save(out_wall)

    print(f"Generating LUNARA lock screen wallpaper -> {out_lock}")
    lock = create_lock_screen()
    lock.save(out_lock)

    print(f"Generating LUNARA user avatar -> {out_avatar}")
    try:
        create_avatar(out_avatar)
    except Exception as e:
        print(f"avatar warning: {e}")

    print("LUNARA visual assets created successfully.")


if __name__ == "__main__":
    main()
