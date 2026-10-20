# Execution evidence — vitals-guided-disks.md

Scope: only `apps/vitals/src/tui.rs`, the relevant `README.md` section, and this evidence file. Existing `.handoff/` and plan files preserved. No git writes, deployment, app control, dependencies, producer/model or unit changes.

Pre-flight: Summary rendering, keyboard routing and the header share `DiskOverview.primary`; Review/Details share the native snapshot. All are changed together without altering its schema.

Ruling: execution evidence lives here instead of skill-generated workspace/scripts, and final review is handed back to the parent — the explicit file allowlist and prohibition on further agents supersede those generic workflow steps. UI CLI discovery was not run because it could install dependencies; this is execution of an accepted Ratatui plan, not a new design.

## RED

Approved host command (shared low-priority runner):

`python3 config/bin/.local/bin/agent-build cargo test --offline --release --manifest-path apps/Cargo.toml -p vitals tui::tests`

- Request `a55bc06cfe0d`: `test result: FAILED. 25 passed; 9 failed; 0 ignored; 0 measured; 87 filtered out; finished in 0.01s` (exit 101).
- Strengthened stale selection to press Enter before navigation; made the healthy fixture actually complete; added incomplete/unknown report coverage.
- Request `24cf68c8d999`: `test result: FAILED. 24 passed; 11 failed; 0 ignored; 0 measured; 87 filtered out; finished in 0.00s` (exit 101).

The second RED failures were:

- `a_running_check_is_not_requested_again_and_request_errors_are_visible`
- `access_errors_stay_explicit_but_diagnostics_are_only_in_details`
- `details_manual_controls_cannot_bypass_critical_or_running_guards`
- `details_manual_controls_dispatch_only_the_two_reports`
- `entering_a_check_requests_only_that_check_and_never_claims_it_completed`
- `guided_review_does_not_leak_invalid_report_errors`
- `partial_and_unknown_reports_are_explained_without_counts_or_paths`
- `stale_secondary_selection_never_overrides_the_summary_recommendation`
- `summary_header_keeps_critical_machine_alerts_without_technical_totals`
- `summary_navigation_cannot_select_hidden_actions_on_small_terminals`
- `the_default_disk_view_has_one_recommendation_and_no_wall_of_details`

Failures showed the old three actions/schedule, diagnostic leaks, missing Details controls/feedback, hidden selection launch, partial report shown as healthy, and missing global critical header. No compilation errors or real service starts.

## GREEN and final verification

- Request `443637ef7117`, same targeted command: `test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 87 filtered out; finished in 0.00s` (exit 0). The approval wait exceeded 120 seconds; continued with `ask-to-run --wait 443637ef7117`, not a duplicate request.
- Self-review found that the inherited request-failure override hid a real 85% space warning: `request_failed` excluded only critical status, and the hint branch also replaced warning explanations with optional feedback. Added `request_feedback_never_replaces_real_space_or_ssd_warnings` before changing this behavior, plus a two-size guided-state matrix.
- Request `c1ca0c6746f5`, same targeted command: `test result: FAILED. 36 passed; 1 failed; 0 ignored; 0 measured; 87 filtered out; finished in 0.00s` (exit 101). Only `request_feedback_never_replaces_real_space_or_ssd_warnings` failed; its rendered 85% warning was replaced by “Vérification non lancée”. Fixed at the shared Summary title/hint precedence: both warning and critical statuses now stay visible; raw refusal detail stays in Details.
- Request `809e4a1d8167`: `python3 config/bin/.local/bin/agent-build cargo test --offline --release --manifest-path apps/Cargo.toml -p vitals` (exit 0):
  - `test result: ok. 124 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s`
  - `test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s` (`collect`)
  - `test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` (`disk_review`)
  - `test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s` (`maintenance`)
  - Binary and doc-test targets: 0 tests, all successful. All 37 UI tests passed within the 124 unit tests.
