#!/bin/bash
set -euo pipefail
mkdir -p ~/.local/share/templogs

ts="$(date -Is)"
cpu="$(sensors 2>/dev/null | awk '/Tctl:/ {gsub(/[+°C]/,"",$2); print $2; exit}')"
gpu="$(sensors 2>/dev/null | awk '/edge:/ {gsub(/[+°C]/,"",$2); print $2; exit}')"

echo "$ts cpu=$cpu gpu=$gpu" >> ~/.local/share/templogs/temps.log

