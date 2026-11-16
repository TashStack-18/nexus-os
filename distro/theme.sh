#!/usr/bin/env bash
# Configure the modern, clean NEXUS OS desktop experience:
# - Slim top panel with NEXUS menu, centered date & clock, systray, and lock/power controls.
# - Centered bottom application dock with launchers + icon-only active/minimized tasklist.
# - Clean, minimal desktop with beautiful wallpaper and NO permanent dashboard widgets.
# - Full-screen NEXUS lock screen with digital clock, branding, user avatar, and PAM auth.
# - Working window lifecycle (minimize, maximize, restore, close) with zero window freezing.
set -euo pipefail
R="$1"
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(dirname "$HERE")"
PROXY="http://127.0.0.1:3142"
log() { echo "[theme] $*"; }

log "installing necessary UI packages"
chroot "$R" env DEBIAN_FRONTEND=noninteractive apt-get -o Acquire::http::Proxy="$PROXY" -qq -y \
    --no-install-recommends install xfce4-screensaver fonts-dejavu-core papirus-icon-theme librsvg2-common mupdf python3 python3-gi python3-gi-cairo gir1.2-gtk-3.0 >/dev/null 2>&1 || true

log "rendering modern wallpaper, lock-screen background, and user avatar"
mkdir -p "$R/usr/share/backgrounds" "$R/usr/share/pixmaps"
python3 "$HERE/theme/make_art.py" "$R/usr/share/backgrounds/nexus.png" "$R/usr/share/backgrounds/nexus-lock.png" "$R/usr/share/pixmaps/nexus-avatar.png"

GTK=Adwaita-dark
[ -d "$R/usr/share/themes/Adwaita-dark" ] || GTK=Adwaita

log "configuring LightDM login and lock screen"
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
font-name=DejaVu Sans 11
clock-format=%A, %d %B   %H:%M
indicators=~host;~spacer;~clock;~spacer;~session;~a11y;~power
user-background=false
position=50%,center
default-user-image=/usr/share/pixmaps/nexus-avatar.png
screensaver-timeout=300
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

# 4. Lock Screen action launcher
cat > "$R/usr/share/applications/nexus-lock.desktop" <<CONF
[Desktop Entry]
Version=1.0
Type=Application
Name=Lock Screen
Comment=Lock the current session
Exec=xflock4
Icon=system-lock-screen
Terminal=false
Categories=System;
StartupNotify=false
CONF

log "configuring student account and desktop environment"
H="$R/home/student"
X="$H/.config/xfce4/xfconf/xfce-perchannel-xml"
P="$H/.config/xfce4/panel"
A="$H/.config/autostart"
mkdir -p "$X" "$P" "$A" "$H/.config/gtk-3.0" "$H/nexus_rust_projects"

# Remove any old Conky desktop widget autostarts to keep the wallpaper 100% clean
rm -f "$A"/conky*.desktop 2>/dev/null || true
rm -rf "$H/.config/conky" 2>/dev/null || true

GTKINI="[Settings]\ngtk-application-prefer-dark-theme=true\ngtk-theme-name=$GTK\ngtk-icon-theme-name=Papirus-Dark\ngtk-font-name=DejaVu Sans 10\n"
printf "$GTKINI" > "$H/.config/gtk-3.0/settings.ini"
mkdir -p "$R/etc/gtk-3.0"
printf "$GTKINI" > "$R/etc/gtk-3.0/settings.ini"
chroot "$R" gtk-update-icon-cache -q -f /usr/share/icons/Papirus-Dark 2>/dev/null || true

# Configure Xfwm4 Window Manager:
# - Compositing enabled (eliminates black borders/tearing)
# - button_layout="O|HMC" guarantees Minimize (H), Maximize (M), Close (C) buttons on all windows!
cat > "$X/xfwm4.xml" <<CONF
<?xml version="1.0" encoding="UTF-8"?>
<channel name="xfwm4" version="1.0">
  <property name="general" type="empty">
    <property name="use_compositing" type="bool" value="true"/>
    <property name="button_layout" type="string" value="O|HMC"/>
    <property name="button_offset" type="int" value="0"/>
    <property name="shadow_delta_height" type="int" value="0"/>
    <property name="shadow_delta_width" type="int" value="0"/>
    <property name="shadow_delta_opacity" type="int" value="50"/>
    <property name="show_dock_shadow" type="bool" value="false"/>
    <property name="show_frame_shadow" type="bool" value="true"/>
    <property name="theme" type="string" value="Default-hdpi"/>
    <property name="title_font" type="string" value="DejaVu Sans Bold 9"/>
    <property name="activate_action" type="string" value="bring"/>
  </property>
</channel>
CONF

# System settings
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

# Desktop wallpaper (clean, minimal, no dashboard widgets)
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

launcher() { echo "<property name=\"plugin-$1\" type=\"string\" value=\"launcher\"><property name=\"items\" type=\"array\"><value type=\"string\" value=\"$1.desktop\"/></property></property>"; }

