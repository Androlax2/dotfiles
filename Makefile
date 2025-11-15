.PHONY: dotfiles

stow = cd config && stow -v -t ~

dotfiles:
	$(stow) --restow */
