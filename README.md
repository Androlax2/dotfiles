# dotfiles

Everything needed to rebuild my Arch Linux desktop running Hyprland: user dotfiles, system files under `/etc`, package lists for eight package managers, and a Makefile that provisions, audits and cleans the machine.

![kitty with fastfetch, btop and yazi tiled under the bar](assets/terminals.png)

> [!WARNING]
> This is a personal setup for one machine: AMD CPU, Realtek RTL8922AE Wi-Fi, LG ultrawide, French keyboard.
> Several files hardcode `/home/theo` and expect the repo at `~/dotfiles`.
> Borrow freely, but read before running `make setup` elsewhere: it overwrites files in `/etc`.

## What's inside

| Path | Contents |
| --- | --- |
| [`config/`](config) | One [GNU Stow](https://www.gnu.org/software/stow/) package per program, symlinked into `~` |
| [`system/`](system) | Files installed under `/` as root: boot, initramfs, locales, pacman, networking, Bluetooth, SDDM, keyd |
| [`packages/`](packages) | Package lists for pacman, AUR, npm, pnpm, pip, cargo, go, composer and agent skills |
| [`services.txt`](services.txt) | System units enabled beyond their vendor preset |
| [`system-ignore.txt`](system-ignore.txt) | `/etc` paths deliberately left out of the repo, each with a reason |
| [`Makefile`](Makefile) | Provision, capture, audit and clean the machine; `make help` lists every target |
| [`tools/`](tools) | Helpers for the Makefile: the `make help` renderer, the palette generator, the security check, the README screenshots |

The desktop itself:

- **Hyprland** with hypridle, hyprlock, hyprsunset and a wallpaper that rotates every 30 minutes
- **Waybar**, the **Walker** launcher with Elephant providers, and **swaync** notifications
- **kitty**, **fish** and **helix**, plus lazygit, btop, fzf, bat, delta, yazi and tmux, all on the same palette
- **Mac-style shortcuts** through keyd, with Ctrl and Command swapped
- **Calendars** synced by vdirsyncer, shown in khal, with desktop notifications
- **Hardware extras**: an OpenRGB profile at boot, the Thermalright LCD, AirPods kept as the default audio sink
- **Boot**: the SDDM `silent` theme with an on-screen keyboard

![Walker launcher open over the wallpaper](assets/launcher.png)

## Design

One palette, Tokyo Night (night), applied the same way everywhere:

- **Tokens** live in [`config/waybar/.config/waybar/colors.css`](config/waybar/.config/waybar/colors.css). `make palette` renders kitty, GTK, qt6ct, swaync, Walker, Hyprland, fish, lazygit, fzf, btop, the OSD, slurp and Zen from the templates in [`tools/palette`](tools/palette); `make check-palette` reports drift.
- **Ground** `#1a1b26` at 85 % behind blurred islands, a `rgba(192, 202, 245, 0.10)` hairline, radius 10.
- **Icons are muted** (`#a9b1d6`), **values carry the colour**: blue for CPU, purple for GPU, yellow for volume, red for recording.
- **One colour family per element**, no gradients across families.
- **The grid**: 15 px outer gaps, 6 px between the bar and the windows, 8 px between islands.
- **Type**: SF Pro Text in the interface and notifications, JetBrainsMono Nerd Font in the bar and terminals.

![A floating terminal with fastfetch over the wallpaper](assets/desktop.png)

![Two windows tiled under the Waybar islands](assets/layout.png)

![Bar](assets/bar.png)

![Notifications](assets/notification.png)

![Apple Music scratchpad](assets/music.png)

![Bar with a recording running](assets/recording.png)

## Keybindings

`Super` is the physical Command key: keyd swaps it with Ctrl, so Hyprland sees these as `CTRL`. `Super + Ctrl + K` (or the keyboard icon at the end of the bar) shows this list on screen, and choosing an entry runs it; it comes from `hyprctl binds` and the `description` of each bind in [`bindings.lua`](config/hyprland/.config/hypr/bindings.lua).

| Keys | Action |
| --- | --- |
| `Alt + 0` | Workspace 10 |
| `Alt + 1` | Workspace 1 |
| `Alt + 2` | Workspace 2 |
| `Alt + 3` | Workspace 3 |
| `Alt + 4` | Workspace 4 |
| `Alt + 5` | Workspace 5 |
| `Alt + 6` | Workspace 6 |
| `Alt + 7` | Workspace 7 |
| `Alt + 8` | Workspace 8 |
| `Alt + 9` | Workspace 9 |
| `Alt + F` | Fullscreen |
| `Alt + G` | Game mode by hand (it turns on by itself when a game opens) |
| `Alt + J` | Toggle window split |
| `Alt + Q` | Terminal |
| `Alt + Scroll down` | Next workspace |
| `Alt + Scroll up` | Previous workspace |
| `Alt + Shift + 0` | Move window to workspace 10 |
| `Alt + Shift + 1` | Move window to workspace 1 |
| `Alt + Shift + 2` | Move window to workspace 2 |
| `Alt + Shift + 3` | Move window to workspace 3 |
| `Alt + Shift + 4` | Move window to workspace 4 |
| `Alt + Shift + 5` | Move window to workspace 5 |
| `Alt + Shift + 6` | Move window to workspace 6 |
| `Alt + Shift + 7` | Move window to workspace 7 |
| `Alt + Shift + 8` | Move window to workspace 8 |
| `Alt + Shift + 9` | Move window to workspace 9 |
| `Alt + Shift + H` | Toggle HDR |
| `Alt + Shift + S` | Scratch workspace |
| `Alt + Shift + TAB` | Previous workspace |
| `Alt + TAB` | Next workspace |
| `Alt + W` | Close window |
| `Ctrl + down` | Focus down |
| `Ctrl + left` | Focus left |
| `Ctrl + right` | Focus right |
| `Ctrl + Shift + down` | Move window down |
| `Ctrl + Shift + left` | Move window left |
| `Ctrl + Shift + right` | Move window right |
| `Ctrl + Shift + up` | Move window up |
| `Ctrl + up` | Focus up |
| `Super + ²` | Dropdown terminal |
| `Super + Ctrl + B` | Bluetooth picker |
| `Super + Ctrl + C` | Calculator |
| `Super + Ctrl + E` | Emoji picker |
| `Super + Ctrl + K` | Keybinding cheat sheet |
| `Super + Ctrl + Q` | Power menu |
| `Super + Ctrl + SPACE` | Emoji picker |
| `Super + Ctrl + V` | Clipboard history |
| `Super + Ctrl + W` | Wi-Fi picker |
| `Super + M` | Apple Music |
| `Super + Shift + 3` | Screenshot the monitor |
| `Super + Shift + 4` | Screenshot an area or a window |
| `Super + Shift + 5` | Start / stop recording |
| `Super + SPACE` | Launcher |
| `XF86AudioLowerVolume` | Volume down |
| `XF86AudioMicMute` | Mute the microphone |
| `XF86AudioMute` | Mute |
| `XF86AudioNext` | Next track |
| `XF86AudioPause` | Play / pause |
| `XF86AudioPlay` | Play / pause |
| `XF86AudioPrev` | Previous track |
| `XF86AudioRaiseVolume` | Volume up |
| `XF86MonBrightnessDown` | Brightness down |
| `XF86MonBrightnessUp` | Brightness up |

## Claude Code cockpit

Workspace 6 is a cockpit for every Claude Code session on the machine. `claude-sessions` keeps a tmux session, `claude-manager`, with a column of panels on the left and the selected session on the right. Every session runs inside greywall; Claude Code hooks report each session's state to the OSD daemon, which keeps the registry the column reads.

![The cockpit: plan usage, grouped sessions, pull requests and the todo list beside a session](assets/claude-cockpit.png)

![The Todo panel focused: priorities, in-progress items and GitHub issues](assets/claude-cockpit-todo.png)

![A guide window beside the cockpit, one step at a time](assets/claude-guide.png)

The screenshots come from `tools/capture-claude.sh`, which builds a made-up cockpit (`claude-demo` in `apps/cockpit`: invented sessions, todos, pull requests and usage) in a tmux server and a home folder of its own, so no real session, task or cost appears.

### The column

| Key | Panel | What it shows |
| --- | --- | --- |
| `0` | Status | Plan usage per window (5 hours, week, per model), the cost at API prices today, this week, this month and all time, plus separate Claude Status and Codex Status rows with affected products and open incidents, refreshed every two minutes. Claude uses [status.claude.com](https://status.claude.com); Codex uses [status.openai.com](https://status.openai.com), marked OpenAI because it reports provider-wide health, not just Codex. Both pages open from the `:` action menu |
| `1` | Sessions | One card per running session: state, project, branch, title, model, effort (click to change), context use, and badges for `/compact`, an installed update, a needed restart, Remote Control, API credits (`$ credits`), pins, `◆ figcraft` on the sessions of the project the running FigCraft serves, when it answers (the hook checks the connection inside the sandbox at each event; every session has the server, so the others show nothing) and commits not pushed yet (`↑2 unpushed`, counted by the hook inside the sandbox at each start and turn end, cleared by a `U` push; in a dotfiles clone, `↑2 unapplied`: the commits whose change `~/dotfiles` lacks, compared by patch id since `git am` gives an applied commit a new hash, and `↑2 unpushed` beside it: what `~/dotfiles` holds and GitHub lacks, counted by the column in the live checkout every 5 s, which `U` pushes after the apply), and background work still running when its turn ended (`◷ 2 agents + background`: the background agents counted, then shells, monitors or workflows, from the list Claude hands the hook at each turn end). The time spent with Claude on the project (`⏱ 34h`, or `⏱ 1h20 today · 34h`) sits on the card, or on the project's heading when it has several sessions: the gaps under 5 minutes between its sessions' replies, from the cost archive, so past months count too and a session left open overnight does not. The Status panel adds `Time with Claude`, today, this week and this month, all sessions together: their active stretches merged where they overlap, so three sessions through the same hour count one hour, never the sum of the projects' times. Once a turn ends, the card says what it left: commits, uncommitted changes, whether its last test run passed or failed, and a todo item closed (worked out by the hook inside the sandbox, from git and the turn's transcript). A repository's sessions sit under one heading |
| `2` | Pull requests | The pull requests of the sessions' repositories, with CI and review state; `c` starts a review session in a separate clone |
| `3` | Todo | Typed items and the open GitHub issues of the viewed session's project, by priority (`+` up to `!!!!`), in tabs (`[` `]`): Open (Proposed, To do and In progress together) · Proposed · To do · In progress · Done. An item has a title and notes, written in your editor in a popup (`t` adds, `e` edits); Enter shows its details, `h` hands it to Claude (the project's running session, or a new one), `N` to a new session in a parallel clone of its repository, on a branch named after it (`todo-<id>`, `issue-<n>`), never to a running one. A new session started this way (`N`, or `h` when none runs) starts out of sight as a card of its own: the session on screen and the Todo panel's project stay as they were. Claude updates the list itself with `claude-todo`, and proposes ideas (`claude-todo propose`, with why) that wait in Proposed for `y` (approve) or `n` (decline; not proposed again). `Q` queues an item: a session of its project idle at its prompt for 5 minutes takes the first queued one by priority, one at a time (never the session on screen). In a project designed in FigCraft, sessions declare their role (`claude-design role designer|developer`, shown `✎ designer` / `⌨ developer` on the card); a gap a developer finds in the design is filed with `--design` (`✎ design` on the row), and `h` or the queue hand it to the designer session, starting one with the `figcraft-design` skill when none runs. A session of any project that the setup blocks (greywall, claude-sessions, the launcher…) files `claude-todo blocked '<what>' --why '<what stopped it>'`: the report lands in the dotfiles list (`⛔ <project>`), queued for the dotfiles session, which gets it framed as data from another session (the same open report once, 3 open per project at most) |