# Panels:
# Panel 1: Slim top bar (size=28) with NEXUS menu, centered clock & date, systray, actions (Lock/Power)
# Panel 2: Centered bottom dock (size=52) with launchers + icon-only active/minimized tasklist!
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
      <property name="size" type="uint" value="28"/>
      <property name="background-style" type="uint" value="1"/>
      <property name="background-rgba" type="array"><value type="double" value="0.04"/><value type="double" value="0.06"/><value type="double" value="0.12"/><value type="double" value="0.88"/></property>
      <property name="plugin-ids" type="array">
        <value type="int" value="1"/><value type="int" value="2"/><value type="int" value="3"/><value type="int" value="4"/><value type="int" value="5"/><value type="int" value="6"/><value type="int" value="7"/>
      </property>
    </property>
    <property name="panel-2" type="empty">
      <property name="position" type="string" value="p=10;x=0;y=0"/>
      <property name="position-locked" type="bool" value="true"/>
      <property name="length-adjust" type="bool" value="true"/>
      <property name="size" type="uint" value="52"/>
      <property name="icon-size" type="uint" value="38"/>
      <property name="background-style" type="uint" value="1"/>
      <property name="background-rgba" type="array"><value type="double" value="0.07"/><value type="double" value="0.09"/><value type="double" value="0.16"/><value type="double" value="0.82"/></property>
      <property name="plugin-ids" type="array">
        <value type="int" value="11"/><value type="int" value="17"/><value type="int" value="16"/><value type="int" value="18"/><value type="int" value="12"/><value type="int" value="14"/><value type="int" value="20"/><value type="int" value="21"/>
      </property>
    </property>
  </property>
  <property name="plugins" type="empty">
    <property name="plugin-1" type="string" value="applicationsmenu">
      <property name="button-title" type="string" value=" NEXUS"/>
      <property name="button-icon" type="string" value="system-run"/>
      <property name="show-button-title" type="bool" value="true"/>
      <property name="show-menu-icons" type="bool" value="true"/>
    </property>
    <property name="plugin-2" type="string" value="separator"><property name="expand" type="bool" value="false"/><property name="style" type="uint" value="0"/></property>
    <property name="plugin-3" type="string" value="separator"><property name="expand" type="bool" value="true"/><property name="style" type="uint" value="0"/></property>
    <property name="plugin-4" type="string" value="clock">
      <property name="mode" type="uint" value="2"/>
      <property name="digital-layout" type="uint" value="3"/>
      <property name="digital-time-format" type="string" value="%a, %b %-d   %H:%M"/>
    </property>
    <property name="plugin-5" type="string" value="separator"><property name="expand" type="bool" value="true"/><property name="style" type="uint" value="0"/></property>
    <property name="plugin-6" type="string" value="systray"/>
    <property name="plugin-7" type="string" value="actions">
      <property name="appearance" type="uint" value="0"/>
      <property name="items" type="array">
        <value type="string" value="+lock-screen"/>
        <value type="string" value="+switch-user"/>
        <value type="string" value="+logout"/>
      </property>
    </property>
    $(launcher 11)
    $(launcher 17)
    $(launcher 16)
    $(launcher 18)
    $(launcher 12)
    $(launcher 14)
    <property name="plugin-20" type="string" value="separator"><property name="expand" type="bool" value="false"/><property name="style" type="uint" value="1"/></property>
    <property name="plugin-21" type="string" value="tasklist">
      <property name="grouping" type="uint" value="1"/>
      <property name="show-labels" type="bool" value="false"/>
      <property name="flat-buttons" type="bool" value="true"/>
      <property name="show-handle" type="bool" value="false"/>
      <property name="show-tooltips" type="bool" value="true"/>
      <property name="window-scrolling" type="bool" value="false"/>
      <property name="include-all-workspaces" type="bool" value="true"/>
    </property>
  </property>
</channel>
CONF

# Configure xfce4-screensaver (Inactivity lock after 5 minutes + secure PAM lock)
cat > "$X/xfce4-screensaver.xml" <<CONF
<?xml version="1.0" encoding="UTF-8"?>
<channel name="xfce4-screensaver" version="1.0">
  <property name="saver" type="empty">
    <property name="enabled" type="bool" value="true"/>
    <property name="idle-activation" type="empty">
      <property name="enabled" type="bool" value="true"/>
      <property name="delay" type="int" value="5"/>
    </property>
  </property>
  <property name="lock" type="empty">
    <property name="enabled" type="bool" value="true"/>
    <property name="saver-activation" type="empty">
      <property name="enabled" type="bool" value="true"/>
      <property name="delay" type="int" value="0"/>
    </property>
  </property>
</channel>
CONF

# Autostart xfce4-screensaver in user session
cat > "$A/xfce4-screensaver.desktop" <<CONF
[Desktop Entry]
Type=Application
Name=Screensaver
Comment=Launch screensaver and locker daemon
Exec=xfce4-screensaver
OnlyShowIn=XFCE;
CONF

# Configure keyboard shortcut: Super + L (and Ctrl+Alt+L) to lock screen immediately
cat > "$X/xfce4-keyboard-shortcuts.xml" <<CONF
<?xml version="1.0" encoding="UTF-8"?>
<channel name="xfce4-keyboard-shortcuts" version="1.0">
  <property name="commands" type="empty">
    <property name="custom" type="empty">
      <property name="&lt;Super&gt;l" type="string" value="xflock4"/>
      <property name="&lt;Primary&gt;&lt;Alt&gt;l" type="string" value="xflock4"/>
      <property name="&lt;Super&gt;t" type="string" value="xfce4-terminal"/>
      <property name="&lt;Super&gt;r" type="string" value="nexus-studio"/>
      <property name="&lt;Super&gt;m" type="string" value="nexus-tasks"/>
    </property>
  </property>
</channel>
CONF

# Screen resolution auto-detection
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
button.suggested-action, button.primary { background-color: #3366ff; color: #fff; border-color: #4d79ff; }
entry, textview, treeview.view { background-color: #0e1424; color: #e8edf8; border-radius: 6px; }
headerbar, .titlebar { background-image: none; background-color: #0e1424; color: #e8edf8; border: none; }
scrollbar slider { background-color: #293860; border-radius: 6px; min-width: 8px; }
menu, .menu, menuitem { background-color: #12182b; color: #e8edf8; }
menuitem:hover { background-color: #3366ff; }
CONF

chroot "$R" chown -R student:student /home/student
log "NEXUS theme, lock screen, and desktop configured successfully."
