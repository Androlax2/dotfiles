#!/usr/bin/env bash
# Waybar image module: bluetooth icon path on line 1, tooltip on line 2. State comes from
# rfkill and sysfs (instant); bluetoothctl is only asked for device names, under a timeout,
# because it blocks while bluetoothd is still starting at login.
ICONS="$HOME/.config/waybar/icons"

if rfkill list bluetooth 2>/dev/null | grep -q "blocked: yes"; then
    printf '%s/bluetooth-off.svg\nbluetooth off\n' "$ICONS"
    exit 0
fi

# each connected device shows up as hci0:<handle> under the adapter
connections=$(ls -d /sys/class/bluetooth/hci*/hci*:* 2>/dev/null | wc -l)
if (( connections == 0 )); then
    printf '%s/bluetooth.svg\nno device connected\n' "$ICONS"
    exit 0
fi

names=$(timeout 0.3 bluetoothctl devices Connected 2>/dev/null | cut -d' ' -f3- | paste -sd ', ')
printf '%s/bluetooth-connected.svg\n%s\n' "$ICONS" "${names:-$connections device(s) connected}"
