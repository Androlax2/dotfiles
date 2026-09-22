#!/usr/bin/env bash
# Waybar image module: the updates icon while updates are pending, nothing otherwise.
cache="/tmp/pending_updates_count"
[[ -f "$cache" ]] || exit 0
IFS=: read -r _ count < "$cache"
(( count > 0 )) && echo "$HOME/.config/waybar/icons/updates.svg"
exit 0
