#!/bin/bash
set -euo pipefail

# check-temp-trends (apps/desktop) exits 2 on a warning; `|| code=$?` keeps that code under set -e.
code=0
out="$("$HOME/.local/bin/check-temp-trends")" || code=$?

mkdir -p "$HOME/.local/share/templogs"
echo "$(date -Is) rc=$code $out" >> "$HOME/.local/share/templogs/trend-check.log"

if [ "$code" -eq 2 ]; then
  if command -v notify-send >/dev/null 2>&1; then
    notify-send -u critical "Cooling trend warning" "$out"
  fi
fi

exit 0

