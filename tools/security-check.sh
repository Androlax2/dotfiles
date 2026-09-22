#!/usr/bin/env bash
# Compromise check for this machine: compares the live system with the known-good baseline in
# security/ and runs a few checks that need no baseline. Read-only, runs as the user, no sudo.
#
#   security-check.sh check    print findings, exit 1 if there are any (make check-security)
#   security-check.sh dump     rewrite security/ from the live system  (make dump-security)
#   security-check.sh notify   like check, plus a desktop notification when something differs
#
# /etc is covered by `make check-system` and `make check-untracked`; enabled system units by
# `make check-services`. This script covers what those do not.
# No -e: diff and grep exit 1 for 'differs' / 'no match', which is data here, not an error.
set -uo pipefail

repo=$(cd "$(dirname "$(readlink -f "$0")")/.." && pwd)
baseline_dir="$repo/security"
findings=0

# --- live snapshots, one function per baseline file ------------------------------------------

listening_ports() {
    # protocol and local address:port, without the ephemeral range (32768+) that browsers and
    # Tailscale churn through; process names need root so they are left out. Normalised so a
    # reboot or an iproute2 upgrade does not churn the baseline: "*" and "[::]" are the same
    # wildcard, "%iface" scope suffixes go, and link-local addresses lose their random suffix.
    ss -tulnH | awk '{ split($5, a, ":"); port = a[length(a)]; if (port + 0 < 32768) print $1, $5 }' \
        | sed -E 's/ \*:/ [::]:/; s/%[A-Za-z0-9_.-]+:/:/; s/\[fe80:[0-9a-f:]+\]/[fe80::link-local]/' | sort -u
}

users_with_shell() {
    awk -F: '$7 !~ /(nologin|false)$/ {print $1 ":" $3 ":" $7}' /etc/passwd | sort
}

admin_group_members() {
    for group in wheel sudo docker; do
        members=$(getent group "$group" 2>/dev/null | cut -d: -f4)
        echo "$group: ${members:-<none>}"
    done
}

suid_sgid_files() {
    find /usr /opt /var /home /srv /tmp -xdev -type f \( -perm -4000 -o -perm -2000 \) 2>/dev/null | sort
}

unowned_executables() {
    find /usr/bin /usr/sbin /usr/lib /usr/local /opt -xdev -type f -perm /111 -print0 2>/dev/null \
        | xargs -0 pacman -Qo 2>&1 >/dev/null | sed -n 's/^error: No package owns //p' | sort
}

kernel_modules() {
    # Netfilter, bridge and veth modules come and go with the firewall ruleset and with Docker
    # containers, so they only add noise; an unexpected module is anything outside those.
    lsmod | awk 'NR > 1 && $1 !~ /^(nf_|nft_|xt_|veth$|bridge$|br_netfilter$|overlay$|ip_tables$|ip6_tables$|iptable_|ip6table_)/ {print $1}' | sort
}

enabled_user_units() {
    systemctl --user list-unit-files --state=enabled --no-legend --no-pager | awk '{print $1}' | sort
}

autostart_entries() {
    ls -1 ~/.config/autostart 2>/dev/null | sort
}

ssh_authorized_keys() {
    # fingerprints only, so the baseline does not carry the public keys themselves
    [[ -f ~/.ssh/authorized_keys ]] || return 0
    ssh-keygen -lf ~/.ssh/authorized_keys | awk '{print $2, $3}' | sort
}

system_cron_entries() {
    find /etc/cron.d /etc/cron.hourly /etc/cron.daily /etc/cron.weekly /etc/cron.monthly -type f 2>/dev/null | sort
}

# baseline name -> function
baselines=(
    listening-ports:listening_ports
    users:users_with_shell
    admin-groups:admin_group_members
    suid-sgid:suid_sgid_files
    unowned-executables:unowned_executables
    kernel-modules:kernel_modules
    user-units:enabled_user_units
    autostart:autostart_entries
    ssh-authorized-keys:ssh_authorized_keys
    system-cron:system_cron_entries
)

# --- reporting -------------------------------------------------------------------------------

section() { echo "== $1 =="; }
finding() { echo "  $1"; findings=$((findings + 1)); }

report_lines() {
    # each line of stdin becomes a finding; done without a pipe so the counter survives
    local line
    while IFS= read -r line; do
        [[ -n "$line" ]] && finding "$line"
    done
}

