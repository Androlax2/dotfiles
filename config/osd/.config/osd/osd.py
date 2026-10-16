#!/usr/bin/env python3
"""On-screen display: a volume/brightness island, Claude banners and the Claude session registry.

Bottom centre: a 320x36 island for volume, mute and brightness changes.
Top centre: a stack of banners, one per Claude Code session. Both are GTK4 + layer
shell windows, namespace "osd".

Reads one command per line from $XDG_RUNTIME_DIR/osd.fifo, written by osd-show and
by the Claude Code hook ~/.claude/hooks/session-state.sh:

    volume <0-100>          yellow line, icon by level
    mute                    mute icon, empty line
    brightness <0-100>      blue line, sun icon
    session <json>          a Claude session's state report (see below)
    sessions-clear          forget every session (claude-sessions reset)
    focus-next              show the most urgent Claude session in the manager

A session report carries session, project, usually a state (working | waiting |
needs-you | gone; absent for an informational event that leaves the state as it
is), and optionally cwd, task, message, sticky and the focus target of the
session's window: kitty_pid, kitty_window, kitty_listen, tmux_pane. The kitty
fields are only a fallback: claude-auto-retry keeps a session alive in tmux when
its kitty window closes, so the environment the hook sees can name a window that
is gone. The registry asks tmux which client is attached to the pane and reads
that client's kitty identity instead. A message raises a banner (sticky stays
until the session reports again or it is clicked); no message drops it. The registry mirrors every change to
$XDG_RUNTIME_DIR/claude-sessions.json for the waybar chip (signal RTMIN+16) and
the claude-sessions manager, and to Hyprland through `hyprctl eval`
(CLAUDE_WINDOW_STATE in ~/.config/hypr/apps/claude.lua), which colours the
session's terminal border by state. Hooks run inside greywall with no Hyprland or
tmux socket, so all of that has to happen here.

Clicking a banner shows that session in the manager (`claude-sessions show`) and
drops the banner.
The island hides 1.2 s after the last command. Icons are the bar's SVGs.
"""
import ctypes
import json
import os
import re
import stat
import subprocess
import sys
import time

# The layer-shell library must be loaded before GTK opens the Wayland display.
ctypes.CDLL("libgtk4-layer-shell.so", mode=ctypes.RTLD_GLOBAL)

import gi  # noqa: E402

gi.require_version("Gdk", "4.0")
gi.require_version("Gtk", "4.0")
gi.require_version("Gtk4LayerShell", "1.0")
from gi.repository import Gdk, GLib, Gtk, Gtk4LayerShell as LayerShell, Pango  # noqa: E402

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
COMMAND_TIMEOUT_S = 3

SESSIONS_SNAPSHOT_PATH = os.path.join(RUNTIME_DIR, "claude-sessions.json")
WAYBAR_SESSIONS_SIGNAL = "-RTMIN+16"
SESSION_PRUNE_INTERVAL_S = 5
STATE_URGENCY = {"working": 0, "waiting": 1, "needs-you": 2}
TARGET_FIELDS = ("kitty_pid", "kitty_window", "kitty_listen", "tmux_pane")
# What Claude Code prints at the bottom of its pane: while busy, while a permission dialog is open,
# and the input prompt line it shows when idle (also shown while busy, so busy wins over it).
# Busy: the spinner line, "✻ Spinning… (8m 31s · ↓ 29.7k tokens)", or the interrupt hint as a whole
# line. The bare phrases are not enough: they can scroll by inside ordinary conversation text.
BUSY_PATTERNS = (
    re.compile(r"^\s*\S{0,2}\s*[A-Z][a-z]+… \(\d+[smh]", re.MULTILINE),
    re.compile(r"^\s*(?:esc|enter) to interrupt(?: and send)?(?:\s*·.*)?$", re.MULTILINE),
)
# The permission dialog: its question alone on a line, then its first option within a few lines.
# The phrase alone is not enough: it can scroll by inside ordinary conversation text.
PERMISSION_DIALOG = re.compile(r"^[\s│]*Do you want to proceed\?\s*$(?:\n.*){0,3}?\n[\s│❯]*1\.\s*Yes", re.MULTILINE)
PROMPT_LINE = re.compile(r"^\s*❯[ \u00a0]", re.MULTILINE)  # Claude pads its prompt with a no-break space
PANE_TAIL_LINES = 20

TMUX_PANE_PATTERN = re.compile(r"%[0-9]{1,10}")
KITTY_LISTEN_PATTERN = re.compile(r"unix:[@/][A-Za-z0-9_./@-]{1,200}")
ASCII_NUMBER_PATTERN = re.compile(r"[0-9]{1,10}")
SESSION_ID_PATTERN = re.compile(r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}")
MAX_TEXT_LENGTH = 300


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


