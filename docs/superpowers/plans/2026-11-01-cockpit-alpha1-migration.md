# Cockpit reset plan (dotfiles side)

> Status: **draft for Theo's review**. Nothing here is applied. The edits go on a feature branch
> only after Theo approves this plan.

**Decision (Theo, 2026-11-01):** no adapting. Most of what the dotfiles do for the cockpit exists
because of the legacy codebase's architecture (OSD registry, `hyprctl eval` hooks, a tmux manager
session, workspace 6 as an app). The dotfiles drop **all** of it, and Theo adds pieces back one at
a time, when he needs a specific one, against the new Cockpit's public surface (report 19,
"Integration contract"; issue #105).

**Goal:** after the reset, the dotfiles contain no reference to the legacy cockpit, and nothing
that only made sense for it. The new Cockpit works with none of it (report 19: "a new Cockpit
install works with the table's left column absent").

**Sources** (read 2026-11-01, `main` of Androlax2/agent-cockpit; no release or tag exists yet):
report 19, report 08 (agent context), report 20 (milestones), `crates/cockpit-cli/src/main.rs`
(`USAGE`), issues #49, #61, #62, #63, #66, #71, #97, #105, #134, #196, #305, #307, #390.

## 0. What the research changed: the switch is not Alpha 1

The task assumed Alpha 1 could become the only cockpit. The repository says otherwise:

| Fact | Source |
| --- | --- |
| The cutover (switch `make install`, delete `apps/`) is **#66, milestone "Beta 2: cutover"**, open. Report 19, #101 and #105 call it "#71", but #71 is the FigCraft plugin. | #66, #71 |
| Alpha state is not guaranteed to migrate to the beta. | #97 |
| Profiles (`COCKPIT_PROFILE`, `--profile`, `cockpit-next`) are **specified, not implemented**: `COCKPIT_PROFILE` appears nowhere in `crates/`. Today the new Cockpit uses the **same paths as the legacy one**: `~/.config/cockpit`, `~/.local/state/cockpit`, `$XDG_RUNTIME_DIR/cockpit/`. | #134, #196 (Alpha 1, open) |
| `cockpit-notify` and its status file, `cockpit notify away`, hold rules: Beta 2. No path, field names or version are specified yet. | #62 |
| Not in the CLI yet: `cockpit session handoff --all`, `cockpit session next`, `cockpit notify away on\|off`, `cockpit daemon install`, `cockpit doctor`. | `main.rs` `USAGE` |
| In the CLI: `cockpit tui`, `cockpit desktop-session import`, `cockpit ask run\|guide`, `cockpit todo …`, `cockpit agent report …`, `cockpit session …`. | `main.rs` `USAGE` |
| `cockpit tui` needs `cockpit-tui` next to `cockpit`, and `tools/release.py` leaves `cockpit-tui` out of the bundle. | `main.rs`, `tools/release.py` |
| Agents launched by the daemon run the first `claude` on its `PATH`, in a systemd user scope, **with no sandbox** until #61 (Alpha 1, open). | #61, daemon launch code |
| Phone client (`cockpit-web`, #63), GitHub tokens (#60), per-project Docker (#390): Beta 2. Teams (#53): Beta 1. | report 20 |

So the plan has two phases with different gates:

- **Phase A (side by side)** starts when Alpha 1 ships **with #134/#196 closed**. Installing
  any earlier build would share the legacy cockpit's directories and socket.
- **Phase B (the reset)** removes every legacy-cockpit piece from the dotfiles in one go. Its
  timing is open question 1: the legacy cockpit loses its desktop integration the moment the
  reset lands, whatever the new Cockpit can do by then.


## Global constraints

- Nothing in the dotfiles may depend on a path under `apps/` or on `cockpit-osd` (report 19).
- No change to the running setup until Theo approves this plan; the reset goes through the usual
  flow: branch, commit, `git format-patch`, `cockpit apply`, `make -C ~/dotfiles test`.
- The pieces agent-cockpit owns (§1.1) are removed by agent-cockpit's own uninstall, from a session
  in `~/IdeaProjects/agent-cockpit`, never from here.
- A test goes with the behaviour it covers: tests of removed behaviour are deleted in the same
  commit as the behaviour, never disabled while it is still there.
- "Add back later" items are not designed here. Each one becomes its own small change when Theo
  asks for it.

---

## 1. Inventory

### 1.1 On the host, owned by agent-cockpit (linked from `~/agent-cockpit`)

The dotfiles call these but do not own them. The reset removes every call from the dotfiles;
agent-cockpit's uninstall removes the pieces themselves (§3, step 6).

| Host path | What it is | Called from the dotfiles by |
| --- | --- | --- |
| `~/.local/bin/cockpit` | legacy CLI (`show`, `handoff`, `prs`, `away`, `log`, `setting`, `queue-prompt-ask`, `apply`, `doctor`) | bindings.lua, scratchpads.lua, lock.sh, claude-idle.sh, game-performance, tmux.conf, project CLAUDE.md |
| `~/.local/bin/cockpit-{agents,codex-hook,demo,design,fade,failsafe,game-guard,guide,health-check,idle-prompt,left,needs-input,remote,run-approve,session-doing,session-label,sidebar,subscription,team,todo}` | legacy helpers | claude-idle.sh (`cockpit-idle-prompt`), tmux.conf (`cockpit-left`), global CLAUDE.md, skills |
| `~/.local/bin/claude-{audit,demo,design,fade,guide,health-check,idle-prompt,needs-input,remote,run-approve,session-label,sessions,todo,update}` | older aliases of the above | nothing in the dotfiles |
| `~/.local/bin/{ask-to-run,ask-to-guide,craft-open}` | approval, guide and FigCraft launchers | global CLAUDE.md, skills, project CLAUDE.md |
| `~/.local/bin/{osd,osd-show}` | GTK4 OSD daemon (also the legacy registry) | volume.sh, brightness-adjust-focused.sh, waybar `on-click` |
| `~/.local/bin/{agent-build,agent-docker-build,clipboard-image-serve,greyproxy-prune,opencode-update}` | sandbox helpers, updater | opencode.jsonc comment, disk-report.sh hint |
| `~/.local/bin/greywall-agents/{claude,codex,opencode,figcraft.sh,launcher-common.sh}` | greywall launchers | fish `config.fish`, bash `.bashrc` and `.profile` (PATH) |
| `~/.local/bin/{greywall,greyproxy}` (+ `.previous`, `greywall.broken-bb85cd4`, `greywatch`) | built binaries, not links | README |
| `~/.config/systemd/user/`: `cockpit.service`, `osd.service`, `osd-report.socket`, `cockpit-remote.service`, `cockpit-failsafe.{service,timer}`, `cockpit-game-guard.service`, `cockpit-health-check.{service,timer}`, `claude-health-check.service`, `claude-remote.service`, `claude-update.{service,timer}`, `clipboard-image-bridge.service`, `greyproxy.service`, `greyproxy-prune.{service,timer}` | units and timers | autostart.lua (`osd`, `cockpit`), game-performance (`cockpit-health-check`) |
| `~/.config/hypr/figcraft-review.lua`, `~/.config/hypr/apps/craft.lua` | FigCraft window rules | loaded by the Hyprland config |
| `~/.config/waybar/scripts/{cockpit.sh,claude-usage.sh,codex-usage.sh,codex-quota.py,codex-session.py,native-subscription.py}` | the bar's cockpit chip and usage | waybar `custom/cockpit` |
| `~/.config/waybar/scripts/{on-claude-workspace.sh,workspace-listener.sh,waybar-visibility.sh}` | "am I on workspace 6", hide the bar there | claude-idle.sh, lock-if-not-gaming.sh, lock-session-if-not-gaming.sh, screen-off-if-not-gaming.sh, waybar `custom/workspace-listener` |
| `~/.config/fish/functions/{claude,opencode}.fish` | shell wrappers | interactive shells |
| `~/.config/kitty/cockpit.conf` | the manager window's kitty config (modifyOtherKeys chords) | the legacy `cockpit show` window |
| `~/.config/greywall/greywall.json` | sandbox policy | the launchers |
| `~/.claude/statusline-command.sh`, `~/.claude/hooks/{session-state.sh,agent-environment.sh,billing-capture.py}` | Claude status line and hooks | `~/.claude/settings.json` (not versioned anywhere I can see: `SessionStart`, `UserPromptSubmit`, `PostToolUse`, `Notification`, `Stop`, `StopFailure`, `SessionEnd` all run `session-state.sh`) |
| `~/.codex/…`, `~/.config/opencode/` plugin | Codex hook, opencode plugin | not inventoried: hidden from this sandbox (open question 1) |


### 1.2 In the dotfiles: what the reset removes, what stays

"Stays" lists only what is left in a file that loses its cockpit part. Every "add back later"
candidate is in §4.

#### Hyprland (`config/hyprland/.config/hypr/`)

| Path | Removed | Stays |
| --- | --- | --- |
| `bindings.lua:79-81` | `SUPER+C` (`cockpit show`), `SUPER+SHIFT+C` (`cockpit handoff`) and their comment | the rest of the binds |
| `scratchpads.lua` | the `prs`, `claude`, `greyproxy` scratchpads, `GREYPROXY_DASHBOARD`, `CLAUDE_WORKSPACE`, the `cockpit` flag of `web_scratchpad`, `on_claude_workspace`, the cockpit gate in `toggle`, `OPEN_IN_SCRATCHPAD` and `open_in`, `SUPER+CTRL+P/A/G`, the Escape handler, the `workspace.active` hook, header lines 7-8, 13-14 | `term`, `music`, `vitals`, `web_scratchpad`, `TOGGLE_SCRATCHPAD` (`tools/capture.sh`), Zen tagging, placeholders, the fullscreen hook |
| `apps/claude.lua` | the whole file (state tags, `CLAUDE_WINDOW_STATE`, workspace 6 rules, `cockpit`, `claude-run`, `cockpit-guide`, `figcraft-chat`, `claude-idle` rules) | nothing |
| `apps/osd.lua` | the whole file | nothing |
| `windows.lua:27,38,43` | `require("apps/claude")`, `require("apps/osd")`, `require("apps/craft")` (`craft.lua` is agent-cockpit's link: the require breaks the config once agent-cockpit uninstalls) | the other requires |
| `autostart.lua:8-23` | `restart osd.service`, `start cockpit.service`, the comment naming the OSD and the session manager | `import-environment` and the restart of `elephant.service walker.service` |
| `scripts/volume.sh:19,21`, `brightness-adjust-focused.sh:38`, `bindings.lua:95` comment | the `osd-show` calls (no on-screen indicator until Theo picks one) | the volume and brightness changes, the bar refresh |
| `scripts/lock.sh` | `cockpit log lock`, `cockpit away on\|off`, `locked_at`, the header about the phone page | `pidof hyprlock >/dev/null && exit 0; hyprlock` |
| `scripts/claude-idle.sh`, `scripts/claude-idle.css` | both files | nothing |
| `hypridle.conf:24-31` | the workspace 6 listener (`claude-idle.sh prompt\|cancel`) and its comment | the other listeners |
| `scripts/{lock-if-not-gaming,lock-session-if-not-gaming,screen-off-if-not-gaming}.sh` | the `on-claude-workspace.sh` early exit and its comment: workspace 6 locks like any other | the gaming checks |

#### Waybar (`config/waybar/.config/waybar/`)

| Path | Removed | Stays |
| --- | --- | --- |
| `config.jsonc:17-26,198-210` | `group/claude`, `custom/cockpit` and the comments about the Claude island | `group/right` (becomes the only right group) |
| `config.jsonc:54,341-345` | `custom/workspace-listener` (agent-cockpit's script that hides the bar on workspace 6) | the other listeners |
| `style.css:155,191-198,238-245,329` | `#custom-cockpit*`, `#custom-workspace-listener` | the rest |
| `../../hypr/apps/waybar.lua:4-6` | `no_anim = true` and its comment (only there so the bar vanished on workspace 6) | the blur rule |

#### tmux, kitty, shells

| Path | Removed | Stays |
| --- | --- | --- |
| `config/tmux/.config/tmux/tmux.conf:15-50` | `claude-*` status hook, the passthrough hook (`claude-manager`, `claude-retry-opencode-*`), `window-size latest`, `extended-keys`, `focus-events`, `extkeys`, `C-Tab`/`C-S-Tab`/`C-BTab`/`C-1..3`, `Left` → `cockpit-left`, `M-q` → `cockpit queue-prompt-ask` | lines 1-13: terminal, mouse, titles, theme, `set-clipboard on` |
| `config/tmux/.config/tmux/border-junction.sha256`, `packages/tmux/` (patch, test, README) | the local tmux border patch, its hash baseline and `cockpit doctor` check (it exists for the cockpit's pane borders) | nothing; host step: reinstall stock `tmux` |
| `tools/palette/kitty.conf.tpl:29-32` → `config/kitty/.config/kitty/kitty.conf` | `allow_remote_control socket-only`, `listen_on unix:@kitty` and the OSD-banner comment (`make palette` re-renders) | the rest |
| `config/fish/.config/fish/config.fish:20-21`, `config/bash/.bashrc:1-2`, `config/bash/.profile:1-2` | `~/.local/bin/greywall-agents` at the front of `PATH` | the rest; `claude`, `codex`, `opencode` then run unsandboxed when typed in a shell (open question 3) |

#### Agents: instructions, skills, packages

| Path | Removed | Stays |
| --- | --- | --- |
| `config/claude/.claude/CLAUDE.md` (also `~/.config/opencode/AGENTS.md`) | "Operating mode" bullets for `cockpit-needs-input`, `cockpit-session-label`, `cockpit-session-doing`; the whole "Sandbox environment (greywall)" section (greywall, greyproxy, Docker, GitHub token, dotfiles/cockpit/vitals apply, handoff, `ask-to-run`, todo list, hand steps, FigCraft, teams, Claude Design) | everything else: skill activation, operating mode, tone, ADHD rules, naming, comments, architecture, testing, git hygiene, hard rules, push-backs |
| `config/claude/.claude/skills/design-first/SKILL.md` | `cockpit-design role`, `craft-open`, `cockpit-todo add\|propose --design` steps | the design-first method (open question 5) |
| `config/claude/.claude/skills/figcraft-design/SKILL.md` + `evals/evals.json` | `cockpit-design role`, `cockpit design-review start\|stop` float, `cockpit-todo` handoffs, `ask-to-run` for `hyprctl` | the design method (open question 5) |
| `CLAUDE.md` (project) | "Applying completed work" (`cockpit apply`, `.outgoing`), "Todo trailers", the paragraph on agent-cockpit and vitals sessions | "Git authorization", the host-only Python tests note |
| `config/opencode/.config/opencode/opencode.jsonc:2-4,9-17` | `autoupdate: false` (held for the cockpit plugin) and the `figcraft-app` MCP entry (the launcher's forwarded port) | the anthropic auth plugin, other MCP servers |
| `packages/npm.txt:2` | `claude-auto-retry` (only the greywall launcher used it) | `@openai/codex` and the rest; host step: `npm uninstall -g claude-auto-retry` |

#### Machine scripts, Makefile, docs, tests

| Path | Removed | Stays |
| --- | --- | --- |
| `config/bin/.local/bin/game-performance` | sandbox freezing (`sandbox_scopes`, `FROZEN_LIST`, thaw), `freeze-while-gaming`, `cockpit-health-check.*` in both lists; `cockpit log game` → `logger -t game-performance` | paused timers, idle priorities, Hyprland game mode |
| `tools/disk-report.sh:3,9,25-32,41-43` + `tools/tests/test_disk_report.py:20` | the agent clones section (`COCKPIT_CLONES`), greyproxy in the sizes, the `greyproxy-prune` hint | caches, `~/.claude`, system, games |
| `config/osd/`, `tools/palette/osd-style.css.tpl`, `tools/palette/claude-idle-style.css.tpl`, `tools/palette/targets.txt:14,19` | the OSD and idle-prompt styles | the other palette targets |
| `Makefile` | `dotfiles-work` target and its `.PHONY` entry; "OSD" in the `palette` help; the comments at lines 7-9 and 107-108 reworded (agent-cockpit's links stay until its uninstall) | everything else |
| `.gitignore:17-18` | `.outgoing/` | the rest |
| `config/zen/.config/zen/scratchpad.user.js:1,17` | "prs, claude, greyproxy" and "cockpit' pull requests" from the comments | the prefs (they serve the music scratchpad too) |
| `README.md:131,139,573` + keybindings rows for `Super+C`, `Super+Shift+C`, `Super+Ctrl+P/A/G` + the OSD mentions | the cockpit, greywall and game-sandbox paragraphs | the rest |
| `apps/config-tests/tests/` | `cockpit_windows.rs`, `claude_idle.rs`; in `cockpit_calls.rs` the lock, tmux and two opencode tests (`figcraft_app…`, `leaves_updates…`); `tmux_clipboard.rs::only_opencode_and_cockpit_windows_allow_terminal_passthrough`; the sandbox, health-timer and freeze tests of `game_performance.rs` | `cockpit_calls.rs` renamed `agent_instructions.rs` with the two shared-rules tests; the timer/priority/Hyprland tests of `game_performance.rs` (log assertion → `logger`); `systemd_units.rs` (comment reworded) |
| `docs/superpowers/plans/2026-10-*.md` | nothing (history) | as is |

Cron: the dotfiles hold no cron entries. Env vars that disappear: `PATH` (greywall-agents),
`COCKPIT_CLONES`, `CLAUDE_IDLE_GRACE_S`, `CLAUDE_IDLE_SCREEN_OFF_AFTER_S`.

---

## 2. Phase A: Alpha 1 side by side (`cockpit-next`)

**Gate:** Alpha 1 released with #134 and #196 closed (profiles), `cockpit-tui` in the bundle.

The dotfiles add **nothing** for phase A: Theo opens `cockpit-next tui` from any terminal. Two
known side effects while the legacy cockpit is still installed, neither fixed in the dotfiles:

- **Legacy hooks fire inside cockpit-next sessions.** The new Cockpit passes its hooks with
  `--settings` and leaves `~/.claude/settings.json` alone, so Claude runs both:
  `session-state.sh` reports cockpit-next sessions into the legacy registry, and
  `agent-environment.sh` says "greywall is active" for an unsandboxed session. Gone after
  agent-cockpit's uninstall (§3, step 6); before that, agent-cockpit can skip both when
  `COCKPIT_LAUNCH_TOKEN` is set (to propose to agent-cockpit after approval).
- **Which `claude` the daemon starts.** It runs the first `claude` on its `PATH`; while
  `~/.local/bin/greywall-agents` leads it, that is the legacy launcher. The reset's `PATH` change
  (§1.2) ends this.

Alpha state may be discarded at the beta (#97): no real work only in `cockpit-next`.

---

## 3. Phase B: the reset

One branch, one commit per group of §1.2, so any group can be reverted alone. About 2 hours of
edits, plus the host steps.

1. **Hyprland** group (`windows.lua` requires first, so the config never loads a missing file).
2. **Waybar** group.
3. **tmux, kitty, shells** group; `make palette` for kitty.
4. **Agents** group (instructions, skills, project `CLAUDE.md`, opencode, npm list).
5. **Machine scripts, Makefile, docs, tests** group; `cargo test` in `apps/config-tests` and
   `make test` green in the clone.
6. **Apply and clean the host**, in this order, each through `ask-to-run`:
   1. `cockpit apply <clone>` (the legacy flow, used one last time), then `make -C ~/dotfiles test`.
   2. `make -C ~/dotfiles dotfiles` (restow; drops links to deleted files).
   3. `hyprctl reload`, restart waybar, `tmux source-file ~/.config/tmux/tmux.conf` on the running server.
   4. Host packages: stock `tmux` back from pacman, `npm uninstall -g claude-auto-retry`.
   5. agent-cockpit uninstall, from a session in `~/IdeaProjects/agent-cockpit` (filed there with
      `cockpit-todo add … --project agent-cockpit`): its `~/.local/bin` links, units (§1.1),
      `~/.claude/settings.json` hooks and status line, greywall launchers and policy.
7. **New Cockpit** (when Theo wants it as the only one): install the default profile once #66
   allows it; nothing in the dotfiles changes for that.

### Rollback

- Before 6.5: `git revert` the group's commit in a clone of `~/dotfiles`, apply, restow, reload.
  The legacy cockpit still runs, so this restores its integration fully.
- After 6.5: `make -C ~/agent-cockpit install` relinks the legacy pieces (the repository is not
  deleted), then the revert above.

### Global CLAUDE.md after the reset

The cockpit rules leave with the "Sandbox environment" section; how an agent talks to the new
Cockpit comes from its instruction bundle (#49), installed per provider. Personal rules stay on
top. For reference when a rule comes back, the new Cockpit's agent commands are:

| Legacy | New Cockpit (`main.rs` `USAGE`) |
| --- | --- |
| `cockpit-needs-input`, `cockpit-session-label`, `cockpit-session-doing` | `cockpit agent report --needs-input '<q>' \| --label '<l>' \| --doing '<d>'` |
| `ask-to-run --why '<why>' [--cwd DIR] '<cmd>'`, `--wait <id>` | `cockpit ask run --reason '<why>' [--cwd DIR] '<cmd>'`, `--wait <id>` (exit codes 125/124/126/3 unchanged) |
| `ask-to-guide`, `--wait <id>` | `cockpit ask guide < steps`, `--wait <id>` |
| `cockpit-todo list\|add\|propose\|start\|done\|delete\|after\|blocked` | `cockpit todo …`, same flags; `blocked` lands in `tasks.blocker_project` |
| `cockpit-todo stack <repo>` | `cockpit todo propose … --project <repo>` |
| `cockpit-design link` | owner verb `cockpit session link SESSION URL` |
| `cockpit-team`, `craft-open`, Docker and GitHub token helpers, `cockpit apply` | none yet (#53 Beta 1, #71, #390, #60 Beta 2); `cockpit apply` is dropped from the product |

---

## 4. Add back later (not part of the reset)

Pieces Theo may want again, each its own change when asked, against the new Cockpit's surface:

| Need | New surface | Available |
| --- | --- | --- |
| A key to open the Cockpit | `kitty --class cockpit -e cockpit tui` | Alpha 1 |
| Agents' display after login | `cockpit desktop-session import` in `autostart.lua` (lost at daemon restart, #307) | now |
| Bar chip | the versioned status file of `cockpit-notify` | Beta 2 (#62), schema unspecified |
| "Next session needing me" | `cockpit session next` | not built |
| Hand off to Remote Control | `cockpit session handoff --all` | not built |
| Quiet while locked | `cockpit notify away on\|off` in `lock.sh` | Beta 2 (#62) |
| Lower agents while gaming | `cockpit resources profile foreground\|normal` in `game-performance` | now |
| Volume/brightness indicator | any desktop OSD | not a Cockpit feature |
| Applying dotfiles clones | a dotfiles script (product drops `cockpit apply`) | dotfiles work |
| Phone notifications | `cockpit-web` | Beta 2 (#63) |

---

## 5. Open questions for Theo

1. When does the reset land: now (the legacy cockpit keeps running but loses its binds, bar chip, idle lock, tmux keys and agent rules), or when you install the new Cockpit as your only one?
2. Without "Applying completed work", how should dotfiles changes reach `~/dotfiles` after the reset: you pull from a clone by hand, or sessions work in `~/dotfiles` directly?
3. With greywall-agents off `PATH`, should agents you start by hand run unsandboxed, or should the shell `claude` stay sandboxed until #61?
4. Can a session with access to `~/.codex` and `~/.config/opencode` list the legacy Codex hook and opencode plugin, so agent-cockpit's uninstall covers them?
5. Keep the `design-first` and `figcraft-design` skills without their cockpit steps, or remove them too?
6. Report 19, #101 and #105 cite #71 as the cutover, but #71 is the FigCraft plugin and the cutover is #66; should I file a fix for the cockpit repo?
