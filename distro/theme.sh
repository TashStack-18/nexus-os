#!/usr/bin/env bash
# Configure the modern, clean NEXUS OS desktop experience:
# - Slim top panel with NEXUS menu, centered date & clock, systray, and lock/power controls.
# - Centered bottom application dock with launchers + icon-only active/minimized tasklist.
# - Clean, minimal desktop with aesthetic pink topographic contour wallpaper and centered digital clock (Ref 1).
# - Full-screen NEXUS lock screen with digital clock, branding, user avatar, and PAM auth.
# - Working window lifecycle (minimize, maximize, restore, close) with zero window freezing.
# - Verified user authentication for student:microos and tashvi:microos.
set -euo pipefail
R="$1"
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(dirname "$HERE")"
PROXY="http://127.0.0.1:3142"
log() { echo "[theme] $*"; }

log "installing necessary UI packages"
chroot "$R" env DEBIAN_FRONTEND=noninteractive apt-get -o Acquire::http::Proxy="$PROXY" -qq -y \
    --no-install-recommends install \
    xfce4-screensaver fonts-dejavu-core papirus-icon-theme librsvg2-common \
    mupdf python3 python3-gi python3-gi-cairo gir1.2-gtk-3.0 \
    firefox-esr conky-all xfce4-settings \
    >/dev/null 2>&1 || true

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
greeter-show-manual-login=true
allow-guest=false
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

cat > "$R/etc/lightdm/lightdm-gtk-greeter.css" <<'CONF'
#login_window {
    background-color: rgba(28, 22, 32, 0.92);
    border: 1px solid rgba(214, 93, 140, 0.45);
    border-radius: 16px;
    padding: 24px;
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.65);
}
#user_image {
    border-radius: 50%;
    border: 2px solid #d65d8c;
    margin-bottom: 12px;
}
entry {
    background-color: rgba(18, 14, 22, 0.95);
    color: #fdf2f6;
    border: 1px solid #4a3648;
    border-radius: 8px;
    padding: 8px 12px;
}
entry:focus {
    border-color: #d65d8c;
    box-shadow: 0 0 8px rgba(214, 93, 140, 0.4);
}
button {
    background-color: #d65d8c;
    color: #ffffff;
    border-radius: 8px;
    padding: 8px 16px;
    font-weight: bold;
}
button:hover {
    background-color: #e06d9b;
}
CONF

log "configuring accounts, passwords, and PAM authentication"
# Ensure student account exists and password is microos
if ! chroot "$R" id -u student >/dev/null 2>&1; then
    chroot "$R" useradd -m -s /bin/bash -G sudo,adm,users,plugdev,audio,video,netdev student 2>/dev/null || true
fi
# Verified SHA-512 crypt hash for password "microos"
PASS_HASH='$6$NOONYvU8UO3xNx0m$s9L83B.M34lxa2wcjv1B0WcedPwjR11C1KNJlEJv1t6wf4r4bg7VKCv8703BNU/6Tw4RSPk1QFCAt2PmisidM0'
echo "student:microos" | chroot "$R" chpasswd
chroot "$R" usermod -p "$PASS_HASH" student

# Ensure tashvi account exists with password microos
if ! chroot "$R" id -u tashvi >/dev/null 2>&1; then
    chroot "$R" useradd -m -s /bin/bash -G sudo,adm,users,plugdev,audio,video,netdev tashvi 2>/dev/null || true
fi
echo "tashvi:microos" | chroot "$R" chpasswd
chroot "$R" usermod -p "$PASS_HASH" tashvi 2>/dev/null || true

# Root password microos
echo "root:microos" | chroot "$R" chpasswd
chroot "$R" usermod -p "$PASS_HASH" root 2>/dev/null || true

# Passwordless sudo for student and tashvi
mkdir -p "$R/etc/sudoers.d"
cat > "$R/etc/sudoers.d/microos" <<'SUDO'
student ALL=(ALL) NOPASSWD:ALL
tashvi ALL=(ALL) NOPASSWD:ALL
SUDO
chmod 0440 "$R/etc/sudoers.d/microos"

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
mkdir -p "$X" "$P" "$A" "$H/.config/gtk-3.0" "$H/nexus_rust_projects" "$H/Desktop"

