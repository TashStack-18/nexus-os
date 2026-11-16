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
    --no-install-recommends install xfce4-screensaver fonts-dejavu-core papirus-icon-theme librsvg2-common conky-std python3 python3-gi python3-gi-cairo gir1.2-gtk-3.0 >/dev/null

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
install -m 0755 "$HERE/theme/microos-lab" "$R/usr/local/bin/microos-lab"
cat > "$R/usr/share/applications/microos-lab.desktop" <<CONF
[Desktop Entry]
Type=Application
Name=Scheduler Lab
Comment=Fairness vs throughput (Micro-OS research) on this kernel
Exec=microos-lab
Icon=applications-science
Categories=System;
CONF
mk_launcher 16 "Scheduler Lab" "microos-lab" "applications-science"
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
      <property name="length" type="uint" value="34"/>
      <property name="length-adjust" type="bool" value="true"/>
      <property name="size" type="uint" value="56"/>
      <property name="icon-size" type="uint" value="40"/>
      <property name="background-style" type="uint" value="1"/>
      <property name="background-rgba" type="array"><value type="double" value="0.07"/><value type="double" value="0.09"/><value type="double" value="0.16"/><value type="double" value="0.8"/></property>
      <property name="plugin-ids" type="array">
        <value type="int" value="11"/><value type="int" value="12"/><value type="int" value="13"/><value type="int" value="14"/><value type="int" value="16"/><value type="int" value="15"/>
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
    $(launcher 12)
    $(launcher 13)
    $(launcher 14)
    $(launcher 15)
    $(launcher 16)
  </property>
</channel>
CONF
# Desktop widgets (conky): big clock, system card, scheduler card, like the NEXUS desktop
C="$H/.config/conky"; A="$H/.config/autostart"
mkdir -p "$C" "$A"
cat > "$C/clock.conf" <<'CONF'
conky.config = {
  alignment = 'top_middle', gap_y = 44, minimum_width = 520, maximum_width = 520,
  own_window = true, own_window_type = 'desktop', own_window_transparent = true,
  own_window_argb_visual = true, own_window_argb_value = 0, own_window_colour = '000000',
  own_window_hints = 'undecorated,below,sticky,skip_taskbar,skip_pager',
  double_buffer = true, update_interval = 1, use_xft = true,
  default_color = 'F6F8FC', draw_shades = true, default_shade_color = '000000',
}
conky.text = [[
${alignc}${font DejaVu Sans:size=16}${time %A, %-d %B}
${voffset -6}${alignc}${font DejaVu Sans:size=88:bold}${time %H:%M}
${voffset -10}${alignc}${font DejaVu Sans:size=11}${color A7B2C8}Micro-OS Linux · up ${uptime}
]]
CONF
cat > "$C/system.conf" <<'CONF'
conky.config = {
  alignment = 'top_left', gap_x = 20, gap_y = 48, minimum_width = 232, maximum_width = 232,
  own_window = true, own_window_type = 'desktop', own_window_argb_visual = true,
  own_window_argb_value = 150, own_window_colour = '0A1128',
  own_window_hints = 'undecorated,below,sticky,skip_taskbar,skip_pager',
  double_buffer = true, update_interval = 1, use_xft = true, font = 'DejaVu Sans:size=10',
  default_color = 'F6F8FC', border_inner_margin = 16, border_outer_margin = 0,
}
conky.text = [[
${color A7B2C8}${font DejaVu Sans:size=9:bold}SYSTEM${font}${color}
${voffset 6}CPU ${alignr}${cpu cpu0}%
${cpubar cpu0 6,232}
${voffset 4}Memory ${alignr}${memperc}%
${membar 6,232}
${voffset 4}Load ${alignr}${loadavg}
Tasks ${alignr}${running_processes} / ${processes}
${voffset 10}${color A7B2C8}${font DejaVu Sans:size=9:bold}${time %B %Y}${font}${color}
${font DejaVu Sans Mono:size=10}${execpi 3600 python3 -c "import calendar,datetime as d; t=d.date.today(); print('\n'.join(calendar.TextCalendar(0).formatmonth(t.year,t.month).splitlines()[1:]))"}
]]
CONF
cat > "$C/sched.conf" <<'CONF'
conky.config = {
  alignment = 'top_right', gap_x = 20, gap_y = 48, minimum_width = 232, maximum_width = 232,
  own_window = true, own_window_type = 'desktop', own_window_argb_visual = true,
  own_window_argb_value = 150, own_window_colour = '0A1128',
  own_window_hints = 'undecorated,below,sticky,skip_taskbar,skip_pager',
  double_buffer = true, update_interval = 1, use_xft = true, font = 'DejaVu Sans:size=10',
  default_color = 'F6F8FC', border_inner_margin = 16, border_outer_margin = 0,
}
conky.text = [[
${color A7B2C8}${font DejaVu Sans:size=9:bold}SCHEDULER${font}${color}
${voffset 6}${font DejaVu Sans:size=13:bold}Linux EEVDF${font}
${color A7B2C8}${kernel} · preemptive
${voffset 8}${color}Context switches ${alignr}${execi 1 awk '/^ctxt/{print $2}' /proc/stat > /tmp/.ctx1; echo $(( $(cat /tmp/.ctx1) - $(cat /tmp/.ctx0 2>/dev/null || cat /tmp/.ctx1) )); cp /tmp/.ctx1 /tmp/.ctx0}/s
Runnable tasks ${alignr}${running_processes}
CPU cores ${alignr}${exec nproc}
${voffset 8}${color 34D1BF}Scheduler Lab (dock): fairness
${color 34D1BF}vs throughput for FCFS, SJF,
${color 34D1BF}SRTF, HRRN, RR and Linux.
]]
CONF
for n in clock system sched; do
cat > "$A/conky-$n.desktop" <<CONF
[Desktop Entry]
Type=Application
Name=NEXUS widget $n
Exec=sh -c "sleep 3; conky -c /home/student/.config/conky/$n.conf"
X-GNOME-Autostart-enabled=true
CONF
done
# Use the largest screen mode (up to 1920 wide) as soon as the desktop starts
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
chroot "$R" chown -R student:student /home/student
log "done (GTK theme $GTK)"
