# Agent Completion Gates Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use executing-plans inline; no additional workers. Steps use checkbox syntax for tracking.

**Goal:** Require native `/code-review` and `/simplify` before final implementation completion claims.

**Architecture:** Extend the existing shared global instruction file, already linked into OpenCode's `AGENTS.md`. Add explicit command-local recursion exclusions and check the deployed link through the existing config-tests suite. This is instruction policy, not automated enforcement.

**Tech Stack:** Markdown, existing Stow packages, Rust config-tests; no new dependencies.

**Spec:** Approved Option A for todo `38a4e84f` in this conversation. Prerequisite `fdafd33f` is complete; its historical results are not verification of this change.

## Current checkpoint

Native review `ses_e730a2827ffewlsRWzszG4dpul` completed without actionable findings;
native `/simplify` completed with no changes. Fresh clone verification receipt
`f7ae4d431da9` passed. Theo has resumed the parent workflow for final evidence,
commit/export and separately approved live apply/verification. The chronological
investigation below preserves superseded diagnoses and their corrections; it is
not a claim that earlier blocked stages are still the current checkpoint.
Old child `ses_e7326a3a4ffeef94ogWT8I65FK` remains unresolved and untouched.

## Global Constraints

- Preserve unrelated untracked files and existing command names, models, permissions and approvals.
- No hooks, installs, providers, manifest edits, unrelated policy or additional workers.
- Native invocation must be verified per harness; prose and discovery are not execution.
- Feature-branch commit and normal reviewed host apply only; no push or default-branch writes.

## Review Focus

- Full task scope includes committed, staged, unstaged and relevant untracked artifacts; unrelated work remains untouched.
- Review/simplify command work must not recursively launch the completion pipeline.
- Unavailable, failed, blocked or still-running gates cannot produce a final success claim.
- Conversation-only and non-implementation tasks must not trigger arbitrary edits or fictitious tests.
- Shared-path checks demonstrate deployment wiring, not agent compliance or cross-harness invocation.

## Task 1: Shared completion pipeline

**Files:**
- Modify `config/claude/.claude/CLAUDE.md` (also exposed by the existing `config/opencode/.config/opencode/AGENTS.md` symlink).
- Modify `config/opencode/.config/opencode/commands/code-review.md` and `simplify.md` only for explicit recursion exclusions.
- Modify `apps/config-tests/tests/cockpit_calls.rs` to exercise the real Stow-deployed shared instruction link in a scratch home.

**Interfaces:** Parent owns review → in-scope fixes → behavior-preserving simplify → relevant verification → review resulting edits. Claude uses native bundled skills; OpenCode uses the existing named commands, only via supported/discovered invocation routes.

- [x] Add the scratch-home Stow link regression; run the focused existing checks before editing instructions. Existing wiring may already pass: this is a deployment regression, not a claim of RED for new prose behavior.
- [x] Add concise shared policy and command-local exclusions; manually check the failure/recursion/non-implementation cases above.
- [x] Run focused `cockpit_calls` and `opencode_commands` checks, then `make test`, preserving host approval requirements and reporting failures verbatim.
- [x] Run native completion commands if a supported invocation route is available; otherwise report the missing route and leave final completion pending. Do not substitute self-review for execution.
- [ ] Inspect the full delta and diff stat; commit only task files, export the new commit, request normal `cockpit apply`, then live `make test` and shared-path/discovery verification through `ask-to-run`. Use `Refs: 38a4e84f` rather than automatic completion (see the parent ruling below).

## Evidence limits

Regression tests cover Stow link wiring and native OpenCode command discovery, not model compliance. No Claude invocation support is inferred from OpenCode discovery. Global instruction edits affect host-started agents; the Rust regression invokes Stow in a scratch home. No executable host hooks or installation scripts are changed.

## Progress and outcomes

