# Global OpenCode Review Commands Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use executing-plans for native execution after Theo approves this plan. Steps use checkbox syntax for tracking.

**Goal:** Add global OpenCode V2 `/code-review` and `/simplify` commands with the boundaries in Theo's task request.

**Architecture:** Two Markdown prompt templates in the existing OpenCode Stow package; no plugin, shell expansion, agent/model override, or dependency changes. Run review in an independent child session and simplification in the current session, retaining the invocation model and existing permissions.

**Tech Stack:** OpenCode V2 command frontmatter, GNU Stow, existing Rust config-tests and serde_json.

**Spec:** Theo's task request in this session: read-only independent diff review; behavior-preserving, task-scoped simplification; authoritative research; regression/discovery checks; reviewed cockpit deployment only.

## Global Constraints

- Edit only `/home/theo/IdeaProjects/dotfiles`; preserve existing untracked work.
- Use `config/opencode/.config/opencode/commands/`, not the legacy singular directory.
- No installs, manifest edits, provider activation, permission changes, automatic fixes during review, Git publishing, or cloud review.
- Keep host/security/install/spending/protected Git approvals; command invocation grants no additional authorization.
- Commit on a new feature branch from fetched `origin/main`; do not rebase the existing branch's applied commits or export them again.
- Host apply remains an explicit `ask-to-run` review. No direct live-checkout edits.

## Research findings (2026-10-29)

