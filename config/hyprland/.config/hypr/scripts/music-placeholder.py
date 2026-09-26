#!/usr/bin/env python3
"""Loading placeholder for the Apple Music scratchpad.

A cold start of the Zen "music" profile takes a few seconds before it has a window, and
Hyprland closes an empty special workspace, so nothing would show until then. This window
maps in a fraction of a second with a spinner; scratchpads.lua spawns it next to
music-launch.sh, shows the special workspace when it opens, and closes it once the Zen
window is there. It quits by itself after TIMEOUT_SECONDS in case that never happens.
"""

import os
import signal
import sys

import gi  # noqa: E402

gi.require_version("Gdk", "4.0")
gi.require_version("Gtk", "4.0")
from gi.repository import Gdk, Gio, GLib, Gtk  # noqa: E402

CONFIG_DIR = os.path.dirname(os.path.realpath(__file__))
TIMEOUT_SECONDS = 60


def main():
    app = Gtk.Application(application_id="dotfiles.music-placeholder", flags=Gio.ApplicationFlags.NON_UNIQUE)

    def on_activate(application):
        provider = Gtk.CssProvider()
        provider.load_from_path(os.path.join(CONFIG_DIR, "music-placeholder.css"))
        Gtk.StyleContext.add_provider_for_display(
            Gdk.Display.get_default(), provider, Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION
        )

        window = Gtk.Window(application=application, title="Apple Music")
        window.add_css_class("music-placeholder")
        window.set_default_size(1400, 900)

        spinner = Gtk.Spinner(spinning=True)
        spinner.set_size_request(40, 40)
        label = Gtk.Label(label="Apple Music")
        label.add_css_class("title")
        box = Gtk.Box(
            orientation=Gtk.Orientation.VERTICAL, spacing=16, halign=Gtk.Align.CENTER, valign=Gtk.Align.CENTER
        )
        box.append(spinner)
        box.append(label)
        window.set_child(box)
        window.present()

        GLib.timeout_add_seconds(TIMEOUT_SECONDS, application.quit)

    app.connect("activate", on_activate)
    GLib.unix_signal_add(GLib.PRIORITY_DEFAULT, signal.SIGTERM, app.quit)
    sys.exit(app.run(None))


if __name__ == "__main__":
    main()
