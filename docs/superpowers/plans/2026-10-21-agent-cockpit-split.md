# Agent Cockpit split and rename — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `~/dotfiles` holds only the configuration of Theo's machine. The agent cockpit moves to its own
repo `agent-cockpit` (command `cockpit`, UI "Agent Cockpit"), the machine monitor to its own repo `vitals`.

**Architecture:** Five phases, each shipped and applied on its own through the reviewed apply flow, so the
machine works after every phase: (1) rename `claude-sessions` to `cockpit` inside dotfiles, (2) make the
reviewed apply work for any live checkout, (3) cut the crate dependencies that cross the future repo
boundary, (4) extract the two repos with their history, (5) cut the host over and delete the moved files
from dotfiles.

**Tech Stack:** Rust workspace (`apps/`), POSIX/bash scripts, systemd user units, GNU stow, greywall.

**Todos:** f4f67e5a (move), bc95a4d0 (rename).

## Decisions (taken with Theo, 2026-10-21)

| Question | Decision |
|---|---|
| What leaves dotfiles | Every program that is not machine configuration: the cockpit and vitals |
| Desktop widgets (`mpris-card`, `web-placeholder`, `cooler-overlay-mask`) | Stay in dotfiles |
| `osd`, `claude-idle-prompt` | Move with the cockpit (osd is the cockpit's daemon) |
| Repo names | `agent-cockpit`, `vitals` (GitHub, Androlax2) |
| Live checkouts (run by the host, never by a session) | `~/agent-cockpit`, `~/vitals` |
| Work clones (sessions work there) | `~/IdeaProjects/agent-cockpit`, `~/IdeaProjects/vitals` |
| Command | `cockpit`; repo and UI say "agent-cockpit" / "Agent Cockpit" |
| History | Kept: per-file history of every moved path |

## Global Constraints

- The security model does not weaken: a live checkout changes only through `cockpit apply <clone-root>`
  after Theo read the diff; the launcher refuses a session inside any live checkout; no sandbox gains
  read or write access it did not have.
- Compatibility kept: `claude-sessions` stays as a forwarding script (`exec cockpit "$@"`, one stderr line
  "claude-sessions is now cockpit") until the follow-up todo removes it; the old state folder is moved
  once, never copied.
- Out of scope for the rename: `claude-todo`, `claude-remote`, `claude-audit`, `claude-design`,
  `claude-guide`, `claude-fade`, `claude-run-approve`, `claude-update`, `claude-health-check`,
  `claude-needs-input`, `claude-session-label`, `ask-to-run`, `ask-to-guide`, and the audit folder
  `~/.local/state/claude-audit` (greywall denies it by that name). A follow-up todo is proposed for them.
- Every phase ends with `make test` green in the work clone, then `make -C ~/dotfiles cockpit test` green
  on the live checkout after apply (phase 5: the equivalent targets of each repo).

## Review Focus

1. **A running session at apply time**: it keeps its old `CLAUDE_SESSIONS_*` environment until it
   restarts (a Codex session reports as Claude meanwhile). Phase 1 ends with `cockpit restart-all`.
2. **State folder already migrated, or both folders exist**: never merge, never overwrite. If both exist,
   `cockpit prepare` logs one error naming both and keeps using the new one (test in Task 1.2).
3. **Sandbox visibility of renamed paths**: `~/.local/state/cockpit` must be exactly as hidden as
   `~/.local/state/claude-sessions` was (test in Task 1.4, `greywall_launcher.rs`).
4. **A session started inside `~/agent-cockpit` or `~/vitals`**: refused like `~/dotfiles` today
   (test in Task 2.2).
5. **Apply of a patch series to the wrong running copy** (a vitals clone's patches onto agent-cockpit):
   impossible, because the clone's folder decides the target (Phase 2 tests).

---

## Phase 1 — Rename `claude-sessions` to `cockpit` (in dotfiles)  ·  about half a day

Mechanical, but 1,328 mentions in 127 files. Mapping:

| Old | New |
|---|---|
| `claude-sessions` (command, kitty class, tmux name, systemd unit `claude-sessions.service`) | `cockpit` (`cockpit.service`) |
| `claude-sessions-sidebar`, `claude-sessions-left`, `claude-sessions-agents/` | `cockpit-sidebar`, `cockpit-left`, `cockpit-agents/` |
| `~/.local/state/claude-sessions/` | `~/.local/state/cockpit/` |
| `$XDG_RUNTIME_DIR/claude-sessions.*`, `claude-sessions-*` runtime files | `cockpit.*`, `cockpit-*` |
| `CLAUDE_SESSIONS_*` (24 variables) | `COCKPIT_*` |
| `kitty/claude-sessions.conf`, `waybar/scripts/claude-sessions.sh`, waybar module `custom/claude-sessions` | `cockpit.conf`, `cockpit.sh`, `custom/cockpit` |
| UI and docs: "claude-sessions", "Claude cockpit" | "Agent Cockpit" (prose), `cockpit` (commands) |

### Task 1.1: Rename files and identifiers

**Files:** every file listed by `grep -rIl 'claude-sessions\|claude_sessions\|CLAUDE_SESSIONS' .` except
`docs/superpowers/**` and `design-plans/**` (historical records stay as written).

- [ ] `git mv` the files of the mapping table; rename the bin `claude-sessions-sidebar.rs` to
  `cockpit-sidebar.rs` and test files `claude_sessions_script.rs` → `cockpit_script.rs`.
- [ ] Replace identifiers per the table (paths, env vars, unit, class, help text, README, both `CLAUDE.md`).
- [ ] Add `config/bin/.local/bin/claude-sessions`: `#!/bin/sh`, one stderr line, `exec cockpit "$@"`.
- [ ] Run `make test`; expected: PASS.
- [ ] Commit `Rename claude-sessions to cockpit` with `Todo: bc95a4d0`.

### Task 1.2: Move the state folder once (done)

**Files:** `config/bin/.local/bin/cockpit` (`move_old_state_folder`, run first on every call),
`apps/cockpit/tests/cockpit_script.rs`

- [x] Tests: `the_first_run_after_the_rename_moves_the_old_state_folder_and_links_its_name`,
  `two_state_folders_are_both_kept_and_the_start_reports_them`, `the_old_claude_sessions_name_runs_cockpit`.
- [x] The old folder name stays as a link to `cockpit`, so the programs started before the rename (old
  sidebar, old osd) keep writing into the moved folder until `cockpit reload` restarts them. This
  replaces the planned Task 1.3 (old runtime names read by osd): osd and sidebar restart together on
  reload, and the hooks are live files, so no reader of old runtime names is left after it.

### Task 1.4: Sandbox visibility unchanged

**Files:** `config/greywall/.config/greywall/greywall.json`, `apps/cockpit/tests/greywall_launcher.rs`

- [ ] Test `a_session_cannot_read_the_cockpit_state_folder` (parametrized over the new folder and
  `claude-audit`).
- [ ] Update every renamed path in `greywall.json` and `learned/*.json`; `make test` PASS, commit.

### Task 1.5: Ship

- [ ] `git format-patch origin/main -o .outgoing`; `ask-to-run` `claude-sessions apply-dotfiles <clone>`
  (the live name until the apply lands).
- [ ] Then, in one `ask-to-run`: `stow --restow systemd kitty waybar` from `~/dotfiles/config` (apply
  restows only `bin`); `systemctl --user daemon-reload; systemctl --user disable claude-sessions.service;
  systemctl --user enable cockpit.service`; `make -C ~/dotfiles cockpit test`; `pkill -SIGUSR2 -x waybar`;
  close the manager window still named `claude-sessions`
  (`hyprctl dispatch 'hl.dsp.window.close({ window = "class:^claude-sessions$" })'`),
  then `cockpit reload` and `cockpit restart-all`.

## Phase 2 — Reviewed apply for any running copy (in dotfiles)  ·  done

- [x] `cockpit apply <clone>` (`apply-dotfiles` stays as its other name): `REVIEWED_REPOSITORIES=(dotfiles
  agent-cockpit vitals)` in the script; the clone's folder (`~/IdeaProjects/<name>` or
  `<clones>/<name>/<clone>`) decides the running copy (`~/<name>`), never the clone's git config, which
  the sandbox writes. A missing running copy is refused. dotfiles keeps its own after-apply steps
  (restow `bin`, rebuild `apps/`); the others run their Makefile's `after-apply BEFORE=<commit>`.
- [x] The launcher refuses a session in any of the three running copies and gives a clone read access
  to its own running copy only; the hook counts "↑N unapplied" against each; the cards count
  "↑N unpushed" per running copy (`sessions::running_copy_of`, `paths::REVIEWED_REPOSITORIES`).
- [x] Tests: `apply_takes_a_work_clones_commits_into_its_own_running_copy`,
  `apply_takes_a_parallel_clones_commits_into_the_running_copy_of_its_repository`,
  `apply_refuses_a_repository_whose_running_copy_is_missing`,
  `apply_runs_the_running_copys_after_apply_target_with_the_commit_before`, `no_session_starts_in_a_running_copy`,
  `the_clones_of_a_reviewed_repository_read_its_running_copy_and_no_other`, and the hook and card tests
  parametrized over the three.

## Phase 3 — Cut the crate dependencies (in dotfiles)  ·  done

- [x] New crate `apps/osd` (lib `fifo`, `registry`, `teams` and their tests; bins `osd`,
  `claude-idle-prompt`), moved with `git mv`. `apps/desktop` keeps `mpris-card`, `web-placeholder`,
  `cooler-overlay-mask` with its own `host.rs` (`home`, `runtime_dir`, `output_within`) and `fifo::open`:
  no cockpit dependency.
- [x] `apps/vitals/src/host.rs` holds the helpers vitals used from the cockpit crate;
  `host::send_notice` writes the osd protocol line to `$XDG_RUNTIME_DIR/osd-host.fifo` (the contract
  between the two repos). vitals reads `XDG_STATE_HOME` only, not `COCKPIT_STATE_HOME`. No cockpit
  dependency.
- [x] New crate `apps/config-tests`: the tests of files that stay in dotfiles (hotcorner, web_launch,
  game_mode_lua, waybar_jobpilot, system_install, tmux_clipboard, switch_audio), with a small sandbox
  of its own.
- [ ] Left for Phase 4's extraction lists: the cockpit-only config files (`hypr/scripts/claude-idle.*`,
  `figcraft-review.lua`, `apps/craft.lua`, the Waybar `cockpit.sh`, `claude-usage.sh`, `codex-usage.sh`,
  `codex-quota.py`, `on-claude-workspace.sh`, `waybar-visibility.sh`) and the integration files whose
  tests sit in the cockpit crate (fish `claude` function, tmux bindings, opencode plugins, codex
  hooks, `lock.sh`). Decide per file there, with Theo.

