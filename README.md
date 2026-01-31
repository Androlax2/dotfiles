# Linux machine setup

## Plymouth

```bash
sudoedit /etc/mkinitcpio.conf
```

```
HOOKS=(base udev plymouth ... fsck)
```

```bash
sudoedit /boot/loader/entries/{date}.conf
```

```bash
options root=PartUUID=... quiet splash
```

```
sudo plymouth-set-default-theme -R motion
```

```bash
sudo mkinitcpio -p linux
```

## Fonts

```
sudo ln -s /usr/share/fontconfig/conf.avail/10-sub-pixel-rgb.conf /etc/fonts/conf.d/
sudo ln -s /usr/share/fontconfig/conf.avail/10-hinting-slight.conf /etc/fonts/conf.d/
sudo ln -s /usr/share/fontconfig/conf.avail/10-autohint.conf /etc/fonts/conf.d/
sudo ln -s /usr/share/fontconfig/conf.avail/11-lcdfilter-default.conf /etc/fonts/conf.d/
sudo ln -s /usr/share/fontconfig/conf.avail/75-apple-color-emoji.conf /etc/fonts/conf.d/
```

```bash
sudoedit /etc/profile.d/freetype2.sh
```

```bash
export FREETYPE_PROPERTIES="truetype:interpreter-version=40"
```

```bash
sudo fc-cache -fv
```

Note: Do NOT use `70-no-bitmaps.conf` or similar configs that disable embedded bitmaps globally - this breaks color emoji fonts (like Apple Color Emoji) which rely on embedded bitmaps.

## SDDM

```bash
sudoedit /etc/sddm.conf
```

```
[General]
InputMethod=qtvirtualkeyboard
GreeterEnvironment=QML2_IMPORT_PATH=/usr/share/sddm/themes/silent/components/,QT_IM_MODULE=qtvirtualkeyboard

[Theme]
Current=silent
```

# Keybinds

Remap binds for mac keyboard

```bash
sudoedit /etc/keyd/default.conf
```

```bash
[ids]
*

[main]

# Swap Ctrl and Command keys
control = layer(meta)
meta = layer(control)

# Tab switching
[meta]
tab = C-pagedown

[meta+shift]
tab = C-pageup

# Insertion point movement
[control]
left = home
right = end
up = C-home
down = C-end

[alt]
left = C-left
right = C-right
```

## DNS (AdGuard Home)

Configure NetworkManager to use systemd-resolved with custom DNS server:

```bash
sudo mkdir -p /etc/NetworkManager/conf.d
sudoedit /etc/NetworkManager/conf.d/dns.conf
```

```ini
[main]
dns=systemd-resolved
```

```bash
sudo mkdir -p /etc/systemd/resolved.conf.d
sudoedit /etc/systemd/resolved.conf.d/adguard.conf
```

```ini
[Resolve]
DNS=192.168.1.15 # IP of the DNS server AdGuard Home
DNSStubListener=no
```

```bash
sudo systemctl enable systemd-resolved
sudo systemctl restart NetworkManager systemd-resolved
```

Verify :

```bash
systemctl is-enabled systemd-resolved NetworkManager
resolvectl status  
```

## Useful

Uninstall package and deps :

```bash
sudo pacman -Rcns <package>
```

## Systemd

```bash
systemctl --user enable --now dns-check.timer
systemctl --user enable --now update-check.timer
```
