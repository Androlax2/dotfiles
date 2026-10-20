# Vitals Maintenance Visibility Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use executing-plans inline, without subagents as previously chosen. No git writes or host deployment without further explicit approval.

**Goal:** Make installed disk maintenance visible in Vitals without adding any cleanup controls.

**Architecture:** A small Rust module gathers read-only native systemd properties and the existing monthly JSON report. The existing Disques view renders that snapshot, refreshed with facts, with scrolling. Native retention and running services stay unchanged.

**Tech Stack:** Existing Rust/Ratatui/chrono/serde_json and systemd; no dependencies added.

**Spec:** `docs/superpowers/specs/2026-10-20-vitals-maintenance-design.md`

## Global Constraints

- French, existing Disques tab, read-only, no deletion or maintenance-trigger controls.
- Preserve SMART/space information, existing tab navigation, and Care selection keys.
- Do not infer success from a timer trigger or systemd's default Result=success.
- Unknown/missing/unavailable data stays explicit; partial estimates stay partial.
- No root commands, new daemon, dependency changes, live installation, or git writes.

## Review Focus

- Enabled versus running versus completed: unit fixtures must prove these are independent.
- Missing privileges/history/units must not produce a green successful task.
- Missing, partial, malformed, oversized, future-dated, or hostile-name reports must render safely.
- A short terminal must still reach every maintenance/review section and retain its footer.
- TUI collection must use both correct systemd scopes without running any state-changing command.

---

### Task 1: Read-only maintenance snapshot

**Files:**
- Create: `apps/vitals/src/maintenance.rs`
- Modify: `apps/vitals/src/lib.rs` (export module)
- Create: `apps/vitals/tests/maintenance.rs`

**Interfaces:**
- `pub fn gather() -> Maintenance`: snapshot containing `tasks: Vec<Task>`, `journal_retention: Result<Option<String>, String>`, and `review: Result<Option<Review>, String>`.
- `Task`: fixed known timer/service IDs and French label; `enabled: Option<bool>`, `last_run: Option<i64>`, `next_run: Option<i64>`, `state: RunState`, `detail: Option<String>`, approved-policy description. `RunState` variants: `Running`, `Succeeded`, `Failed`, `Unknown`.
- `Review`: `completed_at: i64`, `complete: bool`, measured/omitted game/checkout counts, unavailable/error counts, and bounded game/checkout entries. Each entry exposes `path: String`, `allocated_bytes: u64`, `complete: bool`.
- Query `paccache`, `package-cache-report`, `fstrim` timer/service pairs in system scope, and `cache-maintenance-report`, `disk-space-review` pairs in user scope. Batch each scope with `systemctl show --no-pager --timestamp=unix --property=Id,LoadState,UnitFileState,ActiveState,LastTriggerUSec,NextElapseUSecRealtime,Result,ExecMainStartTimestamp,ExecMainExitTimestamp,ExecMainStatus` followed by the exact unit names. Parse blank-separated property blocks by Id, not output order.
- Running service takes precedence; a completed successful service needs both successful result/exit status and a real completion timestamp. Failed result/exit status remains failed. Last run uses a known service start or timer trigger, labeled independently from the completion result; missing values are unknown, not never-run claims.
- Read `systemd-analyze cat-config systemd/journald.conf`, selecting the last active `MaxRetentionSec=` value. Ignore comments. Missing explicit value means native/default policy, not disabled retention.
- Review path is `cockpit::paths::state_home()/disk-maintenance/latest.json`, bounded at 1 MiB, schema version 1 and advisory_only=true. Validate required counts, entry types, and positive completion timestamp <= now. Keep at most three entries per category for display. No project content or journal reads.

- [ ] Write native-boundary tests using temporary HOME/state/runtime and executable systemctl/systemd-analyze shims, with an environment mutex and restore guard local to this integration-test process. Exercise `gather()`, not a duplicate parser or framework mock.
- [ ] Cover enabled-but-never-completed, running, successful completion, failed exit, disabled-but-previously-successful, missing units, command failure, reboot-lost timestamps, mixed property order, and both system/user scopes. Assert shim command logs contain no start/enable/reload/delete operations.
- [ ] Cover valid complete/partial review, no report, bad JSON/schema/counts, >1 MiB, future time, and control characters in paths. Use literal fixture values, not expected values derived from implementation helpers.
- [ ] Run `cargo test --offline --manifest-path apps/Cargo.toml -p vitals --test maintenance`; expected RED: missing module/behavior. Request build authorization first if not granted.
- [ ] Implement the module using existing command/path conventions, retaining meaningful read errors in the snapshot instead of swallowing them. Keep parsing helpers local; no service/repository facade or global hook.
- [ ] Rerun the integration test and Vitals suite; expected all pass. No production monitoring/retention changes.

### Task 2: Existing Disques tab and activation record

**Files:**
- Modify: `apps/vitals/src/tui.rs`
- Modify: `services.txt`
- Modify: `README.md` (maintenance visibility and now-active deployment status)
- Modify: `docs/superpowers/plans/2026-10-20-disk-maintenance.md` (activation outcome record)

**Interfaces:**
- View owns `Option<maintenance::Maintenance>` and a disk scroll offset. Refresh the snapshot only in the existing facts refresh branch; manual `r` refresh invalidates both.
- Existing Disques content remains, followed by `Maintenance` rows: label, enabled/disabled/unknown, running/completed-result, last run, next run, approved rule. Journal retention gets a separate native-policy line.
- `Revue mensuelle` shows report timestamp, complete/partial status, counts, top three game/checkout paths and allocated sizes (with partial labels), and error/unavailable counts. Pending report plus running service displays in progress; missing report without a running job displays unavailable.
- Scroll is clamped to rendered content and viewport; use Ratatui's installed API (verify wrap/line-count APIs before calling them). Arrows/j/k and paging apply only to Disques; Care controls and other tabs stay unchanged.