- Source base: `3a6187c312d8108576665ba140c98d1f970d9895` (`origin/main` after fetch); work is in the existing dedicated clone on `feature/agent-completion-gates`. Branch creation succeeded with `.git/config: Device or resource busy` warnings; no attempt was made to change the protected config. Existing unrelated untracked work remains excluded and untouched.
- Ruling: keep the approved existing clone and work inline; no additional worker, model override or new workspace. Add a real Stow boundary regression, not assertions that grep policy wording or pretend to prove agent compliance. The test passed before the prose edit because shared wiring already exists; no RED→GREEN claim is made.
- Sandbox focused Cargo command failed before compilation, exit 101: `error: no matching package named libc found`. No dependency installation attempted.
- Approved host baseline receipt `9b3762b12202`: `cockpit_calls`: `6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`; `opencode_commands`: `3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`; exit 0.
- Approved host post-edit receipt `ea7fbdd23e17`: focused tests followed by `env CARGO_NET_OFFLINE=true TMPDIR=/home/theo/IdeaProjects/dotfiles/apps/target make test`, exit 0. Every reported suite group passed, with no ignored tests. This verifies clone source through host execution, not applied live configuration or native gate execution.
- Native invocation blocker: sandbox and approved host receipt `4324a746a4e3` both returned `{"location":{"directory":"/home/theo/IdeaProjects/dotfiles"},"data":[]}` from `/api/command`; the host confirmed this session ID on the same service. The installed API schema exposes `POST /api/session/{sessionID}/command`, requiring `name` and `text`. One `code-review` request on `ses_e732b7b11ffeeJr8ePam5QQl8X` specified the full base/head/worktree task scope and explicit unrelated-file exclusions. It returned exit 0/no output, but no command result or inbox item was observed. This is NOT verified review execution. `/simplify` has not been started; it remains behind the unresolved review stage.
- Blocker filed as `095904fc` via `cockpit-todo blocked`, receipt `91bfeb668020` accepted/queued in agent-cockpit. No installation, plugin/provider/model change, substitute prose review or extra worker was used to satisfy the missing native command gate.
- Pending: native review result, in-scope fixes if any, native simplify, final-tree verification/review, commit/export and reviewed host apply/live checks. Todo `38a4e84f` remains in progress, not done. No live configuration change or Claude native invocation is claimed.

### Resume investigation

- Correction to the earlier registry diagnosis: two consecutive CLI `/api/info` calls returned different temporary server PIDs/URLs (`22241` / `127.0.0.1:34343`, then `23115` / `127.0.0.1:43573`). Thus the empty registries and silent command callback did not establish what the active session can discover, and did not invoke its native review.
- Approved host receipt `853628a037c1`: `cockpit --help` returned usage/exit 2, stopping before its following service check. No supported session-command route appeared in that usage.
- Approved host receipt `c1e69f683aac`: managed OpenCode service is `stopped`; this session's standalone UI process is `215453`, with server process `215643` using `serve --stdio --port 0`.
- Approved host receipt `a6efdc2b36aa` confirmed `215643` has parent `215453`; the targeted host listener lookup found no entry and exited 1. Those host PIDs are not visible inside this sandbox. No port scanning, foreign-session connection, service launch or configuration change was attempted.
- Remaining limitation: no verified route from this session's available tools to its active native command registry/execution. Do not replay the temporary-server callback as review evidence. Submission through the actual session's command UI is needed unless the harness supplies a supported route. Native gates, commit/apply and final completion remain pending.

### Original-TUI route and no-replay correction

- Installed-source receipts `cd2c76d61d2d` and `d7edab4fa613` establish `cockpit say <session-id> <message>` dispatches through subscription ownership routing, then `agent_opencode_say` to the existing pane. That handler permits working/waiting or explicit needs-input states, calls `submit_to_pane`, and does not reinterpret slash commands as API prose. The launcher deliberately disables shared-service use; no enablement/restart is needed or authorized.
- Read-only host receipts `6284002cb7a4` and `cb5e63faa49f` verify the original conversation `ses_e732b7b11ffeeJr8ePam5QQl8X`, project/cwd `/home/theo/IdeaProjects/dotfiles`, pane `%285`, tmux session `claude-retry-opencode-1793364100434973388`, matching `CLAUDE_CODE_SESSION_ID`, and routing `{"held":false,"session":"ses_e732b7b11ffeeJr8ePam5QQl8X"}`. `type_into_pane` uses the existing composer/paste safeguards and `submit_to_pane` submits Enter. No messages were sent by these inspections.
- **Correction: the earlier `session.command` attempt WAS durably submitted.** Child `ses_e7326a3a4ffeef94ogWT8I65FK` was created at `1793364417644` with parent `ses_e732b7b11ffeeJr8ePam5QQl8X`, native code-review title and `busy` timestamp `1793364417651`. Its recorded user message `msg_18cd95c6e001sP96aeZhhnd1Pg` at `1793364417661` contains the expanded native `/code-review` template and the exact task scope from the earlier API request. An empty one-shot registry and empty parent inbox did not prove non-submission.
- Read-only child-history inspection returned only that user message: no assistant/review result or recorded completion was present. Child `outcome` is null. Whether an execution remains active or was stranded when the temporary server ended is not established; no result or passing review is claimed.
- In accordance with the no-replay instruction, no second `/code-review` was submitted through cockpit say or any other route. Coordination message `410a33b1df60e0b831baac856b5e3497` was not resent. `/simplify`, commit/export, host apply and final completion remain pending until the existing review submission is resolved through a supported, approved route.

