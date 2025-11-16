# Linux machine setup

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