A session started with `claude` and `t` ("this terminal") still gets a card (badge `▭ terminal`) you can open and message, but `⌘R`, the reboot restore, the Todo queue and the error hand-off leave it alone. `ask-to-run` uses the same per-command approval as managed sessions; `ask-to-guide` remains unavailable, so Claude gives you the steps instead. An approval pane shows only under the session that asked, and follows you back to it. Requests from several sessions (a project's parallel clones too) do not wait for each other's commands: one approval asks you at a time, and once you approve a command it runs on in its pane while the next request shows; a command that then asks for input (a sudo password) comes back to you next. When `ask-to-run` cannot reach you, it says why and what fixes it, and `ask-to-run --check` at each session start tells the session before it needs it.

Above the cards, the Sessions panel warns about requests greyproxy is holding, sessions to bring back after a reboot (`B`), a Claude incident, and sessions started before a change to CLAUDE.md, the settings, plugins, hooks or the sandbox (`⌘R` restarts the ones waiting at their prompt).

Sessions keys: Enter opens a session, `n` peeks at the next one waiting for you, `a` starts one in a project, `A` / `+` another one on the same repository in its own clone, `r` restarts in place, `e` changes the effort, `$` bills the session to the Console's API credits or back to the subscription (it restarts on the same conversation; restarts and the reboot restore keep it; the key comes from the keyring, `make agent-anthropic-key`, and stays a greyproxy placeholder in the sandbox; with no key the session does not start), `x` exits, `R` toggles Remote Control, `p` pins, `g` opens lazygit (inside greywall: review and commit, but no push, since the sandbox has no SSH key), `U` pushes the session's branch to its GitHub origin with your key after a yes (hooks off, settings that could run a program reset); on a dotfiles clone it exports, inside greywall, the commits `~/dotfiles` lacks and runs `apply-dotfiles` on them, `C` sends `/compact`, `z` zooms, `Y` / `N` answer a permission request, `m` sends a message (while the session works it waits, `✉ N queued` on the card, and goes out when the turn ends, one per turn; Alt+Q inside a session does the same, `M` drops the queue), `f` keeps only the sessions waiting for you. Everywhere: `O` starts an opencode session in a project (also a bare `opencode` in a terminal, which asks like `claude`; `claude-sessions --agent opencode new|parallel`): its card wears `◇ opencode` and opens, restarts, exits, comes back after a reboot and takes messages like a Claude one, while `e`, `$`, `C` and the `:` list's Remote Control say they are Claude Code's; `H` resumes a past session (searchable), `/` filters, `:` lists every action, `D` shows the daily digest, `S` the settings (on or off, also `claude-sessions setting`: the phone on lock, the todo queue, errors handed to dotfiles, freezing sessions while gaming, the idle lock on workspace 6, the daily digest; and under Phone, the phone page: whether its server answers and the tailnet publishes it, Pair a phone, and the paired phones with when each was last seen, `x` forgets one after a yes), `K` lists the clones of `A`, `+` and reviews and deletes, after a yes, those with no session, no uncommitted change and nothing unpushed (checked by git inside greywall), `P` `W` `G` open the pull requests, Claude on the web and greyproxy's dashboard, `?` the help.

Moving around (⌘ is the physical Command key, which keyd turns into Ctrl): ⌘Tab between the column and the session, ⌘1/2/3 to a panel from inside the session, `l` or → into the session, and ← back to the column from the start of Claude's prompt (an empty prompt, or the cursor before the first character). Anywhere else in the session the arrows are Claude's, to move in your text. The side with the keys is the lit one: the session's title line turns into a filled bar, or the column's focused panel keeps its blue border (it greys while you are in the session), and the divider between them is orange on the focused side. The other pane fades, colours included: the column fades itself, and the session goes through `claude-fade`, a relay between the pane and its nested tmux client that adds dim to every colour while the focus is elsewhere.

**New-session defaults:** `S` chooses Claude Code or OpenCode as the default agent (initially Claude), plus OpenCode's model and effort. `a`, parallel sessions, reviews, new Todo handoffs, and manager commands without `--agent` use that default. Existing sessions keep their agent and model; an explicit `--agent`, the `O` key, bare `claude`/`opencode` terminal commands, and the `:` action **new-with-agent** still override it.

The model and effort pickers load the active project's available OpenCode models inside its sandbox, without starting the shared service. Press Enter again after loading; with no active project, pick one for the catalog. Effort lists only the selected model's supported variants. Changing the model clears the old effort; **OpenCode default** clears an override. On either Settings row, `d` clears it without needing a working catalog, including when the saved model is unavailable. Choices persist in `~/.local/state/claude-sessions/settings.json`, alongside the existing toggles (`claude-sessions setting default-agent claude|opencode`, `setting opencode-model <provider/model>`, `setting opencode-effort <variant>`; an empty value clears the model or effort).

### Picking work back up

The searchable `:` menu has **return-note** and **clear-return-note** for the
selected session. A note stays in the session's selected/full details across
cockpit restarts and view switches; it is never sent to the agent. Notes accept
up to 280 characters. Empty input cancels; clearing is a separate action.
An unreadable continuity file produces a persistent warning and is preserved,
not silently replaced. These actions do not change existing keyboard shortcuts.

**updates** lists unseen latest-turn results and explicit questions, plus saved
return notes (including exited sessions). Enter opens the chosen update's
actions: mark that displayed update reviewed, or open its live session when
there is one. **reviewed** acknowledges the selected session's displayed
updates; viewing or focusing alone never clears them, and acknowledgement is
not an answer or approval. A turn can have both a result and a question. The
summary counts unseen updates separately from sessions needing input. This is
latest-result tracking, not a lossless activity log. Older sessions show that
result tracking is unavailable until restarted on the new reporting hook.

**queue** explains the viewed project's queued items (all known projects when
none is viewed), in dispatch priority order and with designer/developer routing.
It names the reason a session cannot take work, including your pending manual
messages, questions, approved commands still running, the viewed-session
exclusion, and the five-minute idle window. Eligibility is checked every 30
seconds, not an exact promised start time. The view never dispatches work or
fetches GitHub data; unavailable issue data is labelled explicitly. `Q` still
queues or unqueues a To do item.