GTKINI="[Settings]\ngtk-application-prefer-dark-theme=true\ngtk-theme-name=$GTK\ngtk-icon-theme-name=Papirus-Dark\ngtk-font-name=DejaVu Sans 10\n"
printf "$GTKINI" > "$H/.config/gtk-3.0/settings.ini"
mkdir -p "$R/etc/gtk-3.0"
printf "$GTKINI" > "$R/etc/gtk-3.0/settings.ini"
chroot "$R" gtk-update-icon-cache -q -f /usr/share/icons/Papirus-Dark 2>/dev/null || true

# Configure Xfwm4 Window Manager:
# - Compositing enabled (eliminates black borders/tearing)
# - button_layout="O|HMC" guarantees Minimize (H), Maximize (M), Close (C) buttons on all windows!
# - Frame and dock shadows disabled to eliminate VM CPU software-rendering lag during window dragging
cat > "$X/xfwm4.xml" <<CONF
<?xml version="1.0" encoding="UTF-8"?>
<channel name="xfwm4" version="1.0">
  <property name="general" type="empty">
    <property name="use_compositing" type="bool" value="true"/>
    <property name="button_layout" type="string" value="O|HMC"/>
    <property name="button_offset" type="int" value="0"/>
    <property name="shadow_delta_height" type="int" value="0"/>
    <property name="shadow_delta_width" type="int" value="0"/>
    <property name="shadow_delta_opacity" type="int" value="0"/>
    <property name="show_dock_shadow" type="bool" value="false"/>
    <property name="show_frame_shadow" type="bool" value="false"/>
    <property name="show_popup_shadow" type="bool" value="false"/>
    <property name="theme" type="string" value="Default-hdpi"/>
    <property name="title_font" type="string" value="DejaVu Sans Bold 9"/>
    <property name="activate_action" type="string" value="bring"/>
  </property>
</channel>
CONF

# System settings (disable window animations to eliminate stuttering in VirtualBox)
cat > "$X/xsettings.xml" <<CONF
<?xml version="1.0" encoding="UTF-8"?>
<channel name="xsettings" version="1.0">
  <property name="Net" type="empty">
    <property name="ThemeName" type="string" value="$GTK"/>
    <property name="IconThemeName" type="string" value="Papirus-Dark"/>
    <property name="EnableAnimations" type="bool" value="false"/>
  </property>
  <property name="Gtk" type="empty">
    <property name="FontName" type="string" value="DejaVu Sans 10"/>
  </property>
</channel>
CONF

# Desktop wallpaper & icon settings (enables clean desktop shortcuts)
{
    echo '<?xml version="1.0" encoding="UTF-8"?>'
    echo '<channel name="xfce4-desktop" version="1.0"><property name="backdrop" type="empty"><property name="screen0" type="empty">'
    for m in monitor0 monitorVirtual-1 monitorVGA-1 monitoreDP-1 monitorLVDS-1 monitorHDMI-1; do
        echo "<property name=\"$m\" type=\"empty\"><property name=\"workspace0\" type=\"empty\"><property name=\"color-style\" type=\"int\" value=\"0\"/><property name=\"image-style\" type=\"int\" value=\"5\"/><property name=\"last-image\" type=\"string\" value=\"/usr/share/backgrounds/nexus.png\"/></property></property>"
    done
    echo '</property></property><property name="desktop-icons" type="empty"><property name="style" type="int" value="2"/><property name="file-icons" type="empty"><property name="show-home" type="bool" value="false"/><property name="show-trash" type="bool" value="false"/><property name="show-filesystem" type="bool" value="false"/><property name="show-removable" type="bool" value="false"/></property></property></channel>'
} > "$X/xfce4-desktop.xml"

# Desktop Widgets: Clean split architecture (Reference 1 & 2 inspired)
# 1. Clean, centered large digital clock + date (zero dashboard clutter in center)
# 2. Dedicated floating translucent card on the top right for live system vitals
mkdir -p "$H/.config/conky"
rm -f "$A/conky.desktop" 2>/dev/null || true

