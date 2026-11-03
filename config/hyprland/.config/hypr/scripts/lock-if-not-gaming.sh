#!/bin/bash

source ~/.config/hypr/scripts/is-gaming.sh

if ! is_gaming; then
    ~/.config/hypr/scripts/lock.sh
fi
