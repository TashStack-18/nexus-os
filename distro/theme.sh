#!/usr/bin/env bash
# Configure the modern, clean LUNARA OS desktop experience:
# - Matches Primary Reference Collage:
#   * Desktop: Atmospheric midnight plum wallpaper with silky ribbon wave topography, glowing crescent moon,
#     centered LUNARA OS branding, clean digital clock, left shortcut stack, right System Vitals panel, and bottom dock.
#   * Lock Screen: Full-screen cherry blossom moonlit mountain/lake landscape with System Status & Quick Info cards,
#     central login card with real LightDM/PAM authentication, and bottom quick controls.
# - High-DPI subpixel anti-aliasing (resolving blurriness across all virtual and native displays).
# - Working window lifecycle with zero window freezing and lightweight footprint.
# - Verified user authentication for student:microos and tashvi:microos.
set -euo pipefail
R="$1"
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(dirname "$HERE")"
PROXY="http://127.0.0.1:3142"
log() { echo "[lunara-theme] $*"; }

log "installing necessary UI packages"
chroot "$R" env DEBIAN_FRONTEND=noninteractive apt-get -o Acquire::http::Proxy="$PROXY" -qq -y \
    --no-install-recommends install \
    xfce4-screensaver fonts-dejavu-core papirus-icon-theme librsvg2-common \
    mupdf python3 python3-gi python3-gi-cairo gir1.2-gtk-3.0 \
    firefox-esr conky-all xfce4-settings xfce4-appfinder \
    >/dev/null 2>&1 || true

log "rendering modern LUNARA wallpaper, lock-screen background, and user avatar"
mkdir -p "$R/usr/share/backgrounds" "$R/usr/share/pixmaps"
python3 "$HERE/theme/make_art.py" "$R/usr/share/backgrounds/lunara.png" "$R/usr/share/backgrounds/lunara-lock.png" "$R/usr/share/pixmaps/lunara-avatar.png"

# Symlinks for backwards compatibility
ln -sf /usr/share/backgrounds/lunara.png "$R/usr/share/backgrounds/nexus.png" 2>/dev/null || true
ln -sf /usr/share/backgrounds/lunara-lock.png "$R/usr/share/backgrounds/nexus-lock.png" 2>/dev/null || true
ln -sf /usr/share/pixmaps/lunara-avatar.png "$R/usr/share/pixmaps/nexus-avatar.png" 2>/dev/null || true

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
background=/usr/share/backgrounds/lunara-lock.png
theme-name=$GTK
icon-theme-name=Papirus-Dark
font-name=DejaVu Sans 11
clock-format=%A, %d %B   %H:%M
indicators=~host;~spacer;~clock;~spacer;~session;~a11y;~power
user-background=false
position=50%,center
default-user-image=/usr/share/pixmaps/lunara-avatar.png
screensaver-timeout=300
CONF

cat > "$R/etc/lightdm/lightdm-gtk-greeter.css" <<'CONF'
#login_window {
    background-color: rgba(26, 18, 30, 0.92);
    border: 1px solid rgba(214, 93, 140, 0.45);
    border-radius: 18px;
    padding: 24px;
    box-shadow: 0 10px 32px rgba(0, 0, 0, 0.70);
}
#user_image {
    border-radius: 50%;
    border: 2px solid #d65d8c;
    margin-bottom: 12px;
}
entry {
    background-color: rgba(18, 12, 22, 0.95);
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
    padding: 8px 18px;
    font-weight: bold;
}
button:hover {
    background-color: #e06d9b;
}
CONF

log "configuring accounts, passwords, and PAM authentication"
if ! chroot "$R" id -u student >/dev/null 2>&1; then
    chroot "$R" useradd -m -s /bin/bash -G sudo,adm,users,plugdev,audio,video,netdev student 2>/dev/null || true
fi
PASS_HASH='$6$NOONYvU8UO3xNx0m$s9L83B.M34lxa2wcjv1B0WcedPwjR11C1KNJlEJv1t6wf4r4bg7VKCv8703BNU/6Tw4RSPk1QFCAt2PmisidM0'
echo "student:microos" | chroot "$R" chpasswd
chroot "$R" usermod -p "$PASS_HASH" student

if ! chroot "$R" id -u tashvi >/dev/null 2>&1; then
    chroot "$R" useradd -m -s /bin/bash -G sudo,adm,users,plugdev,audio,video,netdev tashvi 2>/dev/null || true
