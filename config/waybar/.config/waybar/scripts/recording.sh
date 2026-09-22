#!/usr/bin/env bash
# Waybar custom module: "REC mm:ss" while wl-screenrec is running, nothing otherwise
# (an empty output hides the module). Elapsed time is the process age of the
# oldest wl-screenrec, so it survives Waybar restarts.
pid=$(pgrep -x -o wl-screenrec) || exit 0
elapsed=$(ps -o etimes= -p "$pid" | tr -d ' ')
printf 'REC %02d:%02d\n' $((elapsed / 60)) $((elapsed % 60))
