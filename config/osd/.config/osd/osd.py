#!/usr/bin/env python3
"""On-screen display: a volume/brightness island and per-session Claude banners.

Bottom centre: a 320x36 island for volume, mute and brightness changes.
Top centre: a stack of banners, one per Claude Code session, raised by
~/.claude/hooks/notify.sh. Both are GTK4 + layer shell windows, namespace "osd".

Reads one command per line from $XDG_RUNTIME_DIR/osd.fifo, written by osd-show:

    volume <0-100>          yellow line, icon by level
    mute                    mute icon, empty line
    brightness <0-100>      blue line, sun icon
    banner <json>           raise (or replace) the banner of one session
    banner-clear <session>  drop that session's banner

The banner JSON carries session, title, message, sticky, and the focus target of
the session: kitty_pid, kitty_window, kitty_listen and tmux_pane, each optional.
A sticky banner stays until cleared or clicked; the others hide after 6 s.
Clicking a banner focuses the session's kitty window through Hyprland, its kitty
tab through kitty remote control, and its tmux pane, then drops the banner.

The island hides 1.2 s after the last command. Icons are the bar's SVGs.
"""
import ctypes
import json
import os
import re
import stat
import subprocess
import sys

# The layer-shell library must be loaded before GTK opens the Wayland display.
ctypes.CDLL("libgtk4-layer-shell.so", mode=ctypes.RTLD_GLOBAL)

import gi  # noqa: E402

gi.require_version("Gdk", "4.0")
gi.require_version("Gtk", "4.0")
gi.require_version("Gtk4LayerShell", "1.0")
from gi.repository import Gdk, GLib, Gtk, Gtk4LayerShell as LayerShell, Pango  # noqa: E402

CONFIG_DIR = os.path.dirname(os.path.realpath(__file__))
ICON_DIR = os.path.expanduser("~/.config/waybar/icons")
FIFO_PATH = os.path.join(os.environ.get("XDG_RUNTIME_DIR", "/tmp"), "osd.fifo")
HIDE_AFTER_MS = 1200
WIDTH, HEIGHT = 320, 36
ISLAND_PADDING, ICON_SIZE, ROW_SPACING = 14, 18, 12
TRACK_WIDTH = WIDTH - 2 * ISLAND_PADDING - ICON_SIZE - ROW_SPACING - 2  # 2 = the hairline border

BANNER_HIDE_AFTER_MS = 6000
BANNER_MAX_CHARS = 64
BANNER_SPACING = 8
BANNER_ICON_SIZE = 22
FOCUS_COMMAND_TIMEOUT_S = 3

TMUX_PANE_PATTERN = re.compile(r"^%\d+$")
KITTY_LISTEN_PATTERN = re.compile(r"^unix:[@/][\w./@-]+$")


def layer_window(app: Gtk.Application, edge: LayerShell.Edge, margin: int) -> Gtk.Window:
    window = Gtk.Window(application=app)
    window.set_resizable(False)
    # No client-side decoration: its shadow margins would inflate the layer surface.
    window.set_decorated(False)
    window.add_css_class("osd-window")  # not "osd": GTK styles that class itself

    LayerShell.init_for_window(window)
    LayerShell.set_namespace(window, "osd")
    LayerShell.set_layer(window, LayerShell.Layer.OVERLAY)
    LayerShell.set_anchor(window, edge, True)
    LayerShell.set_margin(window, edge, margin)
    LayerShell.set_keyboard_mode(window, LayerShell.KeyboardMode.NONE)
    return window


def volume_icon(level: int) -> str:
    if level == 0:
        return "volume-mute.svg"
    if level < 34:
        return "volume-low.svg"
    if level < 67:
        return "volume-mid.svg"
    return "volume-high.svg"


def parse_banner(raw: str) -> dict:
    """Validate a banner command; the focus fields end up in shell and Lua, so they are strict."""
    banner = json.loads(raw)
    if not isinstance(banner, dict):
        raise ValueError("banner is not an object")
    for key in ("session", "title", "message"):
        if not isinstance(banner.get(key), str) or not banner[key]:
            raise ValueError(f"banner.{key} missing")
    banner["sticky"] = bool(banner.get("sticky", False))
    for key in ("kitty_pid", "kitty_window"):
        if key in banner and not str(banner[key]).isdigit():
            raise ValueError(f"banner.{key} is not a number")
    if "kitty_listen" in banner and not KITTY_LISTEN_PATTERN.match(str(banner["kitty_listen"])):
        raise ValueError("banner.kitty_listen is not a unix socket address")
    if "tmux_pane" in banner and not TMUX_PANE_PATTERN.match(str(banner["tmux_pane"])):
        raise ValueError("banner.tmux_pane is not a pane id")
    return banner


