# Broader Disk Maintenance Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use executing-plans to implement this plan task-by-task, inline in the existing clone. No subagents, git writes, host activation, builds, or dependency changes without further authorization.

**Goal:** Prepare native log/package retention, earlier Vitals alerts, and an approval-only monthly space review.

**Architecture:** Journald and vendor paccache handle only the approved automatic pruning. Vitals keeps its existing health checks, forecasts, and notification path. An unprivileged inventory script feeds a bounded monthly report into that existing notification path; no game or checkout deletion exists.

**Tech Stack:** Existing systemd/pacman-contrib, Rust Vitals workspace, Python standard library tests/report.

**Spec:** `docs/superpowers/specs/2026-10-20-disk-maintenance-design.md`

## Global Constraints

- Logs: 30 days; package archives: two versions plus everything accessed or modified within 30 days.
- Disk usage: warning at 85%, critical at 95%; preserve growth forecasting and Vitals notifications.
- Monthly game/checkout reports are advisory; removal always requires fresh, explicit approval.
- Primary security audit files retain 90 days; Turbo and all other caches remain report-only.
- No installation, enabling/starting units, live cleanup, builds, dependency changes, or git writes under this plan's preparation approval.
- Reuse existing SMART and weekly TRIM monitoring. No additional health or TRIM daemon.
- No auto-removal of personal/project/game/Docker/backup data or installed packages.

## Review Focus

- Recent access to an old package must preserve it; test both independent freshness guards with the real native tool against disposable archives.
- Exactly 85% and 95% must warn/escalate through the public collector, not only a private predicate.
- Missing/skipped/inaccessible scan roots must be visibly incomplete, never look like zero bytes or a safe-to-delete finding.
- Interrupted or concurrent reviews must preserve the last valid report; reports must never authorize deletion.
- Daily collectors must not repeat monthly notifications, and game mode must defer without losing the announcement.

---

### Task 1: Native retention and aligned package reporting

**Files:**
- Create: `system/etc/systemd/journald.conf.d/60-disk-maintenance.conf`
- Create: `system/etc/systemd/system/paccache.service.d/retention.conf`
- Create: `system/etc/systemd/system/paccache.timer.d/schedule.conf`
- Modify: `system/etc/systemd/system/package-cache-report.service`
- Modify: `tools/tests/test_cache_maintenance.py` (existing report argv contract)
- Create: `tools/tests/test_disk_maintenance.py`

**Interfaces:**
- Journald drop-in: `[Journal] MaxRetentionSec=30day`, `MaxFileSec=1day`; no new size/free-space cap.
- paccache service drop-in: reset `ExecStart`, then `/usr/bin/paccache --remove --cachedir /var/cache/pacman/pkg --keep 2 --min-atime "30 days ago" --min-mtime "30 days ago"`. Set `ProtectSystem=strict`, `ReadWritePaths=/var/cache/pacman/pkg`. Root executes only the vendor binary, not checkout code.
- Timer override resets `OnCalendar` to `daily`, sets `Persistent=true`, `RandomizedDelaySec=1h`.
- Existing package-report command stays `--dryrun --verbose`, with the same explicit cachedir and keep/freshness arguments; its timer is not a second deletion owner.

- [ ] Write tests that load drop-in settings and validate the exact systemd command boundary, writable directory, daily schedule, and journal retention/rotation. Expected RED: missing drop-ins and report argv missing the explicit cachedir.
- [ ] Add native paccache fixture tests using the production argument vector with **only cachedir replaced by the fixture path**. Create five versions named `fixture-1.0-1-any.pkg.tar.zst` through `fixture-5.0-1-any.pkg.tar.zst`; timestamp the two newest old and versions 1–3 according to each case. Assert two newest versions survive; fresh access alone preserves an older version; fresh modification alone preserves an older version; an old excess version is removed; dry-run changes no archives. Confirm the installed tool accepts fixture archives before relying on the fixture; do not mock deletion or point tests at the real cache.
- [ ] Run `python3 -m unittest discover -s tools/tests -p 'test_disk_maintenance.py' -v`; expected FAIL for missing policies. If native tools are absent inside greywall, request approved host fixture tests and report the limitation rather than weaken tests or silently skip them.
- [ ] Implement the three drop-ins and align the report service/old argv test. Never edit the vendor files, `services.txt`, dependency manifests, or Makefile. No live prune commands.
- [ ] Rerun both Python suites. Validate the merged vendor/drop-in units using `systemd-analyze verify` in a disposable unit search path containing vendor units and the proposed overrides; do not copy into `/etc` or reload the running manager. Expected: tests pass and no new validation diagnostics. Document unsupported validation steps verbatim.

