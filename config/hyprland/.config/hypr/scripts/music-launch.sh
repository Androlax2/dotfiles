#!/usr/bin/env bash
# Launch the Apple Music window in its own Zen profile (created on first use with the
# prefs from ~/.config/zen/music.user.js). exec keeps the PID, which the spawn rules
# in scratchpads.lua match on.
set -euo pipefail

profile_dir="$HOME/.zen/music"
if [[ ! -d $profile_dir ]]; then
    zen-browser -CreateProfile "music $profile_dir" >/dev/null 2>&1
    cp ~/.config/zen/music.user.js "$profile_dir/user.js"
fi
exec zen-browser -P music --no-remote https://music.apple.com
