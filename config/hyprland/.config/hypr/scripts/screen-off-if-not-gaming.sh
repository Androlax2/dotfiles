#!/bin/bash

source ~/.config/hypr/scripts/is-gaming.sh

if ! is_gaming; then
    hyprctl dispatch dpms off
fi
