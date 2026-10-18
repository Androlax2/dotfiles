#!/usr/bin/env python3
"""The "still there?" screen of the Claude workspace: claude-idle-prompt.py <seconds>.

A layer-shell window over the whole screen (Hyprland blurs what is behind it, apps/claude.lua)
with a card in the middle: the question, a countdown, and what to do. Any key, click or mouse
movement answers it and the process exits 0; when the countdown runs out it exits 1, which
claude-idle.sh turns into the lock. The window takes the keyboard so the key goes nowhere else.
"""

import os
import signal
import sys

import gi  # noqa: E402

gi.require_version("Gdk", "4.0")
gi.require_version("Gtk", "4.0")
gi.require_version("Gtk4LayerShell", "1.0")
from gi.repository import Gdk, Gio, GLib, Gtk  # noqa: E402
from gi.repository import Gtk4LayerShell as LayerShell  # noqa: E402

CONFIG_DIR = os.path.dirname(os.path.realpath(__file__))
ANSWERED, TIMED_OUT = 0, 1


def main():
    if len(sys.argv) != 2 or not sys.argv[1].isdigit():
        sys.exit("usage: claude-idle-prompt.py <seconds>")
    remaining = {"seconds": int(sys.argv[1])}
    app = Gtk.Application(application_id="dotfiles.claude-idle-prompt", flags=Gio.ApplicationFlags.NON_UNIQUE)
    outcome = {"code": TIMED_OUT}

    def finish(code: int):
        outcome["code"] = code
        app.quit()

    def on_activate(application):
        provider = Gtk.CssProvider()
        provider.load_from_path(os.path.join(CONFIG_DIR, "claude-idle.css"))
        Gtk.StyleContext.add_provider_for_display(
            Gdk.Display.get_default(), provider, Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION
        )

        window = Gtk.Window(application=application, title="Still there?")
        window.add_css_class("claude-idle")
        LayerShell.init_for_window(window)
        LayerShell.set_namespace(window, "claude-idle")
        LayerShell.set_layer(window, LayerShell.Layer.OVERLAY)
        for edge in (LayerShell.Edge.TOP, LayerShell.Edge.BOTTOM, LayerShell.Edge.LEFT, LayerShell.Edge.RIGHT):
            LayerShell.set_anchor(window, edge, True)
        LayerShell.set_keyboard_mode(window, LayerShell.KeyboardMode.EXCLUSIVE)

        countdown = Gtk.Label(label=str(remaining["seconds"]))
        countdown.add_css_class("countdown")
        title = Gtk.Label(label="Still there?")
        title.add_css_class("title")
        hint = Gtk.Label(label="The screen locks when the count reaches zero.\nAny key, click or move keeps the session on screen.")
        hint.add_css_class("hint")
        hint.set_justify(Gtk.Justification.CENTER)
        card = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=18, halign=Gtk.Align.CENTER, valign=Gtk.Align.CENTER)
        card.add_css_class("card")
        for widget in (countdown, title, hint):
            card.append(widget)
        window.set_child(card)

        keys = Gtk.EventControllerKey()
        keys.connect("key-pressed", lambda *_: finish(ANSWERED) or True)
        window.add_controller(keys)
        click = Gtk.GestureClick()
        click.connect("pressed", lambda *_: finish(ANSWERED))
        window.add_controller(click)
        motion = Gtk.EventControllerMotion()
        motion.connect("motion", lambda *_: finish(ANSWERED))
        window.add_controller(motion)

        def tick():
            remaining["seconds"] -= 1
            if remaining["seconds"] <= 0:
                finish(TIMED_OUT)
                return GLib.SOURCE_REMOVE
            countdown.set_label(str(remaining["seconds"]))
            return GLib.SOURCE_CONTINUE

        GLib.timeout_add_seconds(1, tick)
        window.present()

    app.connect("activate", on_activate)
    GLib.unix_signal_add(GLib.PRIORITY_DEFAULT, signal.SIGTERM, lambda: finish(ANSWERED))
    app.run(None)
    sys.exit(outcome["code"])


if __name__ == "__main__":
    main()
