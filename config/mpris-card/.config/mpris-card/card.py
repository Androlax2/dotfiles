#!/usr/bin/env python3
"""Now-playing card: cover art, titles, progress and prev/play/next for the last
active MPRIS player (via playerctld). A GTK4 layer-shell window under the bar's
left islands, toggled by clicking the bar's now-playing island.

    card.py           run the daemon (autostart): follows the player, window hidden
    card.py toggle    show or hide the card through the running daemon (instant);
                      starts the daemon if it is not running
"""
import ctypes
import os
import signal
import stat
import subprocess
import sys

ctypes.CDLL("libgtk4-layer-shell.so", mode=ctypes.RTLD_GLOBAL)

import gi  # noqa: E402

gi.require_version("Gdk", "4.0")
gi.require_version("Gtk", "4.0")
gi.require_version("Gtk4LayerShell", "1.0")
from gi.repository import Gdk, Gio, GLib, Gtk, Gtk4LayerShell as LayerShell  # noqa: E402

CONFIG_DIR = os.path.dirname(os.path.realpath(__file__))
RUNTIME_DIR = os.environ.get("XDG_RUNTIME_DIR", "/tmp")
PID_FILE = os.path.join(RUNTIME_DIR, "mpris-card.pid")
FIFO_PATH = os.path.join(RUNTIME_DIR, "mpris-card.fifo")
PLAYER = ["playerctl", "--player=playerctld"]
FIELDS = "{{status}}\t{{artist}}\t{{title}}\t{{album}}\t{{mpris:artUrl}}\t{{mpris:length}}"
COVER_SIZE = 120
# Under the bar (32 + 6) plus the 6 px gap, left-aligned with the bar's now-playing
# island (15 px margin + the five-chip workspaces island + 8 px).
CARD_TOP, CARD_LEFT = 44, 160
ICONS = {"prev": "󰒮", "play": "󰐊", "pause": "󰏤", "next": "󰒭"}