- Request `00238cc30840`: `env CARGO_NET_OFFLINE=true make test` (exit 0). Agent-build fixtures: `Ran 12 tests in 0.439s` / `OK`. The real OSD prerequisite build succeeded; all workspace Rust targets passed (1,277 tests total; zero failures/ignored tests). Full output inspected, including the beginning hidden by the tool's tail preview.
  - Existing out-of-scope compiler warnings: `unused Result that must be used` at `apps/desktop/src/tests.rs:1561` and `:1566`; `variable does not need to be mutable` at `apps/cockpit/src/tests.rs:260`. These files were not edited; warnings were not suppressed.
  - FigCraft “started” messages in this suite come from temporary test fixtures, not opening a user's app.
- Request `1c274b1b8f3b`: `python3 -m unittest discover -s tools/tests -v` (exit 0): `Ran 65 tests in 2.484s` / `OK`. Maintenance tests operate on temporary fake data only.
- `git diff --check`: exit 0, no output.

## Delivered behavior and preserved boundaries

- Summary renders exactly one recommendation from `DiskOverview.primary` and never dispatches stale secondary selection. Home/End/arrows can no longer select secondary tools. Raw paths, error/scan totals, dates and schedules stay out of Summary/Review.
- Review keeps human-readable game/project sizes, parallel-project context, partial-size labels, safe manual-decision wording, and count-free incomplete/unknown notes. Raw diagnostics and policy/schedule data remain in Details.
- Details displays `a : bilan d’espace` and `t : rapport Turbo`, clearly labeled read-only. Both use the same typed `Check` dispatch and SSD-critical/running protections. Successful requests still mean queued, not completed. Feedback is visible at the top even after scrolling.
- Summary's machine header is count-free, preserves critical red alerts even without collection history, and makes unknown state explicit. Other headers, seven tab shortcuts, Care handling, and read-only refresh behavior are preserved.
- No host-executed script, Makefile, unit, installed setting, dependency, producer or schema was edited. No deployment, install, scan, app control, stage or commit was performed.

Ruling: legacy reports without a symlink count remain explicitly uncertain even when they report no errors; incomplete reports no longer inherit the healthy headline. This follows the accepted unknown-data requirement; the cost is a conservative “prepare/update review” recommendation for a legacy report that might otherwise be usable.

Final review: implementation self-review only (no further agents permitted); parent retains independent review. No unresolved test failures. No live-window validation was attempted because app opening/restarting/deployment is explicitly excluded. Parent should review the diff and, after separate deployment approval, validate a freshly launched Vitals window.

Tracked diff stat at verification: `README.md | 22 +--`, `apps/vitals/src/tui.rs | 391 ++++++++++++++++++++++++++++++++++---------------`; 2 tracked files, 283 insertions(+), 130 deletions(-). This new evidence file is untracked alongside the preserved original plans and `.handoff/`.

## Parent-review corrections — optional action or nothing needed

The parent clarified the accepted contract in `vitals-guided-disks.md`: **at most** one action, not an always-present recommendation. This section supersedes earlier “exactly one” delivery descriptions; earlier results remain execution history.

Root cause: Summary unconditionally rendered “Prochain pas” and `primary` could not express absence. The running-first-report branch therefore reused `SpaceCheck`, whose Enter handler merely reported that it was already running.

Changes:

- `DiskOverview.primary` is now `Option<DiskAction>`. Existing warnings keep their priority and actions; a running first report has `None`.
- Healthy report reading and a previous report available during a new scan use “Si tu veux libérer de la place”. Enter still opens the optional report without requesting a check; the footer no longer calls this a “prochain pas”.
- A running first scan shows “Rien à faire pour le moment” and explains that Vitals may be closed while waiting for the notification. It renders no action and no Enter instruction, retaining `d`, `r` and `q`. Optional stale request feedback cannot replace this waiting reassurance; raw feedback is still available in Details.
- Enter, selection/navigation keys and hidden report shortcuts leave this no-action state and its request feedback unchanged. Empty selection navigation returns safely rather than indexing an empty action list. Details-only reports, typed whitelist, running/SSD-critical guards, and queue-not-completion semantics remain unchanged.
- README now documents optional reading and waiting without a redundant action.

### RED → GREEN evidence

All compilation/test requests used `ask-to-run` approval and the shared low-priority runner where applicable, with Cargo offline. No service starts, producer/schema changes, dependency changes, git writes, installation, deployment or user-window control.