**task-trail** works from Sessions, Pull requests and Todo. It connects a task
to every recorded session, unambiguous observed results, and exact
repository/head-branch PR matches. Enter lists navigation actions; opening a PR
never merges it. Multiple matches remain choices rather than a guessed owner.
When Todo retention or a closed/deleted issue removes the source, its link opens
archived details instead of recreating work. Missing or stale GitHub data is not
presented as proof that there is no PR or that a vanished PR was merged.

New handoffs ask the agent to run `claude-todo start <id>` before working so a
background session can identify its assignment. Until that update (or an exact
task branch with a known repository identity), the trail says “Starting;
awaiting assignment”. A result arriving at the same time as a changed assignment
is retained as a session update, not guessed onto either task. Task trails do
not copy full transcripts or silently restart archived conversations.

**defer-20m** and **bring-back** defer or restore a selected explicit question or
PR requiring review. A deferred item stays visible and counted; `n` and the
**attention-filter** action skip deferred questions until their twenty-minute timer expires. A different
question, a new PR head commit or a changed review decision invalidates the old
timer. Deferrals survive cockpit restarts; overdue reminders return on launch.
Approvals, permission dialogs, password/command input and unknown states cannot
be deferred. Neither deferral nor acknowledgement answers anything, and a
deferred question still blocks automatic queued work.

Deferral affects **cockpit navigation only**: it does not change desktop banners,
phone notifications or digests, and it promises no background notification while
the cockpit is closed. PR deferral requires a known head revision. Existing keys
keep their meaning: `f` remains unbound following the earlier key audit; use
**attention-filter** in `:` instead.

### Out of the sandbox, with my approval

| Command (inside a session) | What happens outside |
| --- | --- |
| `ask-to-run --why '…' '<command>'` | An approval terminal shows the command and Claude's explanation, in a pane under the session in the cockpit and in a floating window elsewhere. Nothing runs without "Run it"; the output then goes back to Claude, with tokens, keys, passwords and credentials replaced by `[redacted]` first. Hidden characters are shown escaped |
| `ask-to-guide` | Steps I do by hand (a browser, a console) in a floating window, one at a time, with links to open, values to copy and questions to answer; the answers, or the step I got stuck on, go back to Claude |
| `claude-todo start/done/add` | Updates the Todo panel |
| `claude-needs-input '<question>'` | Marks the session as needing your answer, with the question on its card. Claude and OpenCode keep it through tool completion, turn end and idle notifications; your next message clears it |
| `craft-open figcraft [--session <id>]` (through `ask-to-run`) | Starts an independent FigCraft per checkout (parallel clones included), with confined file access, private preferences/recovery storage, and a permanently reserved control port. Existing windows stay open. The launcher forwards only that checkout's verified port and connects its `figcraft-app` MCP server to it; occupied ports and foreign stale forwards fail closed. Reservations live in host-only `~/.config/greywall-agents/figcraft` and must not be recycled while old sandboxes exist. When connecting an agent, always pass `--session <id>`, even if its tools are listed: it restarts the session once its turn ends, then tells it to carry on. Without it, existing sessions keep their old forwards and MCP connections. Tell a session "connect to FigCraft" |

TODO editing survives cockpit updates: `claude-sessions reload` waits until the editor closes and the item is saved. Editor buffers and their original project are kept in `~/.local/state/claude-sessions/todo-drafts/` until saving succeeds; interrupted drafts are recovered when the cockpit restarts (or once a still-open editor closes). Save or editor errors leave the draft intact and show a message instead of claiming success. Older untracked runtime drafts are left untouched because they have no project metadata.

Agents must signal blocking chat questions with `claude-needs-input` immediately before ending their reply, not optional follow-up offers. Questions must be a nonblank single line of at most 300 characters, without control characters or tmux format sequences. This does not replace approval or guide requests. After installing this command and rule, restart existing sessions so their sandbox can read the command and their agent loads the updated instructions.

Files the sandbox cannot write (CLAUDE.md, hooks, greywall, fish, systemd) go through `.handoff/` and `make handoff-install`, which shows each diff and asks first.

Nothing outside the sandbox runs git on a session's folder, since a session can write that repository's configuration: the hook works out what a turn changed inside the sandbox, the daemon reads project names from the `.git` files, `g` opens lazygit inside greywall, `A` and `+` switch a clone's branch inside greywall, `K` checks clones inside greywall, and a pull request review (`c`) always gets a fresh clone, so its fetch (which needs your credentials) never runs in a folder a session had.

What sessions can and cannot do to the machine (audits of 2026-10-18 and 2026-10-19):

