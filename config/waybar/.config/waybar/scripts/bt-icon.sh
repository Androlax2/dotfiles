#!/usr/bin/env bash
# Waybar image module: prints the bluetooth icon path on line 1 and the tooltip on line 2.
ICONS="$HOME/.config/waybar/icons"

if ! bluetoothctl show 2>/dev/null | grep -q "Powered: yes"; then
    printf '%s/bluetooth-off.svg\nbluetooth off\n' "$ICONS"
    exit 0
fi

connected=$(bluetoothctl devices Connected 2>/dev/null | cut -d' ' -f3- | paste -sd ', ')
if [[ -n "$connected" ]]; then
    printf '%s/bluetooth-connected.svg\n%s\n' "$ICONS" "$connected"
else
    printf '%s/bluetooth.svg\nno device connected\n' "$ICONS"
fi
