.PHONY: dotfiles dump install update

stow = cd config && stow -v -t ~

dotfiles:
	$(stow) --restow */

dump:
	pacman -Qqen > packages.txt
	yay -Qam > aur-packages.txt

install:
	sudo pacman -S --needed - < packages.txt
	yay -S --needed - < aur-packages.txt

update:
	sudo pacman -Syu
	yay -Syu
