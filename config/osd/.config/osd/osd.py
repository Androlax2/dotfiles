#!/usr/bin/env python3
"""On-screen display for volume, mute and brightness changes.

A single 320x36 island at the bottom centre (GTK4 + layer shell, namespace "osd").
Reads one command per line from $XDG_RUNTIME_DIR/osd.fifo, written by osd-show:

    volume <0-100>       yellow line, icon by level
    mute                 mute icon, empty line
    brightness <0-100>   blue line, sun icon

Hides 1.2 s after the last command. Icons are the bar's SVGs (already fg_dark).
"""
import ctypes
import os
import stat
import sys

# The layer-shell library must be loaded before GTK opens the Wayland display.
ctypes.CDLL("libgtk4-layer-shell.so", mode=ctypes.RTLD_GLOBAL)

import gi  # noqa: E402

gi.require_version("Gdk", "4.0")
gi.require_version("Gtk", "4.0")
gi.require_version("Gtk4LayerShell", "1.0")
from gi.repository import Gdk, GLib, Gtk, Gtk4LayerShell as LayerShell  # noqa: E402

CONFIG_DIR = os.path.dirname(os.path.realpath(__file__))
ICON_DIR = os.path.expanduser("~/.config/waybar/icons")
FIFO_PATH = os.path.join(os.environ.get("XDG_RUNTIME_DIR", "/tmp"), "osd.fifo")
HIDE_AFTER_MS = 1200
WIDTH, HEIGHT = 320, 36
ISLAND_PADDING, ICON_SIZE, ROW_SPACING = 14, 18, 12
TRACK_WIDTH = WIDTH - 2 * ISLAND_PADDING - ICON_SIZE - ROW_SPACING - 2  # 2 = the hairline border


def volume_icon(level: int) -> str:
    if level == 0:
        return "volume-mute.svg"
    if level < 34:
        return "volume-low.svg"
    if level < 67:
        return "volume-mid.svg"
    return "volume-high.svg"


class Osd:
    def __init__(self, app: Gtk.Application):
        self.window = Gtk.Window(application=app)
        self.window.set_default_size(WIDTH, HEIGHT)
        self.window.set_size_request(WIDTH, HEIGHT)
        self.window.set_resizable(False)
        # No client-side decoration: its shadow margins would inflate the layer surface.
        self.window.set_decorated(False)
        self.window.add_css_class("osd-window")  # not "osd": GTK styles that class itself

        LayerShell.init_for_window(self.window)
        LayerShell.set_namespace(self.window, "osd")
        LayerShell.set_layer(self.window, LayerShell.Layer.OVERLAY)
        LayerShell.set_anchor(self.window, LayerShell.Edge.BOTTOM, True)
        LayerShell.set_margin(self.window, LayerShell.Edge.BOTTOM, 15)
        LayerShell.set_keyboard_mode(self.window, LayerShell.KeyboardMode.NONE)

        self.icon = Gtk.Image()
        self.icon.set_pixel_size(ICON_SIZE)
        # Two plain boxes instead of Gtk.ProgressBar: the GTK theme's progress-bar
        # rules would win over this file's, boxes take the CSS as written.
        self.track = Gtk.Box(hexpand=True, valign=Gtk.Align.CENTER)
        self.track.add_css_class("track")
        self.track.set_size_request(TRACK_WIDTH, 2)
        self.fill = Gtk.Box(halign=Gtk.Align.START)
        self.fill.add_css_class("fill")
        self.track.append(self.fill)
        row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=ROW_SPACING)
        row.add_css_class("island")
        row.append(self.icon)
        row.append(self.track)
        self.window.set_child(row)

        self.hide_timer = 0

    def show(self, kind: str, icon_file: str, fraction: float):
        for css_class in ("volume", "brightness", "mute"):
            self.fill.remove_css_class(css_class)
        self.fill.add_css_class(kind)
        self.icon.set_from_file(os.path.join(ICON_DIR, icon_file))
        self.fill.set_size_request(round(TRACK_WIDTH * fraction), 2)
        self.window.present()
        if self.hide_timer:
            GLib.source_remove(self.hide_timer)
        self.hide_timer = GLib.timeout_add(HIDE_AFTER_MS, self.hide)

    def hide(self):
        self.hide_timer = 0
        self.window.set_visible(False)
        return GLib.SOURCE_REMOVE

    def handle(self, line: str):
        parts = line.split()
        if not parts:
            return
        command = parts[0]
        try:
            level = max(0, min(100, int(parts[1]))) if len(parts) > 1 else 0
        except ValueError:
            print(f"osd: bad level in {line!r}", file=sys.stderr)
            return
        if command == "volume":
            self.show("volume", volume_icon(level), level / 100)
        elif command == "mute":
            self.show("mute", "volume-mute.svg", 0)
        elif command == "brightness":
            self.show("brightness", "brightness.svg", level / 100)
        else:
            print(f"osd: unknown command {line!r}", file=sys.stderr)


def open_fifo() -> int:
    if os.path.exists(FIFO_PATH) and not stat.S_ISFIFO(os.stat(FIFO_PATH).st_mode):
        os.remove(FIFO_PATH)
    if not os.path.exists(FIFO_PATH):
        os.mkfifo(FIFO_PATH, 0o600)
    # Read-write so the daemon never sees EOF when a writer closes.
    return os.open(FIFO_PATH, os.O_RDWR | os.O_NONBLOCK)


def main():
    app = Gtk.Application(application_id="dotfiles.osd")
    state = {"osd": None, "buffer": b""}

    def on_activate(application):
        provider = Gtk.CssProvider()
        provider.load_from_path(os.path.join(CONFIG_DIR, "style.css"))
        Gtk.StyleContext.add_provider_for_display(
            Gdk.Display.get_default(), provider, Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION
        )
        state["osd"] = Osd(application)
        application.hold()

        fd = open_fifo()

        def on_readable(_fd, _condition):
            chunk = os.read(fd, 4096)
            state["buffer"] += chunk
            while b"\n" in state["buffer"]:
                line, state["buffer"] = state["buffer"].split(b"\n", 1)
                state["osd"].handle(line.decode(errors="replace").strip())
            return GLib.SOURCE_CONTINUE

        GLib.io_add_watch(fd, GLib.PRIORITY_DEFAULT, GLib.IOCondition.IN, on_readable)

    app.connect("activate", on_activate)
    app.run(None)


if __name__ == "__main__":
    main()