def validate_focus_target(fields: dict):
    """The focus fields end up in shell arguments and a Lua string, so they are strict."""
    for key in ("kitty_pid", "kitty_window"):
        if key in fields and not ASCII_NUMBER_PATTERN.fullmatch(str(fields[key])):
            raise ValueError(f"{key} is not a number")
    if "kitty_listen" in fields and not KITTY_LISTEN_PATTERN.fullmatch(str(fields["kitty_listen"])):
        raise ValueError("kitty_listen is not a unix socket address")
    if "tmux_pane" in fields and not TMUX_PANE_PATTERN.fullmatch(str(fields["tmux_pane"])):
        raise ValueError("tmux_pane is not a pane id")


def validate_text(key: str, value: str):
    """Session text reaches tmux and the session manager, where #( and #{ are live tmux formats."""
    if len(value) > MAX_TEXT_LENGTH:
        raise ValueError(f"session.{key} is too long")
    if any(ord(character) < 32 or ord(character) == 127 for character in value):
        raise ValueError(f"session.{key} contains control characters")
    if "#(" in value or "#{" in value:
        raise ValueError(f"session.{key} contains a tmux format sequence")


def parse_session(raw: str) -> dict:
    report = json.loads(raw)
    if not isinstance(report, dict):
        raise ValueError("session report is not an object")
    for key in ("session", "project"):
        if not isinstance(report.get(key), str) or not report[key]:
            raise ValueError(f"session.{key} missing")
    if "state" in report and report["state"] not in STATE_URGENCY and report["state"] != "gone":
        raise ValueError(f"session.state {report['state']!r} unknown")
    for key in ("cwd", "task", "message"):
        if key in report and not isinstance(report[key], str):
            raise ValueError(f"session.{key} is not a string")
    if not SESSION_ID_PATTERN.fullmatch(report["session"]):
        raise ValueError("session is not a Claude session id")
    for key in ("session", "project", "cwd", "task", "message"):
        if key in report:
            validate_text(key, report[key])
    report["sticky"] = bool(report.get("sticky", False))
    validate_focus_target(report)
    return report


def tmux_output(arguments: list[str]):
    try:
        result = subprocess.run(["tmux", *arguments], capture_output=True, text=True, timeout=COMMAND_TIMEOUT_S)
    except (OSError, subprocess.TimeoutExpired):
        return None
    if result.returncode != 0:
        return None
    return result.stdout


def process_environment(pid: str) -> dict[str, str]:
    try:
        with open(f"/proc/{pid}/environ", "rb") as environ:
            entries = environ.read().split(b"\0")
    except OSError:
        return {}
    pairs = (entry.decode(errors="replace").partition("=") for entry in entries if entry)
    return {name: value for name, _, value in pairs}


def process_is_alive(pid: str) -> bool:
    try:
        os.kill(int(pid), 0)
    except ProcessLookupError:
        return False
    except PermissionError:
        pass
    return True


def without_dead_kitty(target: dict) -> dict:
    """A KITTY_PID inherited from a tmux server's environment can name a long closed terminal."""
    if target.get("kitty_pid") and not process_is_alive(target["kitty_pid"]):
        return {key: value for key, value in target.items() if not key.startswith("kitty_")}
    return target


def claude_panes() -> set[str]:
    """The tmux panes in which a claude process is running right now, read from their environment."""
    try:
        result = subprocess.run(["pgrep", "-x", "claude"], capture_output=True, text=True, timeout=COMMAND_TIMEOUT_S)
    except (OSError, subprocess.TimeoutExpired):
        return set()
    panes = set()
    for pid in result.stdout.split():
        pane = process_environment(pid).get("TMUX_PANE")
        if pane:
            panes.add(pane)
    return panes


def pane_state(tail: str):
    """needs-you on a permission dialog, working on the spinner, waiting at the idle prompt, else None."""
    if PERMISSION_DIALOG.search(tail):
        return "needs-you"
    if any(pattern.search(tail) for pattern in BUSY_PATTERNS):
        return "working"
    if PROMPT_LINE.search(tail):
        return "waiting"
    return None


def resolve_target(target: dict) -> dict:
    """The kitty window currently attached to the session's tmux pane, or the reported one.

    The attached tmux client inherited KITTY_* from the shell of the window it runs in, so
    its environment names the live window. With no client attached yet (the launcher
    attaches right after creating the session) the hook's values are the right ones."""
    tmux_pane = target.get("tmux_pane")
    if not tmux_pane:
        return without_dead_kitty(target)
    session_name = tmux_output(["display-message", "-p", "-t", tmux_pane, "#{session_name}"])
    if not session_name:
        return without_dead_kitty(target)
    client_pids = (tmux_output(["list-clients", "-t", session_name.strip(), "-F", "#{client_pid}"]) or "").split()
    for client_pid in client_pids:
        environment = process_environment(client_pid)
        if not environment.get("KITTY_PID") or not process_is_alive(environment["KITTY_PID"]):
            continue
        resolved = {"tmux_pane": tmux_pane, "kitty_pid": environment["KITTY_PID"]}
        for key, variable in (("kitty_window", "KITTY_WINDOW_ID"), ("kitty_listen", "KITTY_LISTEN_ON")):
            if environment.get(variable):
                resolved[key] = environment[variable]
        try:
            validate_focus_target(resolved)
        except ValueError as error:
            print(f"osd: ignoring tmux client {client_pid}: {error}", file=sys.stderr)
            continue
        return resolved
    return without_dead_kitty(target)