- They report their state on a socket (`osd-report.socket`, in `$XDG_RUNTIME_DIR/claude-report/`, a folder each sandbox mounts whole so a socket made again reaches running sessions too): the daemon asks the kernel which process sent a report and takes the tmux pane from its first ancestor outside the sandbox, so no session can claim another's pane or card. Run and guide requests go on that socket too, whole: the daemon keeps its own copy in the runtime directory at once, with the folder the sender's sandbox started in, and the approval and guide windows read only that copy (no session can rewrite another's request; a command for a folder outside the session's is flagged in red). Nothing of a sandbox's is taken on `osd.fifo` any more; banners, focus, clearing the column and approval phases only come through `osd-host.fifo`, which no sandbox can open.
- `~/.ssh/known_hosts` is read-only for sessions.
- Theo's answers (run results, guide progress) go to `$XDG_RUNTIME_DIR/claude-answers/<project>`, written only outside the sandbox: each sandbox has its own project's folder mounted read-only and sees no other, so no session forges an approved output or reads another project's.
- Both launchers, `greywall-agents/claude` and `greywall-agents/opencode`, source `launcher-common.sh`: an opencode session gets the same guards, answers folder, tokens, CA bundle, Docker and forwards as a Claude one, in its own greywall profile, and never the Console's API key. Its Anthropic API key comes from the keyring (`make agent-opencode-key`) and stays a greyproxy placeholder in the sandbox, since opencode's own credential store (`opencode.db`) is open to every opencode session; without one it uses `opencode auth login`. Its replies join Claude's in the usage archive (cost at API prices from their tokens, time spent per project), read-only from `~/.local/share/opencode/opencode.db`. The cockpit starts it with `--standalone` (a server of its own, not opencode 2's shared background one) and a session id it picks, exported as `CLAUDE_CODE_SESSION_ID`. Its plugin (`config/opencode/.config/opencode/plugins/cockpit.ts`, opencode 2's `{id, setup}` plugin API) sends its events through Claude's hook, `session-state.sh`, as the payloads Claude Code sends, with `agent: opencode`: the daemon keeps an opencode session alive while an `opencode` process runs in its pane, and reads its state from those reports only, never from Claude's screen patterns.
- Background build tools (`cargo`, `make`, `npm`, `pnpm`, `go`, `composer`, `pytest`) resolve through CPU-guard shims in the launchers' existing read-only cache. They and their children share the first half of the available logical CPUs (minimum one) at nice +10 or lower priority; the cockpit and interactive agents remain unrestricted. On a 16-thread machine, all guarded builds together can use eight threads, not eight per session. Nested tools keep the same budget through `AGENT_BUILD_CPUS`. `make cockpit` and `make test` also guard host Cargo builds. Other tools can use `agent-build COMMAND [ARGUMENTS...]`. This trades build speed for responsiveness, not memory limits; it is a scheduling safeguard, not sandbox enforcement. Absolute tool paths and work in separate Docker daemons bypass it. Launcher setup affects new sessions; already-running shells use the shims only if their cache directory is visible and first on PATH.
- `~/.claude/projects/<folder>` of every other project (memory, transcripts) is read-only in a session: the launcher writes a settings file of its own per project and agent (`$XDG_RUNTIME_DIR/greywall-agents`), greywall.json plus those folders in `denyWrite`; an opencode session's drops greywall.json's `denyRead` entries for opencode's own data and config, which hide them from Claude sessions. Memory found for a fresh parallel clone is set aside.
- `.git/config` and `.git/hooks` are read-only in the folder a session starts in; every host git call in claude-sessions resets the settings that run a program, and a push redirected by a push URL or URL rewrite is refused.
- A session's folder (and so where a restart resumes it, with which project's token) is the folder its sandbox started in, read from `/proc`, never the one a report claims.
- Commands are shown as they run: run through `sh` exactly as quoted, in the folder resolved before Theo answers, with hidden characters, blank-line padding and non-ASCII letters flagged; output goes back with secrets redacted.
- Claude Code updates only from outside the sandbox (`claude-update.timer`); sessions cannot write the versions folder.
- `make system` and `make handoff-install` show every file (new ones whole, control characters visible) and refuse setuid bits and symlinked destinations.
- Todo items lose control characters, show "by Claude" when a session filed them, and go to sessions by folder, never by project name; the queue waits while Theo games.
- `claude-todo` changes go on the report socket too: the daemon queues them in the runtime directory with the folder the sender's sandbox started in, and that folder's project is the only one an update touches (an item lands there, a status changes only on that project's items and issues). Blocked reports: 3 open and 5 a day per project; at most 200 updates per apply, and every refusal is logged.
- What the column and the daemon read from folders a sandbox writes (transcripts, history, status lines, design links, `.git` files, credentials) goes through `util::read_untrusted`: no final symlink, no FIFO or device, a size cap. Report connections are read off the daemon's main thread within 2 s and 1 MiB, 16 at a time; a sandbox folder has at most 3 requests waiting.
- An audit log, apart from the debug log: `~/.local/state/claude-audit/audit-YYYY-MM-DD.jsonl` (0600, hidden from every sandbox) and journald (`journalctl -t claude-audit`). One JSON line per security event: requests held or refused (with the folder the sender's sandbox started in), approvals shown, declined, approved and run, guide steps, todo updates applied or refused, blocked reports, refused or corrected reports, a session's first report, messages said and prompts queued or sent, error hand-offs, greyproxy holds, refused folders, dotfiles applied, pushes, session starts, exits, restarts and hand-offs. Texts a session chose are kept as `{sha256, len}`; each approved command whole in `runs/<id>.json`. Each line names the hash of the one before it: `claude-audit verify` says whether lines were changed or removed. Gzipped after 7 days, deleted after 90. The debug log (`claude-sessions logs`) keeps no control character and no successful `label-state` or `focus-viewer`.

Dotfiles changes (audit of 2026-10-19, `docs/security-audit-2026-10-19.md`): no session runs in `~/dotfiles`, the live checkout stowed into `~/.local/bin` and `~/.config` (the launcher refuses it). Claude's dotfiles session works in the clone `~/IdeaProjects/dotfiles` (`make dotfiles-work` creates it), exports its commits with `git format-patch origin/main -o .outgoing`, and `claude-sessions apply-dotfiles` (through `ask-to-run`; `apply-dotfiles <clone>` for a parallel clone, `<clones>/dotfiles/<name>`, which reads `~/dotfiles` too) shows them with control characters visible and applies them to `~/dotfiles` with `git am` after a yes. No host git runs in the clone. Still open from that audit: every session can write `~/.claude.json`, the memory folders and the shared todo updates (fixes planned in the audit's order). Also accepted (2026-10-19): `jobpilot-apply.timer` runs `~/IdeaProjects/jobpilot` code hourly with my rights, code that a Claude session working in jobpilot can edit.

### Linked sessions (Designer, Developer, QA)

Sessions of one repository (parallel clones included) that you link to one task message each other without you relaying. Any of Claude Code, OpenCode and native Codex CLI can take any role.

- **Linking**: `:` → `team` opens the Team view. `n` creates a request from verified live sessions (task id, then a Designer, a Developer and a QA, any of them left out); an agent can submit a prefilled one with `cockpit-team propose`. Every request stays pending until you approve its exact task id, session ids and roles (`a`, after a yes); `e` edits (a new revision, approved again), `x` rejects, `u` unlinks a member at once and cancels its pending messages and retries. A session belongs to one team at a time.
- **Delivery**: each app takes its own messages at a moment it considers safe — OpenCode through its native inbox (queued behind the turn, as a labelled non-user message), Claude and Codex when a turn ends (their Stop hook continues with the message; Claude's channels stay off, they are a research preview Anthropic has not enabled here). A session waiting for you is never woken. Messages are kept in `~/.local/state/claude-audit/teams.sqlite` (hidden from every sandbox), deduplicated across restarts, retried at most three times (2, 10, 30 s), and never resent when a delivery may have arrived without a receipt: the Team view asks you to mark it handled or cancel it.
- **Clarifications** are threads of at most three request/reply rounds; the asker marks each reply resolved or not, and an unresolved third round (or a fourth request) escalates to you once (`s` resolves it with a note).
- **Handoffs**: the Developer's `cockpit-team handoff` (branch, full commit id, checkout, summary, test outcome, FigCraft frames) requests QA once; a newer handoff cancels the unsent review of the older one. QA reports findings to the Developer or Designer and does not change product code.
- **Authority**: a team message is collaborator content, labelled as such. It never approves tool use, scope changes, installs, permissions, Git writes, merges or releases; those still need you, as before. Sessions reach the team service only through a socket hard-linked into their own project's read-only answers folder; the daemon identifies the sender from the kernel and its launcher, never from what it claims. Your controls go through a second socket no sandbox can reach.
- The cards show `team <role> · <task>`, and the Sessions summary says when the team needs you. Not yet: waking a Claude or Codex session that is already idle (it gets messages when its next turn ends).

### Native Codex CLI

`claude-sessions --agent codex new|parallel|background` starts Codex's own terminal UI inside greywall (`greywall-agents/codex`, the same project guards as the other launchers, its own profile, `~/.codex` hidden from other agents and theirs from it, `--no-daemon`). Its hooks (`config/codex/.codex/hooks.json`, reviewed and trusted once in Codex's `/hooks`) report its state through `cockpit-codex-hook`, never answering a permission request. Its card carries `cdx_<sha256 of the thread id>`, so its ids never collide with Claude's, and a restart resumes that exact thread, never `--last`. Effort, billing, `/compact`, Remote Control and typed messages are refused by name.

### Around it

- **Banners** at the top of the screen when a session needs me or finishes, held during a meeting (khal) or a game.
- **A loading screen** over the session pane while a session restarts, changes effort or starts.
- **Error hand-off**: an error in the claude-sessions log goes to a dotfiles session to be fixed, once a day per error and three a day at most.
- **Phone** (`claude-remote`, `claude-remote.service`): a page for the phone, from outside the home, over Tailscale only: `tailscale serve` publishes it with HTTPS on the tailnet, and the server listens on 127.0.0.1:7380 and refuses any other address. `claude-remote pair`, or the cockpit's `S` (Phone, Pair a phone) or `pair-phone` in its `:` list, shows a one-time link and its QR code (10 minutes), and a 6-digit code for the Home Screen app (2 minutes, 5 tries; a scanned link always opens in Safari, which keeps cookies apart from that app: its pairing screen takes the digits) ( in the cockpit, drawn over the column, which says `iPhone is paired ✓` once the phone opened it); the phone names itself from its browser (iPhone, iPad, Android); opening it gives the phone's browser a long secret (a cookie: Secure, HttpOnly, SameSite=Strict), `claude-remote pair iPhone` names the phone (`Phone 2` otherwise), `claude-remote phones` lists them with when each was last seen, `claude-remote unpair <id>` forgets one and `claude-remote unpair` every phone, and only the SHA-256 of codes and secrets is kept, in `~/.local/state/claude-remote` (hidden from every sandbox). The page shows the session cards and a session's screen, live and in colour (every 1.5 s, following its newest lines unless you scroll up; of the terminal's escapes only the colour ones reach the phone), interrupts a session that works (Escape, after a second tap), turns its Remote Control on or off (`claude-sessions remote`), sends a message (queued while it works), answers a permission dialog (Yes only while its cursor is on "1. Yes", read from the pane right before), and runs or declines an `ask-to-run` asking now: the same command as the approval window (from its `<id>.asking`, host-only), a second tap to run, refused if the command changed. Tabs at the bottom: Sessions (with `+ New session`: a project, a first message, in its own clone on a `phone-<time>` branch or in the folder itself, started out of sight like the column's `+`; a waiting card shows what its turn left), Todo (the open items per project; add one, sent through the same update queue as `claude-todo` so the column stays the only writer of `todo.json`; Start in a new session is the column's `N`: a clone on `todo-<id>`, then in progress), Pull requests (gh, the repositories sessions run in or the review requests, kept 5 minutes unless refreshed; checks, review, a link to GitHub) and Usage (the plan's limits from the column's cache, the cost estimate and the time with Claude, read from its archive). Notifications (Web Push): the page's This phone section turns them on, sends a test or turns them off; on an iPhone only the page added to the Home Screen gets them, and since that app keeps cookies of its own, the paired page in Safari gives a one-time code (`Get a code for the Home Screen app`, copied) that the app's paste box takes, pairing it as `iPhone (Home Screen)`. While the computer is locked (`phone-on-lock`), every 3 s the server looks for a session that needs you, an `ask-to-run` that asks, or the end of any session's turn; at the desk only the turn ends of the sessions the phone watches (`Tell me when it finishes`, on its screen: "proj-a finished: 2 commits, tests pass"), and pushes the new ones (`proj-a needs you: Bash make deploy`; a tap opens the session), encrypted for the phone's browser alone (RFC 8291) and signed with the server's own key (VAPID, `vapid.json` in the same hidden folder); only Apple's, Google's, Mozilla's and Microsoft's push services are accepted as endpoints, and a subscription the service forgot is dropped. The page's own files load without a phone (they hold no data); everything under `/api` needs one. Four workers answer, so a slow pane read does not hold the other requests; `/health` answers `ok` without a phone. Every POST needs the page's own header, and everything the phone does is in the audit log.
- **Idle**: workspace 6 locks after 10 minutes, after a full-screen "Still there?" prompt; while locked, the phone page notifies (`claude-sessions away on`, from `lock.sh`; the lock itself first, every time: `Your sessions are on the phone`, with how many need you (by project), wait for a prompt, work and ask to run; nothing while the setting `phone-on-lock` is off), and it stays quiet once unlocked; `claude-sessions handoff` (Super+Shift+C) still hands every session to Claude's Remote Control by hand.
- **Web windows** for pull requests, Claude on the web and greyproxy, one tab each.
- **Design reviews**: after consent, the FigCraft agent requests `claude-sessions design-review start <session-id>` through `ask-to-run`. A small, movable Kitty window on FigCraft's workspace shows **only that conversation's existing CLI**, with reading and replies: no session list, TODO panel, second agent, or resumed conversation. Claude and OpenCode both use their existing tmux session. The cockpit, original CLI window, and FigCraft keep their desktop layout. Without an ID, start uses the cockpit's current conversation; agents should pass their own ID explicitly. Live process ownership of the pane and tmux server is verified before launch; stale or unverifiable targets are refused. With several FigCraft windows, use `start <session-id> --figcraft-window <address>` with the intended window's address from the host's `hyprctl -j clients`; this survives the approval terminal taking focus. Ambiguity is refused rather than choosing another project. Repeated start reuses the float. Close it normally or request `claude-sessions design-review stop [session-id]`: only the floating client closes, not the agent; without an ID, stop refuses if several chats are open. Initial size is 560 logical pixels wide and 85% of the monitor's usable height, clamped to fit. The two clients share a terminal: the CLI text may reflow to the last-used client's size under the existing tmux policy. No document, zoom, MCP or sandbox permissions change, and no layout restoration is needed.
- **Waybar** shows the sessions and the plan usage.
- **Health**: `claude-sessions doctor` checks every piece, `claude-sessions logs` shows the log, and a weekly timer runs the checks.

