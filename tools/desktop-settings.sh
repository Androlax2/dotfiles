#!/usr/bin/env bash
# Desktop settings that live in dconf and mimeapps.list rather than in a stowed file.
# Run by `make dotfiles`; every line is idempotent.
set -euo pipefail

# Icons: WhiteSur (macOS-style folders), also set in gtk settings.ini and xsettingsd.conf.
gsettings set org.gnome.desktop.interface icon-theme WhiteSur-dark
# Interface font, also set in gtk settings.ini, xsettingsd.conf and qt6ct.conf. Zen and other
# GTK apps on Wayland read it from here.
gsettings set org.gnome.desktop.interface font-name 'SF Pro Text 11'

# elementary Files: Finder-like column view, thumbnails, readable row height.
gsettings set io.elementary.files.preferences default-viewmode miller_columns
gsettings set io.elementary.files.preferences show-local-thumbnails true
gsettings set io.elementary.files.preferences show-remote-thumbnails true
gsettings set io.elementary.files.preferences singleclick-select false
gsettings set io.elementary.files.column-view default-zoom-level small
gsettings set io.elementary.files.column-view zoom-level small
gsettings set io.elementary.files.column-view preferred-column-width 280
gsettings set io.elementary.files.list-view default-zoom-level small
gsettings set io.elementary.files.list-view zoom-level small

# Default apps: Loupe for images, mpv for video and audio files.
xdg-mime default org.gnome.Loupe.desktop image/png image/jpeg image/gif image/webp image/bmp image/svg+xml image/tiff image/avif
xdg-mime default mpv.desktop video/mp4 video/x-matroska video/webm video/quicktime video/x-msvideo video/mpeg audio/mpeg audio/flac audio/x-wav audio/ogg