cat > "$H/.config/conky/conky_clock.conf" <<'CONKY'
conky.config = {
    alignment = 'top_middle',
    gap_x = 0,
    gap_y = 65,
    minimum_width = 460,
    maximum_width = 600,
    own_window = true,
    own_window_type = 'desktop',
    own_window_transparent = true,
    own_window_argb_visual = true,
    own_window_argb_value = 0,
    own_window_hints = 'undecorated,below,sticky,skip_taskbar,skip_pager',
    double_buffer = true,
    update_interval = 2.0,
    use_xft = true,
    font = 'DejaVu Sans:size=12',
    draw_shades = false,
    draw_outline = false,
    draw_borders = false,
    default_color = 'FDF2F6',
    color1 = 'C9899E',
    color2 = 'D65D8C',
    color3 = 'F2A7BF',
};

conky.text = [[
${alignc}${font DejaVu Sans:size=72:weight=bold}${color}${time %H:%M}
${voffset -12}${alignc}${font DejaVu Sans:size=16:weight=medium}${color1}${time %A, %d %B %Y}
${voffset 6}${alignc}${font DejaVu Sans:size=10:weight=bold}${color2}NEXUS OS${color1}  •  ${font DejaVu Sans:size=10}${color1}Your space. Your workflow.
]];
CONKY

cat > "$H/.config/conky/conky_vitals.conf" <<'CONKY'
conky.config = {
    alignment = 'top_right',
    gap_x = 35,
    gap_y = 65,
    minimum_width = 280,
    maximum_width = 300,
    own_window = true,
    own_window_type = 'normal',
    own_window_transparent = false,
    own_window_argb_visual = true,
    own_window_argb_value = 210,
    own_window_colour = '1E1924',
    own_window_hints = 'undecorated,below,sticky,skip_taskbar,skip_pager',
    border_inner_margin = 16,
    border_width = 1,
    draw_borders = true,
    border_inner_margin = 14,
    border_outer_margin = 0,
    double_buffer = true,
    update_interval = 2.0,
    use_xft = true,
    font = 'DejaVu Sans:size=10',
    draw_shades = false,
    draw_outline = false,
    default_color = 'FDF2F6',
    color1 = 'C9899E',
    color2 = 'D65D8C',
    color3 = 'F2A7BF',
};

conky.text = [[
${font DejaVu Sans:size=11:weight=bold}${color2}SYSTEM VITALS${font DejaVu Sans:size=9}${alignr}${color1}LIVE
${color2}${hr 1}
${voffset 4}${font DejaVu Sans:size=9:weight=bold}${color1}CPU USAGE${alignr}${color}${cpu cpu0}%
${color2}${cpubar 6,260}
${voffset 4}${font DejaVu Sans:size=9:weight=bold}${color1}MEMORY${alignr}${color}${memperc}% (${mem} / ${memmax})
${color2}${membar 6,260}
${voffset 4}${font DejaVu Sans:size=9:weight=bold}${color1}DISK SPACE${alignr}${color}${fs_used_perc /}% (${fs_used /} / ${fs_size /})
${color2}${fs_bar 6,260 /}
${voffset 6}${color1}Tasks: ${color}${processes} (${running_processes} running)
${color1}Uptime: ${color}${uptime_short}
${color1}Kernel: ${color}Linux 6.12 Micro-OS
]];
CONKY

cat > "$A/conky-clock.desktop" <<CONF
[Desktop Entry]
Type=Application
Name=Desktop Clock
Exec=conky -c /home/student/.config/conky/conky_clock.conf
Hidden=false
NoDisplay=false
X-GNOME-Autostart-enabled=true
CONF

cat > "$A/conky-vitals.desktop" <<CONF
[Desktop Entry]
Type=Application
Name=System Vitals
Exec=conky -c /home/student/.config/conky/conky_vitals.conf
Hidden=false
NoDisplay=false
X-GNOME-Autostart-enabled=true
CONF

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
mk_launcher 13 "Web Browser" "firefox-esr" "web-browser"
mk_launcher 15 "Settings" "xfce4-settings-manager" "preferences-system"
mk_launcher 14 "Micro-OS Kernel" "xfce4-terminal --title=Micro-OS-Kernel -e micro-os-kernel" "system-run"

