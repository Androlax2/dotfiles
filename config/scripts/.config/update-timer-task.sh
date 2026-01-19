#!/bin/bash

# Define paths
CACHE_FILE="/tmp/pending_updates_count"
PACMAN_LOG="/var/log/pacman.log"

# 1. Calculate days since last upgrade
LAST_UPGRADE=$(grep "starting full system upgrade" "$PACMAN_LOG" | tail -n 1 | awk '{print $1}' | tr -d '[]')
if [[ -n "$LAST_UPGRADE" ]]; then
    LAST_SEC=$(date -d "$LAST_UPGRADE" +%s)
    NOW_SEC=$(date +%s)
    DIFF_DAYS=$(( (NOW_SEC - LAST_SEC) / 86400 ))
else
    DIFF_DAYS=0
fi

# 2. Check for pending updates (network call)
PENDING_COUNT=$(checkupdates 2>/dev/null | wc -l)

# 3. Save to cache file for Fish to read
echo "$DIFF_DAYS:$PENDING_COUNT" > "$CACHE_FILE"

# DEBUG OUTPUT (Only seen when run manually)
if [[ "$1" == "--debug" ]]; then
    echo "--- DEBUG INFO ---"
    echo "Last Upgrade Date: $LAST_UPGRADE"
    echo "Days Since Upgrade: $DIFF_DAYS"
    echo "Pending Packages: $PENDING_COUNT"
    echo "Writing to: $CACHE_FILE"
    echo "File Content: $(cat $CACHE_FILE)"
fi