- RED request `c64d416d2333`:
  `python3 config/bin/.local/bin/agent-build cargo test --offline --release --manifest-path apps/Cargo.toml -p vitals tui::tests`
  → `test result: FAILED. 35 passed; 4 failed; 0 ignored; 0 measured; 87 filtered out; finished in 0.01s` (exit 101).
  Failing tests:
  - `a_running_first_review_reassures_without_an_action_or_enter_instruction`
  - `all_guided_states_keep_at_most_one_next_step_and_refresh_never_starts_a_scan`
  - `healthy_and_running_previous_reports_are_optional_to_read`
  - `the_default_disk_view_has_one_recommendation_and_no_wall_of_details`
  Rendered failures showed the compulsory-looking heading and redundant first-scan action/Enter instruction, not compilation errors.
- GREEN request `a9008dee1d9e`, same command:
  `test result: ok. 39 passed; 0 failed; 0 ignored; 0 measured; 87 filtered out; finished in 0.00s` (exit 0).
- Strengthened the waiting key test to assert unchanged request feedback, not just absence of callback dispatch; verified this in the complete package and repository runs below.
- Full package request `b8d16c0afef1`:
  `python3 config/bin/.local/bin/agent-build cargo test --offline --release --manifest-path apps/Cargo.toml -p vitals` (exit 0):
  - `test result: ok. 126 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s`
  - `test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s` (`collect`)
  - `test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` (`disk_review`)
  - `test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s` (`maintenance`)
  - Binary/doc-test targets: 0 tests, all successful. Total: 157 tests, including all 39 UI tests.
- Full repository request `f4bebd788124`:
  `env CARGO_NET_OFFLINE=true make test` (exit 0). Build-runner fixtures: `Ran 12 tests in 0.595s` / `OK`. Real OSD prerequisite build succeeded; all 1,279 Rust workspace tests passed, zero failed/ignored. Full output inspected including the truncated head. The same three existing out-of-scope warnings remain at `apps/desktop/src/tests.rs:1561`, `:1566` and `apps/cockpit/src/tests.rs:260`; nothing was suppressed.
- Python request `eed34f476102`:
  `python3 -m unittest discover -s tools/tests -v` (exit 0): `Ran 65 tests in 3.403s` / `OK`.
- `git diff --check`: exit 0, no output.

Current tracked diff from HEAD: `README.md | 30 ++--`, `apps/vitals/src/tui.rs | 479 ++++++++++++++++++++++++++++++++++++-------------`; **2 tracked files changed, 373 insertions(+), 136 deletions(-)**. Evidence appended only to this file; the parent-edited plan and existing untracked work are preserved.

No deviations from the clarified product contract; no unresolved test failures. Independent parent review remains the next step. Live-window validation is still excluded until separate authorization.

## Final parent correctness check — warning/report availability dead end

Confirmed before correction: at 85% with a missing report, Summary rendered “Lire le bilan d’espace”; Enter opened an empty Review without requesting `Check::SpaceReview`. The early 85%/95% branches selected `ReadReview` independently of report availability. The same branches covered invalid reports and a first report already running.

### Regression coverage and correction

- Added parametrized rendered/key regressions crossing **85% and 95% × usable, missing, invalid, and first-report-running states**. Render checks run at both 100×26 and 40×12, asserting the real warning headline and yellow/red severity, exactly one useful action, no raw report error leak, and pending-notification text for running reports.
- Enter reads an existing report, requests only typed `Check::SpaceReview` for missing/invalid reports, or opens Details without requesting anything when the first report is running. The key matrix also verifies `/boot` and `/boot/efi` still open Details in every case, and SSD-critical warnings block preparation even through the manual Details key.
- Production change is local to `disk_overview`: derive report availability/running state from the unchanged snapshot and choose the safe space-warning action before returning the existing warning. Preserve SSD-critical and SMART-warning priority, `/boot` behavior, and all request dispatch/guards. Running-first-report space warnings retain their headline/severity and show “Bilan en cours · notification à venir.”
- README explains the contextual space-warning actions. No model, producer, schema, dependency, permission, root-unit, git, deployment or user-window changes.

### RED → GREEN and full verification