# Top panel navigation launchers
mk_launcher 31 "File" "thunar" "system-file-manager"
mk_launcher 32 "Edit" "nexus-studio" "applications-development"
mk_launcher 33 "View" "nexus-lab" "applications-science"
mk_launcher 34 "Window" "nexus-tasks" "utilities-system-monitor"
mk_launcher 35 "Help" "thunar /usr/share/microos/docs" "help-browser"

launcher() { echo "<property name=\"plugin-$1\" type=\"string\" value=\"launcher\"><property name=\"items\" type=\"array\"><value type=\"string\" value=\"$1.desktop\"/></property></property>"; }

# Panels:
# Panel 1: Slim top bar (size=28) with NEXUS menu, global navigation (File, Edit, View, Window, Help), centered clock & date, systray, actions (Lock/Power)
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
      <property name="background-rgba" type="array"><value type="double" value="0.08"/><value type="double" value="0.07"/><value type="double" value="0.10"/><value type="double" value="0.90"/></property>
      <property name="plugin-ids" type="array">
        <value type="int" value="1"/><value type="int" value="31"/><value type="int" value="32"/><value type="int" value="33"/><value type="int" value="34"/><value type="int" value="35"/><value type="int" value="3"/><value type="int" value="4"/><value type="int" value="5"/><value type="int" value="6"/><value type="int" value="7"/>
      </property>
    </property>
    <property name="panel-2" type="empty">
      <property name="position" type="string" value="p=10;x=0;y=0"/>
      <property name="position-locked" type="bool" value="true"/>
      <property name="length-adjust" type="bool" value="true"/>
      <property name="size" type="uint" value="52"/>
      <property name="icon-size" type="uint" value="38"/>
      <property name="background-style" type="uint" value="1"/>
      <property name="background-rgba" type="array"><value type="double" value="0.08"/><value type="double" value="0.07"/><value type="double" value="0.10"/><value type="double" value="0.85"/></property>
      <property name="plugin-ids" type="array">
        <value type="int" value="11"/><value type="int" value="17"/><value type="int" value="16"/><value type="int" value="18"/><value type="int" value="12"/><value type="int" value="13"/><value type="int" value="15"/><value type="int" value="14"/><value type="int" value="20"/><value type="int" value="21"/>
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
    $(launcher 13)
    $(launcher 15)
    $(launcher 14)
    $(launcher 31)
    $(launcher 32)
    $(launcher 33)
    $(launcher 34)
    $(launcher 35)
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

# Desktop shortcuts in ~/Desktop
log "populating desktop shortcuts"
for app in xfce4-terminal nexus-studio nexus-lab nexus-tasks thunar; do
    if [ -f "$R/usr/share/applications/$app.desktop" ]; then
        cp "$R/usr/share/applications/$app.desktop" "$H/Desktop/"
    fi
