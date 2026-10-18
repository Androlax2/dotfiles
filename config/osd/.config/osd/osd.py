#!/usr/bin/env python3
"""On-screen display: a volume/brightness island and the Claude banners.

Bottom centre: a 320x36 island for volume, mute and brightness changes.
Top centre: a stack of banners, one per Claude Code session. Both are GTK4 + layer
shell windows, namespace "osd".

Reads one command per line from $XDG_RUNTIME_DIR/osd.fifo, written by osd-show and
by the Claude Code hook ~/.claude/hooks/session-state.sh:

    volume <0-100>          yellow line, icon by level
    mute                    mute icon, empty line
    brightness <0-100>      blue line, sun icon
    session <json>          a Claude session's state report (claude_registry.py)
    sessions-clear          forget every session (claude-sessions reset)
    focus-next              show the most urgent Claude session in the manager
    run-request <id>        Claude asks to run a command outside the sandbox (ask-to-run): a banner,
                            and claude-run-approve in a tmux session shown in the cockpit, and in a
                            floating window while it needs Theo
    run-phase <id> <phase>  claude-run-approve's asking / running / done
    guide-request <id>      steps for Theo to do by hand (ask-to-guide): a banner, and claude-guide
                            in a floating window of its own
    notice-clear <id>       drop the banner a notice raised
    notice <json>           a sticky banner of its own ({"id", "title", "message"}); a click
                            dismisses it (the session manager's budget alerts)

The session commands feed the registry in claude_registry.py, which raises and drops
the banners here. Clicking a banner shows that session in the manager
(`claude-sessions show`) and drops the banner. The island hides 1.2 s after the
last command. Icons are the bar's SVGs.
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
from gi.repository import Gdk, GLib, Gtk, Gtk4LayerShell as LayerShell, Pango  # noqa: E402

from claude_registry import NOTICE_ID, SessionRegistry, parse_notice, parse_session, show_in_manager  # noqa: E402

CONFIG_DIR = os.path.dirname(os.path.realpath(__file__))
ICON_DIR = os.path.expanduser("~/.config/waybar/icons")
RUNTIME_DIR = os.environ.get("XDG_RUNTIME_DIR", "/tmp")
FIFO_PATH = os.path.join(RUNTIME_DIR, "osd.fifo")
HIDE_AFTER_MS = 1200
WIDTH, HEIGHT = 320, 36
ISLAND_PADDING, ICON_SIZE, ROW_SPACING = 14, 18, 12
TRACK_WIDTH = WIDTH - 2 * ISLAND_PADDING - ICON_SIZE - ROW_SPACING - 2  # 2 = the hairline border

BANNER_HIDE_AFTER_MS = 6000
BANNER_MAX_CHARS = 64
BANNER_SPACING = 8
BANNER_ICON_SIZE = 22


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
        self.column.remove(row)
        if not self.rows:
            self.window.set_visible(False)

    def showing(self, session: str) -> bool:
        return session in self.rows

    def activate(self, session: str):
        show_in_manager(session)
        self.drop(session)

    def raise_notice(self, notice: dict):
        """A sticky banner that is not a session (budget alerts); a click dismisses it. Shown on
        every workspace, the Claude one included."""
        key = f"notice:{notice['id']}"
        self.drop(key)
        row = build_banner_row({"title": notice["title"], "message": notice["message"], "sticky": True})
        click = Gtk.GestureClick()
        click.connect("released", lambda *_: self.drop(key))
        row.add_controller(click)
        self.column.append(row)
        self.rows[key] = row
        self.window.present()


class Osd:
    def __init__(self, app: Gtk.Application):
        self.window = layer_window(app, LayerShell.Edge.BOTTOM, 15)
        self.window.set_default_size(WIDTH, HEIGHT)
        self.window.set_size_request(WIDTH, HEIGHT)
        self.banners = BannerStack(app)
        self.registry = SessionRegistry(self.banners, GLib.timeout_add_seconds)

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
        if command == "session":
            try:
                self.registry.update(parse_session(rest))
            except ValueError as error:
                print(f"osd: bad session {rest!r}: {error}", file=sys.stderr)
            return
        if command == "focus-next":
            self.registry.focus_most_urgent()
            return
        if command == "sessions-clear":
            self.registry.clear()
            return
        if command == "notice":
            try:
                self.banners.raise_notice(parse_notice(rest))
            except ValueError as error:
                print(f"osd: bad notice {rest!r}: {error}", file=sys.stderr)
            return
        if command == "notice-clear":
            if NOTICE_ID.fullmatch(rest.strip()):
                self.banners.drop(f"notice:{rest.strip()}")
            return
        if command == "run-phase":
            parts = rest.split()
            if len(parts) == 2:
                self.registry.runs.phase_changed(parts[0], parts[1])
            return
        if command == "guide-request":
            try:
                self.registry.guides.request(rest.strip())
            except ValueError as error:
                print(f"osd: bad guide request {rest!r}: {error}", file=sys.stderr)
            return
        if command == "run-request":
            try:
                self.registry.runs.request(rest.strip())
            except ValueError as error:
                print(f"osd: bad run request {rest!r}: {error}", file=sys.stderr)
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
