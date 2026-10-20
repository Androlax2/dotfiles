# Report-only cache maintenance

Approved scope: daily reports for Hartprint's Turbo cache and pacman's package cache. Nothing is installed or enabled by this change. Deletion requires a separate approval and implementation.

## Policies

- Inspect only `~/IdeaProjects/hartprint/.turbo/cache` for Turbo; never discover other projects automatically.
- Report regular files whose access and modification timestamps are both strictly older than 30 days. These are provisional observations, not verified safe deletion candidates: the live entry format and build coordination are not yet verified.
- Reject symlinked path components, symlinked entries, and unexpected nested directories. Missing cache is normal; permission and inspection errors fail visibly.
- Pacman uses the installed `/usr/bin/paccache` dry-run operation, retaining at least two versions and everything accessed or modified within 30 days.
- Daily persistent timers with randomized delay, low CPU/I/O priority, and journal output. Separate unprivileged user service and hardened system service. No root execution of scripts from the user's checkout.
- Prevent overlapping user report runs with a runtime lock; systemd already serializes each service's activations.
- Report counts and allocated-byte estimates for Turbo, explicitly not guaranteed reclaimable bytes. Do not inspect file contents. Pacman reports native dry-run results.
- No deletion mode, broad cleanup commands, dependency changes, Makefile changes, service-list changes, or installation steps.

Excluded: games, project checkouts, dependencies, Docker data, IDE indexes, Rust build output, pnpm/uv stores, AUR build directories, and personal files. Existing manual cleanup stays unchanged.

Implementation choice: Python standard library for the user report and regression tests; vendor paccache executable directly in the system unit. Units follow the existing config/systemd and system/etc/systemd layouts. Document that these files will run outside the sandbox if installed later.
