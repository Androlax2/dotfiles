//! Now-playing card: cover art, titles, progress and prev/play/next for the last active MPRIS
//! player (via playerctld). A GTK4 layer-shell window under the bar's left islands, toggled by
//! clicking the bar's now-playing island.
//!
//!     mpris-card           run the daemon (autostart): follows the player, window hidden
//!     mpris-card toggle    show or hide the card through the running daemon (instant); starts the
//!                          daemon if it is not running

use desktop::mpris::{FIELDS, Track, format_time};
use gtk4::prelude::*;
use gtk4::{Align, Application, Box as GtkBox, Button, CssProvider, DrawingArea, Fixed, GestureClick, Label, Orientation,
           Overflow, PickFlags, Picture, Window, gdk, gdk_pixbuf, gio, glib, pango};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::os::fd::{AsRawFd, FromRawFd};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::rc::Rc;
use std::time::Duration;

const PLAYER: [&str; 2] = ["playerctl", "--player=playerctld"];
const COVER_SIZE: i32 = 120;
// Under the bar (32 + 6) plus the 6 px gap, left-aligned with the bar's now-playing island (15 px
// margin + the five-chip workspaces island + 8 px).
const CARD_TOP: f64 = 44.0;
const CARD_LEFT: f64 = 160.0;
const POSITION_TIMEOUT: Duration = Duration::from_secs(1);
const SIGTERM: i32 = 15;
const ICON_PREVIOUS: &str = "󰒮";
const ICON_PLAY: &str = "󰐊";
const ICON_PAUSE: &str = "󰏤";
const ICON_NEXT: &str = "󰒭";

fn pid_file() -> PathBuf {
    cockpit::paths::runtime_dir().join("mpris-card.pid")
}

fn fifo_path() -> PathBuf {
    cockpit::paths::runtime_dir().join("mpris-card.fifo")
}

fn player_command(arguments: &[&str]) -> Command {
    let mut command = Command::new(PLAYER[0]);
    command.args(&PLAYER[1..]).args(arguments);
    command
}

struct Card {
    window: Window,
    island: GtkBox,
    cover: Picture,
    title: Label,
    artist: Label,
    album: Label,
    track: DrawingArea,
    elapsed: Label,
    remaining: Label,
    times: GtkBox,
    play_button: Button,
    fraction: Cell<f64>,
    length: Cell<Option<u64>>,
    art_url: RefCell<String>,
    textures: RefCell<HashMap<String, gdk::Texture>>,
    position_timer: RefCell<Option<glib::SourceId>>,
}

fn label(css_class: &str) -> Label {
    let label = Label::builder().xalign(0.0).ellipsize(pango::EllipsizeMode::End).max_width_chars(28).build();
    label.add_css_class(css_class);
    label
}

fn control(icon: &str, action: &'static str) -> Button {
    let button = Button::builder().label(icon).has_frame(false).can_focus(false).build();
    button.add_css_class("control");
    button.connect_clicked(move |_| {
        if let Err(error) = player_command(&[action]).spawn() {
            eprintln!("mpris-card: playerctl {action}: {error}");
        }
    });
    button
}