fi
echo "tashvi:microos" | chroot "$R" chpasswd
chroot "$R" usermod -p "$PASS_HASH" tashvi 2>/dev/null || true

echo "root:microos" | chroot "$R" chpasswd
chroot "$R" usermod -p "$PASS_HASH" root 2>/dev/null || true

mkdir -p "$R/etc/sudoers.d"
cat > "$R/etc/sudoers.d/microos" <<'SUDO'
student ALL=(ALL) NOPASSWD:ALL
tashvi ALL=(ALL) NOPASSWD:ALL
SUDO
chmod 0440 "$R/etc/sudoers.d/microos"

log "installing LUNARA Core Applications"
mkdir -p "$R/usr/local/bin" "$R/usr/share/applications" "$R/usr/local/share/nexus/examples"

# 1. Rust Development Studio
install -m 0755 "$HERE/theme/nexus-studio" "$R/usr/local/bin/lunara-studio"
ln -sf /usr/local/bin/lunara-studio "$R/usr/local/bin/nexus-studio" 2>/dev/null || true
install -m 0644 "$HERE/theme/lunara-studio.desktop" "$R/usr/share/applications/lunara-studio.desktop"
ln -sf /usr/share/applications/lunara-studio.desktop "$R/usr/share/applications/nexus-studio.desktop" 2>/dev/null || true

# 2. Unified NEXUS Lab (REMAINS COMPLETELY UNTOUCHED)
install -m 0755 "$HERE/theme/nexus-lab" "$R/usr/local/bin/nexus-lab"
install -m 0755 "$HERE/theme/nexus-lab" "$R/usr/local/bin/microos-lab"
install -m 0644 "$HERE/theme/nexus-lab.desktop" "$R/usr/share/applications/nexus-lab.desktop"
install -m 0644 "$HERE/theme/nexus-lab.desktop" "$R/usr/share/applications/microos-lab.desktop"

# 3. Task Manager & System Monitor
install -m 0755 "$HERE/theme/nexus-tasks" "$R/usr/local/bin/lunara-tasks"
ln -sf /usr/local/bin/lunara-tasks "$R/usr/local/bin/nexus-tasks" 2>/dev/null || true
install -m 0644 "$HERE/theme/lunara-tasks.desktop" "$R/usr/share/applications/lunara-tasks.desktop"
ln -sf /usr/share/applications/lunara-tasks.desktop "$R/usr/share/applications/nexus-tasks.desktop" 2>/dev/null || true

# 4. Floating System Vitals Desktop Widget
install -m 0755 "$HERE/theme/lunara-vitals" "$R/usr/local/bin/lunara-vitals"
install -m 0644 "$HERE/theme/lunara-vitals.desktop" "$R/usr/share/applications/lunara-vitals.desktop"

# 5. Lock Screen launcher
cat > "$R/usr/share/applications/lunara-lock.desktop" <<CONF
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
ln -sf /usr/share/applications/lunara-lock.desktop "$R/usr/share/applications/nexus-lock.desktop" 2>/dev/null || true

log "configuring razor-sharp font rendering and anti-aliasing"
# Configure Fontconfig for crisp, anti-aliased LCD subpixel text across all applications
mkdir -p "$R/etc/fonts"
cat > "$R/etc/fonts/local.conf" <<'CONF'
<?xml version="1.0"?>
<!DOCTYPE fontconfig SYSTEM "fonts.dtd">
<fontconfig>
  <match target="font">
    <edit mode="assign" name="antialias"><bool>true</bool></edit>
    <edit mode="assign" name="hinting"><bool>true</bool></edit>
    <edit mode="assign" name="hintstyle"><const>hintslight</const></edit>
    <edit mode="assign" name="rgba"><const>rgb</const></edit>
    <edit mode="assign" name="lcdfilter"><const>lcddefault</const></edit>
  </match>
</fontconfig>
CONF

log "configuring student account and desktop environment"
H="$R/home/student"
X="$H/.config/xfce4/xfconf/xfce-perchannel-xml"
P="$H/.config/xfce4/panel"
A="$H/.config/autostart"
mkdir -p "$X" "$P" "$A" "$H/.config/gtk-3.0" "$H/nexus_rust_projects" "$H/Desktop"