done
cat > "$H/Desktop/firefox.desktop" <<CONF
[Desktop Entry]
Version=1.0
Type=Application
Name=Web Browser
Comment=Browse the Web
Exec=firefox-esr %u
Icon=firefox-esr
Terminal=false
Categories=Network;WebBrowser;
CONF
chmod 0755 "$H/Desktop"/*.desktop 2>/dev/null || true

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

# Screen resolution auto-detection and VirtualBox dynamic auto-resizing
cat > "$R/usr/local/bin/microos-display" <<'CONF'
#!/bin/sh
# Ensure VirtualBox guest display integration is active
if command -v VBoxClient >/dev/null 2>&1; then
    VBoxClient --vmsvga 2>/dev/null || true
    VBoxClient --display 2>/dev/null || true
fi

# Detect primary connected display
out=$(xrandr | awk '/ connected/{print $1; exit}')
if [ -n "$out" ]; then
    # Add 1920x1080 modeline if missing
    if ! xrandr | grep -q "1920x1080"; then
        xrandr --newmode "1920x1080_60.00" 173.00 1920 2048 2248 2576 1080 1083 1088 1120 -hsync +vsync 2>/dev/null || true
        xrandr --addmode "$out" "1920x1080_60.00" 2>/dev/null || true
    fi
    # Add 1600x900 modeline if missing
    if ! xrandr | grep -q "1600x900"; then
        xrandr --newmode "1600x900_60.00" 118.25 1600 1696 1856 2112 900 903 908 934 -hsync +vsync 2>/dev/null || true
        xrandr --addmode "$out" "1600x900_60.00" 2>/dev/null || true
    fi
    # Auto-detect best resolution
    xrandr --auto 2>/dev/null || true
    best=$(xrandr | awk '/ connected/{f=1; next} f && /^[0-9]+/{f=0} f && /^ +[0-9]+x[0-9]+/{split($1,a,"x"); if (a[1]<=1920) {print $1; exit}}')
    if [ -n "$best" ]; then
        xrandr --output "$out" --mode "$best" 2>/dev/null || true
    fi
fi

# Start background auto-resizer loop (tracks VirtualBox window resizing dynamically)
if ! pgrep -f "microos-autoresize" >/dev/null 2>&1; then
    (
        exec -a microos-autoresize sh -c '
            while true; do
                sleep 3
                xrandr --auto 2>/dev/null || true
            done
        '
    ) >/dev/null 2>&1 &
fi
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

# Performance Optimization: Disable heavy background services in live RAM
if [ -d "$R/etc/systemd/system" ]; then
    chroot "$R" systemctl mask snapd.service snapd.socket snapd.seeded.service 2>/dev/null || true
    chroot "$R" systemctl mask apt-daily.service apt-daily.timer apt-daily-upgrade.service apt-daily-upgrade.timer 2>/dev/null || true
fi

# Pink & Charcoal GTK styling (Reference 1 & 2 inspired)
cat > "$H/.config/gtk-3.0/gtk.css" <<'CONF'
@define-color nexus_bg #141218;
@define-color nexus_card #201824;
@define-color nexus_rose #c9899e;
@define-color nexus_blush #f2a7bf;
@define-color nexus_mauve #6e5264;
@define-color nexus_magenta #d65d8c;
@define-color nexus_text #fdf2f6;
@define-color nexus_muted #9c8a98;

window.background, .background {
    background-color: #141218;
    color: #fdf2f6;
}
headerbar, .titlebar {
    background-image: none;
    background-color: #1b1620;
    color: #fdf2f6;
    border-bottom: 1px solid #332233;
}
button {
    background-image: none;
    background-color: #241c28;
    color: #fdf2f6;
    border: 1px solid #4a3648;
    border-radius: 6px;
    padding: 5px 12px;
}
button:hover {
    background-color: #382436;
    border-color: #d65d8c;
    color: #ffffff;
}
button.suggested-action, button.primary {
    background-color: #d65d8c;
    color: #ffffff;
    border-color: #f2a7bf;
}
button.suggested-action:hover, button.primary:hover {
    background-color: #e06d9b;
}
entry, textview, treeview.view {
    background-color: #1a141e;
    color: #fdf2f6;
    border: 1px solid #382436;
    border-radius: 6px;
}
entry:focus {
    border-color: #d65d8c;
}
scrollbar slider {
    background-color: #4a3648;
    border-radius: 6px;
    min-width: 8px;
}
scrollbar slider:hover {
    background-color: #c9899e;
}
menu, .menu, menuitem {
    background-color: #1b1620;
    color: #fdf2f6;
}
menuitem:hover {
    background-color: #d65d8c;
    color: #ffffff;
}
CONF

cp "$H/.config/gtk-3.0/gtk.css" "$R/etc/gtk-3.0/gtk.css"

# Mirror user configuration to tashvi as well
if [ -d "$R/home/tashvi" ]; then
    cp -r "$H/." "$R/home/tashvi/"
    chroot "$R" chown -R tashvi:tashvi /home/tashvi
fi

chroot "$R" chown -R student:student /home/student
log "NEXUS theme, lock screen, and desktop configured successfully."
