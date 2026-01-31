#!/bin/bash

# Listen for pulseaudio events and signal waybar to update
pactl subscribe | grep --line-buffered "sink" | while read -r line; do
    pkill -RTMIN+8 waybar
done