### Task 2: Vitals warning/critical thresholds, preserving forecasts

**Files:**
- Modify: `apps/vitals/src/checks.rs`
- Modify: `apps/vitals/tests/collect.rs`
- Existing regression coverage: `apps/vitals/src/collect.rs` alert transition tests.

**Interfaces:**
- Preserve `checks::system(&Facts, now) -> Vec<Finding>` and `checks::filling(mount, history) -> Option<Finding>`.
- Keep each existing disk-space alert key; change its severity to `warn` for 85–<95% and `crit` at >=95%. No disk-space finding below 85%.
- Forecast remains active below 85%, with the current 14-day input window, 3-day minimum history, and <=30-day horizon. Percentage-based alert takes over at 85% to avoid duplicates.

- [ ] Add parametrized checks for usage 84, 85, 94, 95, 100: expected space severities `None, warn, warn, crit, crit`, stable key across escalation. Test forecast just below 85%, and no duplicate forecast at 85%/95%.
- [ ] Add public-entry-point integration tests using the existing `Machine` fixture and `df` shim: collect at 85%, inspect `vitals status` as warning; collect at 95%, inspect status as critical; collect at 84%, verify recovery. Keep healthy SMART fixtures so unrelated failures cannot satisfy assertions.
- [ ] Request explicit build authorization before `cargo test --manifest-path apps/Cargo.toml -p vitals`. Expected RED: boundary tests fail under the old 90% rule. If authorization is withheld, stop implementation of this task rather than claim TDD or passing tests.
- [ ] Implement the two thresholds and forecast cutoff, preserving French messages, existing keys, collection frequency, growth algorithm, SMART checks, TRIM units, gaming deferral, and warning-to-critical notification behavior.
- [ ] Rerun Vitals tests; expected all pass. Do not modify unrelated Rust code or add dependencies.

### Task 3: Monthly review report and existing Vitals announcement

**Files:**
- Create: `config/bin/.local/bin/disk-space-review`
- Create: `config/systemd/.config/systemd/user/disk-space-review.service`
- Create: `config/systemd/.config/systemd/user/disk-space-review.timer`
- Modify: `tools/tests/test_disk_maintenance.py`
- Modify: `apps/vitals/src/collect.rs`
- Modify: `apps/vitals/tests/collect.rs`
- Modify: `README.md` next to cache maintenance guidance.

**Interfaces:**
- CLI: `/usr/bin/python3 %h/.local/bin/disk-space-review`, no arguments; HOME/state/runtime from the normal XDG environment. No apply/delete/acknowledgement command.
- Output: `${XDG_STATE_HOME:-$HOME/.local/state}/disk-maintenance/latest.json`, atomically replaced, owner-only directory/file, with schema version 1. Required keys: `completed_at` (Unix seconds), `complete` (boolean), `games`, `checkouts` (ranked lists), `errors` (bounded list), `omitted_errors`, `measured_games`, `measured_checkouts`, `omitted_games`, `omitted_checkouts`, `advisory_only: true`. At most 20 entries per category. Each entry has `path`, `allocated_bytes`, `device`, and `git` (null for games; object for checkouts).
- Git summary keys: `is_repository`, `dirty` (boolean or null if unknown), `has_upstream` (boolean or null), `ahead` (integer or null), `status` (`ok` or `unknown`). Local refs do not prove remote backup; ignored files and active sessions are not certified safe. No status is named `safe_to_delete`.
- Before any Git command, reject symlinked Git metadata/`gitdir` or commondir indirection escaping the approved project roots; unsupported layouts report `unknown`. Use read-only commands with `--no-optional-locks`, disabled fsmonitor, and a 10-second timeout; never fetch, execute hooks, or mutate config/index. Cap Git output; unknown/error is not clean.
- Service: oneshot, daily-report style low priority/hardening, runtime lock, `TimeoutStartSec=15min`; writes only its state directory and runtime lock. Timer: `OnCalendar=monthly`, `Persistent=true`, `RandomizedDelaySec=1h`.
- New Vitals helper `announce_disk_review(store: &Store, path: &Path, now: i64, is_gaming: bool, send: &mut dyn FnMut(&str, &str) -> bool) -> Result<(), String>` reads that JSON and uses existing `notify` delivery. The report path is an explicit dependency for fixture isolation. Store successful delivered timestamp under `disk_review_notified_at`; retry only if delivery failed or gaming deferred it. Fixed French title/message, local report path; no filenames from the report in notifications. Call it from `collect::run` using the existing gaming detection and notification transport; do not add UI components or recurring disk-failure findings for a routine review.

