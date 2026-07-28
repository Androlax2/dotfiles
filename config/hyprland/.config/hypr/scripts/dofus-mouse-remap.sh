#!/bin/bash
# Save as: ~/.config/hypr/scripts/dofus-mouse-remap.sh
# While a Dofus window is focused, bind the mouse side button to send "Y".
# Binds are global in Hyprland, so the bind is added/removed on focus events
# to keep the button's normal behavior (e.g. browser Back) everywhere else.

DOFUS_CLASS_REGEX='^Dofus\.x64$'
MOUSE_BUTTON='mouse:275'
SENT_KEY='Y'

bound=0

bind_key() {
    if [ "$bound" -eq 0 ]; then
        hyprctl keyword bind ",$MOUSE_BUTTON,sendshortcut,,$SENT_KEY,activewindow" >/dev/null
        bound=1
    fi
}

unbind_key() {
    if [ "$bound" -eq 1 ]; then
        hyprctl keyword unbind ",$MOUSE_BUTTON" >/dev/null
        bound=0
    fi
}

apply_for_class() {
    if [[ "$1" =~ $DOFUS_CLASS_REGEX ]]; then
        bind_key
    else
        unbind_key
    fi
}

sync_with_active_window() {
    apply_for_class "$(hyprctl activewindow -j 2>/dev/null | jq -r '.class' 2>/dev/null)"
}

# Cover the case where the script starts while Dofus is already focused
sync_with_active_window

socat -U - "UNIX-CONNECT:$XDG_RUNTIME_DIR/hypr/$HYPRLAND_INSTANCE_SIGNATURE/.socket2.sock" |
while read -r event; do
    case "$event" in
        activewindow\>\>*)
            window_class=${event#activewindow>>}
            window_class=${window_class%%,*}
            apply_for_class "$window_class"
            ;;
        configreloaded\>\>*)
            # hyprctl reload wipes keyword-added binds; re-apply if needed
            bound=0
            sync_with_active_window
            ;;
    esac
done
