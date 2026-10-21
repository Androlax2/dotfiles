SHELL := /bin/bash
.SHELLFLAGS := -euo pipefail -c
.DEFAULT_GOAL := help

# vitals lives in ~/vitals, the running copy: its files are linked into ~/.local/bin and
# ~/.config/systemd/user by stow, one link per file (--no-folding), beside the dotfiles' own.
stow = cd config && stow -v --no-folding -t ~
packages = $(shell ls config)

.PHONY: help build test install system after-apply

help:
	@use_color=0; if [ -t 1 ] && [ -z "$${NO_COLOR:-}" ]; then use_color=1; fi; \
	awk -v use_color="$$use_color" -f tools/make-help.awk $(MAKEFILE_LIST)

##@ Build and install

build: ## Build vitals (apps/target/release/vitals)
	cargo build --release --manifest-path apps/Cargo.toml

test: ## Run the test suite
	cargo test --release --quiet --manifest-path apps/Cargo.toml

install: build ## Link the command and the user units into your home and reload them
	$(stow) --restow $(packages)
	systemctl --user daemon-reload

system: ## [sudo] Copy system/ into / (vitals-root.timer): each changed file shows its diff and asks first
	@tools/system-install.sh

# cockpit apply runs this in the running copy after git am, with BEFORE the commit it started from.
after-apply: ## Relink, and rebuild when apps/ changed: make after-apply BEFORE=<commit>
	@test -n "$(BEFORE)" || { echo "usage: make after-apply BEFORE=<commit before the apply>"; exit 2; }
	$(stow) --restow $(packages)
	systemctl --user daemon-reload
	@if ! git diff --quiet "$(BEFORE)" HEAD -- apps; then $(MAKE) build; fi