impl Card {
    fn new(app: &Application) -> Rc<Card> {
        let window = Window::builder().application(app).decorated(false).build();
        window.add_css_class("card-window");
        window.remove_css_class("background"); // the theme paints .background opaque
        // The layer covers the whole output, transparent except for the card, so a click anywhere
        // outside the card closes it (like a macOS popover).
        window.init_layer_shell();
        window.set_namespace(Some("mpris-card"));
        window.set_layer(Layer::Top);
        for edge in [Edge::Top, Edge::Left, Edge::Right, Edge::Bottom] {
            window.set_anchor(edge, true);
        }
        window.set_exclusive_zone(-1);
        window.set_keyboard_mode(KeyboardMode::None);

        let cover = Picture::builder().width_request(COVER_SIZE).height_request(COVER_SIZE).build();
        let cover_frame = GtkBox::new(Orientation::Horizontal, 0);
        cover_frame.add_css_class("cover");
        cover_frame.set_overflow(Overflow::Hidden);
        cover_frame.append(&cover);

        // Drawn, not laid out: a box-based fill needs the track's allocation, which is 0 until the
        // first frame, so the line showed empty until the next tick.
        let track = DrawingArea::builder().hexpand(true).valign(Align::Center).content_height(2).build();
        track.add_css_class("progress");
        let times = GtkBox::new(Orientation::Horizontal, 8);
        let (elapsed, remaining) = (label("time"), label("time"));
        times.append(&elapsed);
        times.append(&track);
        times.append(&remaining);

        let play_button = control(ICON_PLAY, "play-pause");
        let controls = GtkBox::builder().spacing(4).halign(Align::Center).build();
        controls.append(&control(ICON_PREVIOUS, "previous"));
        controls.append(&play_button);
        controls.append(&control(ICON_NEXT, "next"));

        let (title, artist, album) = (label("title"), label("artist"), label("album"));
        let text = GtkBox::builder().orientation(Orientation::Vertical).spacing(2).hexpand(true).valign(Align::Center).build();
        for widget in [&title, &artist, &album] {
            text.append(widget);
        }
        text.append(&GtkBox::builder().vexpand(true).build());
        text.append(&times);
        text.append(&controls);

        let island = GtkBox::new(Orientation::Horizontal, 14);
        island.add_css_class("island");
        island.append(&cover_frame);
        island.append(&text);
        let canvas = Fixed::new();
        canvas.put(&island, CARD_LEFT, CARD_TOP);
        window.set_child(Some(&canvas));

        let card = Rc::new(Card {
            window, island, cover, title, artist, album, track, elapsed, remaining, times, play_button,
            fraction: Cell::new(0.0), length: Cell::new(None), art_url: RefCell::new(String::new()),
            textures: RefCell::new(HashMap::new()), position_timer: RefCell::new(None),
        });

        let drawn = Rc::downgrade(&card);
        card.track.set_draw_func(move |area, context, width, height| {
            let Some(card) = drawn.upgrade() else { return };
            // The line's colour is the CSS `color` of .progress; the track is its 16 % tint.
            let colour = area.style_context().color();
            let (red, green, blue) = (colour.red() as f64, colour.green() as f64, colour.blue() as f64);
            context.set_source_rgba(red, green, blue, 0.16);
            context.rectangle(0.0, 0.0, width as f64, height as f64);
            let _ = context.fill();
            context.set_source_rgba(red, green, blue, 1.0);
            context.rectangle(0.0, 0.0, width as f64 * card.fraction.get(), height as f64);
            let _ = context.fill();
        });

        let click = GestureClick::builder().button(0).build();
        let clicked = Rc::downgrade(&card);
        click.connect_pressed(move |_, _, x, y| {
            let Some(card) = clicked.upgrade() else { return };
            let picked = card.window.pick(x, y, PickFlags::DEFAULT);
            let inside_card = picked.is_some_and(|widget| widget == card.island || widget.is_ancestor(&card.island));
            if !inside_card {
                card.hide();
            }
        });
        card.window.add_controller(click);
        card
    }

    fn show(self: &Rc<Self>) {
        self.window.present();
        self.update_position(); // after present: the position poll skips a hidden window
        if self.position_timer.borrow().is_none() {
            let ticking = Rc::downgrade(self);
            let timer = glib::timeout_add_local(Duration::from_secs(1), move || {
                if let Some(card) = ticking.upgrade() {
                    card.update_position();
                }
                glib::ControlFlow::Continue
            });
            *self.position_timer.borrow_mut() = Some(timer);
        }
    }

    fn hide(&self) {
        self.window.set_visible(false);
        if let Some(timer) = self.position_timer.borrow_mut().take() {
            timer.remove();
        }
    }

    fn toggle(self: &Rc<Self>) {
        if self.window.is_visible() { self.hide() } else { self.show() }
    }

    fn update(self: &Rc<Self>, track: Track) {
        self.title.set_text(if track.title.is_empty() { "Nothing playing" } else { &track.title });
        self.artist.set_text(&track.artist);
        self.album.set_text(&track.album);
        self.album.set_visible(!track.album.is_empty());
        self.play_button.set_label(if track.is_playing { ICON_PAUSE } else { ICON_PLAY });
        self.length.set(track.length);
        self.times.set_visible(track.length.is_some());
        self.remaining.set_text(&track.length.map(|length| format_time(length as f64)).unwrap_or_default());
        self.load_art(&track.art_url);
        self.update_position();
    }

    fn load_art(self: &Rc<Self>, url: &str) {
        if *self.art_url.borrow() == url {
            return;
        }
        *self.art_url.borrow_mut() = url.to_string();
        if url.is_empty() {
            self.cover.set_paintable(None::<&gdk::Paintable>);
            return;
        }
        if let Some(texture) = self.textures.borrow().get(url) {
            self.cover.set_paintable(Some(texture));
            return;
        }
        let (loading, url) = (Rc::downgrade(self), url.to_string());
        gio::File::for_uri(&url).load_contents_async(None::<&gio::Cancellable>, move |result| {
            let Some(card) = loading.upgrade() else { return };
            let texture = match result.and_then(|(contents, _)| square_texture(&contents)) {
                Ok(texture) => texture,
                Err(error) => {
                    eprintln!("mpris-card: cover {url}: {}", error.message());
                    return;
                }
            };
            if *card.art_url.borrow() == url {
                card.cover.set_paintable(Some(&texture));
            }
            card.textures.borrow_mut().insert(url, texture);
        });
    }

    fn update_position(&self) {
        let Some(length) = self.length.get() else { return };
        if !self.window.is_visible() {
            return;
        }
        let mut command = vec![PLAYER[0], PLAYER[1]];
        command.push("position");
        let Some((_, stdout, _)) = desktop::registry::output_within(&command, POSITION_TIMEOUT) else { return };
        let Ok(seconds) = stdout.trim().parse::<f64>() else { return };
        let microseconds = seconds * 1_000_000.0;
        self.elapsed.set_text(&format_time(microseconds));
        self.fraction.set((microseconds / length as f64).min(1.0));
        self.track.queue_draw();
    }
}

