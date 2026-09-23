#!/usr/bin/env bash
# Asks whether to enable game mode when a game window opens (see game-mode.lua).
# One prompt per game launch: games open several windows, so repeats within a
# minute for the same class are ignored.
set -uo pipefail

class=${1:?usage: game-mode-prompt.sh <window class>}
stamp="${XDG_RUNTIME_DIR:-/tmp}/game-mode-prompt.${class//[^A-Za-z0-9_.-]/_}"
if [[ -f $stamp && $(( $(date +%s) - $(stat -c %Y "$stamp") )) -lt 60 ]]; then
    exit 0
fi
touch "$stamp"

answer=$(notify-send -a "Game mode" -i input-gaming -t 15000 \
    -A "enable=Enable game mode" \
    "$class is starting" "Turn off animations, blur and rounding?")
if [[ $answer == enable ]]; then
    hyprctl eval "ENTER_GAME_MODE()" >/dev/null
fi