### Bounded review-child status reconciliation

- Reviewed host receipt `3944680f7e1f` read the exact child and parent metadata, the child's latest messages/inbox, durable event stream and a bounded selection of existing logs. Receipt `75182cca537c` corrected the event request to `after=0` and retained the actual matching log fields instead of an insufficient field summary. Neither request submitted, resumed or restarted any execution.
- Child status remains: `outcome: null`, no recorded error, only input message `msg_18cd95c6e001sP96aeZhhnd1Pg`, no assistant/tool messages, empty inbox, and original `busy: 1793364417651`. This does not establish active execution or a terminal provider/model/tool failure.
- Both event reads returned only `log.synced` at aggregate sequence 5, including with explicit `after=0`; no execution/error/owner event was exposed. The bounded log search covered 12 files, last 2 MiB each. All matching child lines were diagnostic CLI starts or coordination/read commands, not an executor lifecycle or terminal failure. Older uninspected log portions are not covered by this conclusion.
- Established classification: native command input is durably admitted, review execution/result is unverified. Whether a live executor still owns it, or the original temporary command server exited leaving only persisted input, remains unresolved. A persisted busy timestamp is not liveness evidence; a null error is not successful verification.
- Missing capability: a runtime-local execution-owner/error lookup for this child in its actual owning standalone runtime. The temporary CLI API exposes persisted state but has not supplied that owner lookup or historical execution/error events. Smallest additional host-reviewed diagnostic: identify the server-run/PID that admitted this child's original command and read its lifecycle end/error plus its runtime-local active-child state through a documented interface; do not infer ownership from a fresh API process or resume the child merely to inspect it.
- No replay, replacement reviewer, parent reload/restart, shared-service enablement, provider/credential/model/permission change or simplify. The original five-file task delta and prior passing test receipts remain intact. Task `38a4e84f` remains explicitly blocked on review execution ownership/status, not complete.

### Admitting runtime lifecycle identified

- Theo authorized one bounded read-only ownership diagnostic. Installed API schema inspection first confirmed `/api/debug/location` lists loaded locations only, `/api/session/{sessionID}` returns persisted `Session.Info`, and no documented read-only live execution-owner/status endpoint is exposed. No new API server was started for this diagnostic.
- Host-reviewed receipt `b698a985ba51` scanned all 12 existing log files (43,585,720 bytes, within 128 MiB / 15-second scan bounds) and read process metadata. The admission-window portion of the receipt identifies the actual sequence; the script's narrower automatic run-ID summary includes the parent/CLI runs, not the spawned server, so that summary alone is not the owner identification.
- Original parent server run `2b5ed072` launched the earlier `opencode api post .../command` at `2026-10-30T12:46:54.504Z`. API CLI run `d4a2aa3c` started at `12:46:54.557Z` and spawned `opencode serve --stdio --port 0` at `12:46:54.561Z`. That temporary server is run **`72d32d68`**, starting at `12:46:54.613Z` (receipt lines 4–8).
- Child creation/input timestamps `12:46:57.644Z` / `12:46:57.661Z` fall within that server's command request. At **`12:46:57.682Z`**, run `72d32d68` logged **`shutdown started`, `signal=SIGTERM`**, followed by watcher/service teardown through `12:46:57.719Z` (receipt lines 75–100). Thus the original admitting runtime entered shutdown 21 ms after the recorded child input; the original execution is not evidence of a continuously active reviewer.
- A provider-config HTTP 401 warning occurred in that server at `12:46:57.450Z`; it is not a recorded terminal model/provider failure for the review and is not claimed as its cause. No child assistant/tool result or terminal execution error was established. The evidence instead supports admitted input followed immediately by admitting-runtime shutdown.
- No numeric PID was logged for run `72d32d68`, and the process list does not map historical run IDs to PIDs. Parent server PID `215643` belongs to the still-running original parent, not to that temporary admitting runtime; it must not be relabeled as the review owner.
- Remaining capability gap: no supported runtime-local child-owner lookup is exposed to prove whether another runtime subsequently adopted the persisted child. The original temporary runtime's shutdown is established; current adoption/liveness elsewhere is not. No restart, replay, reviewer replacement, provider/security change or simplify was performed. Task delta, prior test evidence, review identity and host-approval receipts remain preserved; implementation completion/apply remains blocked pending an approved resolution of the interrupted original review.

