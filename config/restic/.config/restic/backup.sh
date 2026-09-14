#!/usr/bin/env bash
# Backs up $HOME with restic to two independent repositories, so losing one never loses the backup:
#   nas      the Synology NAS, over SFTP (the "jeancloud" SSH host)
#   hetzner  the Hetzner Storage Box, over SFTP (the "storagebox" SSH host, defined in ~/.ssh/config)
# The NAS backs up its own data separately, from the homelab repository. See "Backups" in ~/dotfiles/README.md.
set -euo pipefail

export RESTIC_PASSWORD_FILE="$HOME/.secrets/restic"
# <name>:<SSH host from ~/.ssh/config>:<repository path on that host>
# The Storage Box path has no leading slash: relative to its home folder, it is the same folder
# whether ~/.ssh/config uses port 22 (home shows as /) or port 23 (home shows as /home).
targets=(
    "nas:jeancloud:/restic/archlinux"
    "hetzner:storagebox:restic/archlinux"
)

keep_daily=7
keep_weekly=4
keep_monthly=12
check_data_subset="5%"
stale_after_days=3
# restic exits with 3 when the snapshot was saved but some files could not be read.
restic_partial_backup_exit_code=3

config_dir="$HOME/.config/restic"
backup_arguments=("$HOME" --exclude-file "$config_dir/excludes.txt" --exclude-caches --one-file-system)
state_dir="${XDG_STATE_HOME:-$HOME/.local/state}/restic-backup"

usage() {
    cat <<USAGE
Usage: backup.sh <command>

  init                                   Create the repositories that don't exist yet
  run [--skip-if-offline] [restic flags] Back up \$HOME to every target (flags go to restic backup, e.g. --dry-run -v)
  maintain [--skip-if-offline]           Forget old snapshots, prune, and verify a data sample, on every target
  list [depth]                           Show the size of every folder a backup includes, 3 levels deep by default
  restic <target> <args>                 Run any restic command against one target: $(target_names)

--skip-if-offline skips an unreachable target quietly; the timers use it.
USAGE
}

target_names() {
    local target names=()
    for target in "${targets[@]}"; do
        names+=("${target%%:*}")
    done
    echo "${names[*]}"
}

# Prints the target whose name is $1, or fails.
find_target() {
    local target
    for target in "${targets[@]}"; do
        if [[ "${target%%:*}" == "$1" ]]; then
            echo "$target"
            return 0
        fi
    done
    echo "Error: unknown target \"$1\" (one of: $(target_names))" >&2
    return 1
}

target_name() {
    echo "${1%%:*}"
}

target_ssh_host() {
    local rest="${1#*:}"
    echo "${rest%%:*}"
}

# Runs restic against the target in $1, with the remaining arguments.
restic_on() {
    local target="$1"
    shift
    RESTIC_REPOSITORY="sftp:$(target_ssh_host "$target"):${target##*:}" restic "$@"
}

notify() {
    local urgency="$1" title="$2" body="$3"
    echo "$title: $body" >&2
    # A missing notification daemon must not turn a finished backup into a failed one.
    notify-send --app-name=restic --urgency="$urgency" "$title" "$body" || true
}

# True when the target's SSH host answers, at the address and port ~/.ssh/config gives it.
target_is_reachable() {
    local hostname port
    read -r hostname port < <(ssh -G "$(target_ssh_host "$1")" 2>/dev/null | awk '$1 == "hostname" { hostname = $2 } $1 == "port" { port = $2 } END { print hostname, port }')
    timeout 5 bash -c "</dev/tcp/${hostname}/${port}" 2>/dev/null
}

last_success_file() {
    echo "$state_dir/$(target_name "$1")/last-success"
}

warn_if_stale() {
    local name last_success_file
    name=$(target_name "$1")
    last_success_file=$(last_success_file "$1")
    if [[ ! -f "$last_success_file" ]]; then
        notify critical "No backup to $name has ever completed" "Run make backup where $name is reachable"
        return
    fi
    local age_days=$(( ($(date +%s) - $(stat -c %Y "$last_success_file")) / 86400 ))
    if (( age_days >= stale_after_days )); then
        notify normal "Last backup to $name was $age_days days ago" "It has been unreachable; run make backup where it is reachable"
    fi
}

# Returns 0 when the target answers. Otherwise a scheduled run skips it (returns 1), because
# the machine is sometimes away from home or offline, while a manual run reports an error (returns 2).
target_availability() {
    local target="$1" skip_if_offline="$2"
    if target_is_reachable "$target"; then
        return 0
    fi
    local description
    description="$(target_name "$target") (SSH host $(target_ssh_host "$target"))"
    if [[ "$skip_if_offline" == true ]]; then
        echo "Skipping $description: it is not reachable."
        warn_if_stale "$target"
        return 1
    fi
    echo "Error: $description is not reachable. Check the network, and that ~/.ssh/config defines the host." >&2
    return 2
}

# Backup and maintenance must not run against the repositories at the same time.
wait_for_lock() {
    exec {lock_fd}>"${XDG_RUNTIME_DIR:?XDG_RUNTIME_DIR must be set}/restic-backup.lock"
    flock "$lock_fd"
}

