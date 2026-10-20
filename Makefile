SHELL := /bin/bash
.SHELLFLAGS := -euo pipefail -c
.DEFAULT_GOAL := help
# Recipes parse pacman and systemctl output and rely on byte-order sorting for comm and diff.
export LC_ALL := C

stow = cd config && stow -v -t ~
backup_script = ~/.config/restic/backup.sh
system_files = $(shell cd system && find . -type f -printf '%P\n' | sort)
enabled_units = systemctl list-unit-files --state=enabled --no-legend --no-pager | awk '$$3 != "enabled" {print $$1}' | sort

pnpm_home = $${PNPM_HOME:-$$HOME/.local/share/pnpm}
# pnpm 11 refuses every global command while its bin directory is missing from PATH.
pnpm_global = env PATH="$(pnpm_home)/bin:$$PATH" pnpm

# One command per package manager, each printing its globally installed packages one per line.
# packages/<manager>.txt is the versioned copy of that output.
package_managers = pacman aur npm pnpm pip cargo go composer skills plugins
installed_pacman = pacman -Qqen
installed_aur = pacman -Qqm
installed_npm = npm ls -g --depth=0 --parseable | tail -n +2 | sed "s|^$$(npm root -g)/||"
installed_pnpm = $(pnpm_global) ls -g --depth=0 --json | jq -r '.[].dependencies // {} | keys[]'
installed_pip = python3 -m pip list --user --not-required --format=freeze | cut -d= -f1
installed_cargo = cargo install --list | awk '/^[^ ].* v[0-9]/ {print $$1}'
installed_go = for binary in "$$(go env GOPATH)"/bin/*; do if [ -f "$$binary" ]; then go version -m "$$binary" | awk '$$1 == "path" {print $$2}'; fi; done
installed_composer = composer --working-dir="$$(composer config --global home)" show --direct --name-only
# Agent skills installed with `npx skills add -g`, as "owner/repo skill-name".
installed_skills = jq -r '.skills | to_entries[] | "\(.value.source) \(.key)"' ~/.agents/.skill-lock.json
# Claude Code plugins, as "owner/repo plugin@marketplace" (the repo is the marketplace's source).
installed_plugins = { jq -r --slurpfile marketplaces ~/.claude/plugins/known_marketplaces.json \
    '.plugins | keys[] as $$plugin | ($$plugin | split("@")[1]) as $$marketplace | "\($$marketplaces[0][$$marketplace].source.repo // $$marketplace) \($$plugin)"' \
    ~/.claude/plugins/installed_plugins.json 2>/dev/null || true; }

# Skills written by hand are versioned as files in the claude stow package. In ~/.claude/skills
# they are the real directories owned by neither the skills CLI (its lock file) nor Claude's
# account sync (synced/); once versioned they become links into the repo.
versioned_skills = config/claude/.claude/skills
unversioned_skills = comm -23 <(find ~/.claude/skills -mindepth 1 -maxdepth 1 -type d ! -name synced -printf '%f\n' | sort) <(jq -r '.skills | keys[]' ~/.agents/.skill-lock.json | sort)
unlinked_skills = comm -23 <(find $(versioned_skills) -mindepth 1 -maxdepth 1 -type d -printf '%f\n' | sort) <(find ~/.claude/skills -mindepth 1 -maxdepth 1 -type l -printf '%f\n' | sort)

.PHONY: help setup packages dotfiles zen palette files-patch global-packages system locale services initramfs install handoff-install cockpit dotfiles-work \
	    test check check-packages check-skills check-services check-system check-untracked check-security check-palette \
	    dump dump-packages dump-skills dump-services dump-system dump-security \
	    update update-skills disk clean clean-packages clean-caches clean-logs clean-docker clean-trash \
	    backup backup-list backup-status backup-check backup-maintain backup-init

# Descriptions after `##` and sections after `##@` are rendered by tools/make-help.awk.
# Tags: [sudo] prompts for a password, [asks] confirms before deleting,
# [read-only] changes nothing, [no undo] deletes without asking.
help:
	@use_color=0; if [ -t 1 ] && [ -z "$${NO_COLOR:-}" ]; then use_color=1; fi; \
	awk -v use_color="$$use_color" -f tools/make-help.awk $(MAKEFILE_LIST)

##@ Set up a machine

setup: packages dotfiles global-packages system locale services initramfs ## [sudo] Provision a fresh machine with the steps below

packages: ## [sudo] Install pacman and AUR packages
	sudo pacman -S --needed - < packages/pacman.txt
	yay -S --needed - < packages/aur.txt

dotfiles: palette zen ## Link every config/ package into your home
	$(stow) --restow */
	~/.config/wireplumber/wireplumber.conf.d/generate-config.sh
	tools/desktop-settings.sh

