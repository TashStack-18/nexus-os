#!/usr/bin/env bash
# Give the Micro-OS Linux desktop the NEXUS look: dark top bar with a NEXUS
# menu and clock, a dock of launchers at the bottom, an original wallpaper, a
# NEXUS login (lock) screen with a clock, and a screen locker.
# Called by build_iso.sh as: theme.sh ROOTFS   (the root's /proc etc. mounted)
set -euo pipefail
R="$1"
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(dirname "$HERE")"
PROXY="http://127.0.0.1:3142"
log() { echo "[theme] $*"; }

log "installing necessary UI packages"
chroot "$R" env DEBIAN_FRONTEND=noninteractive apt-get -o Acquire::http::Proxy="$PROXY" -qq -y \
    --no-install-recommends install xfce4-screensaver fonts-dejavu-core papirus-icon-theme librsvg2-common conky-std mupdf python3 python3-gi python3-gi-cairo gir1.2-gtk-3.0 >/dev/null 2>&1 || true

log "rendering wallpaper and lock-screen background"
mkdir -p "$R/usr/share/backgrounds"
python3 "$HERE/theme/make_art.py" "$R/usr/share/backgrounds/nexus.png" "$R/usr/share/backgrounds/nexus-lock.png"

GTK=Adwaita-dark
[ -d "$R/usr/share/themes/Adwaita-dark" ] || GTK=Adwaita

log "configuring login (lock) screen"
mkdir -p "$R/etc/lightdm/lightdm.conf.d"
cat > "$R/etc/lightdm/lightdm.conf.d/50-microos.conf" <<CONF
[Seat:*]
greeter-session=lightdm-gtk-greeter
user-session=xfce
greeter-hide-users=false
greeter-show-manual-login=false
CONF

cat > "$R/etc/lightdm/lightdm-gtk-greeter.conf" <<CONF
[greeter]
background=/usr/share/backgrounds/nexus-lock.png
theme-name=$GTK
icon-theme-name=Papirus-Dark
font-name=DejaVu Sans 12
clock-format=%A, %d %B   %H:%M
indicators=~host;~spacer;~clock;~spacer;~session;~a11y;~power
user-background=false
position=50%,center
CONF

log "installing NEXUS Core Applications"
mkdir -p "$R/usr/local/bin" "$R/usr/share/applications" "$R/usr/local/share/nexus/examples"

# 1. Rust Development Studio
install -m 0755 "$HERE/theme/nexus-studio" "$R/usr/local/bin/nexus-studio"
install -m 0644 "$HERE/theme/nexus-studio.desktop" "$R/usr/share/applications/nexus-studio.desktop"

# 2. Unified NEXUS Lab
install -m 0755 "$HERE/theme/nexus-lab" "$R/usr/local/bin/nexus-lab"
install -m 0755 "$HERE/theme/nexus-lab" "$R/usr/local/bin/microos-lab"
install -m 0644 "$HERE/theme/nexus-lab.desktop" "$R/usr/share/applications/nexus-lab.desktop"
install -m 0644 "$HERE/theme/nexus-lab.desktop" "$R/usr/share/applications/microos-lab.desktop"

# 3. Task Manager & System Monitor
install -m 0755 "$HERE/theme/nexus-tasks" "$R/usr/local/bin/nexus-tasks"
install -m 0644 "$HERE/theme/nexus-tasks.desktop" "$R/usr/share/applications/nexus-tasks.desktop"

log "configuring student account and desktop environment"
H="$R/home/student"
X="$H/.config/xfce4/xfconf/xfce-perchannel-xml"
P="$H/.config/xfce4/panel"
mkdir -p "$X" "$P" "$H/.config/gtk-3.0" "$H/nexus_rust_projects"

GTKINI="[Settings]\ngtk-application-prefer-dark-theme=true\ngtk-theme-name=$GTK\ngtk-icon-theme-name=Papirus-Dark\ngtk-font-name=DejaVu Sans 10\n"
printf "$GTKINI" > "$H/.config/gtk-3.0/settings.ini"
mkdir -p "$R/etc/gtk-3.0"
printf "$GTKINI" > "$R/etc/gtk-3.0/settings.ini"
chroot "$R" gtk-update-icon-cache -q -f /usr/share/icons/Papirus-Dark || true

