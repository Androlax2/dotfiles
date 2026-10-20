# Disk maintenance visibility in Vitals

## Approved scope

Show what disk maintenance is enabled and its retention rules; last and next runs;
success, failure, or running status; and the monthly space-review summary. Everything
is read-only. Games/projects must never be deleted automatically or from this view.

Maintenance is already installed and enabled on the host. The first monthly review
was still running at the last check. Do not interpret a missing report as success.
Activation recording in `services.txt` and a feature-branch commit was approved
before this additional UI requirement; finish the record alongside the view, but
obtain explicit git/deployment approval again before committing/installing UI changes.

## Implementation choices

- Extend the existing French **Disques** tab, keeping its SMART and space information.
  Add a maintenance section and a bounded monthly-review summary; preserve existing
  typography, colors, tabs, and key conventions. No browser or FigCraft design files
  exist in this checkout, and no new visual identity is needed.
- Include the native pacman cleanup, package dry-run report, Turbo report, monthly
  review, and weekly TRIM. Journald has native age retention, not its own timer:
  display its effective configured retention separately, without inventing run times.
- Obtain schedules/results from systemd in read-only batches for system/user scopes.
  No new daemon, root command, dependency, journal scraping, or automatic operation.
- Refresh maintenance with the existing slower facts refresh, not every frame.
- Absent units, permission failures, absent completion timestamps, and reboot-lost
  history must display unknown/unavailable, never fabricated success or zero.
  A trigger is not proof that a service completed. Distinguish enabled from active.
- Display the agreed non-journal rules as **Politique approuvée**, not as a claim
  that arbitrary custom service commands have been audited. Journald's configured
  value comes from effective config; unavailable/default values are explicit.
- Read the existing bounded `disk-maintenance/latest.json`; show completion time,
  complete/partial status, measured game/checkout totals, unavailable/error counts,
  and at most three largest entries per category. Partial entry sizes remain marked.
  Reject malformed, oversized, or future-dated reports. Sanitize control characters.
- Add vertical scrolling within Disques using arrows/j/k, PageUp/PageDown, Home/End.
  Preserve Care-tab selection and all existing navigation. Footer documents the keys.
- No deletion buttons, enable/disable controls, manual cleanup shortcuts, guessed
  reclaimed bytes, or scanning launched by opening Vitals.

## Delivery

Prepare code/tests in the clone first. Preserve installed maintenance and unrelated
host work. Build/test only when explicitly authorized; no dependency manifest changes.
User-facing changes need a separately approved patch application and live rebuild.
Record native enabled timers in the clone's `services.txt`: `paccache.timer` and
`package-cache-report.timer`; do not stage `.handoff/` or unrelated enablement links.
