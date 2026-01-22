#!/bin/bash

ADGUARD_DNS="192.168.1.15"
CONF_FILE="/etc/systemd/resolved.conf.d/adguard.conf"

case "$1" in
    adguard)
        echo "Switching to AdGuard DNS..."
        sudo sed -i "s/^DNS=.*/DNS=$ADGUARD_DNS/" "$CONF_FILE"
        sudo systemctl restart systemd-resolved
        notify-send "DNS Switched" "Now using AdGuard ($ADGUARD_DNS)"
        ;;
    isp)
        echo "Switching to ISP DNS..."
        sudo sed -i 's/^DNS=.*/DNS=/' "$CONF_FILE"
        sudo systemctl restart systemd-resolved
        notify-send "DNS Switched" "Now using ISP DNS"
        ;;
    *)
        echo "Usage: $0 {adguard|isp}"
        exit 1
        ;;
esac

sleep 1
resolvectl status | grep "Current DNS Server"