# Enable Xfce Window Manager Compositing (Eliminates black widget backgrounds!)
cat > "$X/xfwm4.xml" <<CONF
<?xml version="1.0" encoding="UTF-8"?>
<channel name="xfwm4" version="1.0">
  <property name="general" type="empty">
    <property name="use_compositing" type="bool" value="true"/>
    <property name="shadow_delta_height" type="int" value="0"/>
    <property name="shadow_delta_width" type="int" value="0"/>
    <property name="shadow_delta_opacity" type="int" value="50"/>
    <property name="show_dock_shadow" type="bool" value="false"/>
    <property name="show_frame_shadow" type="bool" value="true"/>
    <property name="theme" type="string" value="Default-hdpi"/>
    <property name="title_font" type="string" value="DejaVu Sans Bold 9"/>
  </property>
</channel>
CONF

cat > "$X/xsettings.xml" <<CONF
<?xml version="1.0" encoding="UTF-8"?>
<channel name="xsettings" version="1.0">
  <property name="Net" type="empty">
    <property name="ThemeName" type="string" value="$GTK"/>
    <property name="IconThemeName" type="string" value="Papirus-Dark"/>
  </property>
  <property name="Gtk" type="empty">
    <property name="FontName" type="string" value="DejaVu Sans 10"/>
  </property>
</channel>
CONF

{
    echo '<?xml version="1.0" encoding="UTF-8"?>'
    echo '<channel name="xfce4-desktop" version="1.0"><property name="backdrop" type="empty"><property name="screen0" type="empty">'
    for m in monitor0 monitorVirtual-1 monitorVGA-1 monitoreDP-1 monitorLVDS-1 monitorHDMI-1; do
        echo "<property name=\"$m\" type=\"empty\"><property name=\"workspace0\" type=\"empty\"><property name=\"color-style\" type=\"int\" value=\"0\"/><property name=\"image-style\" type=\"int\" value=\"5\"/><property name=\"last-image\" type=\"string\" value=\"/usr/share/backgrounds/nexus.png\"/></property></property>"
    done
    echo '</property></property><property name="desktop-icons" type="empty"><property name="style" type="int" value="0"/></property></channel>'
} > "$X/xfce4-desktop.xml"

# Dock launchers
icon_path() {
    local f
    f=$(find "$R/usr/share/icons/Papirus/48x48" -name "$1.svg" 2>/dev/null | head -1)
    if [ -n "$f" ]; then echo "${f#"$R"}"; else echo "$1"; fi
}
mk_launcher() { # id name exec icon
    local d="$P/launcher-$1"
    local ic
    ic=$(icon_path "$4")
    mkdir -p "$d"
    cat > "$d/$1.desktop" <<CONF
[Desktop Entry]
Version=1.0
Type=Application
Name=$2
Exec=$3
Icon=$ic
Terminal=false
StartupNotify=true
CONF
}

mk_launcher 11 "Terminal" "xfce4-terminal" "utilities-terminal"
mk_launcher 17 "Rust Studio" "nexus-studio" "applications-development"
mk_launcher 16 "NEXUS Lab" "nexus-lab" "applications-science"
mk_launcher 18 "Task Manager" "nexus-tasks" "utilities-system-monitor"
mk_launcher 12 "Files" "thunar" "system-file-manager"
mk_launcher 14 "Micro-OS Kernel" "xfce4-terminal --title=Micro-OS-kernel -e micro-os-kernel" "system-run"
mk_launcher 15 "Network" "nm-connection-editor" "preferences-system-network"

launcher() { echo "<property name=\"plugin-$1\" type=\"string\" value=\"launcher\"><property name=\"items\" type=\"array\"><value type=\"string\" value=\"$1.desktop\"/></property></property>"; }

