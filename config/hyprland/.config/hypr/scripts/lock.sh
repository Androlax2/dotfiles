#!/usr/bin/env bash
# The one way the screen locks: hypridle's lock_cmd (so loginctl lock-session and before sleep),
# lock-if-not-gaming.sh and the hot corner all run this.
pidof hyprlock >/dev/null && exit 0
hyprlock
