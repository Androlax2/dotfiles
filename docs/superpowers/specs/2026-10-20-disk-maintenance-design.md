# Broader disk maintenance

## Approved policy

The user approved automatic pruning of system logs older than **30 days**, keeping
at least **two cached versions per package** plus packages accessed or modified
within **30 days**, disk-space notifications through **Vitals** at **85% warning**
and **95% critical**, preserving growth forecasts and existing disk-health/TRIM
monitoring, and a **monthly review of large games and project checkouts**. Removal
of games or checkouts always requires explicit approval.

This document and its companion plan do not authorize installation, enabling
services, live deletion, dependency changes, builds, or git writes.

## Architecture and boundaries

1. Native journald retention, not another vacuum timer. Set `MaxRetentionSec=30day`
   and `MaxFileSec=1day` in a tracked drop-in. Journald works on journal files;
   retention is not an exact per-record deletion deadline. Existing size/default
   free-space limits can shorten history. Do not add the previously suggested
   4 GiB cap or free-space thresholds: those were not approved.
2. The vendor `paccache.service` and timer own automatic package cleanup after
   separately approved activation. Override its command to prune only
   `/var/cache/pacman/pkg`, keeping two versions and both 30-day freshness guards.
   Do not remove installed packages, orphans, AUR builds, or uninstalled-package
   archives under a special zero-version policy. The existing package report
   remains dry-run, aligned to that same fixed directory and retention arguments.
3. Extend existing Vitals findings, alert escalation, and notification plumbing.
   Preserve the current disk alert key for each mount so warning-to-critical
   transitions use the existing escalation behavior. Preserve growth forecasting
   below the disk warning threshold, and avoid duplicate forecast/full alerts.
   Do not introduce a new monitor, UI, or notification channel.
4. A new unprivileged monthly report inventories approved roots and publishes a
   bounded local report. It never supplies a delete/apply mode. Use Vitals' existing
   collector to announce each new report once, deferring announcements while gaming.
   A missing notification does not erase or invalidate the report.

## What stays unchanged

- Existing SMART checks, the ten-minute root collector, and weekly `fstrim.timer`.
  Source and live inspection confirmed these already exist; no duplicate services.
  A past timer trigger is not proof of successful discard on every filesystem.
- Existing 90-day primary security audit retention. The journald copy will obey
  the new journal limits; do not shorten the primary audit files' retention.
- Turbo stays report-only because cache-entry and active-build safety are unverified.
- No automatic cleanup of personal files, projects, games, build output, dependencies,
  Docker data, backups, pnpm/uv stores, or IDE caches.
- Existing manual Makefile cleanup commands; do not schedule them automatically.

## Implementation choices (recommended, not separately chosen by the user)

- Package cleanup runs daily with up to one hour of jitter, persistent across shutdown.
- Monthly review uses Python's standard library, no new dependencies, low CPU/I/O
  priority, a runtime lock, and a 15-minute service timeout. Retain a single atomically
  replaced `latest.json` under the user's state directory, not an expanding archive.
- Review root inventory: direct game directories under `~/Games` and
  `~/.local/share/Steam/steamapps/common`; direct projects under `~/IdeaProjects`
  excluding `.parallel`; checkouts at `~/IdeaProjects/.parallel/*/*`.
- Roots are explicit, not discovered from arbitrary configs. Do not follow symlinks
  or cross devices; report inaccessible or skipped paths, never substitute zero.
  Allocated-byte estimates are non-additive across roots with shared hardlinks.
  Preserve measured partial sizes with per-entry `complete=false`; such sizes may
  underestimate usage, and Git state for incomplete scans remains unknown.
- Rank the largest 20 game directories and 20 project/checkout directories. Show
  filesystem identity, scan time, errors, total measured counts, omitted counts,
  and limited Git status summaries. Git state cannot establish deletion safety:
  ignored files, stale remote refs, and active sessions remain explicit caveats.
- Git commands use read-only arguments, disable optional locks and fsmonitor, never
  fetch, and have a 10-second timeout. Do not run hooks, checkout scripts, or launchers.
  Inspect only untracked/staged state and local refs. Worktree comparisons can invoke
  clean filters, so tracked worktree contents remain unknown instead of running them.
- Monthly notification contains a fixed French message and report location, not
  untrusted filenames or instructions from project content. Existing Vitals disk
  warnings retain French wording and existing gaming deferral.

## Delivery

Prepare source/configuration and tests in the clone only. Mark all host-executed
changes in the README. Provide a separate, approval-gated activation/rollback
checklist without executing it. Installation must show diffs before approval.
Pruning is irreversible: disabling units prevents future pruning but cannot restore
expired logs or package archives. No clean Git status, successful report, or
month-old approval grants permission for later game/project deletion.