- [ ] Write CLI fixture tests before implementation: rank game/project roots correctly; exclude `.parallel` from the direct-project category; discover parallel checkouts two levels down; preserve fixture content, names, and mtime; refuse any delete/apply argument; report missing/skipped/error roots explicitly; ignore symlinks without following them; skip mount boundaries; account for hardlinks once within each entry and label cross-entry estimates non-additive.
- [ ] Add fixtures for dirty/untracked/ignored Git data, no upstream, ahead of upstream, gitfile escaping roots, hostile control characters in filenames, and timed-out/failed Git reads. Assert bounded JSON, `unknown` where appropriate, no network/Git writes, and no claim of deletion safety. For races/permission/mount boundaries use narrow filesystem-operation mocks only where real fixtures are impractical.
- [ ] Add output-safety tests: runtime lock conflict fails visibly; symlink state/report paths are rejected; interrupted scan leaves the previous valid report intact; failed roots publish `complete=false` with errors and return nonzero, never zero-byte success. Missing roots are listed as unavailable (not measured); a successful scan can be complete for available roots. Enforce the documented timeout in unit-policy tests.
- [ ] Run the Python suite; expected RED because the script and units do not exist. Implement the report and units using standard-library descriptor-based traversal, no symlink following, same-device limits, atomic publication, bounded error/name lists, and explicit error status. Never delete or start anything except disposable fixture cleanup owned by the tests.
- [ ] Add Vitals tests proving a report announces once, failed delivery retries, gaming defers, missing report is quiet, malformed/future-dated report fails visibly, and a later successful report announces again. A report timestamp must be positive and no later than `now`; acknowledge only successful delivery. Existing collectors without this optional report remain healthy. Expected RED: helper/collector behavior absent.
- [ ] Implement and wire the helper with the existing store state methods and notification sender; preserve existing collection and UI behavior. Report partial-review warnings honestly but never imply automatic game/project action.
- [ ] Run both Python suites, authorized Vitals tests, authorized full `make test`, syntax checks, unit validation, and `git diff --check`; record actual outputs including failures. If a build is not approved or existing unrelated tests fail, say so and do not claim the full suite passed.
- [ ] Document native retention trade-offs, forecast thresholds, unchanged SMART/TRIM/audit/Turbo behavior, report location, and approval-only game/checkout removal. Mark every proposed host-executed script/unit/config change. Include an activation checklist as a future approval-gated operation: review/install system files; reload journald (restart only if reload unsupported, through approval); enable only vendor paccache timer and monthly review timer; keep report services read-only; verify effective policies/results. Do not run activation, add enabled units to `services.txt`, or invoke broad `make clean`.
- [ ] Document rollback: disable future cleanup, remove only these drop-ins after review, reload configuration, and preserve reports. Explain that expired logs/package archives cannot be restored by rollback. No git writes, commits, export, host installation, or live cleanup as part of implementation without further approval.

## Plan self-review

Coverage: Task 1 owns the only new automatic deletion policies; Task 2 owns
thresholds/forecast coexistence; Task 3 owns bounded advisory reporting and delivery.
SMART/TRIM and 90-day primary audit files remain untouched. No new absolute
free-space threshold, journal size cap, broader cache pruning, or automatic
personal/project deletion was approved or added. Native-tool fixtures and Rust
builds require explicit execution authorization; source preparation is not
authorization to activate maintenance on the PC.