- [ ] Add TestBackend rendering tests before changes: complete snapshot, failed/unknown jobs, running/no report, partial report, and a short terminal scrolled to the final review rows. Assert real rendered buffer text/style and accessible footer, not private string helpers alone.
- [ ] Add navigation tests for clamping, page/home/end behavior, and preserving Care selection. Add an explicit manual-refresh test proving maintenance is refreshed with facts and not on every frame.
- [ ] Run new tests; expected RED: maintenance not visible or scroll unavailable. Implement rendering and narrowly scoped navigation without redesigning the UI or adding destructive actions.
- [ ] Add `paccache.timer` and `package-cache-report.timer` to the alphabetically ordered system `services.txt`, reflecting the already-approved host activation. Do not track unrelated user-generated wants links or `.handoff/`.
- [ ] Update README with where to see maintenance, scroll keys, unknown/partial behavior, and read-only boundaries. Record installed/active settings and the first-review state honestly; verify live result with read-only host approval if needed. Keep historical test results as historical evidence.
- [ ] Run all Vitals tests, full `CARGO_NET_OFFLINE=true make test`, Python fixture suites through approved host execution where greywall ancestor checks require it, and `git diff --check`. Record actual outcomes and failures verbatim; never skip or baseline failures.
- [ ] Self-review as previously chosen; show diff stat and summarize preserved behavior. Stop before staging/committing/exporting/applying new UI changes until explicitly authorized.

## Self-review

Both tasks share only the explicit Maintenance snapshot. Native statuses are kept
separate from approved policies; journal has no fabricated timer. Monthly data
reuses the existing report without scanning. Every review-focus case has an owning
test step. No new UI framework or FigCraft file is introduced. Implementation can
stay within this module, existing TUI, tests, and the approved activation record.

## Execution record

Implemented in the existing clone, inline and self-reviewed without subagents.
No UI installation, git writes, retention changes, or maintenance-trigger controls.
The two enabled system timers are recorded in `services.txt`; untracked `.handoff/`
deployment artifacts and unrelated host links remain untouched.

### Decisions

- Used an external scratch ledger instead of helpers that could write git metadata;
  this document is the durable record. No additional git authorization was inferred.
- The optional UI-skills CLI was unavailable offline (`ENOTCACHED`); reported it
  through `claude-todo blocked`, without package installation or a network workaround.
  No FigCraft files exist; the current Ratatui layout/style is retained.
- Ratatui's wrapped-line count API requires an unenabled unstable feature. Avoided
  manifest changes: existing styled-grapheme iteration wraps the content before
  calculating scroll bounds. This wraps at grapheme rather than word boundaries;
  long Unicode paths remain reachable and styles are preserved.
- The cache gate accepts a plain read closure for direct refresh/TTL regression
  tests, not a service abstraction or global test hook. Facts and maintenance
  refresh together every 30 seconds, with explicit `r` invalidation.
- Failed systemd batches preserve any valid unit data while retaining diagnostics;
  absent units stay unknown. A timer trigger or default successful result is never
  treated as proof of completed maintenance.

### Verification history

- Native-boundary RED: `unresolved import vitals::maintenance`, exit 101.
- Initial native-boundary GREEN: `test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s`.
- UI RED: missing maintenance/scroll fields, cache helper and key handler (exit 101).
- First rendered UI run: `test result: FAILED. 12 passed; 1 failed; 0 ignored; 0 measured; 87 filtered out; finished in 0.05s`. The long path was visibly wrapped between terminal borders; corrected the assertion to ignore border glyphs rather than require a particular word break.
- Subsequent Vitals GREEN: 102 unit tests, 9 collector, 6 delivery, and 9 maintenance integration tests passed.
- First full repository run: 1,085 passed, 0 failed, 0 ignored.
- Approved host Python run: `Ran 50 tests in 2.892s` / `OK`.
- Added coverage for fractional timestamps, reversed unit order, valid units retained
  in failed batches, and the existing red failure style in actual TestBackend cells.
- Self-review found journald parsing needed whitespace, section, and empty-reset
  semantics. Regression RED: `test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.00s` (got `60day`, expected `30day`). Implemented last active `[Journal]` assignment/reset parsing rather than hiding the incorrect value.

Live read-only verification confirmed Unix timestamp formatting and the partial
monthly report (28 games, 71 checkouts, service exit 1, 92,713 omitted diagnostics).
Sampled entries were skipped Wine symlinks. A separate follow-up was proposed to
distinguish expected skips from real read failures; this change does not alter that
reporting or deletion policy. Vitals shows the partial state, not fabricated success.

### Final verification

`CARGO_NET_OFFLINE=true make test` passed **1,088 Rust tests, 0 failures, 0 ignored**.
Vitals includes `103 passed; 0 failed` unit tests, 9 collector integration tests,
6 review-delivery tests, and 11 maintenance integration tests. The final native
maintenance group reported: `test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s`.

The approved host Python suite passed all 50 tests, as recorded above.
`git diff --check` passed. Author self-review completed; no deferred minor findings.
The one journal-parser finding is covered by its RED→GREEN regression and the
green full suite. No test was disabled, no manifest or baseline changed, and no
live maintenance settings were modified by this UI implementation.

Both implementation tasks are complete in the clone. Commit, patch application,
and live rebuild of this UI revision still require separate authorization.
