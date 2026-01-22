#!/bin/bash
EXPECTED_DNS="192.168.1.15"
CURRENT_DNS=$(resolvectl status | grep "Current DNS Server:" | head -1 | awk '{print $4}')

if [[ "$CURRENT_DNS" != "$EXPECTED_DNS" ]]; then
    notify-send -u critical "DNS Warning" "DNS changed from $EXPECTED_DNS to $CURRENT_DNS"
    echo "$(date): DNS is $CURRENT_DNS (expected $EXPECTED_DNS)" >> ~/.local/share/dns-check.log
    exit 1
fi
