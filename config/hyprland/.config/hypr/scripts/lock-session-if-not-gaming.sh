#!/bin/bash


# On the Claude workspace the lock comes later and asks first: scripts/claude-idle.sh.
if "$HOME/.config/waybar/scripts/on-claude-workspace.sh"; then exit 0; fi
source ~/.config/hypr/scripts/is-gaming.sh

if ! is_gaming; then
    loginctl lock-session
fi
