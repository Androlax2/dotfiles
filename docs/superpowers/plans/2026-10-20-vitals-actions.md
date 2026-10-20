# Vitals Action-First Disk View Implementation Plan

> **For agentic workers:** Execute inline with test-driven-development. Preserve the existing no-subagents choice. Do not commit, install, or change live maintenance without separate approval.

**Goal:** Give the Disques tab an ADHD-friendly default: one status, at most three next actions, optional technical details, and explicitly requested read-only checks.

**Architecture:** Keep the native maintenance snapshot and existing detailed view. Add two fixed, unprivileged report-start commands, a compact action selector, and an optional skipped-symlink counter. Opening Vitals or refreshing data never starts a job.

**Tech Stack:** Existing Rust/Ratatui/systemd and existing test fixtures. No dependencies.

**Spec:** User's approved scope in this conversation: one status, up to three actions, details hidden by default, direct read-only checks. This supersedes the earlier display-only restriction solely for the two user report services.

## Global Constraints

- Never expose deletion, package pruning, root service start, or arbitrary command/unit input.
- Only `disk-space-review.service` and `cache-maintenance-report.service` may be requested, in user scope, asynchronously, after Enter on a selected action.
- Preserve SMART warnings, 85%/95% disk alerts, all raw details and scrolling behind `d`.
- Do not hide genuine failures or invent zero skipped links for legacy reports.
- Preserve existing cleanup policies and symlink protection. The producer fix is already deployed by commit b7878b9; do not duplicate or overwrite it from this older checkout.

## Review Focus

- A stale report must suggest refresh, not claim current success.
- A full/failed disk takes priority over optional cache checks.
- Enter must never launch a destructive or privileged service.
- Running checks must not be duplicated; command failures must be visible, not optimistic success.
- Compact, narrow-terminal navigation must keep the selected action and footer accessible; details remain reachable.

### Task 1: Read-only checks and skip compatibility

**Files:** `apps/vitals/src/maintenance.rs`, `apps/vitals/tests/maintenance.rs`.

- [ ] Add failing native-boundary tests for `request_check(Check::SpaceReview|TurboReport)`, asserting exact `systemctl --user start --no-block` arguments and useful command failure results.
- [ ] Add tests for optional `skipped_symlinks`: positive/zero values are explicit; absent is legacy unknown; invalid values fail. Preserve real errors and partial flags.
- [ ] Watch RED, implement only the two fixed commands and optional counter, then verify GREEN with authorized builds/tests.

### Task 2: Compact actionable view

**Files:** `apps/vitals/src/tui.rs`, `README.md`.

- [ ] Add TestBackend/key tests before implementation: details hidden by default; no more than three selectable actions; truthful status at healthy/85%/95%/SMART-failure/unknown/error states; expected skipped links are not errors.
- [ ] Actions are bounded to show the report/details, request a new space report, or request the Turbo dry-run report. Prioritize reading data when available and retrying an absent/failed report when needed. Showing details never launches a check.
- [ ] Add tests for `d` detail toggling, Enter on display-only actions, selection and scroll bounds, already-running job protection, failed request feedback, and preserving Care-tab/navigation behavior.
- [ ] Watch RED, implement the compact default with French one-line labels and one short explanation for the selected action. Keep native policies, raw paths, timestamps and diagnostics in the existing expanded details.
- [ ] Show queued requests as requested, not completed; refresh native facts after a successful request. No job is launched on refresh or opening a tab.
- [ ] Update README; run Vitals tests, full make test when build-authorized, existing Python fixtures through host approval where needed, and git diff --check. Record all failures. Stop before git writes/deployment pending approval.

## Execution notes

The user approved the action-first scope and direct read-only checks. Implementation
is in the clone only; no git writes or UI deployment are authorized by that approval.

