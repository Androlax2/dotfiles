"""Render the cooler LCD mask: two translucent pills with CPU/GPU labels.

trcc draws the live temperature values on top, inside each pill.
Usage: python make-overlay-mask.py [output.png]
"""
import sys

from PIL import Image, ImageDraw, ImageFont

SCREEN_SIZE = 320
SUPERSAMPLE = 4
PILL_WIDTH = 148
PILL_HEIGHT = 34
PILL_FILL = (6, 12, 28, 190)
PILL_BORDER = (40, 190, 255, 230)
LABEL_COLOR = (190, 235, 255, 255)
LABEL_SIZE = 13
TOP_PILL_CENTER_Y = 22
BOTTOM_PILL_CENTER_Y = 298
FONT_CANDIDATES = [
    "/usr/share/fonts/TTF/JetBrainsMono-Bold.ttf",
    "/usr/share/fonts/TTF/DejaVuSans-Bold.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
]


def scaled(value):
    return round(value * SUPERSAMPLE)


def load_label_font():
    for font_path in FONT_CANDIDATES:
        try:
            return ImageFont.truetype(font_path, scaled(LABEL_SIZE))
        except OSError:
            continue
    return ImageFont.load_default(scaled(LABEL_SIZE))


def pill_box(center_y):
    left = (SCREEN_SIZE - PILL_WIDTH) / 2
    top = center_y - PILL_HEIGHT / 2
    return [scaled(left), scaled(top), scaled(left + PILL_WIDTH), scaled(top + PILL_HEIGHT)]


def draw_pill(draw, center_y, label, font):
    box = pill_box(center_y)
    draw.rounded_rectangle(box, radius=scaled(PILL_HEIGHT / 2), fill=PILL_FILL, outline=PILL_BORDER, width=scaled(1.5))
    label_x = box[0] + scaled(18)
    draw.text((label_x, scaled(center_y)), label, font=font, fill=LABEL_COLOR, anchor="lm")
    divider_x = box[0] + scaled(54)
    draw.line(
        [(divider_x, scaled(center_y - 9)), (divider_x, scaled(center_y + 9))],
        fill=PILL_BORDER,
        width=scaled(1),
    )


def render_mask():
    canvas_size = scaled(SCREEN_SIZE)
    mask = Image.new("RGBA", (canvas_size, canvas_size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(mask)
    font = load_label_font()
    draw_pill(draw, TOP_PILL_CENTER_Y, "CPU", font)
    draw_pill(draw, BOTTOM_PILL_CENTER_Y, "GPU", font)
    return mask.resize((SCREEN_SIZE, SCREEN_SIZE), Image.LANCZOS)


if __name__ == "__main__":
    output_path = sys.argv[1] if len(sys.argv) > 1 else "overlay-mask-320.png"
    render_mask().save(output_path)
