# Report-only Cache Maintenance Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use executing-plans to implement this plan task-by-task in this session. Do not delegate without user authorization.

**Goal:** Prepare daily, strictly report-only Turbo and pacman maintenance units without installing or enabling them.

**Architecture:** An unprivileged Python script inventories only Hartprint's Turbo cache. A separate hardened system service invokes the vendor paccache executable in dry-run mode. Both emit reports to the journal and have daily persistent timers.

**Tech Stack:** Python standard library, systemd, installed pacman-contrib.

**Spec:** `docs/superpowers/specs/2026-10-20-cache-maintenance-design.md`

## Global Constraints

- Retention is 30 days; pacman retains at least two versions.
- No deletion mode, installation, enabling timers, dependency manifests, Makefile changes, or git writes.
- Never execute a script in the user's checkout as root.
- Turbo observations are provisional until entry format and coordination with builds are verified.
- Preserve all existing cleanup commands and unrelated files.

## Review Focus

- Missing cache after manual cleanup: report zero files, not failure.
- Symlinked ancestors or entries: never traverse or count the target.
- Recently accessed old files and exact cutoff timestamps: preserve as ineligible.
- Concurrent invocation or inspection failure: report visibly, never silently claim success.
- Unit wiring: paccache is always dry-run; neither timer enables itself.

---

### Task 1: Turbo report and public-entry-point tests

**Files:**
- Create: `config/bin/.local/bin/cache-maintenance-report`
- Create: `tools/tests/test_cache_maintenance.py`

**Interfaces:**
- Consumes: `HOME`, `XDG_RUNTIME_DIR`; fixed relative path `IdeaProjects/hartprint/.turbo/cache`.
- Produces: CLI `python3 config/bin/.local/bin/cache-maintenance-report` with no arguments; stdout report, stderr failures, nonzero exit on invalid invocation or inspection error. No apply/delete flags.
- Tests run the real CLI using temporary home/runtime directories, never the real cache.

- [ ] Write subprocess regression tests: missing cache exits 0 and reports zero; old regular files are provisional; recent access or modification and exact 30-day boundary are not old; symlink ancestors and entries and nested directories fail visibly without following them; unexpected arguments fail; file contents and directory entries remain unchanged. Use timestamps safely away from the boundary in CLI tests and a fixed clock via a separate standard-library mock-based boundary test of the time predicate.
- [ ] Add tests for non-readable cache (skip only when running as root), a disappearing file during inspection (mock the filesystem operation), concurrent runtime lock contention, and allocated bytes explicitly labeled as estimates.
- [ ] Run `python3 -m unittest discover -s tools/tests -p 'test_cache_maintenance.py' -v`; record the expected missing-script failures.
- [ ] Implement `main() -> int` with Python standard-library filesystem inspection, a nonblocking `fcntl.flock` runtime lock, 30-day strict access/modification predicate, no symlink traversal, no content reads, and no deletion calls. Require an existing owned runtime directory. Fail on races/errors instead of producing an incomplete successful report. Use JSON or clearly labeled text output with provisional status and allocated-byte estimates (`st_blocks * 512`). Keep helpers minimal; the time predicate accepts an explicit current timestamp for deterministic tests.
- [ ] Rerun the tests and record actual output. Do not commit.

### Task 2: Timer wiring, policy tests, and usage documentation

**Files:**
- Create: `config/systemd/.config/systemd/user/cache-maintenance-report.service`
- Create: `config/systemd/.config/systemd/user/cache-maintenance-report.timer`
- Create: `system/etc/systemd/system/package-cache-report.service`
- Create: `system/etc/systemd/system/package-cache-report.timer`
- Modify: `tools/tests/test_cache_maintenance.py`
- Modify: `README.md` adjacent to Everyday use cleanup guidance.