- The producer's symlink fix was already deployed (`b7878b9`) and its shared todo
  was already complete. The original report predated it. A separately approved
  fresh read-only run now reports 93,501 ignored symlinks and a much smaller set of
  other diagnostics; it still exits 1 because inaccessible test folders remain.
  No producer code was duplicated from the stale clone, and no permissions changed.
- Optional `skipped_symlinks` preserves backward compatibility: absent means legacy
  unknown, not guessed zero. Genuine aggregate error counts and partial flags remain.
- Two fixed typed check requests target only user report services. An accepted
  request is queued, not completed. Running checks are not repeated, hidden actions
  cannot be triggered, and native command failures remain visible.
- Default Disques is a short status and at most three useful actions. A separate
  readable review puts up to three sanitized diagnostic paths before optional game
  suggestions. Raw SMART, policies, timestamps and job diagnostics remain behind d.
- Actions have no numbered shortcuts, avoiding a conflict with the existing 1–7
  tab navigation. Selection retains action identity across data refreshes.

Verification so far:

- Native action/counter RED: missing Check/request_check/skipped_symlinks, exit 101.
- Native action/counter GREEN: 14 tests passed, 0 ignored.
- Compact-view RED: missing screen/selection/key-handler behavior, exit 101.
- First runtime run: `test result: FAILED. 110 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s`.
  The small viewport scrolled its status away to fit long help text. Compact help
  and bounded header feedback fixed that; no assertions were disabled.
- Subsequent targeted and full Rust runs passed. Python: `Ran 50 tests in 0.944s` / `OK`.
- Diagnostic-sample RED: missing issues field, exit 101. Added a bounded typed sample
  and French access/race guidance; no retrospective rewriting of old reports.

Final self-review caught one display regression: shortening a parallel checkout to
only `parallel-1924` lost its parent project. The new rendered-buffer regression
failed with `0 passed; 1 failed; 115 filtered out`; readable labels now preserve
`dotfiles / parallel-1924` without restoring a wall of full paths. Full paths remain
available in Details.

Final verification:

- `CARGO_NET_OFFLINE=true make test`: **1,181 passed, 0 failed, 0 ignored**.
- Vitals: 116 unit, 9 collector, 6 delivery and 15 maintenance integration tests pass.
- Final maintenance group: `test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s`.
- Python fixtures: `Ran 50 tests in 0.944s` / `OK`; no Python source changed in this update.
- `git diff --check` passed. No test disablement, baseline additions, or dependency changes.

Author self-review completed, retaining the previously chosen no-subagent approach.
No deferred minor findings. Existing scheduled cleanup, root units, permissions and
producer behavior remain unchanged. The only live scan was the separately approved
refresh of the already-installed producer; new UI checks were exercised only with
fixtures. No git writes or UI installation occurred during this implementation.

## Authorized delivery verification

The user subsequently approved committing, fetching/rebasing this unpushed branch,
and opening the live application approval. The UI commit rebased cleanly onto
`a64c3bf`, preserving the already-applied producer fix and newer dotfiles changes.

That base adds a shared low-priority CPU build runner and Python preflight tests to
the Makefile. The earlier entry-point fixture ran make in an empty temporary cwd,
so both regressions failed (`Ran 2 tests in 0.107s` / `FAILED (failures=2)`) before
even reaching their fake cargo. Changed only that fixture's cwd to the repository:
the real preflight and runner now execute, while cargo remains the disposable fake.
No build-budget settings, production recipes, or tests were bypassed. Regression
GREEN: `Ran 2 tests in 1.163s` / `OK`.

Post-rebase full verification, approved host request b3ef467acb0b:

- Rust: **1,269 passed, 0 failed, 0 ignored**.
- Python: `Ran 65 tests in 2.596s` / `OK`.
- Vitals: 116 unit, 10 collector, 6 delivery, and 15 maintenance integration tests.
- No dependency, root policy, permission, or producer changes were introduced by
  this UI/fixture update. Live application still awaits its separate host approval.