### Same-child recovery interface verification

- Theo requested preparation only, with recovery itself requiring a separately reviewed host approval. Installed OpenCode v2.2.2 `session --help` exposes list/delete/export/import, not a recovery subcommand. `run --session` exists but without a verified explicit original-server connection would launch/use a different runtime and is not an acceptable recovery here.
- The inspected API supports `POST /api/session/{sessionID}/prompt` to admit new input and schedule execution in an existing session. That is a continuation interface, not a documented replay-free recovery of the already-admitted input; `interrupt?resume=true` is mutating and not a read-only liveness/recovery probe. Neither was invoked.
- Within this original sandbox, read-only process/listener inspection identified UI PID 2 and its server PID 75 listening on `127.0.0.1:34641`. Explicit `opencode api --server http://127.0.0.1:34641 get /api/info` was rejected: `Server at http://127.0.0.1:34641 requires a password; set OPENCODE_PASSWORD`. No password/credential inspection, extraction, setting change or authentication bypass was attempted.
- Host-reviewed receipt `728abb09c85b` independently verified pane `%285`, matching original conversation environment, UI host PID `215453`, its standalone server host PID `215643`, and `NSpid: 215643 75`. Thus the listener is correlated to the original runtime, not to another temporary API server. The installed CLI version/help was rechecked read-only in that receipt.
- Conclusion: no verified end-to-end supported same-child recovery command is available with the current CLI interface/access. An authenticated original-runtime continuation route might exist, but it has not been verified and must not be presented as ready to execute. Current cross-runtime child adoption is still unverified, so duplicate execution cannot be ruled out. No recovery command was submitted for execution, and no fresh review was authorized or submitted by this preparation step.

### One approved fresh native command delivery

- Theo subsequently authorized preparation of exactly one fresh native `/code-review`, with host execution still requiring review. Approval receipt **`927631805965`** accepted `cockpit say ses_e732b7b11ffeeJr8ePam5QQl8X "/code-review ..."` and returned exit 0. Its approval text explicitly disclosed unresolved old child `ses_e7326a3a4ffeef94ogWT8I65FK` and possible duplicate execution, and excluded simplify/retries/restarts/credential/settings changes.
- The exact command is retained in that receipt. Scope: explicit direct base `3a6187c312d8108576665ba140c98d1f970d9895` to verified same HEAD, separate staged/unstaged inspection, the five-file implementation/plan delta and all five task-created `.handoff/38a4e84f-*.py` diagnostic scripts. Pre-existing unrelated work was explicitly excluded and preserved. The index was empty and `git diff --check` passed before submission.
- Delivery receipt is not a review result. The subsequent durable child-list read exposed only the old child; parent inbox was empty. Read-only original-TUI capture receipt **`840d745df4b9`** confirmed the original pane remained live with a subagent indicator, but did not identify a new child or review result. No new child/message identity is yet established; `927631805965` is the fresh command-delivery identity, not a child execution ID.
- No second delivery/retry, old-child mutation or simplify was performed. Await reconciliation of this single delivery before any further command. This ledger-only evidence update does not change the reviewed implementation; it must be included in resulting-edit review once the native result is available.

### Fresh review result reconciled

