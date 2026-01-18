#!/bin/bash

FLAG="/tmp/gaming_mode_active"

turn_on() {
    if [ ! -f "$FLAG" ]; then
        touch "$FLAG"
        notify-send -t 2000 "Gaming Mode" "ON - F-Keys Active" -i input-gaming
    fi
}

turn_off() {
    if [ -f "$FLAG" ]; then
        rm "$FLAG"
        notify-send -t 2000 "Gaming Mode" "OFF - Media Keys Active" -i input-keyboard
    fi
}

case "$1" in
    on)
        turn_on
        ;;
    off)
        turn_off
        ;;
    *)
        if [ -f "$FLAG" ]; then turn_off; else turn_on; fi
        ;;
esac
