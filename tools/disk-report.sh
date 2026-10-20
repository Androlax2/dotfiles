#!/usr/bin/env bash
# Read-only disk report behind `make disk`, ordered by what has filled the disk before:
# agent clones and their Rust builds, agent logs, caches, then system and games.
# Nothing is deleted; each section ends with the command that reclaims it.
# No pipefail: du exits 1 for any missing or unreadable folder, which is expected here.
set -u

home=${HOME:?HOME must be set}
clones_root=${CLAUDE_SESSIONS_CLONES:-$home/IdeaProjects/.parallel}
projects_root=$home/IdeaProjects
pnpm_home=${PNPM_HOME:-$home/.local/share/pnpm}

section() { printf '\n== %s ==\n' "$1"; }
hint() { printf '   -> %s\n' "$1"; }
largest() { sort -rh | head -n "${1:-10}"; }
sizes() { du -sh "$@" 2>/dev/null | largest 20; }

section "disk"
df -h --output=target,size,used,avail,pcent / "$home" 2>/dev/null | awk '!seen[$0]++'
used_percent=$(df --output=pcent "$home" 2>/dev/null | tail -1 | tr -dc '0-9')
if [ -n "$used_percent" ] && [ "$used_percent" -ge 85 ]; then
    echo "!! $home is ${used_percent}% full"
fi

section "agent clones ($clones_root)"
mapfile -t clones < <(find "$clones_root" -mindepth 2 -maxdepth 2 -type d 2>/dev/null)
if [ "${#clones[@]}" -eq 0 ]; then
    echo "none"
else
    echo "${#clones[@]} clones, $(du -sch "${clones[@]}" 2>/dev/null | tail -1 | cut -f1) in total; largest:"
    sizes "${clones[@]}" | head -n 10
    hint "nothing removes finished clones; delete them, or at least their apps/target"
fi

section "build output in projects (target, .next, node_modules, vendor)"
find "$projects_root" -xdev -type d \( -name target -o -name .next -o -name node_modules -o -name vendor \) \
    -prune -print0 2>/dev/null | xargs -0 -r du -sh 2>/dev/null | largest 10
hint "cargo clean / rm -rf the folder; it is rebuilt on the next build"

section "agent data"
sizes "$home/.local/share/greyproxy" "$home/.cache/claude" "$home/.claude" \
    "$home/.local/share/opencode" "$home/.cache/opencode" "$home/.codex"
hint "greyproxy-prune.timer caps greyproxy.db at 14 days and 4 GB; greyproxy-prune --compact returns the space"

section "largest user caches (~/.cache)"
{ du -sh "$home"/.cache/* 2>/dev/null || true; } | largest 10
hint "make clean-caches; JetBrains: File > Invalidate Caches, or delete old IDE versions"

section "package caches"
sizes /var/cache/pacman/pkg "$home/.cache/yay" "$home/.npm" "$pnpm_home/store" "$home/.pnpm-store" \
    "$home/go/pkg/mod" "$home/.cargo/registry" "$home/.local/share/Trash"
echo "orphaned packages: $({ pacman -Qdtq 2>/dev/null || true; } | wc -l)"
hint "make clean-packages; make clean-trash"

section "system"
journalctl --disk-usage 2>/dev/null
coredumps=$(du -sh /var/lib/systemd/coredump 2>/dev/null | cut -f1)
echo "coredumps: ${coredumps:-unreadable}"
for agent_home in /var/lib/agent-*; do
    [ -d "$agent_home" ] || continue
    if agent_size=$(du -sh "$agent_home" 2>/dev/null); then
        echo "agent docker $(basename "$agent_home"): $(cut -f1 <<<"$agent_size")"
    else
        echo "agent docker $(basename "$agent_home"): unreadable, run sudo du -sh $agent_home"
    fi
done
hint "make clean-logs; sudo rm /var/lib/systemd/coredump/*; make clean-docker"

section "docker (main daemon)"
docker system df 2>/dev/null || echo "docker is not reachable"

section "games"
sizes "$home/.local/share/Steam/steamapps/common"/* "$home/Games"/* | head -n 8