/// The image's centre square: the cover box is square, and a picture that is not would otherwise
/// show bars (GTK's ContentFit::Cover needs GTK 4.8 bindings this crate does not enable).
#[allow(deprecated)] // Texture::for_pixbuf: still the way without the 4.6 bindings
fn square_texture(contents: &[u8]) -> Result<gdk::Texture, glib::Error> {
    let loader = gdk_pixbuf::PixbufLoader::new();
    loader.write(contents)?;
    loader.close()?;
    let image = loader.pixbuf().ok_or_else(|| glib::Error::new(gdk_pixbuf::PixbufError::CorruptImage, "no image"))?;
    let side = image.width().min(image.height());
    let square = image.new_subpixbuf((image.width() - side) / 2, (image.height() - side) / 2, side, side);
    Ok(gdk::Texture::for_pixbuf(&square))
}

fn daemon_running() -> bool {
    use std::os::unix::fs::FileTypeExt;
    let alive = std::fs::read_to_string(pid_file()).ok().and_then(|pid| pid.trim().parse::<u32>().ok())
        .is_some_and(|pid| PathBuf::from(format!("/proc/{pid}")).exists());
    alive && std::fs::metadata(fifo_path()).is_ok_and(|metadata| metadata.file_type().is_fifo())
}

/// Lines of `playerctl --follow` go to the card; the watch ends with playerctl.
fn follow_player(card: Rc<Card>) {
    let follow = player_command(&["--follow", "metadata", "--format", FIELDS]).stdout(Stdio::piped()).spawn();
    let mut child = match follow {
        Ok(child) => child,
        Err(error) => {
            eprintln!("mpris-card: cannot follow the player: {error}");
            return;
        }
    };
    let mut output = child.stdout.take().expect("piped stdout");
    let fd = output.as_raw_fd();
    let mut pending = Vec::new();
    glib_unix::unix_fd_add_local(fd, glib::IOCondition::IN | glib::IOCondition::HUP, move |_, _| {
        let mut chunk = [0u8; 4096];
        let count = output.read(&mut chunk).unwrap_or(0);
        if count == 0 {
            let _ = child.wait();
            return glib::ControlFlow::Break;
        }
        pending.extend_from_slice(&chunk[..count]);
        while let Some(end) = pending.iter().position(|byte| *byte == b'\n') {
            let line: Vec<u8> = pending.drain(..=end).collect();
            card.update(Track::parse(String::from_utf8_lossy(&line).trim_end_matches('\n')));
        }
        glib::ControlFlow::Continue
    });
}

fn listen_for_toggles(card: Rc<Card>) {
    let fifo = match desktop::fifo::open(&fifo_path()) {
        Ok(fifo) => fifo,
        Err(error) => {
            eprintln!("mpris-card: cannot open {}: {error}", fifo_path().display());
            return;
        }
    };
    let fd = fifo.as_raw_fd();
    // SAFETY: `fifo` is an open descriptor handed over to the File, which owns it from here on.
    let mut reader = unsafe { std::fs::File::from_raw_fd(fd) };
    std::mem::forget(fifo);
    glib_unix::unix_fd_add_local(fd, glib::IOCondition::IN, move |_, _| {
        let mut chunk = [0u8; 4096];
        let count = reader.read(&mut chunk).unwrap_or(0);
        for command in String::from_utf8_lossy(&chunk[..count]).split_whitespace() {
            if command == "toggle" {
                card.toggle();
            }
        }
        glib::ControlFlow::Continue
    });
}

fn activate(app: &Application, show_now: bool) {
    let provider = CssProvider::new();
    provider.load_from_path(cockpit::paths::home().join(".config/mpris-card/style.css"));
    gtk4::style_context_add_provider_for_display(&gdk::Display::default().expect("a display"), &provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
    let card = Card::new(app);
    // Held for good: the daemon has no window most of the time.
    std::mem::forget(app.hold());
    follow_player(card.clone());
    listen_for_toggles(card.clone());
    if show_now {
        card.show();
    }
}

fn main() {
    let wants_toggle = std::env::args().nth(1).as_deref() == Some("toggle");
    if wants_toggle && daemon_running() {
        let sent = std::fs::OpenOptions::new().write(true).open(fifo_path()).and_then(|mut fifo| fifo.write_all(b"toggle\n"));
        if let Err(error) = sent {
            eprintln!("mpris-card: cannot reach the daemon: {error}");
            std::process::exit(1);
        }
        return;
    }
    if let Err(error) = std::fs::write(pid_file(), std::process::id().to_string()) {
        eprintln!("mpris-card: cannot write {}: {error}", pid_file().display());
    }
    let app = Application::builder().application_id("dotfiles.mpris-card").flags(gio::ApplicationFlags::NON_UNIQUE).build();
    app.connect_activate(move |app| activate(app, wants_toggle));
    let quitting = app.clone();
    glib_unix::unix_signal_add_local_once(SIGTERM, move || quitting.quit());
    app.run_with_args::<&str>(&[]);
    for path in [pid_file(), fifo_path()] {
        let _ = std::fs::remove_file(path);
    }
}