- The original-runtime completion notification identifies new native review child **`ses_e730a2827ffewlsRWzszG4dpul`**, completed: **"No actionable introduced bugs verified."** This reconciles approved delivery `927631805965`; it is not a result from the unresolved old child.
- Reviewer explicitly verified the direct base/HEAD comparison at `3a6187c312d8108576665ba140c98d1f970d9895`, empty index, all four tracked modifications (69 insertions), the entire untracked plan and five task-created diagnostic scripts, shared AGENTS.md symlink, command-discovery tests, test helpers and Stow caller. It observed no scope change. Pre-existing unrelated work was excluded and untouched.
- Reviewer ran read-only source/Git/symlink inspection and `git diff --check`; it did not run tests, diagnostic scripts, native commands, host execution or additional workers. Its result is not independent reproduction of historical test/runtime receipts. The Stow test proves scratch-home link resolution, not model compliance, native invocation, or default-folding/restow in an existing home.
- No findings require an implementation fix. Next is one native `/simplify` through the original TUI, subject to its own host approval. Old child `ses_e7326a3a4ffeef94ogWT8I65FK` and its logs remain unchanged; possible duplicate execution remains disclosed. Final verification, resulting-edit review, commit/export and reviewed apply/live checks are still pending.
- Host approval receipt **`b470815b1188`** accepted exactly one `cockpit say` native `/simplify` delivery to the original parent and returned exit 0. Scope includes the same task delta and five diagnostic scripts, with no recursive commands, diagnostic replay, workers, provider/settings/dependency changes, commits or apply. The scoped simplify outcome is still pending; delivery alone is not simplify execution/completion evidence.

### Native simplify result and parent continuation

- The expanded native `/simplify` invocation was delivered in this original conversation after receipt `b470815b1188`. Its four-pass assessment found no justified behavior-preserving cleanup and changed no files. It inspected the direct base/HEAD comparison, staged/unstaged deltas, the complete task plan and five diagnostic artifacts; no completion commands or diagnostic scripts were recursively executed.
- Fresh host-approved receipt **`f7ae4d431da9`** ran the focused `cockpit_calls` and `opencode_commands` tests and then `env CARGO_NET_OFFLINE=true TMPDIR=/home/theo/IdeaProjects/dotfiles/apps/target make test`, exit 0. Focused results: **`6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`** and **`3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`**. Every full-suite group passed, with none ignored. `git diff --check` passed.
- SHA-256 checks against the simplify starting manifest confirmed all ten scoped files unchanged after assessment/tests. Parent resumption rechecked that same manifest successfully before this evidence-only update. The four implementation/test files remain exactly the reviewed, simplified and tested bytes; there is no reason to replay the completed gates or diagnostic commands.
- Theo explicitly ended the simplify-only restriction and resumed parent completion: finalize evidence, review the final delta, commit/export under dotfiles authorization, then request separate live apply and verification approvals. No push, install, restart, authentication/settings change or new worker is authorized.
- Final parent review includes the evidence-only ledger additions since native review and all implementation/test diffs. The ledger additions reconcile actual receipts/outcomes without changing the policy or test behavior. Five deliverable files will be committed; diagnostic artifacts remain preserved in `.handoff/`, not deployed as executables. No Makefile, hooks, manifests, providers or security configuration are changed.
- Live application and live verification are still pending at commit time. This commit is not a final task-completion claim; `38a4e84f` must not be marked complete before successful live checks.
- Parent ruling: use `Refs: 38a4e84f`, not the usual `Todo:` trailer, for this commit. Installed apply inspection receipts `822cb7016a99` and `1e450f75b0f7` establish that `close_shipped_todos` marks Todo trailers done immediately after `git am`, before post-apply/live tests. Theo's more specific requirement to mark complete only after live verification takes precedence; manual `cockpit-todo done` will follow successful live checks. This changes no global/project convention or cockpit behavior.
- Apply inspection receipt `5b159dcae701` confirms this test-only `apps/` change triggers `make -C ~/dotfiles build`, with no installation/restart in that function. The live apply approval must disclose that normal build; live tests still need their separate approval. Read-only apply inspection was a new completion-boundary check, not a replay of completed native gates or review-runtime diagnostics.