- RED request `32b7bcdb69bf`:
  `python3 config/bin/.local/bin/agent-build cargo test --offline --release --manifest-path apps/Cargo.toml -p vitals tui::tests::space_warning`
  → `test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 126 filtered out; finished in 0.00s` (exit 101).
  - `space_warning_enter_reads_prepares_or_waits_without_a_dead_end`: `85/missing`, actual request list `[]`, expected `[SpaceReview]`.
  - `space_warnings_offer_a_useful_action_for_the_available_report_state`: `85/missing`, rendered “Lire le bilan” instead of “Préparer le bilan”.
- GREEN request `750c5ac2d707`, same targeted command:
  `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 126 filtered out; finished in 0.00s` (exit 0). All eight combinations and their boot/SSD safety variants passed.
- Full package request `97bc229b48db`:
  `python3 config/bin/.local/bin/agent-build cargo test --offline --release --manifest-path apps/Cargo.toml -p vitals` (exit 0):
  - `test result: ok. 128 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s`
  - `test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s` (`collect`)
  - `test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s` (`disk_review`)
  - `test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s` (`maintenance`)
  - Binary/doc-test targets: 0 tests, all successful. Total: **159 tests**, including all **41 UI tests**.
- Full repository request `4c1cd09a0840`:
  `env CARGO_NET_OFFLINE=true make test` (exit 0). Runner fixtures: `Ran 12 tests in 0.482s` / `OK`; OSD prerequisite build succeeded; all **1,281 Rust workspace tests** passed, zero failed/ignored. Full output inspected, including the truncated head. Same existing warnings at `apps/desktop/src/tests.rs:1561`, `:1566`, and `apps/cockpit/src/tests.rs:260`; no warnings suppressed.
- Python request `4867ca7acf2f`:
  `python3 -m unittest discover -s tools/tests -v` (exit 0): `Ran 65 tests in 3.521s` / `OK`.
- `git diff --check`: exit 0, no output.

Current tracked diff from HEAD: `README.md | 34 ++-`, `apps/vitals/src/tui.rs | 574 ++++++++++++++++++++++++++++++++++++++-----------`; **2 tracked files changed, 470 insertions(+), 138 deletions(-)**. This evidence file remains the only appended execution document; existing untracked `.handoff/` and parent plan work remain preserved. No unresolved failures or implementation deviations; parent review is next, without deployment.

## Final batch — preserve finite usage above 100%

Confirmed the parent's edge case before changing production code: `/` at 101% plus `/boot` at 12% rendered “Aucune action urgente” and described 12% as the fullest volume. The UI's finite-value filter rejected every usage above 100%, masking the actual critical reading.

Smallest production correction: replace `(0.0..=100.0).contains(used)` with `*used >= 0.0`, retaining `used.is_finite()`. No thresholds, producers, model/schema, dependencies, permissions or units changed. README needed no further change for this validity correction.

Regressions:
- `overfull_root_usage_keeps_its_critical_warning_and_useful_action`: 101%/104% root plus 12% `/boot`, crossed with usable/missing report; render at 100×26 and 40×12; assert critical headline/red severity, useful reading/preparation action, and the corresponding safe Enter behavior.
- `nonfinite_and_negative_usage_do_not_become_space_measurements`: NaN, positive/negative infinity and negative usage remain rejected. With no valid reading the state remains unavailable, and with a valid low-usage mount invalid readings do not become spurious space warnings.

### RED → GREEN

- RED request `d6e8077f8766`:
  `python3 config/bin/.local/bin/agent-build cargo test --offline --release --manifest-path apps/Cargo.toml -p vitals tui::tests::overfull_root_usage_keeps_its_critical_warning_and_useful_action`
  → `test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 129 filtered out; finished in 0.00s` (exit 101). Rendered `101/usable` was falsely healthy. This targeted RED command ran only the overfull regression; invalid-value coverage ran in GREEN and complete verification.
- GREEN request `02a8cf94bd16`:
  `python3 config/bin/.local/bin/agent-build cargo test --offline --release --manifest-path apps/Cargo.toml -p vitals tui::tests`
  → `test result: ok. 43 passed; 0 failed; 0 ignored; 0 measured; 87 filtered out; finished in 0.01s` (exit 0).

### Complete verification