# Colours live in one file (config/waybar/.config/waybar/colors.css); the other themed
# files are rendered from tools/palette/*.tpl so a token change reaches every surface.
palette: ## Render kitty, GTK, qt6ct, swaync, walker, Hyprland, fish, lazygit, fzf, btop, OSD, slurp and Zen colours from colors.css
	@tools/palette.sh

# Zen profiles have random names, so stow cannot reach them: link the chrome and
# prefs from config/zen into the default profile named in ~/.zen/installs.ini.
zen: ## Link the Zen userChrome and prefs into the default Zen profile
	@profile="$$(sed -n 's/^Default=//p' ~/.zen/installs.ini | head -n1)"; \
	if [ -z "$$profile" ] || [ ! -d ~/.zen/"$$profile" ]; then echo "zen: no default profile in ~/.zen/installs.ini, skipped"; exit 0; fi; \
	mkdir -p ~/.zen/"$$profile"/chrome; \
	ln -sfn ~/.config/zen/userChrome.css ~/.zen/"$$profile"/chrome/userChrome.css; \
	ln -sfn ~/.config/zen/user.js ~/.zen/"$$profile"/user.js; \
	echo "zen: linked into ~/.zen/$$profile"

# Needs `make dotfiles` first: ~/.npmrc points npm's global prefix at ~/.local/npm.
# pip needs --break-system-packages because Arch marks the system Python as externally
# managed; --user keeps these out of the system site-packages.
# Only what is listed but missing gets installed (same comparison as check-packages),
# so a second run changes nothing.
# Group skills per source repo so each repo is cloned once. Target Codex to populate
# ~/.agents/skills for sibling-script dependencies, plus Claude Code's own directory.
missing = comm -23 <(sort packages/$(1).txt) <($(installed_$(1)) | sort)
global-packages: ## Install npm, pnpm, pip, cargo, go and composer packages, agent skills and Claude plugins
	$(call missing,npm) | xargs --no-run-if-empty npm install -g
	$(call missing,pnpm) | xargs --no-run-if-empty $(pnpm_global) add -g
	$(call missing,pip) | xargs --no-run-if-empty python3 -m pip install --user --break-system-packages
	$(call missing,cargo) | xargs --no-run-if-empty cargo install
	$(call missing,go) | xargs --no-run-if-empty -I{} go install {}@latest
	$(call missing,composer) | xargs --no-run-if-empty composer global require
	$(call missing,skills) | awk '{skills[$$1] = skills[$$1] " --skill " $$2} END {for (source in skills) print source skills[source]}' \
	    | xargs --no-run-if-empty -L1 npx --yes skills add --global --agent claude-code --agent codex --yes
	@# The real binary: the fish `claude` function would run this inside greywall, where ~/.claude/plugins
	@# is read-only. The same goes for the plugin's data folder, which Claude Code creates before running
	@# a plugin's hooks; it is made here once, or every sandboxed session fails at start (EROFS).
	$(call missing,plugins) | while read -r repository plugin; do \
	    ~/.local/bin/claude plugin marketplace add "$$repository"; \
	    ~/.local/bin/claude plugin install "$$plugin"; \
	    mkdir -p ~/.claude/plugins/data/"$${plugin/@/-}"; \
	done

system: ## [sudo] Copy system/ into /: each changed file shows its diff and asks first
	@tools/system-install.sh

locale: ## [sudo] Generate the locales enabled in locale.gen
	sudo locale-gen

services: ## [sudo] Enable the units listed in services.txt
	xargs --no-run-if-empty sudo systemctl enable < services.txt

initramfs: ## [sudo] Rebuild the initramfs after boot changes
	sudo mkinitcpio -P

