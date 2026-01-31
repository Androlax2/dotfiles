#!/bin/bash

HEADSET="alsa_output.usb-SteelSeries_Arctis_Nova_Pro-00.analog-stereo"

current=$(pactl get-default-sink)
muted=$(pactl get-sink-mute @DEFAULT_SINK@ | grep -Po 'yes|no')

if [[ "$muted" == "yes" ]]; then
    echo "󰖁"
elif [[ "$current" == "$HEADSET" ]]; then
    echo "󰋋"
else
    echo "󰕾"
fi
