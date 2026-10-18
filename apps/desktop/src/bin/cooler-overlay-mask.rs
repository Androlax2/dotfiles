//! Renders the cooler LCD mask (config/cooler/overlay-mask-320.png): two translucent pills with
//! CPU/GPU labels; trcc draws the live temperatures on top, inside each pill.
//!
//!     cooler-overlay-mask [output.png]

use gtk4::prelude::*;
use gtk4::{cairo, gdk, glib};
use std::f64::consts::PI;

const SCREEN_SIZE: i32 = 320;
const PILL_WIDTH: f64 = 148.0;
const PILL_HEIGHT: f64 = 34.0;
const PILL_FILL: (f64, f64, f64, f64) = (6.0, 12.0, 28.0, 190.0);
const PILL_BORDER: (f64, f64, f64, f64) = (40.0, 190.0, 255.0, 230.0);
const BORDER_WIDTH: f64 = 1.5;
const LABEL_COLOR: (f64, f64, f64, f64) = (190.0, 235.0, 255.0, 255.0);
const LABEL_FONT: &str = "JetBrains Mono";
const LABEL_SIZE: f64 = 13.0;
const LABEL_INSET: f64 = 18.0;
const DIVIDER_INSET: f64 = 54.0;
const DIVIDER_HALF_HEIGHT: f64 = 9.0;
const TOP_PILL_CENTER_Y: f64 = 22.0;
const BOTTOM_PILL_CENTER_Y: f64 = 298.0;

fn set_color(context: &cairo::Context, (red, green, blue, alpha): (f64, f64, f64, f64)) {
    context.set_source_rgba(red / 255.0, green / 255.0, blue / 255.0, alpha / 255.0);
}

fn draw_pill(context: &cairo::Context, center_y: f64, label: &str) -> Result<(), cairo::Error> {
    let left = (SCREEN_SIZE as f64 - PILL_WIDTH) / 2.0;
    // The border lies inside the box, as Pillow drew it: the path runs half a border in.
    let inset = BORDER_WIDTH / 2.0;
    let radius = PILL_HEIGHT / 2.0 - inset;
    let (inner_left, inner_right) = (left + inset + radius, left + PILL_WIDTH - inset - radius);
    context.new_path();
    context.arc(inner_right, center_y, radius, -PI / 2.0, PI / 2.0);
    context.arc(inner_left, center_y, radius, PI / 2.0, 3.0 * PI / 2.0);
    context.close_path();
    set_color(context, PILL_FILL);
    context.fill_preserve()?;
    set_color(context, PILL_BORDER);
    context.set_line_width(BORDER_WIDTH);
    context.stroke()?;

    context.select_font_face(LABEL_FONT, cairo::FontSlant::Normal, cairo::FontWeight::Bold);
    context.set_font_size(LABEL_SIZE);
    let font = context.font_extents()?;
    // Vertically centred between ascender and descender (Pillow's "lm" anchor).
    context.move_to(left + LABEL_INSET, center_y + (font.ascent() - font.descent()) / 2.0);
    set_color(context, LABEL_COLOR);
    context.show_text(label)?;

    let divider_x = (left + DIVIDER_INSET).round() + 0.5;
    context.move_to(divider_x, center_y - DIVIDER_HALF_HEIGHT);
    context.line_to(divider_x, center_y + DIVIDER_HALF_HEIGHT);
    set_color(context, PILL_BORDER);
    context.set_line_width(1.0);
    context.stroke()
}

fn render(output: &str) -> Result<(), String> {
    let mut surface = cairo::ImageSurface::create(cairo::Format::ARgb32, SCREEN_SIZE, SCREEN_SIZE).map_err(|error| error.to_string())?;
    {
        let context = cairo::Context::new(&surface).map_err(|error| error.to_string())?;
        draw_pill(&context, TOP_PILL_CENTER_Y, "CPU").map_err(|error| error.to_string())?;
        draw_pill(&context, BOTTOM_PILL_CENTER_Y, "GPU").map_err(|error| error.to_string())?;
    }
    surface.flush();
    let stride = surface.stride() as usize;
    let pixels = surface.data().map_err(|error| error.to_string())?.to_vec();
    // Cairo's ARGB32 is premultiplied and native-endian: B, G, R, A in memory on this machine.
    let texture = gdk::MemoryTexture::new(SCREEN_SIZE, SCREEN_SIZE, gdk::MemoryFormat::B8g8r8a8Premultiplied, &glib::Bytes::from_owned(pixels), stride);
    texture.save_to_png(output).map_err(|error| error.to_string())
}

fn main() {
    let output = std::env::args().nth(1).unwrap_or_else(|| "overlay-mask-320.png".into());
    if let Err(error) = render(&output) {
        eprintln!("cooler-overlay-mask: {output}: {error}");
        std::process::exit(1);
    }
}
