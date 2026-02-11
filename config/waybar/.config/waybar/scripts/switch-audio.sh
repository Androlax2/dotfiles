#!/bin/bash

HEADSET="alsa_output.usb-SteelSeries_Arctis_Nova_Pro-00.analog-stereo"
SPEAKERS="alsa_output.usb-bestechnic_EDIFIER_M60_20160406.1-00.analog-stereo"

current=$(pactl get-default-sink)

if [[ "$current" == "$HEADSET" ]]; then
    pactl set-default-sink "$SPEAKERS"
    new_name="Speakers"
else
    pactl set-default-sink "$HEADSET"
    new_name="Headset"
fi

# Move all current playing streams to the new sink
new_sink=$(pactl get-default-sink)
for stream in $(pactl list short sink-inputs | awk '{print $1}'); do
    pactl move-sink-input "$stream" "$new_sink"
done

# Signal waybar to update the icon immediately
pkill -RTMIN+8 waybar

notify-send "Audio Output" "Switched to: $new_name" -t 2000