cat > "$X/xfce4-panel.xml" <<CONF
<?xml version="1.0" encoding="UTF-8"?>
<channel name="xfce4-panel" version="1.0">
  <property name="configver" type="int" value="2"/>
  <property name="panels" type="array">
    <value type="int" value="1"/>
    <value type="int" value="2"/>
    <property name="dark-mode" type="bool" value="true"/>
    <property name="panel-1" type="empty">
      <property name="position" type="string" value="p=6;x=0;y=0"/>
      <property name="position-locked" type="bool" value="true"/>
      <property name="length" type="uint" value="100"/>
      <property name="size" type="uint" value="30"/>
      <property name="background-style" type="uint" value="1"/>
      <property name="background-rgba" type="array"><value type="double" value="0.03"/><value type="double" value="0.04"/><value type="double" value="0.09"/><value type="double" value="0.85"/></property>
      <property name="plugin-ids" type="array">
        <value type="int" value="1"/><value type="int" value="2"/><value type="int" value="3"/><value type="int" value="4"/><value type="int" value="8"/><value type="int" value="6"/><value type="int" value="7"/>
      </property>
    </property>
    <property name="panel-2" type="empty">
      <property name="position" type="string" value="p=10;x=0;y=0"/>
      <property name="position-locked" type="bool" value="true"/>
      <property name="length" type="uint" value="42"/>
      <property name="length-adjust" type="bool" value="true"/>
      <property name="size" type="uint" value="56"/>
      <property name="icon-size" type="uint" value="40"/>
      <property name="background-style" type="uint" value="1"/>
      <property name="background-rgba" type="array"><value type="double" value="0.07"/><value type="double" value="0.09"/><value type="double" value="0.16"/><value type="double" value="0.8"/></property>
      <property name="plugin-ids" type="array">
        <value type="int" value="11"/><value type="int" value="17"/><value type="int" value="16"/><value type="int" value="18"/><value type="int" value="12"/><value type="int" value="14"/><value type="int" value="15"/>
      </property>
    </property>
  </property>
  <property name="plugins" type="empty">
    <property name="plugin-1" type="string" value="applicationsmenu">
      <property name="button-title" type="string" value="NEXUS"/>
      <property name="show-button-title" type="bool" value="true"/>
    </property>
    <property name="plugin-2" type="string" value="tasklist"><property name="expand" type="bool" value="false"/><property name="flat-buttons" type="bool" value="true"/><property name="show-labels" type="bool" value="true"/><property name="grouping" type="uint" value="1"/></property>
    <property name="plugin-3" type="string" value="separator"><property name="expand" type="bool" value="true"/><property name="style" type="uint" value="0"/></property>
    <property name="plugin-4" type="string" value="clock"><property name="mode" type="uint" value="2"/><property name="digital-layout" type="uint" value="3"/><property name="digital-time-format" type="string" value="%H:%M"/></property>
    <property name="plugin-8" type="string" value="separator"><property name="expand" type="bool" value="true"/><property name="style" type="uint" value="0"/></property>
    <property name="plugin-6" type="string" value="systray"/>
    <property name="plugin-7" type="string" value="actions"/>
    $(launcher 11)
    $(launcher 17)
    $(launcher 16)
    $(launcher 18)
    $(launcher 12)
    $(launcher 14)
    $(launcher 15)
  </property>
</channel>
CONF

# Desktop widgets: Perfectly balanced, non-overlapping, transparent Conky config
C="$H/.config/conky"; A="$H/.config/autostart"
mkdir -p "$C" "$A"

cat > "$C/clock.conf" <<'CONF'
conky.config = {
  alignment = 'top_middle', gap_y = 50, minimum_width = 460, maximum_width = 460,
  own_window = true, own_window_type = 'desktop', own_window_transparent = true,
  own_window_argb_visual = true, own_window_argb_value = 0,
  own_window_hints = 'undecorated,below,sticky,skip_taskbar,skip_pager',
  double_buffer = true, update_interval = 1, use_xft = true,
  default_color = 'F6F8FC', draw_shades = false,
}
conky.text = [[
${alignc}${font DejaVu Sans:size=15}${time %A, %-d %B}
${voffset -4}${alignc}${font DejaVu Sans:size=78:bold}${time %H:%M}
${voffset -8}${alignc}${font DejaVu Sans:size=10}${color A7B2C8}NEXUS OS · Rust Micro-OS Kernel & Debian Desktop
]]
CONF

cat > "$C/system.conf" <<'CONF'
conky.config = {
  alignment = 'top_left', gap_x = 24, gap_y = 50, minimum_width = 220, maximum_width = 220,
  own_window = true, own_window_type = 'desktop', own_window_argb_visual = true,
  own_window_argb_value = 160, own_window_colour = '0A1128',
  own_window_hints = 'undecorated,below,sticky,skip_taskbar,skip_pager',
  double_buffer = true, update_interval = 2, use_xft = true, font = 'DejaVu Sans:size=9',
  default_color = 'F6F8FC', border_inner_margin = 14, border_outer_margin = 0,
}
conky.text = [[
${color 5B8CFF}${font DejaVu Sans:size=10:bold}SYSTEM MONITOR${font}${color}
${voffset 4}CPU Load ${alignr}${cpu cpu0}%
${color 5B8CFF}${cpubar cpu0 5,192}${color}
${voffset 4}Memory ${alignr}${memperc}%
${color 34D1BF}${membar 5,192}${color}
${voffset 4}Load Avg ${alignr}${loadavg 1}
Processes ${alignr}${running_processes} / ${processes}
Uptime ${alignr}${uptime}
${voffset 8}${color 5B8CFF}${font DejaVu Sans:size=9:bold}QUICK LAUNCH${font}${color}
${color E8EDF8}• Rust Studio (Dock)
• NEXUS Lab (Dock)
• Task Manager (Dock)
• Micro-OS Kernel (Dock)
]]
CONF

