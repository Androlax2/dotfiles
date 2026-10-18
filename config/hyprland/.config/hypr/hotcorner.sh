#!/bin/bash
# Bottom-left hot corner: locks the screen through lock.sh (which hands the Claude sessions to
# Remote Control). Hyprland sends no cursor events, so this polls: one `hyprctl cursorpos` per
# POLL_S, the monitor layout every LAYOUT_EVERY polls and the gaming check every GAMING_EVERY
# polls. That is about 7 processes a second; the first version, with every lookup on every
# 0.1 s poll, started about 80.
# HOTCORNER_COMMAND replaces the lock (tests).
COMMAND=${HOTCORNER_COMMAND:-$HOME/.config/hypr/scripts/lock.sh}
THRESHOLD=15      # pixels from the corner
DEBOUNCE=3        # seconds after a trigger before watching again
POLL_S=0.15
LAYOUT_EVERY=66   # about 10 s
GAMING_EVERY=13   # about 2 s

source ~/.config/hypr/scripts/is-gaming.sh >/dev/null

monitors=""  # one "x y width height" line per monitor
read_layout() {
    monitors=$(hyprctl monitors -j 2>/dev/null | jq -r '.[] | "\(.x) \(.y) \(.width) \(.height)"' 2>/dev/null)
}

in_corner() { # x y: bottom-left corner of the monitor under the cursor
    local x=$1 y=$2 mx my mw mh
    while read -r mx my mw mh; do
        [[ -n $mx ]] || continue
        if (( x >= mx && x < mx + mw && y >= my && y < my + mh )); then
            (( x - mx < THRESHOLD && y > my + mh - THRESHOLD ))
            return
        fi
    done <<< "$monitors"
    return 1
}

poll=0
gaming=0
while true; do
    (( poll % LAYOUT_EVERY == 0 )) && read_layout
    if (( poll % GAMING_EVERY == 0 )); then
        if is_gaming; then gaming=1; else gaming=0; fi
    fi
    poll=$((poll + 1))
    if (( gaming )); then
        sleep "$POLL_S"
        continue
    fi
    # "x, y", possibly with decimals.
    position=$(hyprctl cursorpos 2>/dev/null)
    x=${position%%,*}; y=${position##*,}
    x=${x%%.*}; y=${y%%.*}; x=${x// /}; y=${y// /}
    if [[ $x =~ ^-?[0-9]+$ && $y =~ ^-?[0-9]+$ ]] && in_corner "$x" "$y"; then
        "$COMMAND"
        sleep "$DEBOUNCE"
    fi
    sleep "$POLL_S"
done