def format_time(microseconds: float) -> str:
    seconds = int(microseconds // 1_000_000)
    return f"{seconds // 60}:{seconds % 60:02d}"


class Card:
    def __init__(self, app: Gtk.Application):
        self.window = Gtk.Window(application=app)
        self.window.add_css_class("card-window")
        self.window.set_decorated(False)
        self.window.remove_css_class("background")  # the theme paints .background opaque
        # The layer covers the whole output, transparent except for the card, so a
        # click anywhere outside the card closes it (like a macOS popover).
        LayerShell.init_for_window(self.window)
        LayerShell.set_namespace(self.window, "mpris-card")
        LayerShell.set_layer(self.window, LayerShell.Layer.TOP)
        for edge in (LayerShell.Edge.TOP, LayerShell.Edge.LEFT, LayerShell.Edge.RIGHT, LayerShell.Edge.BOTTOM):
            LayerShell.set_anchor(self.window, edge, True)
        LayerShell.set_exclusive_zone(self.window, -1)
        LayerShell.set_keyboard_mode(self.window, LayerShell.KeyboardMode.NONE)

        self.cover = Gtk.Picture(content_fit=Gtk.ContentFit.COVER)
        self.cover.set_size_request(COVER_SIZE, COVER_SIZE)
        cover_frame = Gtk.Box()
        cover_frame.add_css_class("cover")
        cover_frame.set_overflow(Gtk.Overflow.HIDDEN)
        cover_frame.append(self.cover)

        self.title = self.label("title")
        self.artist = self.label("artist")
        self.album = self.label("album")

        # Drawn, not laid out: a box-based fill needs the track's allocation, which is
        # 0 until the first frame, so the line showed empty until the next tick.
        self.fraction = 0.0
        self.track = Gtk.DrawingArea(hexpand=True, valign=Gtk.Align.CENTER, content_height=2)
        self.track.add_css_class("progress")
        self.track.set_draw_func(self.draw_progress)
        self.elapsed = self.label("time")
        self.remaining = self.label("time")
        self.times = Gtk.Box(spacing=8)
        self.times.append(self.elapsed)
        self.times.append(self.track)
        self.times.append(self.remaining)

        self.play_button = self.button("play", "play-pause")
        controls = Gtk.Box(spacing=4, halign=Gtk.Align.CENTER)
        controls.append(self.button("prev", "previous"))
        controls.append(self.play_button)
        controls.append(self.button("next", "next"))

        text = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=2, hexpand=True, valign=Gtk.Align.CENTER)
        for widget in (self.title, self.artist, self.album):
            text.append(widget)
        text.append(Gtk.Box(vexpand=True))
        text.append(self.times)
        text.append(controls)

        self.island = Gtk.Box(spacing=14)
        self.island.add_css_class("island")
        self.island.append(cover_frame)
        self.island.append(text)
        canvas = Gtk.Fixed()
        canvas.put(self.island, CARD_LEFT, CARD_TOP)
        self.window.set_child(canvas)

        click = Gtk.GestureClick(button=0)
        click.connect("pressed", self.on_pressed)
        self.window.add_controller(click)

        self.length = 0
        self.status = ""
        self.art_url = None
        self.textures: dict[str, Gdk.Texture] = {}

        self.position_timer = 0

    def draw_progress(self, area, context, width, height):
        # Line colour comes from the CSS `color` of .progress; the track is its 16 % tint.
        colour = area.get_color()
        context.set_source_rgba(colour.red, colour.green, colour.blue, 0.16)
        context.rectangle(0, 0, width, height)
        context.fill()
        context.set_source_rgba(colour.red, colour.green, colour.blue, 1)
        context.rectangle(0, 0, width * self.fraction, height)
        context.fill()

    def on_pressed(self, _gesture, _n_press, x, y):
        picked = self.window.pick(x, y, Gtk.PickFlags.DEFAULT)
        if picked is None or not (picked == self.island or picked.is_ancestor(self.island)):
            self.hide()

    def show(self):
        self.window.present()
        self.update_position()  # after present: the position poll skips a hidden window
        if not self.position_timer:
            self.position_timer = GLib.timeout_add(1000, self.update_position)

    def hide(self):
        self.window.set_visible(False)
        if self.position_timer:
            GLib.source_remove(self.position_timer)
            self.position_timer = 0

    def toggle(self):
        if self.window.get_visible():
            self.hide()
        else:
            self.show()

    @staticmethod
    def label(css_class: str) -> Gtk.Label:
        label = Gtk.Label(xalign=0, ellipsize=3, max_width_chars=28)  # 3 = PANGO_ELLIPSIZE_END
        label.add_css_class(css_class)
        return label

    @staticmethod
    def button(icon: str, command: str) -> Gtk.Button:
        button = Gtk.Button(label=ICONS[icon], has_frame=False, can_focus=False)
        button.add_css_class("control")
        button.connect("clicked", lambda *_: subprocess.Popen(PLAYER + [command]))
        return button

    def update(self, line: str):
        status, artist, title, album, art_url, length = (line.split("\t") + [""] * 6)[:6]
        self.status = status
        self.title.set_text(title or "Nothing playing")
        self.artist.set_text(artist)
        self.album.set_text(album)
        self.album.set_visible(bool(album))
        self.play_button.set_label(ICONS["pause"] if status == "Playing" else ICONS["play"])
        # Firefox reports a 1 s length and no position for web players (Apple Music):
        # the progress row only shows when the length is real.
        self.length = int(length) if length.isdigit() and int(length) > 2_000_000 else 0
        self.times.set_visible(bool(self.length))
        self.remaining.set_text(format_time(self.length) if self.length else "")
        self.load_art(art_url)
        self.update_position()

    def load_art(self, url: str):
        if url == self.art_url:
            return
        self.art_url = url
        if not url:
            self.cover.set_paintable(None)
            return
        if url in self.textures:
            self.cover.set_paintable(self.textures[url])
            return

        def loaded(file, result):
            try:
                _, contents, _ = file.load_contents_finish(result)
                texture = Gdk.Texture.new_from_bytes(GLib.Bytes.new(contents))
            except GLib.Error as error:
                print(f"mpris-card: cover {url}: {error.message}", file=sys.stderr)
                return
            self.textures[url] = texture
            if url == self.art_url:
                self.cover.set_paintable(texture)

        Gio.File.new_for_uri(url).load_contents_async(None, loaded)

    def update_position(self):
        if not self.length or not self.window.get_visible():
            return True
        try:
            position = float(subprocess.run(PLAYER + ["position"], capture_output=True, text=True, timeout=1).stdout or 0)
        except (ValueError, subprocess.TimeoutExpired):
            return True
        microseconds = position * 1_000_000
        self.elapsed.set_text(format_time(microseconds))
        self.fraction = min(1.0, microseconds / self.length)
        self.track.queue_draw()
        return True


