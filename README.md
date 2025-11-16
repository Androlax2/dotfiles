# Linux machine setup

## Fonts

```
sudo ln -s /usr/share/fontconfig/conf.avail/10-sub-pixel-rgb.conf /etc/fonts/conf.d/
sudo ln -s /usr/share/fontconfig/conf.avail/10-hinting-slight.conf /etc/fonts/conf.d/
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

## Keybinds

Use keyd and config /etc/keyd/default.conf

```
[ids]
*

[main]
# Swap Super and CTRL
leftmeta = leftcontrol
leftcontrol = leftmeta
rightmeta = rightcontrol
rightcontrol = rightmeta
```