## Phase 4 — Extract the repos  ·  done

- [x] Rule (Theo, 2026-10-21): files that exist only to plug the cockpit in move with it (hooks,
  opencode and Codex plugins, greywall launchers and profiles, Waybar cockpit scripts and bar
  visibility, figcraft-review.lua, craft.lua, the fish `claude`/`opencode` functions); configuration
  that only calls cockpit commands stays (fish, tmux, Hyprland bindings, lock.sh, claude-idle.sh,
  the Waybar layout, CLAUDE.md, skills, opencode's own settings).
- [x] `tools/extract/<name>.paths` (+ `.history-paths`, the names before the renames),
  `tools/extract/<name>/` (Makefile with build, test, install, system, after-apply; README;
  CLAUDE.md; `.gitignore`; `apps/Cargo.toml`), `tools/extract-repo.sh <name> <dest>`.
- [x] Same layout as in dotfiles (`apps/`, `config/<package>/…`, `tools/`), so every relative path,
  wrapper and test keeps working; install is `stow --no-folding` from each repo's `config/`.
- [x] Trial run on scratch copies: agent-cockpit 388 commits, `make test` green (1601 Rust tests +
  the Python tools); vitals 28 commits, `make test` green (165).
- [x] Theo: `git-filter-repo` (pacman), the public repos Androlax2/agent-cockpit (388 commits) and
  Androlax2/vitals (28), extracted into `~/agent-cockpit` and `~/vitals`, pushed, work clones in
  `~/IdeaProjects/`.

## Phase 5 — Cut over  ·  5b committed in dotfiles (not applied), waiting for 5a

### 5a — in an agent-cockpit session (`~/IdeaProjects/agent-cockpit`), applied first

Inert until 5c: nothing is linked from `~/agent-cockpit` yet. agent-cockpit's todo "Cutover 5a"
lists the files; in short, every `~/dotfiles/…` path of the cockpit becomes `~/agent-cockpit/…`:
- greywall.json and `learned/*.json`: the allowRead of the wrappers and builds, `~/dotfiles/config/claude`
  → `~/agent-cockpit/config/claude` while keeping `~/dotfiles/config/claude/.claude/CLAUDE.md` and
  `…/skills` (they stay in dotfiles), denyWrite `~/dotfiles/config/greywall`; drop the `.json.bak`.
- opencode plugin `cockpit.ts`: the hook path. Every wrapper's "make -C ~/dotfiles cockpit" message.
- `cockpit` script: `OSD_BINARY` default; doctor fixes; `after_dotfiles_apply` restows every changed
  `config/<package>` with `--no-folding` and runs `make -C ~/dotfiles build` when `apps/` changed (no
  osd, no `cockpit` target in dotfiles any more); `rebuild_apps` belongs to agent-cockpit's after-apply.
- launcher-common.sh, greywall-agents/claude, agent-environment.sh (`make -C ~/agent-cockpit agent-*`),
  claude-health-check (both repos' tests).
- Port dotfiles' `tools/tests/test_test_entrypoint.py` (make test builds osd first) to agent-cockpit.
- Ask Theo: error hand-offs and `claude-todo blocked` reports, to the agent-cockpit session instead of
  dotfiles?

### 5b — dotfiles (committed: "Move the Agent Cockpit and vitals out of dotfiles")

The moved paths deleted, `apps/` = `config-tests` + `desktop`, Makefile without the cockpit's targets
and with `stow --no-folding`, README/CLAUDE.md/Claude's rules pointing at the new repos. `make test`
green alone (56 tests); the widgets build.

### 5c — the switch-over (one `ask-to-run`, fish, `;` between steps: the old apply's rebuild fails by design)

```fish
cockpit apply <dotfiles clone>        # 5b; its "make cockpit" fails: dotfiles no longer has it
cd ~/dotfiles/config; and stow -v --no-folding -t ~ --restow */   # unfold waybar, hypr, fish
find ~/.local/bin ~/.local/share ~/.config ~/.claude ~/.codex -xtype l -lname '*dotfiles/config/*' -print -delete
make -C ~/agent-cockpit install; and make -C ~/vitals install
systemctl --user daemon-reload
systemctl --user restart osd.service cockpit.service vitals-collect.timer
cockpit reload; cockpit doctor
```
Then restart every session (CMD+R in the sidebar): their greywall profile changed.
Checks: ALT+C opens the cockpit; `ask-to-run` and `claude-todo list` work from a session; the bar's
vitals dot shows; `make -C ~/agent-cockpit test`, `make -C ~/vitals test`, `make -C ~/dotfiles test`.