install: packages global-packages ## [sudo] Install packages only, without touching config

# Files (pantheon-files) with packages/pantheon-files/finder-clicks.patch: no folder
# deep count in the overlay bar, single click anywhere on a folder row opens it.
# pacman replaces it on the next pantheon-files update, so run this again afterwards.
files-patch: ## [sudo] Build and install the patched Files file manager
	packages/pantheon-files/build.sh

##@ Keep the repo in sync

build_runner = python3 config/bin/.local/bin/agent-build

test: ## Build the OSD classifier and run the apps/ test suite
	python3 -m unittest discover -s tools/tests -p test_agent_build.py
	$(build_runner) cargo build --release --quiet --manifest-path apps/Cargo.toml -p desktop --bin osd
	$(build_runner) cargo test --release --quiet --manifest-path apps/Cargo.toml

cockpit: ## Build the Rust programs in apps/ (the cockpit's column, approval terminal, guide, sandbox commands); claude-sessions reload shows the column
	$(build_runner) cargo build --release --manifest-path apps/Cargo.toml

dotfiles-work: ## Create the clone Claude's dotfiles session works in (~/IdeaProjects/dotfiles); claude-sessions apply-dotfiles brings its commits here
	@if [ -e ~/IdeaProjects/dotfiles ]; then echo "~/IdeaProjects/dotfiles exists already"; else git clone -q ~/dotfiles ~/IdeaProjects/dotfiles && echo "created ~/IdeaProjects/dotfiles (origin: ~/dotfiles)"; fi

handoff-install: ## Install what Claude left in .handoff/ (checksums, diff, then asks; runs with your rights)
	@tools/handoff-install.sh

check: check-packages check-skills check-services check-system check-untracked check-security check-palette test ## [read-only] Show everything that differs from the repo, then run the tests

check-packages: ## Packages listed but not installed, or the reverse
	@echo "== packages =="
	@$(foreach manager,$(package_managers),diff <(sort packages/$(manager).txt) <($(installed_$(manager)) | sort) | sed -n 's/^< /  $(manager): listed, not installed: /p; s/^> /  $(manager): installed, not listed: /p' || true;)

check-skills: ## Hand-written skills in ~/.claude/skills missing from the repo, or the reverse
	@echo "== hand-written skills =="
	@$(unversioned_skills) | sed 's/^/  in ~\/.claude\/skills, not versioned: /'
	@$(unlinked_skills) | sed 's/^/  versioned, not linked into ~\/.claude\/skills: /'

check-services: ## Enabled units missing from services.txt, or the reverse
	@echo "== services =="
	@diff <(sort services.txt) <($(enabled_units)) | sed -n 's/^< /  listed, not enabled: /p; s/^> /  enabled, not listed: /p' || true

check-system: ## Diff each system/ file against the live one
	@echo "== system files that differ from the live system =="
	@for path in $(system_files); do \
	    diff -u --label "repo: system/$$path" --label "live: /$$path" "system/$$path" "/$$path" || true; \
	done