is_dry_run() {
    local flag
    for flag in "$@"; do
        if [[ "$flag" == "--dry-run" || "$flag" == "-n" ]]; then
            return 0
        fi
    done
    return 1
}

backup_to() {
    local target="$1" skip_if_offline="$2"
    shift 2
    local name availability=0
    name=$(target_name "$target")
    target_availability "$target" "$skip_if_offline" || availability=$?
    if (( availability == 1 )); then
        return 0
    elif (( availability == 2 )); then
        return 1
    fi

    echo "Backing up to $name"
    local exit_code=0
    restic_on "$target" backup "${backup_arguments[@]}" "$@" || exit_code=$?

    if is_dry_run "$@"; then
        return "$exit_code"
    fi
    if (( exit_code == 0 || exit_code == restic_partial_backup_exit_code )); then
        mkdir -p "$(dirname "$(last_success_file "$target")")"
        touch "$(last_success_file "$target")"
    fi
    if (( exit_code == restic_partial_backup_exit_code )); then
        notify normal "Backup to $name saved, but some files were unreadable" "See journalctl --user -u restic-backup"
    elif (( exit_code != 0 )); then
        notify critical "Backup to $name failed (restic exit code $exit_code)" "See journalctl --user -u restic-backup"
        return "$exit_code"
    fi
}

run_backup() {
    local skip_if_offline=false
    if [[ "${1:-}" == "--skip-if-offline" ]]; then
        skip_if_offline=true
        shift
    fi
    wait_for_lock
    local target failed=0
    for target in "${targets[@]}"; do
        backup_to "$target" "$skip_if_offline" "$@" || failed=1
    done
    return "$failed"
}

maintain() {
    local target="$1" skip_if_offline="$2" name availability=0
    name=$(target_name "$target")
    target_availability "$target" "$skip_if_offline" || availability=$?
    if (( availability == 1 )); then
        return 0
    elif (( availability == 2 )); then
        return 1
    fi
    echo "Maintaining $name"
    if ! restic_on "$target" forget --prune --keep-daily "$keep_daily" --keep-weekly "$keep_weekly" --keep-monthly "$keep_monthly" \
        || ! restic_on "$target" check --read-data-subset="$check_data_subset"; then
        notify critical "Backup maintenance on $name failed" "See journalctl --user -u restic-maintenance"
        return 1
    fi
}

run_maintenance() {
    local skip_if_offline=false
    if [[ "${1:-}" == "--skip-if-offline" ]]; then
        skip_if_offline=true
    fi
    wait_for_lock
    local target failed=0
    for target in "${targets[@]}"; do
        maintain "$target" "$skip_if_offline" || failed=1
    done
    return "$failed"
}

init_repositories() {
    local target failed=0
    for target in "${targets[@]}"; do
        if restic_on "$target" cat config > /dev/null 2>&1; then
            echo "$(target_name "$target"): the repository already exists"
        elif ! restic_on "$target" init; then
            failed=1
        fi
    done
    return "$failed"
}

# Dry-runs restic's own exclude rules against an empty throwaway repository, so the result
# matches a real backup but never touches a remote. Sizes are on-disk file sizes, summed into
# every parent folder like du, before deduplication and compression.
list_backup_contents() {
    local depth="${1:-3}"
    local scratch_repository
    scratch_repository="$(mktemp -d)"
    # Expanded now: the local variable is gone by the time the EXIT trap runs.
    trap "rm -rf '$scratch_repository'" EXIT

    env -u RESTIC_PASSWORD_FILE restic -r "$scratch_repository" --insecure-no-password init --quiet
    env -u RESTIC_PASSWORD_FILE restic -r "$scratch_repository" --insecure-no-password \
        backup "${backup_arguments[@]}" --dry-run --json -vv \
        | jq -j 'select(.message_type == "verbose_status" and .item != "" and (.item | endswith("/") | not)) | .item, " "' \
        | xargs -0 --no-run-if-empty stat --printf '%s\t%n\n' \
        | awk -F '\t' -v home="$HOME/" -v depth="$depth" '
            {
                part_count = split(substr($2, length(home) + 1), parts, "/")
                folder = "~"
                size[folder] += $1
                for (level = 1; level < part_count && level <= depth; level++) {
                    folder = folder "/" parts[level]
                    size[folder] += $1
                }
            }
            END { for (folder in size) printf "%d\t%s\n", size[folder], folder }' \
        | sort -t $'\t' -k1,1nr \
        | numfmt --delimiter=$'\t' --field=1 --to=iec --padding=6
}

main() {
    local command="${1:-}"
    if (( $# > 0 )); then
        shift
    fi
    case "$command" in
        init) init_repositories ;;
        run) run_backup "$@" ;;
        maintain) run_maintenance "$@" ;;
        list) list_backup_contents "$@" ;;
        restic)
            local target
            target=$(find_target "${1:-}")
            shift
            restic_on "$target" "$@"
            ;;
        *) usage >&2; exit 2 ;;
    esac
}

if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
    main "$@"
fi