**Interfaces:**
- User service executes `/usr/bin/python3 %h/.local/bin/cache-maintenance-report`.
- System service executes `/usr/bin/paccache --dryrun --verbose --keep 2 --min-atime "30 days ago" --min-mtime "30 days ago"` directly, not via a shell.
- Timer filenames bind to matching service filenames. Both use `OnCalendar=daily`, `Persistent=true`, `RandomizedDelaySec=1h`, and `WantedBy=timers.target`.

- [ ] Add policy regression tests parsing unit files: exact executable/arguments, dry-run only, no force/remove flags, correct retention, matching timer/service names, daily persistent schedule, low priority, and read-only service hardening.
- [ ] Run the test command above and confirm new unit tests fail before the unit files exist.
- [ ] Write both oneshot services with `Nice=19`, `IOSchedulingClass=idle`, `NoNewPrivileges=true`, `ProtectSystem=strict`, and `PrivateTmp=true`. User service uses `ProtectHome=read-only` and allows writes only to `%t` for its runtime lock. System service uses `ProtectHome=true`, `PrivateNetwork=true`, and no writable paths. Report output goes to the journal; no retained report files grow indefinitely.
- [ ] Write timers without installing or enabling them. Do not edit `services.txt`.
- [ ] Document scope, retention, provisional Turbo status, journal unit names, no automatic deletion, and later installation being a separate approval. Explicitly note these scripts and units execute outside the sandbox once installed. Do not instruct users to invoke broad `make clean` automatically.
- [ ] Run all new regression tests, `python3 -m py_compile` on the new script, and `systemd-analyze verify` on both unit pairs if available. Missing tools or unresolved live installation paths must be reported verbatim, not hidden. No build/install commands.
- [ ] Review `git diff --check`, `git diff --stat`, and untracked file listing. Summarize changes, preserved behavior, and actual test results. Do not commit, push, export patches, install, enable, or start units without further authorization.

## Execution results

Both tasks implemented in the existing `parallel-1249` checkout, without git writes,
installation, or starting/enabling units. Existing manual cleanup remains unchanged.

Verification summaries (verbatim):

| Run | Result |
| --- | --- |
| Initial missing-script regression run | `Ran 13 tests in 0.289s` / `FAILED (failures=15)` |
| First implementation run inside sandbox | `Ran 13 tests in 0.382s` / `FAILED (failures=10)` |
| Approved host run, task 1 | `Ran 13 tests in 0.301s` / `OK` |
| Missing-unit regression run | `Ran 4 tests in 0.003s` / `FAILED (failures=6)` |
| Unit-policy tests after implementation | `Ran 4 tests in 0.001s` / `OK` |
| Approved host full suite | `Ran 17 tests in 0.305s` / `OK` |
| Journal-size regression before fix | `Ran 1 test in 0.028s` / `FAILED (failures=1)` |
| Final approved host full suite | `Ran 18 tests in 0.321s` / `OK` |

Sandbox failures were `PermissionError: [Errno 13] Permission denied: '/'` during
directory-ancestor validation, reported with `claude-todo blocked`. Approved host
tests used only temporary fixtures and passed without skips. The journal-size
regression caught an unbounded filename list; output now samples at most 20 names
with an explicit omitted count while totals cover all observed files.

Final `python3 -m py_compile` and both `systemd-analyze verify` commands returned
exit 0 with no diagnostics. No live service execution was attempted. The unrelated
Rust suite (`make test`) was not run because it invokes a release build, which was
not authorized. The new tests run separately; the existing Makefile is unchanged.

Execution decisions:

- Self-review, no subagents, as approved; less independent review than delegation.
- Scratch ledger kept outside the checkout; no git helper writes. Execution evidence
  is recorded here so the deliverable does not depend on the scratch ledger.
- Host fixture tests requested rather than weakening path validation for greywall;
  testing requires explicit approval in this environment.
- Filename output bounded following final review; full cache-entry and active-build
  safety verification remains deferred before any deletion implementation.

Broader disk maintenance was added to the project's todo list as an agreed follow-up.
