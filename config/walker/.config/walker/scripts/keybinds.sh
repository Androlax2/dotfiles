#!/usr/bin/env bash
# Keybinding cheat sheet in walker's dmenu mode, generated from the live Hyprland
# binds (`hyprctl binds`; descriptions come from bindings.lua). Choosing an entry
# runs it (RUN_BIND in binds.lua). keyd swaps Ctrl and Super at the keyboard, so
# modifiers are shown as the physical keys, and keycodes are named for AZERTY.
# `--list` prints the sheet instead of opening walker (used to build the README table).
set -uo pipefail

sheet() {
hyprctl binds | awk '
    function flush() {
        if (key != "" && desc != "" && submap == "") {
            n = split(key, parts, " \\+ ")
            k = parts[n]
            if (k in names) k = names[k]
            mods = ""
            if (int(modmask / 4) % 2 == 1)  mods = mods "Super + "
            if (int(modmask / 64) % 2 == 1) mods = mods "Ctrl + "
            if (int(modmask / 8) % 2 == 1)  mods = mods "Alt + "
            if (modmask % 2 == 1)           mods = mods "Shift + "
            printf "%-30s %s\n", mods k, desc
        }
        key = ""; desc = ""; submap = ""; modmask = 0
    }
    BEGIN {
        names["code:10"] = "1"; names["code:11"] = "2"; names["code:12"] = "3"; names["code:13"] = "4"
        names["code:14"] = "5"; names["code:15"] = "6"; names["code:16"] = "7"; names["code:17"] = "8"
        names["code:18"] = "9"; names["code:19"] = "0"; names["code:49"] = "²"
        names["mouse_down"] = "Scroll down"; names["mouse_up"] = "Scroll up"
    }
    /^bind/ { flush() }
    /^\tmodmask: /     { modmask = $2 }
    /^\tsubmap: /      { submap = $2 }
    /^\tkey: /         { key = substr($0, 7) }
    /^\tdescription: / { desc = substr($0, 15) }
    END { flush() }
' | sort -u
}

if [[ ${1:-} == --list ]]; then
    sheet
    exit 0
fi

choice=$(sheet | walker -d -p "Keybindings") || exit 0
description=$(sed -E 's/^.{30} +//' <<<"$choice")
[[ -z $description ]] && exit 0
hyprctl eval "RUN_BIND([[$description]])" >/dev/null
