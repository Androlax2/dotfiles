# vitals

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


## Install

```sh
git clone git@github.com:Androlax2/vitals.git ~/vitals
make -C ~/vitals install   # build, link the command and the user units
make -C ~/vitals system    # [sudo] vitals-root.timer (SMART, the failed system units)
git clone ~/vitals ~/IdeaProjects/vitals   # where sessions work
```

Its alerts reach the screen through the Agent Cockpit's OSD daemon: one line on
`$XDG_RUNTIME_DIR/osd-host.fifo`, `notice {"id": "vitals-<key>", "title": …, "message": …}` or
`notice-clear vitals-<key>`; without a daemon reading, the line is dropped and the bar's dot
still shows the alert. The running copy, `~/vitals`, changes only through `cockpit apply <clone>`
after a yes.