- Full package request `7523440ff72a`:
  `python3 config/bin/.local/bin/agent-build cargo test --offline --release --manifest-path apps/Cargo.toml -p vitals` (exit 0):
  - `test result: ok. 130 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s`
  - `test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s` (`collect`)
  - `test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` (`disk_review`)
  - `test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s` (`maintenance`)
  - Binary/doc-test targets: 0 tests, all successful. Total: **161 tests**, including all **43 UI tests**.
- Full repository request `218e3996b3ff`:
  `env CARGO_NET_OFFLINE=true make test` (exit 0). Initial wait exceeded 120 seconds; resumed the same request with `ask-to-run --wait 218e3996b3ff`, no duplicate execution. Runner fixtures: `Ran 12 tests in 0.469s` / `OK`; real OSD prerequisite build succeeded; all **1,283 Rust workspace tests** passed, zero failures/ignored. Inspected every result, including results hidden by the tail preview. Same three existing out-of-scope compiler warnings remain at desktop test lines 1561/1566 and cockpit test line 260, unsuppressed.
- Python request `781db8278ae0`:
  `python3 -m unittest discover -s tools/tests -v` (exit 0): `Ran 65 tests in 2.480s` / `OK`.
- `git diff --check`: exit 0, no output.

All compilation/test commands used host approval; Cargo stayed offline and used the shared low-priority runner. No service/scan, installation, deployment, git write, user-window control or further agent was requested. No new unrelated observations beyond the already recorded compiler warnings. Existing untracked work preserved.

Current tracked diff from HEAD: `README.md | 34 ++-`, `apps/vitals/src/tui.rs | 618 +++++++++++++++++++++++++++++++++++++++----------`; **2 tracked files changed, 513 insertions(+), 139 deletions(-)**. This batch's production edit is one filter condition plus two regression tests; execution evidence is appended here. No unresolved failures; parent review remains next.

## Fresh verification while independent review is pending

Verification-only continuation; product source unchanged. Loaded `verification-before-completion` and `finishing-a-development-branch`; the explicitly selected host-owned-clone protocol supersedes the generic integration menu and cleanup. No git writes, fetch/rebase, export/apply, deployment, app/window/service control, or further agents.

Current HEAD: `77b42abb9d14df477899976dcf32983ac0f162fb`, branch `parallel-1249`. Status: modified `README.md` and `apps/vitals/src/tui.rs`; untracked `.handoff/` and `design-plans/`, all preserved.

- Approved request `36a7656fc101`: `env CARGO_NET_OFFLINE=true make test` → exit 0. Makefile shared low-priority runner built the real OSD prerequisite and ran the full Rust workspace: **1,283 tests passed, 0 failed, 0 ignored**. Runner fixtures: `Ran 12 tests in 0.449s` / `OK`. Vitals included `test result: ok. 130 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s`, plus integration results of 10/6/15 passing tests. Read the complete output, including the head omitted from the initial tail preview.
- Existing unsuppressed warnings remain: `unused Result that must be used` at `apps/desktop/src/tests.rs:1561` and `:1566`; `variable does not need to be mutable` at `apps/cockpit/src/tests.rs:260`. No test failures.
- Approved request `fd378a9d4564`: `python3 -m unittest discover -s tools/tests -v` → exit 0: `Ran 65 tests in 2.503s` / `OK`; all individual fixture outputs inspected.
- `git diff --check` → exit 0, no output.

Ready for the parent to clear independent review before any authorized integration work proceeds. No commit, fetch/rebase, export, live application approval, or cleanup performed in this continuation.

## Independent review Important finding — Review header diagnostics

Confirmed in source and by RED rendering: `header` enabled the plain-language machine state only for Disques Summary, leaving Review with collection elapsed seconds or alert totals. The reconciled plan applies this header to both non-Details Disques screens.

Smallest product correction in `apps/vitals/src/tui.rs`: change the guided predicate from `disk_screen == DiskScreen::Summary` to `disk_screen != DiskScreen::Details`, still requiring `tab == DISKS_TAB`. No change to other tabs or Details. Critical global machine alerts retain their red prominence even without collection history; history failures remain explicitly unknown.