compare_with_baseline() {
    local name=$1 fn=$2 file="$baseline_dir/$1.txt"
    section "$name"
    if [[ ! -f "$file" ]]; then
        finding "no baseline: run 'make dump-security' once on a machine you trust"
        return
    fi
    report_lines <<< "$(diff <(sort "$file") <("$fn" | sort) | sed -n 's/^> /new: /p; s/^< /gone: /p')"
}

check_package_integrity() {
    section "package files modified outside /etc"
    # -Qkk verifies size, permissions, mtime and checksums; /etc is expected to differ
    report_lines <<< "$(pacman -Qkk 2>/dev/null | grep '^warning' | grep -v ': /etc/' | sed 's/^warning: //' | head -50)"
}

check_preload_and_taint() {
    section "kernel and loader"
    [[ -e /etc/ld.so.preload ]] && finding "/etc/ld.so.preload exists: $(tr '\n' ' ' < /etc/ld.so.preload)"
    local taint
    taint=$(cat /proc/sys/kernel/tainted)
    [[ "$taint" != "0" ]] && finding "kernel tainted (flags $taint); see /proc/sys/kernel/tainted"
    return 0
}

firewall_loaded() {
    # nftables.service is a one-shot unit that exits after loading the ruleset, so "is-active"
    # is always inactive; a successful last run since boot means the rules are in the kernel.
    [[ "$(systemctl is-enabled nftables 2>/dev/null)" == "enabled" ]] \
        && [[ "$(systemctl show nftables -p Result --value 2>/dev/null)" == "success" ]] \
        && [[ "$(systemctl show nftables -p ExecMainStatus --value 2>/dev/null)" == "0" ]]
}

check_firewall_and_ssh() {
    section "network exposure"
    if ! firewall_loaded; then
        finding "nftables ruleset not loaded (systemctl status nftables): every listening port is reachable from any network"
    fi
    if ss -tlnH | awk '{print $4}' | grep -q -E '^(0\.0\.0\.0|\*|\[::\]):22$' && ! firewall_loaded; then
        finding "sshd listens on all interfaces without a firewall"
    fi
    if sshd -T 2>/dev/null | grep -q '^passwordauthentication yes'; then
        finding "sshd accepts passwords (system/etc/ssh/sshd_config.d/10-hardening.conf turns that off)"
    fi
    return 0
}

check_auth_activity() {
    section "authentication, last 24 h"
    local failed sudo_root
    failed=$(journalctl _COMM=sshd --since -24h --no-pager 2>/dev/null | grep -c -i -E 'failed|invalid user' || true)
    (( failed > 10 )) && finding "$failed failed ssh logins (journalctl _COMM=sshd --since -24h)"
    sudo_root=$(journalctl _COMM=sudo --since -24h --no-pager 2>/dev/null | grep 'COMMAND=' | grep -vc " $USER : " || true)
    (( sudo_root > 0 )) && finding "$sudo_root sudo commands by a user other than $USER (journalctl _COMM=sudo --since -24h)"
    return 0
}

check_vulnerable_packages() {
    section "known vulnerable packages"
    if ! command -v arch-audit >/dev/null; then
        finding "arch-audit not installed (pacman -S arch-audit)"
        return
    fi
    report_lines <<< "$(arch-audit --upgradable --quiet 2>/dev/null)"
}

run_checks() {
    for entry in "${baselines[@]}"; do
        compare_with_baseline "${entry%%:*}" "${entry##*:}"
    done
    check_package_integrity
    check_preload_and_taint
    check_firewall_and_ssh
    check_auth_activity
    check_vulnerable_packages
    echo
    if (( findings == 0 )); then
        echo "nothing differs from the baseline"
    else
        echo "$findings finding(s); review them, then 'make dump-security' if they are legitimate"
    fi
    (( findings == 0 ))
}

dump_baseline() {
    mkdir -p "$baseline_dir"
    for entry in "${baselines[@]}"; do
        local name=${entry%%:*} fn=${entry##*:}
        "$fn" > "$baseline_dir/$name.txt"
        echo "wrote security/$name.txt ($(wc -l < "$baseline_dir/$name.txt") lines)"
    done
}

case "${1:-check}" in
    check) run_checks ;;
    dump) dump_baseline ;;
    notify)
        report=$(run_checks 2>&1) && exit 0
        count=$(grep -c '^  ' <<< "$report")
        notify-send -u critical -t 0 "Security check: $count finding(s)" \
            "$(grep '^  ' <<< "$report" | head -8)\n\nmake check-security for the full report" || true
        exit 1
        ;;
    *) echo "usage: $0 check|dump|notify" >&2; exit 2 ;;
esac