cat > "$C/sched.conf" <<'CONF'
conky.config = {
  alignment = 'top_right', gap_x = 24, gap_y = 50, minimum_width = 230, maximum_width = 230,
  own_window = true, own_window_type = 'desktop', own_window_argb_visual = true,
  own_window_argb_value = 160, own_window_colour = '0A1128',
  own_window_hints = 'undecorated,below,sticky,skip_taskbar,skip_pager',
  double_buffer = true, update_interval = 2, use_xft = true, font = 'DejaVu Sans:size=9',
  default_color = 'F6F8FC', border_inner_margin = 14, border_outer_margin = 0,
}
conky.text = [[
${color 5B8CFF}${font DejaVu Sans:size=10:bold}SCHEDULER LAB${font}${color}
${voffset 4}${font DejaVu Sans:size=11:bold}Linux EEVDF + Micro-OS${font}
${color A7B2C8}${kernel} · Preemptive
${voffset 6}${color}CPU Cores ${alignr}${exec nproc}
Running Tasks ${alignr}${running_processes}
${voffset 8}${color 34D1BF}Open NEXUS Lab to
compare FCFS, SJF, SRTF,
HRRN, RR and Linux EEVDF.
${voffset 4}${color WARM}CA-4 Research: H1, H2, H3
Fairness vs Throughput.
]]
CONF

for n in clock system sched; do
cat > "$A/conky-$n.desktop" <<CONF
[Desktop Entry]
Type=Application
Name=NEXUS widget $n
Exec=sh -c "sleep 2; conky -c /home/student/.config/conky/$n.conf"
X-GNOME-Autostart-enabled=true
CONF
done

# Screen mode autoset
cat > "$R/usr/local/bin/microos-display" <<'CONF'
#!/bin/sh
out=$(xrandr | awk '/ connected/{print $1; exit}')
mode=$(xrandr | awk '/ connected/{f=1; next} f && /^ +[0-9]+x[0-9]+/{split($1,a,"x"); if (a[1]<=1920) {print $1; exit}}')
[ -n "$out" ] && [ -n "$mode" ] && xrandr --output "$out" --mode "$mode"
CONF
chmod 0755 "$R/usr/local/bin/microos-display"
cat > "$A/microos-display.desktop" <<CONF
[Desktop Entry]
Type=Application
Name=Screen mode
Exec=/usr/local/bin/microos-display
X-GNOME-Autostart-enabled=true
CONF

# Documentation inside the OS
D="$R/usr/share/microos/docs"
mkdir -p "$D"
cp "$ROOT/results/R3 Research - synced with Micro-OS.pdf" "$D/R3-Research.pdf" 2>/dev/null || true
for f in research_sync report writing-programs architecture linux_comparison x86_port; do
    [ -f "$ROOT/docs/$f.md" ] && cp "$ROOT/docs/$f.md" "$D/$f.md"
done
[ -f "$ROOT/results/research_sync.md" ] && cp "$ROOT/results/research_sync.md" "$D/research_results.md"
[ -f "$ROOT/results/distro_boottime.md" ] && cp "$ROOT/results/distro_boottime.md" "$D/boot_times.md"
mkdir -p "$H/Documents"
cp -r "$D/." "$H/Documents/" 2>/dev/null || true

# Performance Optimization: Disable heavy background snap services from auto-refreshing in live RAM
if [ -d "$R/etc/systemd/system" ]; then
    chroot "$R" systemctl mask snapd.seeded.service 2>/dev/null || true
fi

# Dark GTK styling
cat > "$H/.config/gtk-3.0/gtk.css" <<'CONF'
@define-color nexus_bg #0b1020;
@define-color nexus_card #12182b;
@define-color nexus_accent #5b8cff;
window.background, .background { background-color: #0b1020; color: #e8edf8; }
button { background-image: none; background-color: #1c2744; color: #e8edf8; border: 1px solid #293860; border-radius: 6px; padding: 5px 12px; }
button:hover { background-color: #26365e; border-color: #5b8cff; }
button.suggested-action { background-color: #3366ff; color: #fff; border-color: #4d79ff; }
entry, textview, treeview.view { background-color: #0e1424; color: #e8edf8; border-radius: 6px; }
headerbar, .titlebar { background-image: none; background-color: #0e1424; color: #e8edf8; border: none; }
scrollbar slider { background-color: #293860; border-radius: 6px; min-width: 8px; }
menu, .menu, menuitem { background-color: #12182b; color: #e8edf8; }
menuitem:hover { background-color: #3366ff; }
CONF

chroot "$R" chown -R student:student /home/student
log "NEXUS theme and desktop configured successfully."
