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

## Agent Cockpit

Workspace 6 is the [Agent Cockpit](https://github.com/Androlax2/agent-cockpit): every Claude Code, opencode and Codex session on the machine, each inside greywall, in one tmux window. It is a repository of its own, linked into this home from its running copy `~/agent-cockpit` (`make -C ~/agent-cockpit install`); it brings the session hooks, the opencode and Codex plugins, the greywall launchers and profiles, the bar's cockpit chip and the fish `claude` and `opencode` functions. This repository only calls it: the Hyprland bindings and window rules (`hypr/apps/claude.lua`), the idle lock of workspace 6 (`hypr/scripts/claude-idle.sh`), the lock screen (`cockpit away`), the tmux keys, Claude's rules (`config/claude/.claude/CLAUDE.md`) and skills, opencode's own settings.

## Games

A game window opening turns **game mode** on by itself (`config/hyprland/.config/hypr/game-mode.lua`), and the game's exit turns it off:

- Hyprland drops animations, blur and rounding; banners and approval windows wait.
- Feral GameMode (`config/gamemode`) puts the CPU governor and the GPU (`card1`) in their performance modes while the game's process runs.
- `game-performance` (GameMode's start and end hooks) pauses every agent sandbox, Claude Code, opencode or codex (cgroup freezer on the scope of each `greywall` process, a tmux pane's or a kitty window's: they resume mid-step after the game; the column and approval panes stay usable), the backup, jobpilot and the update, security and health checks, and drops any job already running to idle CPU and disk priority. The Todo queue waits too. `game-performance off` thaws by hand; the cockpit doctor flags sessions left frozen.
- MangoHud (`config/MangoHud`): FPS, frame times, CPU and GPU load and temperatures, with `Left Shift + F12`. Only Steam and its games load it (`config/steam` sets `MANGOHUD=1` in Steam's launcher).

## Machine monitor (vitals)

[vitals](https://github.com/Androlax2/vitals) watches the machine (temperatures and their trends, fans, disks, services, kernel errors, memory, the backup), keeps the history and warns through the Agent Cockpit's OSD and the bar. It is a repository of its own, linked from its running copy `~/vitals` (`make -C ~/vitals install`, `make -C ~/vitals system` for its root timer). This repository keeps its keybind (`Ctrl + Cmd + M`), its bar chip (`custom/vitals`) and the disk review it reads (`disk-space-review`).

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
`make -C ~/vitals test` (in its own repository).

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
| greywall, greyproxy | `~/.local/bin` | The `opencode`, `codex`, and `claude` launchers of the Agent Cockpit (`greywall-agents/`) run through greywall; greyproxy is a user service. Each sandbox logs in to greyproxy as `<agent>-<project>` (`claude-runway`), which its pending page and the cockpit's `⛔ greyproxy holds` line show; a label, not proof (any sandbox can claim any name), so approvals stay per agent (`claude*`), never per project. greywall is built from the `dotfiles` branch of Androlax2/greywall, v0.3.7 plus upstream PRs #103, #112, #113 and #114 (`make -C ~/agent-cockpit greywall-patch`), so a sandbox cannot read greyproxy's keys or `~/.ssh/id_*`. greyproxy is built from the `dotfiles` branch of Androlax2/greyproxy, v0.4.5 plus a block on proxied traffic to its own listeners, so a sandbox cannot reach greyproxy's API through the proxy, an optional API token (`~/.local/share/greyproxy/api-token`) and per-client proxy secrets, both off until greywall sends them, and a fail-closed start: greyproxy exits when its service cannot start (`make -C ~/agent-cockpit greyproxy-patch`) |
| opencode | `~/.opencode/bin` | `curl -fsSL https://opencode.ai/install \| bash`; CLI commands use the Greywall launcher in `~/.local/bin/greywall-agents`, which turns its self-update off |
| Herd Lite | `~/.config/herd-lite` | PHP, Composer and the Laravel installer, added to `PATH` by `~/.profile` |
| uv | `/usr/local/bin` | `make clean-caches` uses it |
| Private Internet Access | `/opt/piavpn` | Its installer also writes `piavpn.service` and `wgpia.conf` |
| Thermalright LCD control | `/usr/local/bin` | Its unit file is versioned in `system/` |

## Notes

- **DNS** goes through systemd-resolved with Quad9 and Cloudflare, set in [`system/etc/systemd/resolved.conf`](system/etc/systemd/resolved.conf). NetworkManager hands DNS over to it through [`dns.conf`](system/etc/NetworkManager/conf.d/dns.conf).
- **User services and timers** are versioned with their enable links in [`config/systemd`](config/systemd/.config/systemd/user), so `make dotfiles` enables them too.
- **The wallpaper** is picked by `wallpaper-rotator.sh`, which writes `~/.config/hypr/wallpaper.conf`. That file is git-ignored because it changes every 30 minutes.
