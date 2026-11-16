#!/usr/bin/env bash
# Give the Micro-OS Linux desktop the NEXUS look: dark top bar with a NEXUS
# menu and clock, a dock of launchers at the bottom, an original wallpaper, a
# NEXUS login (lock) screen with a clock, and a screen locker.
# Called by build_iso.sh as: theme.sh ROOTFS   (the root's /proc etc. mounted)
set -euo pipefail
R="$1"
HERE="$(cd "$(dirname "$0")" && pwd)"
PROXY="http://127.0.0.1:3142"
log() { echo "[theme] $*"; }

log "installing the screen locker"
chroot "$R" env DEBIAN_FRONTEND=noninteractive apt-get -o Acquire::http::Proxy="$PROXY" -qq -y \
    --no-install-recommends install xfce4-screensaver fonts-dejavu-core papirus-icon-theme librsvg2-common >/dev/null

log "rendering wallpaper and lock-screen background"
mkdir -p "$R/usr/share/backgrounds"
python3 "$HERE/theme/make_art.py" "$R/usr/share/backgrounds/nexus.png" "$R/usr/share/backgrounds/nexus-lock.png"

GTK=Adwaita-dark
[ -d "$R/usr/share/themes/Adwaita-dark" ] || GTK=Adwaita

log "login (lock) screen"
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

log "desktop settings for the student account"
H="$R/home/student"
X="$H/.config/xfce4/xfconf/xfce-perchannel-xml"
P="$H/.config/xfce4/panel"
mkdir -p "$X" "$P" "$H/.config/gtk-3.0"
GTKINI="[Settings]\ngtk-application-prefer-dark-theme=true\ngtk-theme-name=$GTK\ngtk-icon-theme-name=Papirus-Dark\ngtk-font-name=DejaVu Sans 10\n"
printf "$GTKINI" > "$H/.config/gtk-3.0/settings.ini"
mkdir -p "$R/etc/gtk-3.0"
printf "$GTKINI" > "$R/etc/gtk-3.0/settings.ini"
chroot "$R" gtk-update-icon-cache -q -f /usr/share/icons/Papirus-Dark || true

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

# Dock launchers: each is a launcher plugin holding one .desktop item.
icon_path() { # resolve a themed icon name to a file inside the image (else keep the name)
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
mk_launcher 12 "Files" "thunar" "system-file-manager"
mk_launcher 13 "Snap Store" "snap run snap-store" "system-software-install"
mk_launcher 14 "Micro-OS kernel" "xfce4-terminal --title=Micro-OS-kernel -e micro-os-kernel" "utilities-system-monitor"
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
        <value type="int" value="1"/><value type="int" value="2"/><value type="int" value="3"/><value type="int" value="4"/><value type="int" value="6"/><value type="int" value="7"/>
      </property>
    </property>
    <property name="panel-2" type="empty">
      <property name="position" type="string" value="p=10;x=0;y=0"/>
      <property name="position-locked" type="bool" value="true"/>
      <property name="length" type="uint" value="34"/>
      <property name="length-adjust" type="bool" value="true"/>
      <property name="size" type="uint" value="56"/>
      <property name="icon-size" type="uint" value="40"/>
      <property name="background-style" type="uint" value="1"/>
      <property name="background-rgba" type="array"><value type="double" value="0.07"/><value type="double" value="0.09"/><value type="double" value="0.16"/><value type="double" value="0.8"/></property>
      <property name="plugin-ids" type="array">
        <value type="int" value="11"/><value type="int" value="12"/><value type="int" value="13"/><value type="int" value="14"/><value type="int" value="15"/>
      </property>
    </property>
  </property>
  <property name="plugins" type="empty">
    <property name="plugin-1" type="string" value="applicationsmenu">
      <property name="button-title" type="string" value="NEXUS"/>
      <property name="show-button-title" type="bool" value="true"/>
    </property>
    <property name="plugin-2" type="string" value="tasklist"><property name="show-labels" type="bool" value="true"/><property name="grouping" type="uint" value="1"/></property>
    <property name="plugin-3" type="string" value="separator"><property name="expand" type="bool" value="true"/><property name="style" type="uint" value="0"/></property>
    <property name="plugin-4" type="string" value="clock"><property name="mode" type="uint" value="2"/><property name="digital-layout" type="uint" value="3"/><property name="digital-time-format" type="string" value="%A %d %B   %H:%M"/></property>
    <property name="plugin-6" type="string" value="systray"/>
    <property name="plugin-7" type="string" value="actions"/>
    $(launcher 11)
    $(launcher 12)
    $(launcher 13)
    $(launcher 14)
    $(launcher 15)
  </property>
</channel>
CONF
chroot "$R" chown -R student:student /home/student
log "done (GTK theme $GTK)"