# A .pacnew is listed even under an ignored path: it is a pending merge, and an ignore
# written for a whole directory must not hide it.
unignored_etc_paths = NR == FNR { if ($$0 !~ /^[[:space:]]*(\#|$$)/) ignored[$$0]; next } \
	/\.pacnew$$/ { print; next } \
	{ for (pattern in ignored) if ($$0 ~ pattern) next; print }
check-untracked: ## Edited /etc files in neither system/ nor system-ignore.txt, and every .pacnew
	@echo "== /etc files configured on this machine but not versioned =="
	@comm -23 \
	    <({ find /etc -xdev -type f -print0 2>/dev/null | xargs -0 pacman -Qo 2>&1 >/dev/null | sed -n 's/^error: No package owns //p'; \
	        pacman -Qii | grep -oE '/[^[:space:]]+ \[modified\]' | cut -d' ' -f1; } \
	      | sort -u | awk '$(unignored_etc_paths)' system-ignore.txt -) \
	    <(printf '/%s\n' $(system_files) | sort)

# Ports, users, SUID files, unowned binaries, kernel modules, persistence spots and SSH keys, plus
# package integrity, firewall state and known-vulnerable packages. The daily security-check.timer
# runs the same script and notifies on findings.
check-security: ## [read-only] Compare the machine with the security baseline in security/
	@tools/security-check.sh check || true

check-palette: ## [read-only] Diff every themed file against its palette template
	@echo "== palette =="
	@tools/palette.sh --check || true

dump: dump-packages dump-skills dump-services dump-system ## Save packages, skills, services and system files into the repo

dump-packages: ## Rewrite packages/*.txt from what is installed
	@$(foreach manager,$(package_managers),$(installed_$(manager)) | sort > packages/$(manager).txt; echo "wrote packages/$(manager).txt";)

dump-skills: ## Move hand-written skills from ~/.claude/skills into the repo and link them back
	@$(unversioned_skills) | while read -r skill; do \
	    mv ~/.claude/skills/"$$skill" $(versioned_skills)/; \
	    echo "moved ~/.claude/skills/$$skill into $(versioned_skills)"; \
	done
	@if [ -n "$$($(unlinked_skills))" ]; then $(stow) --restow claude; fi

dump-services: ## Rewrite services.txt from the enabled units
	$(enabled_units) > services.txt

dump-security: ## Rewrite security/ from the live machine (only when its findings are legitimate)
	@tools/security-check.sh dump

dump-system: ## Copy live edits of system/ files back into the repo
	@for path in $(system_files); do \
	    cmp -s "/$$path" "system/$$path" && continue; \
	    cp "/$$path" "system/$$path"; \
	    echo "updated system/$$path"; \
	done

agent_docker = tools/agent-docker.sh

.PHONY: agent-docker-setup agent-docker-add agent-docker-remove agent-docker-status

##@ Isolate coding agents

# One rootless Docker daemon per project for agents running in greywall (tools/agent-docker.sh).
# The units and the agent_docker_egress firewall table come from system/.
agent-docker-setup: system ## [sudo] Install the agent units and the agent-docker group (log out and back in after the first run)
	sudo $(agent_docker) setup

agent-docker-add: ## [sudo] Give a project its own isolated Docker daemon: make agent-docker-add PROJECT=runway
	@test -n "$(PROJECT)" || { echo "usage: make agent-docker-add PROJECT=<folder in ~/IdeaProjects>"; exit 2; }
	sudo $(agent_docker) add "$(PROJECT)"

agent-docker-remove: ## [no undo] Delete a project's agent daemon, its user, images and containers: make agent-docker-remove PROJECT=runway
	@test -n "$(PROJECT)" || { echo "usage: make agent-docker-remove PROJECT=<folder in ~/IdeaProjects>"; exit 2; }
	sudo $(agent_docker) remove "$(PROJECT)"

agent-docker-status: ## [read-only] List the agent Docker daemons and whether their sockets are up
	@$(agent_docker) status

.PHONY: agent-anthropic-key agent-opencode-key

agent-anthropic-key: ## Store the Console API key that credits sessions (the cockpit's $) bill instead of the subscription; asks for it
	@secret-tool store --label "Claude API key for credits sessions" service agent-anthropic

agent-opencode-key: ## Store the Anthropic API key opencode sessions use, a greyproxy placeholder inside their sandbox; asks for it
	@read -rsp "Anthropic API key for opencode (sk-ant-…, not shown): " key; echo; \
	if [[ ! $$key =~ ^sk-ant-[a-z]+[0-9]*-[A-Za-z0-9_-]+$$ ]] || (( $${#key} < 80 )); then \
	    echo "not stored: an Anthropic API key starts with sk-ant- and has 80 characters or more (this one: $${#key})" >&2; exit 1; \
	fi; \
	printf '%s' "$$key" | secret-tool store --label "Anthropic API key for opencode sessions" service agent-opencode-anthropic; \
	echo "stored ($${#key} characters)"

agent_github = tools/agent-github.sh

.PHONY: agent-github-token agent-github-board-token agent-github-relogin agent-github-protect agent-github-check \
	    agent-dashboard-tailnet agent-dashboard-tailnet-off

agent-github-token: ## Create and store a GitHub token that reaches only this project's repo: make agent-github-token PROJECT=runway
	@test -n "$(PROJECT)" || { echo "usage: make agent-github-token PROJECT=<folder in ~/IdeaProjects>"; exit 2; }
	@$(agent_github) token "$(PROJECT)"

agent-github-board-token: ## Create and store the classic token, project scope only, that lets agents read and move Project board cards
	@$(agent_github) project-board-token

agent-github-relogin: ## Revoke your old GitHub CLI login and log in again with keyring storage
	@$(agent_github) relogin

agent-github-protect: ## Require pull requests and block force pushes on the project's default branch: make agent-github-protect PROJECT=runway
	@test -n "$(PROJECT)" || { echo "usage: make agent-github-protect PROJECT=<folder in ~/IdeaProjects>"; exit 2; }
	@$(agent_github) protect "$(PROJECT)"

agent-github-check: ## [read-only] Check that each agent token reaches only its repo and that gh keeps no token in hosts.yml
	@$(agent_github) check

# greyproxy's dashboard listens on localhost only; `tailscale serve` publishes it to this tailnet
# (and nowhere else) so pending network requests can be approved from the phone.
GREYPROXY_DASHBOARD_PORT ?= 43080
agent-dashboard-tailnet: ## Publish greyproxy's dashboard on the tailnet: make agent-dashboard-tailnet [GREYPROXY_DASHBOARD_PORT=43080] (then open /dashboard)
	tailscale serve --bg --https=443 http://127.0.0.1:$(GREYPROXY_DASHBOARD_PORT)
	tailscale serve status

agent-dashboard-tailnet-off: ## Stop publishing greyproxy's dashboard on the tailnet
	tailscale serve --https=443 off

##@ Maintain

update: ## [sudo] Upgrade pacman and AUR packages
	sudo pacman -Syu
	yay -Syu

# Kept out of `update`: skills are instructions agents act on, pulled unpinned from many
# repositories, so upgrading them is a deliberate step.
update-skills: ## Upgrade the agent skills installed with the skills CLI
	npx --yes skills update --global --yes

disk: ## [read-only] Show what is using disk space
	@echo "== largest user caches =="
	@{ du -sh ~/.cache/* 2>/dev/null || true; } | sort -rh | sed -n "1,10p"
	@echo "== package and download caches =="
	@du -sh /var/cache/pacman/pkg ~/.npm $(pnpm_home)/store ~/go/pkg/mod ~/.cargo/registry ~/.local/share/Trash 2>/dev/null || true
	@echo "== journal =="
	@journalctl --disk-usage
	@echo "== orphaned packages: $$({ pacman -Qdtq || true; } | wc -l) =="
	@echo "== docker =="
	@docker system df

clean: clean-packages clean-caches clean-logs ## [asks] Routine cleanup with the steps below

clean-packages: ## [asks] Remove orphans and old pacman and AUR caches
	yay -Yc
	sudo paccache -rk2
	sudo paccache -ruk0
	yay -Sc --aur

clean-caches: ## Empty npm, pnpm, go, uv, pip and composer caches
	npm cache clean --force
	$(pnpm_global) store prune
	go clean -cache -modcache
	uv cache prune
	python3 -m pip cache purge
	composer clear-cache

clean-logs: ## [sudo] Delete journal entries older than 4 weeks
	sudo journalctl --vacuum-time=4weeks

##@ Back up

backup: ## Back up home to the NAS and Hetzner now
	$(backup_script) run

# DEPTH sets how many folder levels below ~ are listed, for example make backup-list DEPTH=2.
backup-list: ## [read-only] Show the size of each folder a backup includes
	$(backup_script) list $(DEPTH)

backup-status: ## [read-only] Snapshots on both targets, and the next runs
	$(backup_script) restic nas snapshots --compact
	$(backup_script) restic hetzner snapshots --compact
	systemctl --user list-timers 'restic-*' --no-pager

backup-check: ## [read-only] Read back 5% of the data on both targets
	$(backup_script) restic nas check --read-data-subset=5%
	$(backup_script) restic hetzner check --read-data-subset=5%

backup-maintain: ## Forget old snapshots and prune both targets
	$(backup_script) maintain

backup-init: ## Create the repositories that don't exist yet
	$(backup_script) init

##@ Delete data

clean-docker: ## [asks] Remove unused containers, images and volumes
	docker system prune --all
	docker volume prune --all

clean-trash: ## [no undo] Empty the desktop trash
	gio trash --empty