Added rendered Review-header coverage at 100×26 and 40×12 for healthy collected state, warning alerts, critical alerts with/without collection history, missing collection, and history-read failure. Assertions cover the plain-language state, appropriate colors, and absence of diagnostic counts/timing/error paths. Existing Summary/Details coverage retained; an additional regression confirms legacy collection timing remains in Details and other tabs.

- RED request `d5a6e5f03653`:
  `python3 config/bin/.local/bin/agent-build cargo test --offline --release --manifest-path apps/Cargo.toml -p vitals header`
  → `test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 129 filtered out; finished in 0.00s` (exit 101). Only `review_header_uses_plain_machine_state_without_counts_or_collection_timing` failed: healthy Review showed “tout va bien · collecté il y a 123 s”. Existing Summary and unchanged technical-header controls passed.
- GREEN request `d8851ad03591`, same command:
  `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 129 filtered out; finished in 0.00s` (exit 0). Every Review-header case/size passed, alongside Summary and technical-header controls.
- Fresh complete repository request `9ccbe12070e7`:
  `env CARGO_NET_OFFLINE=true make test` → exit 0. Runner fixtures: `Ran 12 tests in 0.439s` / `OK`; OSD prerequisite build succeeded; **1,285 Rust tests passed, 0 failed, 0 ignored**. Vitals results:
  - `test result: ok. 132 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s`
  - `test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s`
  - `test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
  - `test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s`
  - Total Vitals: **163 tests**, including **45 UI tests**. Binary/doc-test targets also passed with 0 tests.
- Existing unsuppressed compiler warnings remain at `apps/desktop/src/tests.rs:1561`, `:1566` (unused Result), and `apps/cockpit/src/tests.rs:260` (unused mut). Full output inspected including the initially truncated head; no failures hidden.
- Python request `3da871114b81`:
  `python3 -m unittest discover -s tools/tests -v` → exit 0: `Ran 65 tests in 4.040s` / `OK`.
- `git diff --check` → exit 0, no output.

All compilation/tests approved through `ask-to-run`, Cargo offline using the shared low-priority runner. No model/producer/schema/root-unit/permission/dependency edits, no further agents, no app control, and no commit/fetch/rebase/export/apply. Existing `.handoff/` and plan work preserved. Ready for parent clearance, which is not assumed.

Current tracked diff from HEAD: `README.md | 34 ++-`, `apps/vitals/src/tui.rs | 663 +++++++++++++++++++++++++++++++++++++++----------`; **2 tracked files changed, 558 insertions(+), 139 deletions(-)**, plus this untracked evidence file.

## Independent parent review

Reviewed the final source diff, README, keyboard guards, optional/waiting states,
warning/report-availability routing, and the finite overfull-usage correction.
The parent independently read the final host outputs: **1,283 Rust tests passed,
0 failed, 0 ignored**; Python **Ran 65 tests in 2.480s / OK**.
`git diff --check` is clean. No unresolved findings were retained.

The read-only review role wrote only files under `design-plans/`; the separate
implementation executor made the product/test changes. No commit, fetch/rebase,
live application, user-window control, scan, permission or cleanup-policy change
has occurred for this guided revision. Deployment and fresh-window validation
remain separate approval steps.

## Final integration-review clearance

The independent reviewer found no Critical issues and one Important issue: Review
still exposed technical counts/timing in its shared header. The tested correction
above now makes both non-Details Disques headers plain-language. Parent source
review confirmed the one-line predicate and retained critical-warning branch.

Reviewer set-aside rulings:
- Other tabs' legacy headers are intentionally outside the guided-surface scope;
  keep them unchanged. Cost: those tabs retain their existing technical wording.
- Details' pre-existing extreme-value space-bar presentation is outside this
  correction; do not widen the patch. The guided warning filter itself was fixed
  and tested for legitimate overfull readings.
- Live-window appearance remains unverified until deployment/fresh-window approval;
  do not present headless evidence as a live visual check.

No Critical or Important findings remain unresolved. The user has explicitly
approved feature-branch commit, fetch/rebase, and opening the live apply approval.
Use that selected dotfiles protocol, not a generic merge/PR menu; never merge or
commit directly on the default branch, push, reset, delete workspaces, or include
the unrelated `.handoff/` artifacts. Reverify after rebase and disclose any failures
before a live apply. Cleanup policies, permissions and producer are not part of
this change.