### Codex quota sign-in

Codex quota uses a separate ChatGPT sign-in, not OpenCode's account or your usual Codex CLI account. After applying the dotfiles, run on the host:

```bash
bash ~/.config/waybar/scripts/codex-usage.sh --login
```

Open the sign-in URL it prints and complete the browser login. No OpenCode or session restart is needed; a successful login immediately refreshes the quota cache. If access is revoked, run the same command again.

The reader needs Python 3 and the official Codex CLI at `~/.local/npm/bin/codex` with the [app-server account interface](https://developers.openai.com/codex/app-server/). It runs a short-lived stdio app-server on the host, outside the agent sandbox, without starting an agent conversation or executing model-generated commands. Codex manages token storage and refresh in a private `codex-quota` directory under `${XDG_DATA_HOME:-~/.local/share}`; treat its `auth.json` as a password and never commit or share it. The reader never opens that file itself.

Polling reuses a successful runtime cache for four minutes. Missing CLI, sign-in or network access leaves the last successful cache unchanged and records an error, rather than switching accounts or presenting failed reads as fresh usage. Check the installed vendor CLI if it is missing or incompatible; `--force` retries a quota read without waiting for the cache to expire.

### Where it lives

| Path | Role |
| --- | --- |
| [`config/bin/.local/bin/claude-sessions`](config/bin/.local/bin/claude-sessions) | The manager: tmux layout, start, resume, restart, logs, doctor |
| [`apps/cockpit`](apps/cockpit) | The Rust programs, built with `make cockpit`: the column (Ratatui), the approval terminal, the guide window and the sandbox commands; each name in `config/bin/.local/bin` is a two-line wrapper that runs its build |
| [`apps/desktop`](apps/desktop) | The GTK programs: the OSD daemon (banners, the session registry, the run and guide queues), the now-playing card, the idle screen, the web placeholder |
| [`config/claude/.claude/hooks`](config/claude/.claude/hooks) | `session-state.sh` reports states; `agent-environment.sh` tells a session what its sandbox has |
| [`config/bin/.local/bin`](config/bin/.local/bin) | `ask-to-run`, `claude-run-approve`, `ask-to-guide`, `claude-guide`, `claude-todo`, `claude-design`: wrappers of the Rust builds |
| [`config/tmux/.config/tmux/tmux.conf`](config/tmux/.config/tmux/tmux.conf) | The cockpit's keys |
| [`config/hyprland/.config/hypr/apps/claude.lua`](config/hyprland/.config/hypr/apps/claude.lua), [`scratchpads.lua`](config/hyprland/.config/hypr/scratchpads.lua) | Workspace 6, window rules, the web windows |
| [`config/systemd/.config/systemd/user`](config/systemd/.config/systemd/user) | `osd`, `claude-sessions`, the health check, the version link |
| [`apps/cockpit/tests`](apps/cockpit/tests), [`apps/desktop/src/tests.rs`](apps/desktop/src/tests.rs) | `make test`: the scripts, hooks and programs through fake commands on PATH, and the daemon's registry |

## Games

A game window opening turns **game mode** on by itself (`config/hyprland/.config/hypr/game-mode.lua`), and the game's exit turns it off:

- Hyprland drops animations, blur and rounding; banners and approval windows wait.
- Feral GameMode (`config/gamemode`) puts the CPU governor and the GPU (`card1`) in their performance modes while the game's process runs.
- `game-performance` (GameMode's start and end hooks) pauses every agent sandbox, Claude Code, opencode or codex (cgroup freezer on the scope of each `greywall` process, a tmux pane's or a kitty window's: they resume mid-step after the game; the column and approval panes stay usable), the backup, jobpilot and the update, security and health checks, and drops any job already running to idle CPU and disk priority. The Todo queue waits too. `game-performance off` thaws by hand; the cockpit doctor flags sessions left frozen.
- MangoHud (`config/MangoHud`): FPS, frame times, CPU and GPU load and temperatures, with `Left Shift + F12`. Only Steam and its games load it (`config/steam` sets `MANGOHUD=1` in Steam's launcher).

## Machine monitor (vitals)

`vitals` ([`apps/vitals`](apps/vitals)) watches the machine so a cooling or hardware problem shows up before it does damage. `Ctrl + Cmd + M` (or the bar's chip) opens its TUI, in French: **Aperçu** (the machine in plain words, then every temperature with its 24 h range and limit, the pump, fans, load), **Refroidissement** (CPU and GPU over 90 days, and the idle trend), **Disques** (SMART health, space), **Système** (failed services, kernel errors, memory, the backup), **Entretien** (upkeep on a schedule), **Jeux** (each game session with its peaks: CPU, GPU, hotspot, VRAM, GPU power, minutes at a limit) and **Journal** (crashes, hard stops, programs killed for memory, GPU hangs, and the alerts that ended, over 30 days, each tagged with the game running at the time). Aperçu also lists the GPU's power, clock and video memory use.

Aperçu opens with a headline ("Ton PC va bien : rien à faire.") and one sentence each for cooling, disk, system, backup, upkeep and stability (the last 7 days of the Journal), saying what to do when something drifts: a CPU idling 5 °C warmer than two weeks ago means dusting the filters and the radiator, then checking the pump.

| Upkeep (Entretien tab) | Every |
| --- | --- |
| Dépoussiérer les filtres et les ventilateurs | 3 months |
| Nettoyer le radiateur | 6 months |
| Vérifier le watercooling (pump noise, tubes, joints) | 6 months |
| Refaire la pâte thermique du CPU | 3 years |
| Tester la restauration d'une sauvegarde | 6 months |
| Chercher une mise à jour du BIOS | 6 months |

`d` records a task done today, `b` moves its date a week earlier (for one done a while ago). A task past its interval raises a banner and the bar's chip like any problem; one never recorded only asks for its date.

| Check | Warns when |
| --- | --- |
| Limits | A temperature stays at its limit for 3 minutes (the driver's, else 90 °C for the CPU, 85 °C otherwise); critical at the driver's critical limit |
| Idle trend | The idle temperature (10th percentile of idle minutes) of the last two weeks is 5 °C above the two before: dust, the pump, the paste |
| Load trend | The temperature under the same load (≥ 70 %) is 5 °C above three weeks earlier |
| Fans | A fan or pump that spun this week reads 0 RPM while its chip is above 60 °C |
| Pump | The pump's idle speed this week is 15 % under three weeks earlier: it slows down before it stops |
| Disks | A SMART critical warning, spare blocks under their threshold, 80 % wear, new media errors, a file system over 90 %, or one that will be full within 30 days at the pace of the last two weeks |
| System | Failed services, kernel GPU resets, disk I/O errors, machine checks, programs killed for memory or throttling, low memory or memory pressure, no successful backup in two days |

- `vitals-collect.timer` (user, every minute) runs `vitals collect`: one SQLite file in `~/.local/state/vitals`, every reading for 35 days and hourly summaries for good. A new problem raises a banner (held while gaming) and a chip in the bar until it clears.
- `vitals-root.timer` (system, every 10 minutes, in `system/`) runs only `smartctl`, `journalctl` and `coredumpctl` as root and leaves their output in `/var/lib/vitals`: no program from the dotfiles runs as root.
- `vitals-shutdown.service` (user) records the boot when the session starts and again when it stops cleanly; a boot recorded only at the start was a hard stop (a freeze, a power cut, the power button held).
- A game session runs while GameMode has the machine (game-performance's list in `$XDG_RUNTIME_DIR`) or a game window has the focus; its name is the game window's title.
- Pump and fan speeds come from the motherboard's fan chip (ITE IT8696E) through the community `it87` driver (AUR `it87-dkms-git`, loaded at boot by `system/etc/modules-load.d/it87.conf`; the kernel's own `it87` does not know this chip). Header 5 carries the pump; the case fans hang off the cooler, out of its sight.
- `vitals import` loaded the temperature log kept since February (`log-temps.sh`, now retired with `check-temp-trends`).

## Quickstart

### Prerequisites

- A base Arch install with network access, a user in the `wheel` group, and `git` and `base-devel` installed
- [yay](https://github.com/Jguer/yay), which `make` uses for AUR packages:

  ```bash
  git clone https://aur.archlinux.org/yay.git /tmp/yay
  cd /tmp/yay && makepkg -si
  ```

- The secret files that some configs read, which never go in the repo:

  | File | Used by |
  | --- | --- |
  | `~/.secrets/airpods` | `make dotfiles`, to generate the WirePlumber AirPods rule |
  | `~/.secrets/icloud.env` | vdirsyncer, iCloud calendar |
  | `~/.secrets/restic` | Backups: the password of both repositories, see [Backups](#backups) |

  `~/.secrets/airpods` holds a single line: `AIRPODS_MAC="XX:XX:XX:XX:XX:XX"`.

### Install

```bash
git clone https://github.com/Androlax2/dotfiles.git ~/dotfiles
cd ~/dotfiles
make setup
```

`make setup` runs these targets in order, and each one can also be run on its own:

1. `packages` installs the pacman and AUR packages
2. `dotfiles` links every `config/` package into `~`
3. `global-packages` installs the npm, pnpm, pip, cargo, go and composer packages, and the agent skills in `packages/skills.txt`
4. `system` copies `system/` into `/` as root
5. `locale` generates the locales enabled in `/etc/locale.gen`
6. `services` enables the units in `services.txt`
7. `initramfs` rebuilds the initramfs after the `system/etc/mkinitcpio.conf` copy

Then finish the [manual steps](#manual-steps) and reboot.

### Verify

```bash
make check
```

Each section should be empty. Anything listed is drift between the machine and the repo.

## Everyday use

```bash
make update   # upgrade pacman and AUR packages
make update-skills   # upgrade the agent skills installed with the skills CLI
make check    # what changed on this machine that the repo doesn't know about?
make dump     # write those changes into the repo, then review them with git diff
make disk     # where is the disk space going?
make clean    # remove orphans, old package caches, dev tool caches, journal older than 4 weeks
```

`make clean` asks before removing packages. Two cleanups stay separate because they destroy data:

```bash
make clean-docker   # stopped containers, unused images, and every unused volume with its data
make clean-trash    # empty the desktop trash for good
```

### Report-only cache maintenance

Two report-only daily timer pairs were installed and enabled on this PC on
2026-10-20:

- `cache-maintenance-report.timer` (user): inventories only
  `~/IdeaProjects/hartprint/.turbo/cache`. Reports files whose access **and**
  modification timestamps are older than 30 days, with an allocated-byte estimate.
  Totals cover all files; filenames are sampled (up to 20, with the omitted count)
  to keep journal output bounded.
  These are provisional observations, not verified safe deletion candidates or
  guaranteed reclaimable space: Turbo's entry format and coordination with running
  builds still need verification. Symlinks and unexpected subdirectories fail
  visibly; a missing cache reports zero files.
- `package-cache-report.timer` (system): runs the installed `paccache` in dry-run
  mode, keeping at least two versions of every package and everything accessed or
  modified within 30 days.

Both use low CPU/I/O priority, catch up after shutdown, and write to the journal.
Reports can be read under the user
`cache-maintenance-report.service` and system `package-cache-report.service` journal
units. The user script prevents overlapping report runs with a runtime lock.

These reports have **no deletion mode**. Games, project checkouts, dependencies, Docker data,
Rust build output, IDE indexes, other caches, and the existing manual cleanup
commands are unchanged. Do not schedule `make clean` as a replacement.

These scripts and units execute **outside the agent sandbox**. Installation on
another machine, activation changes, and adding deletion behavior require separate
approval. The system report service runs only the vendor executable, not checkout code as
root. Run fixture-only regression tests with
`python3 -m unittest discover -s tools/tests -p 'test_cache_maintenance.py' -v`.

### Broader disk maintenance

Installed and enabled on this PC on **2026-10-20**:

- **Logs:** native journald retention of 30 days, with daily file rotation.
- **Package archives:** native `paccache.timer` runs daily, keeping two versions
  plus anything accessed or modified within 30 days, only in `/var/cache/pacman/pkg`.
- **Vitals:** disk warnings at 85%, critical alerts at 95%, retaining growth forecasts,
  existing SMART checks, weekly TRIM, and gaming-aware notifications.
- **Monthly review:** `disk-space-review.timer` inventories explicitly listed games
  and checkouts. The largest 20 of each are written to
  `${XDG_STATE_HOME:-$HOME/.local/state}/disk-maintenance/latest.json`.
  Vitals announces each new report once. **No games or projects are deleted.**
- **greyproxy.db:** greyproxy keeps every captured request forever, so
  `greyproxy-prune.timer` (user, hourly: it grows several GB a day) deletes traffic history (transactions,
  request logs, middleware events, conversations) older than 14 days, then the
  oldest transactions and conversations until the data fits in 4 GB, in small
  batches next to the running proxy. Rules, pending requests, sessions and
  credentials are never touched. It fails (and logs a WARN) when the data still
  exceeds 4 GB. Freed space returns to disk once the file was compacted:
  `systemctl --user stop greyproxy; greyproxy-prune --compact; systemctl --user start greyproxy`.
  Tests: `python3 -m unittest discover -s tools/tests -p 'test_greyproxy_prune.py' -v`.

In this revision, **Vitals → 3 Disques** opens with one plain-language status and
at most one contextual action. Healthy routine maintenance does not become a to-do
list: reading a report is explicitly optional if you want to free space, including
when an older report is available during a new scan. A first scan already running
needs no further action: you can close Vitals and wait for its notification, with
no redundant action or Enter instruction. Low space, SSD warnings, stale reports,
and actual failures take priority.
For a non-boot space warning, the action reads an existing report or prepares a
missing/invalid one. If the first report is already running, the warning instead
offers Details and explains that its notification is still pending; boot-volume
warnings keep their Details action.
Technical diagnostics, counts, paths, schedules, and manual checks stay under Details.

When an action is offered, use **Enter** to read the space review, prepare a first
or updated read-only review when needed, or understand a warning. Under **d** (Details),
**a** requests a space review and **t** requests the Turbo dry-run report.
Only the two known user report services can be requested: a new
space review or the Turbo dry-run report. They run asynchronously; a queued request
is not shown as completed, an already-running check is not started again, and
failure feedback remains visible, with raw errors in Details. An SSD-critical warning
blocks both manual reports to avoid adding load. There are no prune/delete/admin controls.

The review shows the largest games/projects with readable names and sizes, retaining
project context for parallel checkouts. Incomplete data gets a short warning, not a
list of diagnostic paths or counts. Details preserve actual errors and expected
symlink skips separately; old reports remain explicitly uncertain. Analyzed-file permissions
are never changed automatically; game/project removal remains a separate manual decision. Partial sizes stay
labeled; a large directory is not evidence it is unused.

Use **d** for full SMART/space/maintenance details and **b** or **Backspace** to
return to the short view. Arrows scroll in the review/details, **PageUp/PageDown**
page, and **Home/End** jump. **r** rereads the current state; it does not launch a
new scan. Opening Vitals also starts no scan or cleanup. Long paths wrap on narrow
terminals; the optional action or waiting reassurance and essential footer fit a 40×12 summary.

Details preserve enabled/disabled/unknown state, last/next runs, and real run
results. Journald retention is separate because it has no cleanup timer. Approved
policies are labeled as policies, not an audit of custom commands. Missing history
never becomes fabricated success. The updated screen requires rebuilding the live
Vitals binary from this revision.

The review never follows symlinks or nested mount boundaries. Skipped paths produce
explicit errors and partial-size labels, not zero-byte success. It runs as your
user, has a 15-minute timeout, and never runs a launcher or project script. Git
summaries use local refs and untracked/staged state only: tracked worktree changes,
ignored files, unsupported Git indirection, and active sessions can remain unknown.
Nothing in a report certifies that a checkout is safe to delete or backed up.

Journald expires whole journal files, not records exactly on their 30-day birthday;
its size/free-space limits can shorten history. Its secondary security-audit copy
obeys these limits; the primary audit files keep their existing **90-day** policy.
Turbo stays report-only. Existing manual cleanup commands are unchanged.

Host-executed changes are the journald drop-in, vendor paccache service/timer
drop-ins, package-report command, monthly report script/unit pair, and Vitals
collector. They run **outside the sandbox** after reviewed deployment.
No dependency manifest changed. The Makefile test target builds the real OSD
classifier before testing; `services.txt` records the enabled native system timers.

Activation checklist for another machine or a future reinstall (approval required):

1. Review/apply the clone through the normal dotfiles workflow, including the Vitals
   build and updated user links; review each system-file diff before installation.
2. Confirm merged journald/paccache settings and the fixed cache path. Installing
   overrides can affect an already-active timer, so check its state before install.
3. Reload journald if supported; otherwise request approval for a restart. Do not
   run a one-off vacuum or broad `make clean` to "test" retention.
4. After approval, enable system `paccache.timer` and `package-cache-report.timer`,
   and user `cache-maintenance-report.timer` and `disk-space-review.timer`.
   Keep report services dry-run. Record enabled system timers only as part of
   approved deployment.
5. Verify next runs, actual completion results, report location, and Vitals delivery.
   Game/checkout removal remains a new, explicit approval every time.

Rollback: disable future maintenance first, review removal of only these drop-ins,
and reload the affected configurations. Preserve the review report. Disabling a
timer or reverting configuration **cannot restore expired logs or removed package
archives**. Do not roll back unrelated SMART, TRIM, backup, or audit settings.

Fixture tests: `python3 -m unittest discover -s tools/tests -v` (native paccache
tests prune **temporary fake archives only**). Vitals tests:
`cargo test --offline --manifest-path apps/Cargo.toml -p vitals`.

### Keeping the repo in sync

After installing something or editing a config, run `make check`, then `make dump`, and commit what `git diff` shows.

`make dump` rewrites the package lists, `services.txt` and the files already in `system/`.
It also captures [agent skills](#agent-skills).
It never picks up a new `/etc` file by itself. `make check` lists those instead, and each one needs a decision:

- **Track it** by copying it into `system/` at the same path:

  ```bash
  install -D -m 644 /etc/foo/bar.conf system/etc/foo/bar.conf
  ```

- **Ignore it** by adding a regex and the reason to `system-ignore.txt`.

`.pacnew` files and editor leftovers are never ignored: merge or delete them.

### Adding a program's config

Move its config into a new stow package, then relink:

```bash
mkdir -p config/foo/.config
mv ~/.config/foo config/foo/.config/
make dotfiles
```

Don't stow files that programs rewrite by replacing them, such as `mimeapps.list`: the program replaces the symlink with a plain file.

### Agent skills

Skills for Claude Code and the other coding agents come from three places, and only the first two are versioned:

| Kind | Lives in | Versioned as | Restored by |
| --- | --- | --- | --- |
| Installed with the [skills CLI](https://github.com/vercel-labs/skills) | `~/.agents/skills`, linked into `~/.claude/skills` | One `owner/repo skill-name` line in [`packages/skills.txt`](packages/skills.txt) | `make global-packages` |
| Written by hand | `~/.claude/skills` | The files themselves, in [`config/claude/.claude/skills`](config/claude/.claude/skills) | `make dotfiles` |
| Synced from the Claude account | `~/.claude/skills/synced` | Not versioned | Claude Code, on login |

Install a skill from a repository, then record it:

```bash
npx skills add owner/repo --skill skill-name -g -a claude-code
make dump
```

`-a claude-code` is enough: the other agents read `~/.agents/skills` directly, and Claude Code is the only one that needs links elsewhere.

For a skill written by hand, create it in `~/.claude/skills/<name>` and run `make dump`: it moves the folder into `config/claude` and links it back, so later edits land in the repo.

`make check` reports both kinds: CLI skills under `packages`, hand-written ones under their own section.

The list records where each skill comes from, not its version. A fresh machine gets the latest version of each, and `make update-skills` upgrades them here. It is separate from `make update` on purpose: skills are instructions agents act on, pulled unpinned from many repositories.

## Backups

```text
PC ── restic over SFTP, daily ──> NAS share "restic"          (btrfs snapshots, immutable for 7 days)
PC ── restic over SFTP, daily ──> Hetzner Storage Box         (Storage Box snapshots)
NAS ── its own data, nightly, from the homelab repo ──> Hetzner Storage Box
```

- **Two independent copies of the PC.** The NAS and Hetzner each hold a separate restic repository, so losing either one, or the NAS with the house, still leaves a backup.
- **The NAS backs up its own data.** Photos, apps and databases leave the NAS through the [homelab repository](https://github.com/Androlax2/homelab#backups), not from this machine.
- **Encrypted on the PC.** restic encrypts and deduplicates before upload: the NAS and Hetzner only store ciphertext.
- **Everything in `~` is backed up** except the caches, game installs, programs and build output listed in [`excludes.txt`](config/restic/.config/restic/excludes.txt). `make backup-list` shows the size of every included folder, and `make backup-list DEPTH=2` shows fewer levels.
- **One script does the work.** [`backup.sh`](config/restic/.config/restic/backup.sh) runs behind the `make backup*` targets and both timers.

| Job | When | Keeps |
| --- | --- | --- |
| Backup to both, `restic-backup.timer` | 15 minutes after boot, then daily | Every run until maintenance |
| Maintenance on both, `restic-maintenance.timer` | Sundays at 14:00, or at the next boot if missed | 7 daily, 4 weekly and 12 monthly snapshots |
| NAS snapshots of the `restic` share | Daily | 14 days, immutable for 7 |
| Storage Box snapshots | Automatic, set in Hetzner Console | Up to 10 |

A target that can't be reached, away from home or offline, is skipped quietly while the other still runs. After 3 days without a backup to a target you get a notification, and any failure sends a critical one.

> [!CAUTION]
> The backups can't be decrypted without `~/.secrets/restic`. Keep a copy outside the NAS, for example on paper stored away from home: Bitwarden here is the Vaultwarden instance running on the NAS, so it disappears with it.

### First-time setup

On the NAS, in DSM:

1. In Control Panel, open Shared Folder and create `restic` on Volume 1, with data checksum on and read/write access for your account.
2. In Control Panel, open File Services, then FTP, and enable SFTP.
3. In Snapshot Replication, schedule daily snapshots of `restic`, kept 14 days and immutable for 7.

On Hetzner:

4. In Hetzner Console, open the Storage Box and turn on **External Reachability** and **SSH Support**. Without External Reachability, only machines inside Hetzner's network can connect.
5. In the Storage Box's Snapshots tab, turn on automatic snapshots. This machine holds an SSH key to the box, so snapshots are what protect the backups if it is compromised.

On the PC:

6. Install restic and record it in the package list:

   ```bash
   sudo pacman -S restic
   make dump-packages
   ```

7. Generate the repository password, then store a copy outside the NAS:

   ```bash
   (umask 077 && head -c 32 /dev/urandom | base64 > ~/.secrets/restic)
   ```

8. Name the Storage Box `storagebox` in `~/.ssh/config`, with the username from Hetzner Console. Use port 23: there the box accepts the usual one-line OpenSSH key, while port 22 only accepts keys in RFC4716 format:

   ```text
   Host storagebox
       HostName u000000.your-storagebox.de
       Port 23
       User u000000
   ```

   Add this machine's public key, as its one-line `.pub` file, to the box's `.ssh/authorized_keys`. Then connect once to accept its host key, since restic can't answer that prompt:

   ```bash
   sftp storagebox
   ```

9. Create both repositories, check what would be uploaded, then run the first backup. Plug in Ethernet for this one, since it uploads everything twice:

   ```bash
   make backup-init
   ~/.config/restic/backup.sh run --dry-run -v
   make backup
   ```

10. Turn on the schedule:

    ```bash
    systemctl --user enable --now restic-backup.timer restic-maintenance.timer
    ```

### Restoring

Pick the target with `nas` or `hetzner`. They hold the same data, so use the NAS at home, since it's faster:

```bash
make backup-status
~/.config/restic/backup.sh restic nas restore latest --target /tmp/restore --include "$HOME/IdeaProjects/opusline"
```

To browse every snapshot as folders, mount a repository:

```bash
mkdir -p /tmp/restic-mount
~/.config/restic/backup.sh restic nas mount /tmp/restic-mount
```

If the NAS is lost, restore from Hetzner the same way, with `hetzner` as the target.

## Manual steps

These can't be automated safely, so they stay manual.

### Boot parameters

Loader entries contain the disk's PARTUUID, so they are not versioned:

```bash
sudoedit /boot/loader/entries/<date>_linux.conf
```

```text
options root=PARTUUID=... rw rootfstype=ext4 quiet thermal.off=1 acpi_enforce_resources=lax
```

- `quiet`: no boot splash, so this only hides kernel chatter.
- `thermal.off=1`: the board exposes one ACPI thermal zone reporting 17 °C, and probing it
  costs 2 s at every boot (`initcall_debug`). CPU temperatures come from k10temp, unaffected.
- `acpi_enforce_resources=lax`: lets OpenRGB and the Thermalright LCD reach the SMBus.

### Fonts

`system/` only tracks regular files, so the fontconfig links are created by hand:

```bash
sudo ln -s /usr/share/fontconfig/conf.avail/10-sub-pixel-rgb.conf /etc/fonts/conf.d/
sudo ln -s /usr/share/fontconfig/conf.avail/10-hinting-slight.conf /etc/fonts/conf.d/
sudo ln -s /usr/share/fontconfig/conf.avail/10-autohint.conf /etc/fonts/conf.d/
sudo ln -s /usr/share/fontconfig/conf.avail/11-lcdfilter-default.conf /etc/fonts/conf.d/
sudo ln -s /usr/share/fontconfig/conf.avail/75-apple-color-emoji.conf /etc/fonts/conf.d/
sudo fc-cache -fv
```

Don't enable `70-no-bitmaps.conf` or any other config that disables embedded bitmaps. Color emoji fonts such as Apple Color Emoji rely on them.

### Wi-Fi (RTL8922AE)

The `rtw89_8922ae` driver stalls when the access point changes channel or bandwidth.
Traffic stops, but the driver never reports a disconnect, so NetworkManager keeps showing the link as connected.

1. **Stop the Livebox from hopping channels.** On `http://192.168.1.1`, open Wi-Fi, then the 5 GHz advanced settings:
   - Fix the channel to 36, 40, 44 or 48, which avoid radar (DFS) channels.
   - Set the width to 80 MHz, because 160 MHz spans DFS channels and forces moves.
   - Turn off any smart channel or channel optimisation option.
2. **Let the PC recover on its own.** `make system` installs a NetworkManager dispatcher script that power-cycles the radio whenever the connectivity check fails while `wlan0` still claims to be connected. It also installs a module config that disables PCIe and driver power saving.

Check that the recovery works, and that the regulatory database loaded after a reboot:

```bash
journalctl -t wifi-reconnect
iw reg get   # should show "country FR", not "country 00"
```

### Software installed outside package managers

These came from vendor installers, so `make` can't reinstall them:

| Software | Location | Notes |
| --- | --- | --- |
| Claude Code | `~/.local/bin/claude` | Updates itself; CLI commands use the Greywall launcher in `~/.local/bin/greywall-agents` |
| Codex CLI | `~/.local/npm/bin/codex` | CLI commands use the Greywall launcher in `~/.local/bin/greywall-agents` |
| CodeRabbit CLI | `~/.local/bin/coderabbit` | `cr` is a symlink to it |
| greywall, greyproxy | `~/.local/bin` | The `opencode`, `codex`, and `claude` launchers in `config/bin/.local/bin/greywall-agents` run through greywall; greyproxy is a user service |
| opencode | `~/.opencode/bin` | `curl -fsSL https://opencode.ai/install \| bash`; CLI commands use the Greywall launcher in `~/.local/bin/greywall-agents`, which turns its self-update off |
| Herd Lite | `~/.config/herd-lite` | PHP, Composer and the Laravel installer, added to `PATH` by `~/.profile` |
| uv | `/usr/local/bin` | `make clean-caches` uses it |
| Private Internet Access | `/opt/piavpn` | Its installer also writes `piavpn.service` and `wgpia.conf` |
| Thermalright LCD control | `/usr/local/bin` | Its unit file is versioned in `system/` |

## Notes

- **DNS** goes through systemd-resolved with Quad9 and Cloudflare, set in [`system/etc/systemd/resolved.conf`](system/etc/systemd/resolved.conf). NetworkManager hands DNS over to it through [`dns.conf`](system/etc/NetworkManager/conf.d/dns.conf).
- **User services and timers** are versioned with their enable links in [`config/systemd`](config/systemd/.config/systemd/user), so `make dotfiles` enables them too.
- **The wallpaper** is picked by `wallpaper-rotator.sh`, which writes `~/.config/hypr/wallpaper.conf`. That file is git-ignored because it changes every 30 minutes.