def daemon_running() -> bool:
    try:
        with open(PID_FILE) as handle:
            os.kill(int(handle.read()), 0)
        return stat.S_ISFIFO(os.stat(FIFO_PATH).st_mode)
    except (FileNotFoundError, ValueError, ProcessLookupError, PermissionError):
        return False


def open_fifo() -> int:
    if os.path.exists(FIFO_PATH) and not stat.S_ISFIFO(os.stat(FIFO_PATH).st_mode):
        os.remove(FIFO_PATH)
    if not os.path.exists(FIFO_PATH):
        os.mkfifo(FIFO_PATH, 0o600)
    return os.open(FIFO_PATH, os.O_RDWR | os.O_NONBLOCK)  # read-write: never sees EOF


def main():
    want_toggle = len(sys.argv) > 1 and sys.argv[1] == "toggle"
    if want_toggle and daemon_running():
        with open(FIFO_PATH, "w") as fifo:
            fifo.write("toggle\n")
        return
    with open(PID_FILE, "w") as handle:
        handle.write(str(os.getpid()))

    app = Gtk.Application(application_id="dotfiles.mpris-card", flags=Gio.ApplicationFlags.NON_UNIQUE)

    def on_activate(application):
        provider = Gtk.CssProvider()
        provider.load_from_path(os.path.join(CONFIG_DIR, "style.css"))
        Gtk.StyleContext.add_provider_for_display(
            Gdk.Display.get_default(), provider, Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION
        )
        card = Card(application)
        application.hold()

        follow = subprocess.Popen(PLAYER + ["--follow", "metadata", "--format", FIELDS], stdout=subprocess.PIPE, text=True)

        def on_line(_source, _condition):
            line = follow.stdout.readline()
            if line == "":
                return GLib.SOURCE_REMOVE
            card.update(line.rstrip("\n"))
            return GLib.SOURCE_CONTINUE

        GLib.io_add_watch(follow.stdout, GLib.PRIORITY_DEFAULT, GLib.IOCondition.IN, on_line)

        fifo_fd = open_fifo()

        def on_command(_fd, _condition):
            for command in os.read(fifo_fd, 4096).decode(errors="replace").split():
                if command == "toggle":
                    card.toggle()
            return GLib.SOURCE_CONTINUE

        GLib.io_add_watch(fifo_fd, GLib.PRIORITY_DEFAULT, GLib.IOCondition.IN, on_command)
        if want_toggle:
            card.show()

    GLib.unix_signal_add(GLib.PRIORITY_DEFAULT, signal.SIGTERM, app.quit)
    app.connect("activate", on_activate)
    app.run(None)
    for path in (PID_FILE, FIFO_PATH):
        try:
            os.remove(path)
        except FileNotFoundError:
            pass


if __name__ == "__main__":
    main()
