#!/bin/bash
set -euo pipefail

out="$("$HOME/.config/check-temp-trends.py" || true)"
code=$?

mkdir -p "$HOME/.local/share/templogs"
echo "$(date -Is) rc=$code $out" >> "$HOME/.local/share/templogs/trend-check.log"

if [ "$code" -eq 2 ]; then
  if command -v notify-send >/dev/null 2>&1; then
    notify-send -u critical "Cooling trend warning" "$out"
  fi
fi

exit 0