GTKINI="[Settings]\ngtk-application-prefer-dark-theme=true\ngtk-theme-name=$GTK\ngtk-icon-theme-name=Papirus-Dark\ngtk-font-name=DejaVu Sans 10\ngtk-xft-antialias=1\ngtk-xft-hinting=1\ngtk-xft-hintstyle=hintslight\ngtk-xft-rgba=rgb\n"
printf "$GTKINI" > "$H/.config/gtk-3.0/settings.ini"
mkdir -p "$R/etc/gtk-3.0"
printf "$GTKINI" > "$R/etc/gtk-3.0/settings.ini"
chroot "$R" gtk-update-icon-cache -q -f /usr/share/icons/Papirus-Dark 2>/dev/null || true

# User Xresources for crisp vector rendering in X11
cat > "$H/.Xresources" <<'CONF'
Xft.dpi: 96
Xft.antialias: 1
Xft.hinting: 1
Xft.hintstyle: hintslight
Xft.rgba: rgb
Xft.lcdfilter: lcddefault
CONF

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

# System settings (crisp font rendering + no window animation lag in VirtualBox)
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
    <property name="MonospaceFontName" type="string" value="DejaVu Sans Mono 10"/>
  </property>
  <property name="Xft" type="empty">
    <property name="DPI" type="int" value="96"/>
    <property name="Antialias" type="int" value="1"/>
    <property name="Hinting" type="int" value="1"/>
    <property name="HintStyle" type="string" value="hintslight"/>
    <property name="RGBA" type="string" value="rgb"/>
  </property>
</channel>
CONF

# Desktop wallpaper & icon settings (enables clean desktop shortcuts)
{
    echo '<?xml version="1.0" encoding="UTF-8"?>'
    echo '<channel name="xfce4-desktop" version="1.0"><property name="backdrop" type="empty"><property name="screen0" type="empty">'
    for m in monitor0 monitorVirtual-1 monitorVGA-1 monitoreDP-1 monitorLVDS-1 monitorHDMI-1; do
        echo "<property name=\"$m\" type=\"empty\"><property name=\"workspace0\" type=\"empty\"><property name=\"color-style\" type=\"int\" value=\"0\"/><property name=\"image-style\" type=\"int\" value=\"5\"/><property name=\"last-image\" type=\"string\" value=\"/usr/share/backgrounds/lunara.png\"/></property></property>"
    done
    echo '</property></property><property name="desktop-icons" type="empty"><property name="style" type="int" value="2"/><property name="file-icons" type="empty"><property name="show-home" type="bool" value="false"/><property name="show-trash" type="bool" value="false"/><property name="show-filesystem" type="bool" value="false"/><property name="show-removable" type="bool" value="false"/></property></property></channel>'
} > "$X/xfce4-desktop.xml"

# Desktop Widgets: Split architecture matching Primary Reference Collage
# 1. Clean centered large digital clock + date (aligned between LUNARA OS branding and divider line)
mkdir -p "$H/.config/conky"
rm -f "$A/conky.desktop" 2>/dev/null || true

