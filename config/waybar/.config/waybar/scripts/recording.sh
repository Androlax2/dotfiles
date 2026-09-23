#!/usr/bin/env bash
# Waybar custom module: "REC mm:ss" while a recording runs, nothing otherwise (an
# empty output hides the module). screen-record.sh writes "<start epoch>\t<file>"
# to $XDG_RUNTIME_DIR/recording and signals RTMIN+13 on start and stop; the 1 s
# interval keeps the clock ticking. A stale file without a recorder is ignored.
state_file="${XDG_RUNTIME_DIR:-/tmp}/recording"
[[ -f $state_file ]] || exit 0
pgrep -x wl-screenrec >/dev/null || exit 0
start=$(cut -f1 "$state_file")
elapsed=$(( $(date +%s) - start ))
printf 'REC %02d:%02d\n' $((elapsed / 60)) $((elapsed % 60))
