#!/usr/bin/env bash
# Waybar image module: the output device as an icon. Headphones while the headset
# is the default sink, otherwise the mute / level icon for the speakers.
ICONS="$HOME/.config/waybar/icons"
HEADSET="alsa_output.usb-SteelSeries_Arctis_Nova_Pro-00.analog-stereo"

if [[ "$(pactl get-default-sink)" == "$HEADSET" ]]; then
    echo "$ICONS/headphones.svg"
    exit 0
fi

if pactl get-sink-mute @DEFAULT_SINK@ | grep -q "yes"; then
    echo "$ICONS/volume-mute.svg"
    exit 0
fi

volume=$(pactl get-sink-volume @DEFAULT_SINK@ | grep -o -m1 '[0-9]*%' | head -1 | tr -d '%')
if (( volume < 34 )); then
    echo "$ICONS/volume-low.svg"
elif (( volume < 67 )); then
    echo "$ICONS/volume-mid.svg"
else
    echo "$ICONS/volume-high.svg"
fi
