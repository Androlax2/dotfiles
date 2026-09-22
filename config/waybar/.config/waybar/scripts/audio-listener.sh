#!/bin/bash
# Forward pactl sink events to the bar as SIGRTMIN+8, coalesced: a held volume key
# produces dozens of events per second, and each signal makes the bar re-run the
# volume icon script, so events within 150 ms of each other become one signal.
pactl subscribe | grep --line-buffered "sink" | while read -r _; do
    while read -r -t 0.15 _; do :; done
    pkill -RTMIN+8 waybar
done