def run_command(command: list[str], ok_exit_codes=(0,)) -> bool:
    try:
        result = subprocess.run(command, capture_output=True, text=True, timeout=COMMAND_TIMEOUT_S)
    except (OSError, subprocess.TimeoutExpired) as error:
        print(f"osd: {command[0]} failed: {error}", file=sys.stderr)
        return False
    if result.returncode not in ok_exit_codes:
        output = (result.stdout + result.stderr).strip()
        print(f"osd: {' '.join(command)} exited {result.returncode}: {output}", file=sys.stderr)
        return False
    return True


def hyprctl_eval(expression: str) -> bool:
    return run_command(["hyprctl", "eval", expression])


def show_in_manager(session: str) -> bool:
    return run_command(["claude-sessions", "show", session])


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

    def activate(self, session: str):
        show_in_manager(session)
        self.drop(session)


class SessionRegistry:
    """Every live Claude session, keyed by session id, mirrored to the snapshot file and to Hyprland."""

    def __init__(self, banners: BannerStack):
        self.banners = banners
        self.sessions: dict[str, dict] = {}
        self.load_snapshot()
        GLib.timeout_add_seconds(SESSION_PRUNE_INTERVAL_S, self.prune_dead)

    def update(self, report: dict):
        session = report["session"]
        if report.get("state") == "gone":
            self.remove(session)
            return
        entry = self.sessions.get(session)
        previous_state = entry["state"] if entry else None
        if entry is None:
            entry = {"state": report.get("state", "waiting"), "project": report["project"], "cwd": "", "task": "",
                     "since": time.time(), "target": {}}
            self.sessions[session] = entry
            changed = True
        else:
            changed = False
        for key in ("state", "project", "cwd", "task"):
            if key in report and entry[key] != report[key]:
                entry[key] = report[key]
                changed = True
        previous_pid = entry["target"].get("kitty_pid")
        reported_target = {key: str(report[key]) for key in TARGET_FIELDS if key in report}
        entry["target"] = resolve_target({**entry["target"], **reported_target})
        kitty_pid = entry["target"].get("kitty_pid")
        self.drop_pane_predecessors(session, entry["target"].get("tmux_pane"))
        if previous_state != entry["state"]:
            entry["since"] = time.time()

        if "message" in report:
            self.banners.raise_banner({"session": session, "title": f"Claude · {entry['project']}",
                                       "message": report["message"], "sticky": report["sticky"]})
        else:
            self.banners.drop(session)

        # The window a session moved away from (tmux session re-attached elsewhere) loses its
        # tag unless another session still lives there; the new one is tagged.
        window_changed = kitty_pid != previous_pid
        if window_changed and previous_pid:
            self.apply_window_state(previous_pid)
        if kitty_pid and (window_changed or previous_state != entry["state"]):
            self.apply_window_state(kitty_pid)
        if changed or window_changed:
            self.publish()

    def drop_pane_predecessors(self, session: str, tmux_pane):
        """A pane runs one Claude at a time, but /clear, --resume and forks give it a new session
        id without a SessionEnd for the old one: the previous ids of the same pane are gone."""
        if not tmux_pane:
            return
        for other, entry in list(self.sessions.items()):
            if other != session and entry["target"].get("tmux_pane") == tmux_pane:
                print(f"osd: session {other} ({entry['project']}) replaced in {tmux_pane} by {session}", file=sys.stderr)
                self.remove(other)

    def remove(self, session: str):
        entry = self.sessions.pop(session, None)
        if entry is None:
            return
        self.banners.drop(session)
        kitty_pid = entry["target"].get("kitty_pid")
        if kitty_pid:
            self.apply_window_state(kitty_pid)
        self.publish()

    def window_state(self, kitty_pid: str):
        """Most urgent state among the sessions sharing one kitty window (several tmux windows)."""
        states = [entry["state"] for entry in self.sessions.values() if entry["target"].get("kitty_pid") == kitty_pid]
        if not states:
            return None
        return max(states, key=STATE_URGENCY.__getitem__)

    def apply_window_state(self, kitty_pid: str):
        state = self.window_state(kitty_pid)
        hyprctl_eval(f"CLAUDE_WINDOW_STATE({kitty_pid}, {json.dumps(state) if state else 'nil'})")

    def publish(self):
        rows = [
            {"session": session, "state": entry["state"], "project": entry["project"], "cwd": entry["cwd"],
             "task": entry["task"], "since": int(entry["since"]), **entry["target"]}
            for session, entry in self.sessions.items()
        ]
        rows.sort(key=lambda row: (-STATE_URGENCY[row["state"]], row["since"]))
        temporary_path = SESSIONS_SNAPSHOT_PATH + ".tmp"
        with open(temporary_path, "w") as snapshot:
            json.dump({"sessions": rows}, snapshot)
        os.replace(temporary_path, SESSIONS_SNAPSHOT_PATH)
        # Exit 1 is "no waybar running", not an error.
        run_command(["pkill", WAYBAR_SESSIONS_SIGNAL, "waybar"], ok_exit_codes=(0, 1))

    def load_snapshot(self):
        """A daemon restart keeps the sessions; the next hook event refreshes their state."""
        try:
            with open(SESSIONS_SNAPSHOT_PATH) as snapshot:
                rows = json.load(snapshot)["sessions"]
        except FileNotFoundError:
            return
        except (OSError, ValueError, KeyError, TypeError) as error:
            print(f"osd: ignoring snapshot {SESSIONS_SNAPSHOT_PATH}: {error}", file=sys.stderr)
            return
        for row in rows:
            try:
                target = {key: str(row[key]) for key in TARGET_FIELDS if row.get(key)}
                validate_focus_target(target)
                if row["state"] not in STATE_URGENCY:
                    raise ValueError(f"state {row['state']!r} unknown")
                self.sessions[row["session"]] = {
                    "state": row["state"], "project": row["project"], "cwd": row.get("cwd", ""),
                    "task": row.get("task", ""), "since": row.get("since", time.time()),
                    "target": resolve_target(target),
                }
            except (KeyError, TypeError, ValueError) as error:
                print(f"osd: skipping snapshot row {row!r}: {error}", file=sys.stderr)
        self.prune_dead()
        for kitty_pid in {entry["target"].get("kitty_pid") for entry in self.sessions.values()} - {None}:
            self.apply_window_state(kitty_pid)
        self.publish()

    def is_dead(self, target: dict, live_panes: set[str]) -> bool:
        """A session with a pane is alive while a claude process runs in that pane (pane ids
        outlive their programs: a respawned pane keeps its id). Without a pane, the terminal decides."""
        tmux_pane = target.get("tmux_pane")
        if tmux_pane:
            return tmux_pane not in live_panes
        kitty_pid = target.get("kitty_pid")
        return bool(kitty_pid) and not process_is_alive(kitty_pid)

    def prune_dead(self):
        live_panes = claude_panes()
        for session, entry in list(self.sessions.items()):
            if self.is_dead(entry["target"], live_panes):
                print(f"osd: session {session} ({entry['project']}) is gone, dropping it", file=sys.stderr)
                self.remove(session)
        self.reconcile_with_panes()
        return GLib.SOURCE_CONTINUE

    def reconcile_with_panes(self):
        """Hooks can mislead or go missing: "needs you" fires on a permission dialog and nothing follows
        until the tool ends (in auto mode the classifier's thinking time trips it too), and a Stop that
        fires while the daemon restarts is lost. The pane's last lines say what Claude is doing now."""
        for session, entry in list(self.sessions.items()):
            tmux_pane = entry["target"].get("tmux_pane")
            if not tmux_pane:
                continue
            tail = tmux_output(["capture-pane", "-p", "-t", tmux_pane, "-S", f"-{PANE_TAIL_LINES}"]) or ""
            observed = pane_state(tail)
            if observed and observed != entry["state"]:
                self.update({"session": session, "state": observed, "project": entry["project"], "sticky": False})

    def clear(self):
        for session in list(self.sessions):
            self.remove(session)

    def focus_most_urgent(self):
        """Show the most urgent session in the manager; with none, just open the manager."""
        candidates = [
            (STATE_URGENCY[entry["state"]], -entry["since"], session)
            for session, entry in self.sessions.items() if entry["target"].get("tmux_pane")
        ]
        if not candidates:
            run_command(["claude-sessions", "show"])
            return
        session = max(candidates)[2]
        show_in_manager(session)
        self.banners.drop(session)


class Osd:
    def __init__(self, app: Gtk.Application):
        self.window = layer_window(app, LayerShell.Edge.BOTTOM, 15)
        self.window.set_default_size(WIDTH, HEIGHT)
        self.window.set_size_request(WIDTH, HEIGHT)
        self.banners = BannerStack(app)
        self.registry = SessionRegistry(self.banners)

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