cat > "$H/.config/conky/conky_clock.conf" <<'CONKY'
conky.config = {
    alignment = 'top_middle',
    gap_x = 0,
    gap_y = 415,
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
${alignc}${font DejaVu Sans:size=56:weight=bold}${color}${time %H:%M}
${voffset 2}${alignc}${font DejaVu Sans:size=14}${color1}${time %A, %d %B %Y}
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

# Autostart the floating System Vitals panel (matches Right Card in Reference Collage)
cat > "$A/lunara-vitals.desktop" <<CONF
[Desktop Entry]
Type=Application
Name=LUNARA System Vitals
Exec=/usr/local/bin/lunara-vitals
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

# Bottom Dock Launchers (Matching Reference Collage Dock)
mk_launcher 10 "Applications" "xfce4-appfinder" "org.xfce.appfinder"
mk_launcher 11 "Terminal" "xfce4-terminal" "utilities-terminal"
mk_launcher 17 "Rust Studio" "lunara-studio" "applications-development"
mk_launcher 16 "NEXUS Lab" "nexus-lab" "applications-science"
mk_launcher 12 "Files" "thunar" "system-file-manager"
mk_launcher 13 "Web Browser" "firefox-esr" "web-browser"
mk_launcher 15 "Settings" "xfce4-settings-manager" "preferences-system"

# Top panel global navigation launchers (Matching Reference Collage Top Bar)
mk_launcher 30 "Applications" "xfce4-appfinder" "org.xfce.appfinder"
mk_launcher 31 "File" "thunar" "system-file-manager"
mk_launcher 32 "Edit" "lunara-studio" "applications-development"
mk_launcher 33 "View" "nexus-lab" "applications-science"
mk_launcher 34 "Go" "thunar /home/student" "user-home"
mk_launcher 35 "Window" "lunara-tasks" "utilities-system-monitor"
mk_launcher 36 "Help" "thunar /usr/share/microos/docs" "help-browser"

launcher() { echo "<property name=\"plugin-$1\" type=\"string\" value=\"launcher\"><property name=\"items\" type=\"array\"><value type=\"string\" value=\"$1.desktop\"/></property></property>"; }

# Panels:
# Panel 1: Slim top bar (size=28) with LUNARA OS menu, global navigation, centered clock, systray, power
# Panel 2: Centered bottom dock (size=52) with launchers + active/minimized tasklist!
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
      <property name="background-rgba" type="array"><value type="double" value="0.08"/><value type="double" value="0.06"/><value type="double" value="0.10"/><value type="double" value="0.90"/></property>
      <property name="plugin-ids" type="array">
        <value type="int" value="1"/><value type="int" value="30"/><value type="int" value="31"/><value type="int" value="32"/><value type="int" value="33"/><value type="int" value="34"/><value type="int" value="35"/><value type="int" value="36"/><value type="int" value="3"/><value type="int" value="4"/><value type="int" value="5"/><value type="int" value="6"/><value type="int" value="7"/>
      </property>
    </property>
    <property name="panel-2" type="empty">
      <property name="position" type="string" value="p=10;x=0;y=0"/>
      <property name="position-locked" type="bool" value="true"/>
      <property name="length-adjust" type="bool" value="true"/>
      <property name="size" type="uint" value="52"/>
      <property name="icon-size" type="uint" value="38"/>
      <property name="background-style" type="uint" value="1"/>
      <property name="background-rgba" type="array"><value type="double" value="0.08"/><value type="double" value="0.06"/><value type="double" value="0.10"/><value type="double" value="0.85"/></property>
      <property name="plugin-ids" type="array">
        <value type="int" value="10"/><value type="int" value="11"/><value type="int" value="17"/><value type="int" value="16"/><value type="int" value="12"/><value type="int" value="13"/><value type="int" value="15"/><value type="int" value="20"/><value type="int" value="21"/>
      </property>
    </property>
  </property>
  <property name="plugins" type="empty">
    <property name="plugin-1" type="string" value="applicationsmenu">
      <property name="button-title" type="string" value=" LUNARA OS"/>
      <property name="button-icon" type="string" value="system-run"/>
      <property name="show-button-title" type="bool" value="true"/>
      <property name="show-menu-icons" type="bool" value="true"/>
    </property>
    <property name="plugin-2" type="string" value="separator"><property name="expand" type="bool" value="false"/><property name="style" type="uint" value="0"/></property>
    <property name="plugin-3" type="string" value="separator"><property name="expand" type="bool" value="true"/><property name="style" type="uint" value="0"/></property>
    <property name="plugin-4" type="string" value="clock">
      <property name="mode" type="uint" value="2"/>
      <property name="digital-layout" type="uint" value="3"/>
      <property name="digital-time-format" type="string" value="%a, %-d %b   %H:%M"/>
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
    $(launcher 10)
    $(launcher 11)
    $(launcher 17)
    $(launcher 16)
    $(launcher 12)
    $(launcher 13)
    $(launcher 15)
    $(launcher 30)
    $(launcher 31)
    $(launcher 32)
    $(launcher 33)
    $(launcher 34)
    $(launcher 35)
    $(launcher 36)
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

# Desktop shortcuts in ~/Desktop (Matching Reference Collage Left Column!)
log "populating desktop shortcuts matching reference collage"
cat > "$H/Desktop/01-home.desktop" <<CONF
[Desktop Entry]
Version=1.0
Type=Application
Name=Home
Exec=thunar /home/student
Icon=user-home
Terminal=false
Categories=FileManager;
CONF

cat > "$H/Desktop/02-studio.desktop" <<CONF
[Desktop Entry]
Version=1.0
Type=Application
Name=Rust Studio
Comment=LUNARA Rust Development Studio
Exec=lunara-studio
Icon=applications-development
Terminal=false
Categories=Development;
CONF

cat > "$H/Desktop/03-lab.desktop" <<CONF
[Desktop Entry]
Version=1.0
Type=Application
Name=NEXUS Lab
Comment=CA-6 Scheduler & Research Lab
Exec=nexus-lab
Icon=applications-science
Terminal=false
Categories=Science;
CONF

cat > "$H/Desktop/04-terminal.desktop" <<CONF
[Desktop Entry]
Version=1.0
Type=Application
Name=Terminal
Exec=xfce4-terminal
Icon=utilities-terminal
Terminal=false
Categories=System;TerminalEmulator;
CONF

cat > "$H/Desktop/05-files.desktop" <<CONF
[Desktop Entry]
Version=1.0
Type=Application
Name=File Manager
Exec=thunar
Icon=system-file-manager
Terminal=false
Categories=System;FileManager;
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
      <property name="&lt;Super&gt;r" type="string" value="lunara-studio"/>
      <property name="&lt;Super&gt;m" type="string" value="lunara-tasks"/>
    </property>
  </property>
</channel>
CONF

# Screen resolution auto-detection and VirtualBox dynamic auto-resizing
cat > "$R/usr/local/bin/lunara-display" <<'CONF'
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
    # Add 1366x768 modeline if missing
    if ! xrandr | grep -q "1366x768"; then
        xrandr --newmode "1366x768_60.00" 85.50 1366 1436 1579 1792 768 771 774 798 -hsync +vsync 2>/dev/null || true
        xrandr --addmode "$out" "1366x768_60.00" 2>/dev/null || true
    fi
    # Auto-detect best resolution
    xrandr --auto 2>/dev/null || true
    best=$(xrandr | awk '/ connected/{f=1; next} f && /^[0-9]+/{f=0} f && /^ +[0-9]+x[0-9]+/{split($1,a,"x"); if (a[1]<=1920) {print $1; exit}}')
    if [ -n "$best" ]; then
        xrandr --output "$out" --mode "$best" 2>/dev/null || true
    fi
fi

# Start background auto-resizer loop (tracks VirtualBox window resizing dynamically)
if ! pgrep -f "lunara-autoresize" >/dev/null 2>&1; then
    (
        exec -a lunara-autoresize sh -c '
            while true; do
                sleep 2
                xrandr --auto 2>/dev/null || true
            done
        '
    ) >/dev/null 2>&1 &
fi
CONF
chmod 0755 "$R/usr/local/bin/lunara-display"
ln -sf /usr/local/bin/lunara-display "$R/usr/local/bin/microos-display" 2>/dev/null || true

cat > "$A/lunara-display.desktop" <<CONF
[Desktop Entry]
Type=Application
Name=Screen mode
Exec=/usr/local/bin/lunara-display
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

# LUNARA Pink & Dark Plum GTK styling
cat > "$H/.config/gtk-3.0/gtk.css" <<'CONF'
@define-color lunara_bg #16101c;
@define-color lunara_card #22162a;
@define-color lunara_rose #c9899e;
@define-color lunara_blush #f2a7bf;
@define-color lunara_mauve #6e5264;
@define-color lunara_magenta #d65d8c;
@define-color lunara_text #fdf2f6;
@define-color lunara_muted #9c8a98;

window.background, .background {
    background-color: #16101c;
    color: #fdf2f6;
}
headerbar, .titlebar {
    background-image: none;
    background-color: #1c1324;
    color: #fdf2f6;
    border-bottom: 1px solid #362238;
}
button {
    background-image: none;
    background-color: #24172c;
    color: #fdf2f6;
    border: 1px solid #4a3250;
    border-radius: 6px;
    padding: 5px 12px;
}
button:hover {
    background-color: #382242;
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
    background-color: #1a1222;
    color: #fdf2f6;
    border: 1px solid #38243c;
    border-radius: 6px;
}
entry:focus {
    border-color: #d65d8c;
}
scrollbar slider {
    background-color: #4a3250;
    border-radius: 6px;
    min-width: 8px;
}
scrollbar slider:hover {
    background-color: #c9899e;
}
menu, .menu, menuitem {
    background-color: #1c1324;
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
log "LUNARA OS theme, lock screen, and desktop configured successfully."
