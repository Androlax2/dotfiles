//! Loading placeholder for a web scratchpad: web-placeholder <name> <title>.
//!
//! A cold start of a Zen profile takes a few seconds before it has a window, and Hyprland closes an
//! empty special workspace, so nothing would show until then. This window maps in a fraction of a
//! second with a spinner; scratchpads.lua spawns it next to web-launch.sh, shows the special
//! workspace when it opens, and closes it once the Zen window is there (its class is
//! dotfiles.<name>-placeholder). It quits by itself after TIMEOUT in case that never happens.

use gtk4::prelude::*;
use gtk4::{Align, Application, Box as GtkBox, CssProvider, Label, Orientation, Spinner, Window, gdk, gio, glib};
use std::time::Duration;

const TIMEOUT: Duration = Duration::from_secs(60);
const SPINNER_SIZE: i32 = 40;
const SIGTERM: i32 = 15;

fn activate(app: &Application, title: &str) {
    let provider = CssProvider::new();
    provider.load_from_path(cockpit::paths::home().join(".config/hypr/scripts/web-placeholder.css"));
    gtk4::style_context_add_provider_for_display(&gdk::Display::default().expect("a display"), &provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);

    let window = Window::builder().application(app).title(title).default_width(1400).default_height(900).build();
    window.add_css_class("web-placeholder");
    let spinner = Spinner::builder().spinning(true).width_request(SPINNER_SIZE).height_request(SPINNER_SIZE).build();
    let label = Label::new(Some(title));
    label.add_css_class("title");
    let column = GtkBox::builder().orientation(Orientation::Vertical).spacing(16).halign(Align::Center).valign(Align::Center).build();
    column.append(&spinner);
    column.append(&label);
    window.set_child(Some(&column));
    window.present();

    let app = app.clone();
    glib::timeout_add_local_once(TIMEOUT, move || app.quit());
}

fn main() -> glib::ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let [name, title] = arguments.as_slice() else {
        eprintln!("usage: web-placeholder <name> <title>");
        return glib::ExitCode::from(2);
    };
    let app = Application::builder().application_id(format!("dotfiles.{name}-placeholder")).flags(gio::ApplicationFlags::NON_UNIQUE).build();
    let title = title.clone();
    app.connect_activate(move |app| activate(app, &title));
    let quitting = app.clone();
    glib_unix::unix_signal_add_local_once(SIGTERM, move || quitting.quit());
    app.run_with_args::<&str>(&[])
}