def focus_commands(target: dict) -> list[list[str]]:
    commands = []
    if "kitty_pid" in target:
        commands.append(["hyprctl", "dispatch", f'hl.dsp.focus({{ window = "pid:{target["kitty_pid"]}" }})'])
    if "kitty_listen" in target and "kitty_window" in target:
        commands.append([
            "kitten", "@", "--to", target["kitty_listen"],
            "focus-window", "--match", f"id:{target['kitty_window']}",
        ])
    if "tmux_pane" in target:
        commands.append(["tmux", "select-window", "-t", target["tmux_pane"]])
        commands.append(["tmux", "select-pane", "-t", target["tmux_pane"]])
    return commands


def focus_session(target: dict):
    for command in focus_commands(target):
        try:
            result = subprocess.run(command, capture_output=True, text=True, timeout=FOCUS_COMMAND_TIMEOUT_S)
        except (OSError, subprocess.TimeoutExpired) as error:
            print(f"osd: {command[0]} failed: {error}", file=sys.stderr)
            continue
        if result.returncode != 0:
            print(f"osd: {' '.join(command)} exited {result.returncode}: {result.stderr.strip()}", file=sys.stderr)


def build_banner_row(banner: dict) -> Gtk.Box:
    row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=ROW_SPACING)
    row.add_css_class("island")
    row.add_css_class("banner")
    if banner["sticky"]:
        row.add_css_class("attention")

    icon = Gtk.Image(valign=Gtk.Align.CENTER)
    icon.set_pixel_size(BANNER_ICON_SIZE)
    icon.set_from_file(os.path.join(ICON_DIR, "bell-dot.svg" if banner["sticky"] else "bell.svg"))

    title = Gtk.Label(label=banner["title"], xalign=0, max_width_chars=BANNER_MAX_CHARS, single_line_mode=True)
    title.set_ellipsize(Pango.EllipsizeMode.END)
    title.add_css_class("title")
    message = Gtk.Label(label=banner["message"], xalign=0, max_width_chars=BANNER_MAX_CHARS, single_line_mode=True)
    message.set_ellipsize(Pango.EllipsizeMode.END)
    message.add_css_class("message")
    text = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, valign=Gtk.Align.CENTER)
    text.append(title)
    text.append(message)

    row.append(icon)
    row.append(text)
    return row


class BannerStack:
    def __init__(self, app: Gtk.Application):
        self.window = layer_window(app, LayerShell.Edge.TOP, 15)
        self.column = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=BANNER_SPACING)
        self.window.set_child(self.column)
        self.rows: dict[str, Gtk.Box] = {}
        self.targets: dict[str, dict] = {}
        self.timers: dict[str, int] = {}

    def raise_banner(self, banner: dict):
        session = banner["session"]
        self.drop(session)
        row = build_banner_row(banner)
        click = Gtk.GestureClick()
        click.connect("released", lambda *_: self.activate(session))
        row.add_controller(click)
        self.column.append(row)
        self.rows[session] = row
        self.targets[session] = banner
        if not banner["sticky"]:
            self.timers[session] = GLib.timeout_add(BANNER_HIDE_AFTER_MS, self.expire, session)
        self.window.present()

    def expire(self, session: str):
        self.timers.pop(session, None)
        self.drop(session)
        return GLib.SOURCE_REMOVE

    def drop(self, session: str):
        row = self.rows.pop(session, None)
        if row is None:
            return
        timer = self.timers.pop(session, 0)
        if timer:
            GLib.source_remove(timer)
        self.targets.pop(session, None)
        self.column.remove(row)
        if not self.rows:
            self.window.set_visible(False)

    def activate(self, session: str):
        focus_session(self.targets.get(session, {}))
        self.drop(session)


class Osd:
    def __init__(self, app: Gtk.Application):
        self.window = layer_window(app, LayerShell.Edge.BOTTOM, 15)
        self.window.set_default_size(WIDTH, HEIGHT)
        self.window.set_size_request(WIDTH, HEIGHT)
        self.banners = BannerStack(app)

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
        command, _, rest = line.partition(" ")
        if not command:
            return
        if command == "banner":
            try:
                self.banners.raise_banner(parse_banner(rest))
            except ValueError as error:
                print(f"osd: bad banner {rest!r}: {error}", file=sys.stderr)
            return
        if command == "banner-clear":
            self.banners.drop(rest.strip())
            return
        try:
            level = max(0, min(100, int(rest))) if rest else 0
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
