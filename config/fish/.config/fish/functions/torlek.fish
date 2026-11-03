# `torlek tui` opens in its own kitty window of class torlek, which Hyprland keeps fully opaque
# (hypr/apps/torlek.lua). Without a Wayland session (SSH, TTY) it runs in place.
# `command torlek tui` bypasses the window.
function torlek --description 'torlek, with the TUI in its own opaque kitty window'
    if test "$argv[1]" = tui; and set -q WAYLAND_DISPLAY
        kitty --detach --class torlek torlek $argv
        return
    end
    command torlek $argv
end
