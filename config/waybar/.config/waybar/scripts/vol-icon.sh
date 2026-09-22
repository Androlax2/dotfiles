#!/usr/bin/env bash
# Waybar image module: the output device as an icon. Headphones while the headset is the
# default sink, otherwise the mute / level icon. pactl blocks while pipewire is still starting
# at login, and image scripts run synchronously at bar startup, so every call gets a timeout;
# no output hides the icon until the next poll or signal.
ICONS="$HOME/.config/waybar/icons"
HEADSET="alsa_output.usb-SteelSeries_Arctis_Nova_Pro-00.analog-stereo"

sink=$(timeout 0.3 pactl get-default-sink 2>/dev/null) || exit 0
if [[ "$sink" == "$HEADSET" ]]; then
    echo "$ICONS/headphones.svg"
    exit 0
fi

if timeout 0.3 pactl get-sink-mute @DEFAULT_SINK@ 2>/dev/null | grep -q "yes"; then
    echo "$ICONS/volume-mute.svg"
    exit 0
fi

volume=$(timeout 0.3 pactl get-sink-volume @DEFAULT_SINK@ 2>/dev/null | grep -o -m1 '[0-9]*%' | head -1 | tr -d '%')
[[ -n "$volume" ]] || exit 0
if (( volume < 34 )); then
    echo "$ICONS/volume-low.svg"
elif (( volume < 67 )); then
    echo "$ICONS/volume-mid.svg"
else
    echo "$ICONS/volume-high.svg"
fi
