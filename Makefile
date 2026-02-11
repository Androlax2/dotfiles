.PHONY: dotfiles dump install update

stow = cd config && stow -v -t ~

dotfiles:
	$(stow) --restow */
	~/.config/wireplumber/wireplumber.conf.d/generate-config.sh

dump:
	pacman -Qqen > packages.txt
	yay -Qqm > aur-packages.txt

install:
	sudo pacman -S --needed - < packages.txt
	yay -S --needed - < aur-packages.txt

update:
	sudo pacman -Syu
	yay -Syu