## Execution evidence

Source/configuration prepared in `parallel-1249`, without installation, enabling,
live pruning, dependency changes, or git writes. Prior report-only work is preserved.
The current message explicitly authorized builds for verification.

### Decisions during execution

- Kept inline/self-review execution without subagents as approved; this provides
  less independent review than a separate reviewer.
- Used an external scratch ledger rather than git-writing workspace helpers; the
  durable execution evidence is recorded here instead.
- Avoided `git status` and worktree diffs because clean filters can execute project
  commands. Report untracked/staged state and local refs; tracked worktree changes
  can remain unknown and need human review.
- Unsupported gitfiles/commondir/alternates stay unknown rather than following
  metadata indirection. Some worktrees consequently have less Git detail.
- Passed the report path explicitly to the Vitals helper for deterministic tests;
  this adds one public parameter, not a global test-only override.
- Kept partial-size observations when nested links/mounts are skipped, with explicit
  incomplete labels. Otherwise common dependency symlinks would hide whole large
  projects from the ranking. Partial estimates can understate size; no deletion
  decision may rely on them.

### Test history

- Native retention RED: `Ran 7 tests in 0.004s` / `FAILED (failures=7)`.
- Native retention GREEN: `Ran 7 tests in 0.060s` / `OK`.
- Vitals thresholds RED: `test result: FAILED. 91 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s`.
- Public threshold entry point RED: `test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.02s`.
- Threshold GREEN: 93 unit tests and 8 integration tests passed.
- Monthly Python RED: `Ran 19 tests in 0.013s` / `FAILED (failures=21)` (missing implementation; subtest failures counted separately).
- First host Python run: `Ran 44 tests in 0.865s` / `FAILED (failures=1)`. The Git fixture lacked a remote fetch refspec. Native Git reproduced `upstream branch 'refs/heads/main' not stored as a remote-tracking branch`; corrected the fixture, not production defaults.
- Monthly delivery RED: unresolved import `vitals::collect::announce_disk_review`; public collector test failed because it ignored the report. Subsequent fixture failure exposed inherited state-home variables; explicitly isolated both XDG_STATE_HOME and CLAUDE_SESSIONS_STATE_HOME in `Machine`.
- Partial-size regression RED: `Ran 27 tests in 0.562s` / `FAILED (failures=2)`; all other cases, including Git ahead/filter behavior, passed. Fixed omission of partially measured entries.
- Host Python GREEN before final review fix: `Ran 47 tests in 0.937s` / `OK`.
- Final approved host Python suite: `Ran 48 tests in 1.184s` / `OK`; no skipped tests. Repeated Python syntax and user/system unit validation exited 0 without diagnostics.
- Final Git-error regression RED: `Ran 1 test in 0.012s` / `FAILED (failures=1)`. Exit 1 from `ls-files` was wrongly accepted as inventory success; now only the documented diff/upstream commands allow exit 1. Inventory failures remain unknown, not clean.
- Vitals final targeted run: 93 unit, 9 collector integration, and 6 monthly-delivery integration tests passed; no ignored tests.
- Python compilation, approved host user/system unit validation, merged native vendor/drop-in validation, and `git diff --check` passed. User-unit validation inside greywall instead failed with `Failed to create directory '/run/user/1000/systemd/': Permission denied`; reported using `claude-todo blocked`, then validated through approved host execution without installing/starting units.

### Full-suite limitation (not suppressed or fixed out of scope)

`CARGO_NET_OFFLINE=true make test` exited 2 with:
`test result: FAILED. 42 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.03s`.

`cargo test --offline --release --quiet --manifest-path apps/Cargo.toml --no-fail-fast`
exited 101: **969 passed, 15 failed, 0 ignored**, across four unchanged targets.
All Vitals targets passed. No failing tests were disabled. These failures are outside
the modified disk-maintenance code; no clean baseline run was performed.

Failure names, verbatim:

```text
yes_and_no_from_the_phone_press_the_dialog_keys
compact_is_typed_only_at_the_prompt
exit_types_slash_exit_at_an_idle_prompt
handoff_types_rc_at_an_idle_prompt
prompt_commands_pause_between_the_text_and_its_enter
restart_exits_then_resumes_the_same_conversation
say_refuses_while_a_dialog_is_open_and_keeps_the_text_out_of_the_log
say_sends_at_the_prompt_and_queues_while_working
a_session_started_from_a_shell_reports_even_under_the_daemon
tests::a_credits_session_is_marked_in_the_snapshot_and_the_reboot_record_until_it_switches_back
tests::a_request_held_or_refused_is_audited_without_its_command
tests::a_sessions_agent_is_in_the_snapshot_and_the_reboot_record_and_claude_by_default
tests::a_terminal_session_is_marked_in_the_snapshot_and_the_reboot_record
tests::sessions_are_remembered_across_reboots
tests::the_daemon_writes_its_messages_to_the_shared_log
```

The phone test returned 409 instead of 200; script fixtures showed unexpected
`C-x Enter` behavior; desktop fixtures attempted to write read-only real host state
(`Read-only file system (os error 30)`). Reported the verification block using
`claude-todo blocked`; no sandbox workarounds or unrelated fixes applied.

**Status before the authorized follow-up:** all three implementation tasks were
prepared and targeted checks passed. Full-repository verification was red as
recorded above. No deferred
minor review findings; no installation, enabling, live pruning, or git writes.

## Authorized follow-up: test setup fixes

Further investigation identified all 15 failures: eight tests depended on the
missing `apps/target/release/osd` classifier binary (not OpenCode key handling),
six desktop tests inherited `CLAUDE_SESSIONS_STATE_HOME` instead of keeping state
inside their fixture, and one shell-launch hook fixture inherited `OPENCODE=1`.
Without source changes, isolating the state override made all 126 desktop tests
pass; isolating OPENCODE made all 25 hook tests pass. Removing OPENCODE did not fix
the missing-classifier failures, confirming the distinction.

The user then explicitly approved fixture fixes and changing `make test` to build
the classifier first, including test builds. Changes are confined to the Makefile,
desktop/hook test fixtures, and regression tests; no installed program behavior or
disk policy changed.

New regression evidence, before implementation:

- Make entry-point tests: `Ran 2 tests in 0.012s` / `FAILED (failures=2)`.
- Desktop fixture escape test: `test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 126 filtered out; finished in 0.00s` with `the fixture wrote into the inherited host state folder`.
- Deliberately marked shell-launch fixture: `test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 24 filtered out; finished in 0.31s`.

Fixes: build the real OSD classifier before running Rust tests; set the desktop
fixture's state override to its private directory; explicitly remove OPENCODE for
the fixture modeling a user-started shell session. The hook fixture deliberately
injects OPENCODE first so the regression also works outside an agent-launched run.
The Makefile entry-point tests verify command ordering and that a failed classifier
build stops testing. After the change: `Ran 2 tests in 0.009s` / `OK`.

The Makefile runs outside the sandbox when invoked by the user; its new build step
is intentional and the misleading `[read-only]` help label was removed. No install,
reload, service start, git write, disabled test, or static-analysis suppression.

Final follow-up verification:

- `CARGO_NET_OFFLINE=true make test` completed successfully: **985 passed, 0 failed,
  0 ignored** across the Rust targets. This includes all previously failing tests
  and the new desktop fixture regression; no environment-scrubbing wrapper was used
  to hide failures.
- Full Python fixture suite: `Ran 50 tests in 1.388s` / `OK`.
- `git diff --check` passed. The test-fix implementation spans four files:
  **63 insertions(+), 1 deletion(-)**, plus this updated execution record.
- Existing warnings about `unused_mut` and the unused `settings` method remain
  unchanged in cockpit tests; no suppressions or unrelated fixes were added.
- Completed todos `ac640661` and `c8aada6f` after verification.

**Current status:** maintenance implementation and test-setup fixes are verified.
Everything remains in the clone: no installation, enabling, service reload,
automatic live pruning, or git writes have occurred.
