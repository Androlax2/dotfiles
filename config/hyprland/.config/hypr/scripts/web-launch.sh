#!/usr/bin/env bash
# Open a site in its own Zen profile, the way the web scratchpads in scratchpads.lua want it:
#   web-launch.sh <profile> <url>
# The profile is created on first use; on every start its user.js is rewritten from
# ~/.config/zen/scratchpad.user.js plus the site as its home page (the one set at creation: a later
# start on a pull request does not change it), so it is a compact window on that site and nothing
# else, with its own logins. exec keeps the PID, which the spawn rules in scratchpads.lua match on. No --no-remote:
# Zen's remoting is per profile, so a later `zen-browser -P <profile> <url>` opens the page in
# this window (scratchpads.lua's OPEN_IN_SCRATCHPAD) instead of failing on a profile in use.
set -euo pipefail
profile=${1:?usage: web-launch.sh <profile> <url>}
url=${2:?usage: web-launch.sh <profile> <url>}

profile_dir="$HOME/.zen/$profile"
if [[ ! -d $profile_dir ]]; then
    zen-browser -CreateProfile "$profile $profile_dir" >/dev/null 2>&1
fi
homepage=$(grep -h '^user_pref("browser.startup.homepage"' "$profile_dir/user.js" 2>/dev/null || true)
{
    cat ~/.config/zen/scratchpad.user.js
    printf '%s\n' "${homepage:-$(printf 'user_pref("browser.startup.homepage", "%s");' "$url")}"
} > "$profile_dir/user.js.new"
mv "$profile_dir/user.js.new" "$profile_dir/user.js"
exec zen-browser -P "$profile" "$url"
