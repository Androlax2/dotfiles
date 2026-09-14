#!/usr/bin/env bash
# Backs up $HOME to the Synology NAS with restic.
# The NAS copies this repository to the Hetzner Storage Box with Hyper Backup,
# so this machine never holds cloud credentials. See "Backups" in ~/dotfiles/README.md.
set -euo pipefail

nas_host="jeancloud"
export RESTIC_REPOSITORY="sftp:${nas_host}:/restic/archlinux"
export RESTIC_PASSWORD_FILE="$HOME/.secrets/restic"

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
last_success_file="$state_dir/last-success"

usage() {
    cat <<USAGE
Usage: backup.sh <command>

  init                                   Create the repository on the NAS
  run [--skip-if-offline] [restic flags] Back up \$HOME (flags go to restic backup, e.g. --dry-run -v)
  maintain [--skip-if-offline]           Forget old snapshots, prune, and verify a data sample
  list [depth]                           Show the size of every folder a backup includes, 3 levels deep by default
  restic <args>                          Run any restic command against the repository

--skip-if-offline exits quietly when the NAS is unreachable; the timers use it.
USAGE
}

notify() {
    local urgency="$1" title="$2" body="$3"
    echo "$title: $body" >&2
    # A missing notification daemon must not turn a finished backup into a failed one.
    notify-send --app-name=restic --urgency="$urgency" "$title" "$body" || true
}

nas_is_reachable() {
    timeout 5 bash -c "</dev/tcp/${nas_host}/22" 2>/dev/null
}

warn_if_stale() {
    if [[ ! -f "$last_success_file" ]]; then
        notify critical "No backup has ever completed" "Run make backup while at home"
        return
    fi
    local age_days=$(( ($(date +%s) - $(stat -c %Y "$last_success_file")) / 86400 ))
    if (( age_days >= stale_after_days )); then
        notify normal "Last backup was $age_days days ago" "The NAS has been unreachable; run make backup while at home"
    fi
}

# Returns 0 when the NAS answers. Otherwise a scheduled run is skipped, because the
# machine is regularly away from home, while a manual run fails.
nas_available_for_run() {
    local skip_if_offline="$1"
    if nas_is_reachable; then
        return 0
    fi
    if [[ "$skip_if_offline" == true ]]; then
        echo "Skipping: NAS $nas_host is not reachable."
        warn_if_stale
        return 1
    fi
    echo "Error: NAS $nas_host is not reachable on port 22." >&2
    exit 1
}

# Backup and maintenance must not run against the repository at the same time.
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

record_success() {
    mkdir -p "$state_dir"
    touch "$last_success_file"
}

run_backup() {
    local skip_if_offline=false
    if [[ "${1:-}" == "--skip-if-offline" ]]; then
        skip_if_offline=true
        shift
    fi
    nas_available_for_run "$skip_if_offline" || return 0
    wait_for_lock

    local exit_code=0
    restic backup "${backup_arguments[@]}" "$@" || exit_code=$?

    if is_dry_run "$@"; then
        return "$exit_code"
    fi
    if (( exit_code == 0 )); then
        record_success
    elif (( exit_code == restic_partial_backup_exit_code )); then
        record_success
        notify normal "Backup saved, but some files were unreadable" "See journalctl --user -u restic-backup"
    else
        notify critical "Backup failed (restic exit code $exit_code)" "See journalctl --user -u restic-backup"
        return "$exit_code"
    fi
}

run_maintenance() {
    local skip_if_offline=false
    if [[ "${1:-}" == "--skip-if-offline" ]]; then
        skip_if_offline=true
    fi
    nas_available_for_run "$skip_if_offline" || return 0
    wait_for_lock

    if ! restic forget --prune --keep-daily "$keep_daily" --keep-weekly "$keep_weekly" --keep-monthly "$keep_monthly" \
        || ! restic check --read-data-subset="$check_data_subset"; then
        notify critical "Backup maintenance failed" "See journalctl --user -u restic-maintenance"
        return 1
    fi
}

# Dry-runs restic's own exclude rules against an empty throwaway repository, so the result
# matches a real backup but never touches the NAS. Sizes are on-disk file sizes, summed into
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
        | jq -j 'select(.message_type == "verbose_status" and .item != "" and (.item | endswith("/") | not)) | .item, "\u0000"' \
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
        init) restic init ;;
        run) run_backup "$@" ;;
        maintain) run_maintenance "$@" ;;
        list) list_backup_contents "$@" ;;
        restic) restic "$@" ;;
        *) usage >&2; exit 2 ;;
    esac
}

if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
    main "$@"
fi