- [Claude local review](https://code.claude.com/docs/en/code-review-local): `/code-review` is a bundled prompt skill, normally a background reviewer with its own context; correctness first, cleanup coverage varies with model/effort. Default scope includes branch commits ahead of upstream and uncommitted changes. Explicit PR, path, branch and revision-range targets are supported. `--fix` applies findings; `--comment` posts them; `ultra` can launch cloud review. The bundled skill has no editable `SKILL.md` on disk.
- [Claude commands](https://code.claude.com/docs/en/commands.md): `/simplify` applies cleanup, using four parallel reviewers for reuse, simplification, efficiency and abstraction level; it is not a correctness review. Historical descriptions are not a reliable account of the current behavior.
- [OpenCode V2 commands](https://opencode.ai/v2/docs/commands): global Markdown lives in `~/.config/opencode/commands/`; filenames determine names, `$ARGUMENTS` supplies literal argument text, `subagent: true` selects a child, and omitting agent/model avoids overrides. Shell interpolation runs outside tool approvals and must not be used. Project definitions can shadow global commands.
- Installed Claude plugin inspection failed with `Permission denied` for `~/.claude/plugins` and `~/.claude`; reported as blocker `50c7b449`. No bundled prompt internals were verified or copied.
- Accessible `~/.agents/skills/requesting-code-review/code-reviewer.md` supplies a read-only, evidence-based review checklist; reuse its checklist, not its orchestration or strengths-first format. Accessible `ponytail` supplies the minimal-change simplification policy. Discover/load installed guidance without installing anything, and keep this task's stricter boundaries.
- No matching todo or team owner was found. Source OpenCode configuration contains no existing `code-review`/`simplify` definitions; `.opencode/` is empty. Live OpenCode directory listing has no command directories. Existing unrelated root config files remain untouched.

## Review Focus

- Empty or ambiguous diff: state exact scope, include staged/unstaged and relevant untracked files, ask rather than guess a base, and report no changes explicitly.
- Unsupported Claude flags: explain that effort/cloud/fix/comment switches are not implemented; never interpret them as permission to edit, publish or spend.
- Existing unrelated changes: preserve them, constrain simplification to the selected task's changed hunks, and stop if ownership is unclear.
- Read-only is a prompt contract, not an OS/tool denial: no writes or mutating checks during review; existing tool approval enforcement remains intact.
- Tests verify discovery and prompt contracts, not model compliance. No paid model calls, credentials or provider activation in checks.

## Task 1: Ship the two native commands and their checks

**Files:**
- Create `config/opencode/.config/opencode/commands/code-review.md`.
- Create `config/opencode/.config/opencode/commands/simplify.md`.
- Create `apps/config-tests/tests/opencode_commands.rs`.
- Keep this plan as the research/behavior-difference record; add actual check results after execution.

**Interfaces:** `/code-review [target or focus]` runs an independent report-only child (`subagent: true`); `/simplify [target or focus]` runs inline (`subagent: false`). Both expand `$ARGUMENTS` as scope data, not authority. No agent/model/frontmatter permission fields. Defaults: review branch changes plus working changes when a verified base exists; simplify the current task's working changes, not arbitrary branch history.

- [x] Write config-tests that fail while the files are absent: enforce descriptions, `$ARGUMENTS`, execution mode, no agent/model/permission metadata or shell blocks. Check source JSON/JSONC and legacy command paths for name collisions. Safety prose is reviewed separately, not asserted as evidence of model compliance.
- [x] Run `cargo test --offline --manifest-path apps/Cargo.toml -p config-tests --test opencode_commands` and confirm the missing-file failure through host approval (sandbox offline cache lacks libc).
- [x] Add the two concise command templates: review actual diff/callers/tests independently, report concrete severity-ranked `file:line` findings first, distinguish test/reuse concerns, never fix or post; simplify reuse/clarity/efficiency/abstraction only when behavior preservation is demonstrable, preserving public contracts, errors and validation, and run relevant checks without unrelated fixes or security/dependency changes.
- [x] Add native discovery using the installed OpenCode binary and scratch XDG/HOME/DB/config environment (following `opencode_effort_picker.rs`). Copy only the new global command files and compare registered names/descriptions via `/api/command`; check mode/model/agent metadata statically. Keep one private stdio server alive and wait for readiness, because one-shot standalone queries race plugin initialization. No prompts or model requests; no real global plugins/credentials. Scratch files stay inside the requested clone under `apps/target`.
- [x] Run focused tests, then the project's suite; host-only tests use `ask-to-run` as documented in `CLAUDE.md`. Report every failure without disabling tests or fixing unrelated code.
- [ ] Review the complete diff and `git diff --check`; show exact files and diff stat, then commit only requested files on the feature branch.
- [ ] Export only this branch's new commit via `git format-patch origin/main -o .outgoing`; request `cockpit apply /home/theo/IdeaProjects/dotfiles` through `ask-to-run`, preserving host review. If approved, request `make -C ~/dotfiles test` through `ask-to-run`.
- [ ] Report source findings, intentional Claude differences (no exact parity, no automatic four-agent fan-out, no Claude flags or remote posting), exact test output, and applied/pending deployment status. Call out both new global prompt configurations and the Rust test that can execute on the host; no Makefile, hook or executable host script changes.

## Execution evidence

- Native execution approved; no new workers or model changes. Work stayed in the existing dedicated clone, on `feature/opencode-review-commands` from `origin/main` (`790a6edb716df2042de900e58d5ed0717a7e0085`). Existing untracked files were preserved. Branch creation succeeded but printed `.git/config: Device or resource busy` warnings; no attempt was made to modify that protected configuration.
- Sandbox offline Cargo attempt: `error: no matching package named libc found` (exit 101, before compilation). No dependencies were installed. Host-approved focused RED: `test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out`, both `missing global /code-review definition: No such file or directory (os error 2)`.
- Initial post-implementation focused run: `test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out`; `native_discovery_loads_both_global_commands_without_collisions` reported `/code-review is not uniquely discoverable` (0 entries). Diagnostics established that one-shot `api --standalone` exits before asynchronous command plugin initialization, not a missing plural-directory feature. A persistent private server registered both commands after readiness; no runtime upgrade or legacy-directory fallback was introduced.
- Corrected focused host run (`ask-to-run` request `82d042bc32cb`): `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.45s`.
- Host `env CARGO_NET_OFFLINE=true TMPDIR=/home/theo/IdeaProjects/dotfiles/apps/target make test`: exit 0; every reported test group passed. Full output is in the approval receipt for `82d042bc32cb`.
- `/api/config` returns raw discovery entries, not expanded Markdown definitions. Tests therefore verify native names/descriptions and the static frontmatter/template boundary, not mode execution or model compliance. Neither command was submitted to a model or run on refactor `5b93b063`; that session's finishing gate remains separate.
- Self-review only, as explicitly requested: reviewed empty/ambiguous scopes, full base/head/worktree accounting, unsupported flags, preservation of unrelated edits, verification failures and approvals. Read-only is a prompt contract, not a tool-deny policy. No agent/model override is configured; OpenCode's existing agent/model policies still apply.
- Host-facing changes are the two global prompt configurations and the Rust regression test (which starts an isolated private test server). No executable scripts, Makefiles, hooks, dependencies, providers or security settings were changed. Deployment/live registry verification remains pending reviewed `cockpit apply`.
