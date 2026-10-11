"""Render eight 480x480 seamless cooler-screen loops starring Eros, a little grey cat."""
import math
import sys
from functools import lru_cache
from multiprocessing import Pool
from pathlib import Path

import numpy as np
from PIL import Image, ImageChops, ImageDraw, ImageFilter

OUTPUT_SIZE = 480
SCALE = 2
CANVAS = OUTPUT_SIZE * SCALE
TAU = math.tau
FRAME_RATE = 20

FUR = (124, 118, 126)
FUR_DARK = (88, 84, 92)
FUR_LIGHT = (164, 158, 166)
INNER_EAR = (196, 146, 156)
NOSE = (224, 150, 162)
TONGUE = (236, 120, 140)
EYE = (198, 214, 92)
PUPIL = (18, 18, 24)
EYE_SHINE = (240, 250, 255)
CYAN = (40, 190, 255)
ICE = (190, 235, 255)
DEEP_BLUE = (20, 80, 220)
MAGENTA = (255, 70, 190)
ORANGE = (255, 120, 40)


# ---------- generic helpers ----------

def lerp(start, end, amount):
    return start + (end - start) * amount


def lerp_color(start, end, amount):
    return tuple(int(lerp(first, second, amount)) for first, second in zip(start, end))


def scale_color(color, amount):
    return tuple(max(0, min(255, int(channel * amount))) for channel in color)


def ease_in_out(amount):
    return 0.5 - 0.5 * math.cos(math.pi * max(0.0, min(1.0, amount)))


def polar(center_x, center_y, radius, angle):
    return center_x + radius * math.cos(angle), center_y + radius * math.sin(angle)


def rotate_point(point_x, point_y, center_x, center_y, angle):
    offset_x, offset_y = point_x - center_x, point_y - center_y
    return (
        center_x + offset_x * math.cos(angle) - offset_y * math.sin(angle),
        center_y + offset_x * math.sin(angle) + offset_y * math.cos(angle),
    )


def shrink_toward_centroid(points, factor):
    centroid_x = sum(point[0] for point in points) / len(points)
    centroid_y = sum(point[1] for point in points) / len(points)
    return [(centroid_x + (x - centroid_x) * factor, centroid_y + (y - centroid_y) * factor) for x, y in points]


def ellipse(draw, center_x, center_y, radius_x, radius_y, **style):
    radius_x, radius_y = max(radius_x, 0.5), max(radius_y, 0.5)
    draw.ellipse(
        [(center_x - radius_x) * SCALE, (center_y - radius_y) * SCALE, (center_x + radius_x) * SCALE, (center_y + radius_y) * SCALE],
        **style,
    )


def arc(draw, center_x, center_y, radius_x, radius_y, start, end, width, fill):
    draw.arc(
        [(center_x - radius_x) * SCALE, (center_y - radius_y) * SCALE, (center_x + radius_x) * SCALE, (center_y + radius_y) * SCALE],
        start=start,
        end=end,
        fill=fill,
        width=max(1, int(width * SCALE)),
    )


def polygon(draw, points, **style):
    draw.polygon([(x * SCALE, y * SCALE) for x, y in points], **style)


def line(draw, points, width, fill):
    draw.line([(x * SCALE, y * SCALE) for x, y in points], fill=fill, width=max(1, int(width * SCALE)), joint="curve")


def rectangle(draw, left, top, right, bottom, **style):
    if "width" in style:
        style["width"] = max(1, int(style["width"] * SCALE))
    draw.rectangle([left * SCALE, top * SCALE, right * SCALE, bottom * SCALE], **style)


@lru_cache(maxsize=None)
def gradient_background(top_color, bottom_color):
    blend = np.linspace(0, 1, CANVAS, dtype=np.float32)[:, None, None]
    pixels = np.array(top_color, dtype=np.float32) * (1 - blend) + np.array(bottom_color, dtype=np.float32) * blend
    return np.repeat(pixels, CANVAS, axis=1)


@lru_cache(maxsize=1)
def vignette_mask():
    coordinates = np.linspace(-1, 1, CANVAS, dtype=np.float32)
    distance = np.sqrt(coordinates[None, :] ** 2 + coordinates[:, None] ** 2)
    return np.clip(1.25 - distance * 0.5, 0, 1)[:, :, None]


class Frame:
    """Three layers: base (behind glow), glow (blurred light), front (on top)."""

    def __init__(self, top_color, bottom_color):
        self.background = gradient_background(top_color, bottom_color)
        self.base = Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0))
        self.glow = Image.new("RGB", (CANVAS, CANVAS))
        self.front = Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0))
        self.base_draw = ImageDraw.Draw(self.base)
        self.glow_draw = ImageDraw.Draw(self.glow)
        self.front_draw = ImageDraw.Draw(self.front)

    def paste_front_layer(self, layer):
        self.front.alpha_composite(layer)

    def render(self):
        base_pixels = np.asarray(self.base, dtype=np.float32)
        alpha = base_pixels[..., 3:4] / 255
        pixels = self.background * (1 - alpha) + base_pixels[..., :3] * alpha
        pixels = pixels + np.asarray(self.glow.filter(ImageFilter.GaussianBlur(10 * SCALE)), dtype=np.float32) * 1.2
        pixels = pixels + np.asarray(self.glow.filter(ImageFilter.GaussianBlur(30 * SCALE)), dtype=np.float32) * 0.8
        image = Image.fromarray(np.clip(pixels, 0, 255).astype(np.uint8)).convert("RGBA")
        image.alpha_composite(self.front)
        shaded = np.asarray(image.convert("RGB"), dtype=np.float32) * vignette_mask()
        final = Image.fromarray(np.clip(shaded, 0, 255).astype(np.uint8))
        return final.resize((OUTPUT_SIZE, OUTPUT_SIZE), Image.LANCZOS)


def new_layer():
    layer = Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0))
    return layer, ImageDraw.Draw(layer)


# ---------- neon & PC parts ----------

def neon_line(frame, points, width, color, glow_amount=1.0):
    line(frame.base_draw, points, width, color)
    line(frame.glow_draw, points, width, scale_color(color, glow_amount))


def neon_ring(frame, center_x, center_y, radius, width, color, glow_amount=1.0):
    arc(frame.base_draw, center_x, center_y, radius, radius, 0, 360, width, color)
    arc(frame.glow_draw, center_x, center_y, radius, radius, 0, 360, width, scale_color(color, glow_amount))


def draw_stars(frame, progress, seed, count=60):
    generator = np.random.default_rng(seed)
    for _ in range(count):
        star_x, star_y = generator.uniform(0, OUTPUT_SIZE, 2)
        radius = generator.uniform(0.4, 1.0)
        offset = generator.uniform(0, TAU)
        speed = int(generator.integers(1, 3))
        brightness = 0.3 + 0.7 * (0.5 + 0.5 * math.sin(TAU * progress * speed + offset))
        ellipse(frame.base_draw, star_x, star_y, radius, radius, fill=scale_color(ICE, brightness * 0.85))


def draw_fan(frame, center_x, center_y, radius, rotation, glow_amount=1.0):
    draw = frame.base_draw
    ellipse(draw, center_x, center_y, radius, radius, fill=(6, 14, 30))
    blade_count = 7
    inner, outer = radius * 0.3, radius * 0.93
    for blade_index in range(blade_count):
        base_angle = rotation + blade_index * TAU / blade_count
        leading = [polar(center_x, center_y, lerp(inner, outer, step / 10), base_angle + step / 10 * 0.55) for step in range(11)]
        trailing = [polar(center_x, center_y, lerp(inner, outer, step / 10), base_angle + 0.42 + step / 10 * 0.62) for step in range(10, -1, -1)]
        polygon(draw, leading + trailing, fill=(34, 96, 190), outline=(120, 200, 255))
    ellipse(draw, center_x, center_y, inner, inner, fill=(8, 16, 32))
    neon_ring(frame, center_x, center_y, radius, 2.2, CYAN, glow_amount)
    neon_ring(frame, center_x, center_y, inner, 2, CYAN, glow_amount)


def draw_neon_octagon(frame, center_x, center_y, radius, phase, glow_amount=1.0):
    corners = [polar(center_x, center_y, radius, math.pi / 8 + index * TAU / 8) for index in range(8)]
    line(frame.base_draw, corners + [corners[0]], 9, (10, 18, 34))
    for index in range(8):
        start, end = corners[index], corners[(index + 1) % 8]
        brightness = 0.55 + 0.45 * (0.5 + 0.5 * math.sin(phase - index * TAU / 8))
        segment = [
            (lerp(start[0], end[0], 0.14), lerp(start[1], end[1], 0.14)),
            (lerp(start[0], end[0], 0.86), lerp(start[1], end[1], 0.86)),
        ]
        neon_line(frame, segment, 3, scale_color(CYAN, brightness), glow_amount)


# ---------- Eros ----------

def draw_tail(draw, points, base_width, tip_width, tip_share=0.15):
    last_index = len(points) - 1
    for index, (point_x, point_y) in enumerate(points):
        progress = index / last_index
        radius = lerp(base_width, tip_width, progress) / 2
        color = FUR_DARK if progress > 1 - tip_share else FUR
        ellipse(draw, point_x, point_y, radius, radius, fill=color)


def draw_front_ears(draw, center_x, center_y, size):
    for direction in (-1, 1):
        ear = [
            (center_x + direction * size * 0.95, center_y - size * 0.15),
            (center_x + direction * size * 0.2, center_y - size * 0.7),
            (center_x + direction * size * 0.8, center_y - size * 1.3),
        ]
        polygon(draw, ear, fill=FUR)
        polygon(draw, shrink_toward_centroid(ear, 0.5), fill=INNER_EAR)


def draw_eyes(draw, center_x, center_y, size, openness, pupil_shift):
    for direction in (-1, 1):
        eye_x = center_x + direction * size * 0.4
        eye_y = center_y - size * 0.02
        if openness < 0.12:
            arc(draw, eye_x, eye_y - size * 0.06, size * 0.17, size * 0.12, 20, 160, size * 0.06, PUPIL)
            continue
        ellipse(draw, eye_x, eye_y, size * 0.2, size * 0.19 * openness, fill=EYE)
        ellipse(draw, eye_x + pupil_shift * size * 0.08, eye_y, size * 0.06, size * 0.17 * openness, fill=PUPIL)
        shine_x = eye_x + pupil_shift * size * 0.08 - size * 0.05
        ellipse(draw, shine_x, eye_y - size * 0.07 * openness, size * 0.035, size * 0.035, fill=EYE_SHINE)


def draw_muzzle(draw, center_x, center_y, size, tongue):
    ellipse(draw, center_x, center_y + size * 0.38, size * 0.38, size * 0.25, fill=FUR_LIGHT)
    for direction in (-1, 1):
        for whisker_index in range(2):
            start = (center_x + direction * size * 0.3, center_y + size * (0.36 + whisker_index * 0.1))
            end = (center_x + direction * size * 0.95, center_y + size * (0.3 + whisker_index * 0.2))
            line(draw, [start, end], size * 0.02, (210, 210, 216))
    polygon(
        draw,
        [(center_x - size * 0.08, center_y + size * 0.22), (center_x + size * 0.08, center_y + size * 0.22), (center_x, center_y + size * 0.31)],
        fill=NOSE,
    )
    if tongue:
        ellipse(draw, center_x, center_y + size * 0.48, size * 0.08, size * 0.13, fill=TONGUE)
    for direction in (-1, 1):
        arc(draw, center_x + direction * size * 0.08, center_y + size * 0.34, size * 0.08, size * 0.06, 0, 180, size * 0.035, PUPIL)


def draw_cat_head(draw, center_x, center_y, size, openness=1.0, pupil_shift=0.0, tongue=False):
    draw_front_ears(draw, center_x, center_y, size)
    ellipse(draw, center_x, center_y, size, size * 0.85, fill=FUR)
    # Grey tabby forehead marks.
    for mark_index in (-1, 0, 1):
        mark_x = center_x + mark_index * size * 0.16
        line(draw, [(mark_x, center_y - size * 0.72), (mark_x, center_y - size * 0.45)], size * 0.06, FUR_DARK)
    draw_eyes(draw, center_x, center_y, size, openness, pupil_shift)
    draw_muzzle(draw, center_x, center_y, size, tongue)


def back_head_ear_points(head_x, head_y, size, tilt):
    ears = []
    for direction in (-1, 1):
        ear = [
            (head_x + direction * size * 0.55, head_y - size * 0.2),
            (head_x + direction * size * 0.12, head_y - size * 0.4),
            (head_x + direction * size * 0.45, head_y - size * 0.85),
        ]
        ears.append([rotate_point(x, y, head_x, head_y, tilt) for x, y in ear])
    return ears


def draw_cat_back(draw, center_x, base_y, size, head_tilt=0.0, tail_points=None):
    """Eros sitting, seen from behind."""
    if tail_points is None:
        tail_points = [
            (center_x + size * 0.5 + step / 39 * size * 1.1, base_y - size * 0.08 - (step / 39) ** 2 * size * 0.8)
            for step in range(40)
        ]
    ellipse(draw, center_x, base_y - size * 0.5, size * 0.95, size * 0.52, fill=FUR)
    ellipse(draw, center_x, base_y - size * 1.1, size * 0.72, size * 0.85, fill=FUR)
    for stripe_index in range(3):
        stripe_y = base_y - size * (1.55 - stripe_index * 0.3)
        arc(draw, center_x, stripe_y, size * 0.45, size * 0.2, 200, 340, size * 0.07, FUR_DARK)
    head_x = center_x + head_tilt * size * 0.18
    head_y = base_y - size * 2.0
    for ear in back_head_ear_points(head_x, head_y, size, head_tilt * 0.5):
        polygon(draw, ear, fill=FUR)
    ellipse(draw, head_x, head_y, size * 0.6, size * 0.52, fill=FUR)
    draw_tail(draw, tail_points, size * 0.28, size * 0.22)


def draw_loaf(draw, center_x, ground_y, length, height, head_size, head_x, head_y, openness=0.0, tongue=False):
    ellipse(draw, center_x, ground_y - height, length, height, fill=FUR)
    for stripe_index in range(3):
        stripe_x = center_x + (stripe_index - 0.5) * length * 0.3
        arc(draw, stripe_x, ground_y - height * 1.45, length * 0.12, height * 0.5, 200, 340, height * 0.12, FUR_DARK)
    draw_cat_head(draw, head_x, head_y, head_size, openness=openness, tongue=tongue)


def draw_zzz(frame, origin_x, origin_y, progress, opacity=1.0):
    for index in range(3):
        drift = (progress * 2 + index / 3) % 1
        fade = math.sin(drift * math.pi) * opacity
        size = 5 + index * 1.5
        z_x = origin_x - drift * 22
        z_y = origin_y - drift * 34
        points = [(z_x - size, z_y - size), (z_x + size, z_y - size), (z_x - size, z_y + size), (z_x + size, z_y + size)]
        line(frame.front_draw, points, 1.6, scale_color(ICE, fade) + (int(255 * fade),))


# ---------- scenes ----------

def scene_chasing_fan(progress):
    frame = Frame((4, 7, 18), (2, 14, 34))
    draw_stars(frame, progress, seed=1)
    fan_x, fan_y = 240, 190
    draw_neon_octagon(frame, fan_x, fan_y, 128, TAU * progress)
    draw_fan(frame, fan_x, fan_y, 100, progress * TAU / 7 * 3)

    head_tilt = 0.55 * math.sin(TAU * progress * 2)
    swipe = math.sin(math.pi * (progress - 0.55) / 0.2) if 0.55 < progress < 0.75 else 0.0
    cat_x, base_y, size = 240, 478, 58
    draw = frame.front_draw
    tail_swish = math.sin(TAU * progress)
    tail_points = [
        (cat_x - size * 0.5 - step / 39 * size * 1.0 + tail_swish * size * 0.3 * (step / 39) ** 2,
         base_y - size * 0.08 - (step / 39) ** 2 * size * 0.9)
        for step in range(40)
    ]
    draw_cat_back(draw, cat_x, base_y, size, head_tilt=head_tilt, tail_points=tail_points)
    if swipe > 0.02:
        shoulder = (cat_x + size * 0.42, base_y - size * 1.5)
        angle = -math.pi / 2 + 0.35
        reach = size * 2.1 * swipe
        paw = (shoulder[0] + math.cos(angle) * reach, shoulder[1] + math.sin(angle) * reach)
        line(draw, [shoulder, paw], size * 0.3, FUR)
        ellipse(draw, paw[0], paw[1], size * 0.19, size * 0.17, fill=FUR_LIGHT)
    return frame.render()


def draw_donut_cat(draw, center_x, center_y, size):
    """Eros curled up, seen from above."""
    ellipse(draw, center_x, center_y, size, size, fill=FUR)
    for ring_index in range(3):
        radius = size * (0.35 + 0.2 * ring_index)
        arc(draw, center_x, center_y, radius, radius, 120 + ring_index * 25, 300 + ring_index * 25, size * 0.05, FUR_DARK)
    head_x, head_y = center_x, center_y - size * 0.5
    for direction in (-1, 1):
        ear = [
            (head_x + direction * size * 0.14, head_y - size * 0.22),
            (head_x + direction * size * 0.38, head_y - size * 0.12),
            (head_x + direction * size * 0.36, head_y - size * 0.52),
        ]
        polygon(draw, ear, fill=FUR)
        polygon(draw, shrink_toward_centroid(ear, 0.5), fill=INNER_EAR)
    ellipse(draw, head_x, head_y, size * 0.38, size * 0.32, fill=FUR)
    for direction in (-1, 1):
        arc(draw, head_x + direction * size * 0.15, head_y - size * 0.02, size * 0.07, size * 0.05, 20, 160, size * 0.03, PUPIL)
    ellipse(draw, head_x - size * 0.13, head_y + size * 0.3, size * 0.1, size * 0.08, fill=FUR_LIGHT)
    ellipse(draw, head_x + size * 0.13, head_y + size * 0.3, size * 0.1, size * 0.08, fill=FUR_LIGHT)
    tail_points = [polar(center_x, center_y, size * 0.84, lerp(0.6, -1.25, step / 39)) for step in range(40)]
    draw_tail(draw, tail_points, size * 0.3, size * 0.24, tip_share=0.2)


def scene_donut_fan(progress):
    frame = Frame((4, 7, 18), (2, 14, 34))
    draw_stars(frame, progress, seed=2)
    center_x, center_y = 240, 245
    draw_neon_octagon(frame, center_x, center_y, 150, TAU * progress)
    pulse = 0.85 + 0.15 * math.sin(TAU * progress * 2)
    neon_ring(frame, center_x, center_y, 120, 3, scale_color(CYAN, pulse))
    # Dashes rotating the other way, like the fan's outer frame.
    for dash_index in range(24):
        angle = dash_index * TAU / 24 - TAU * progress / 4
        neon_line(frame, [polar(center_x, center_y, 108, angle), polar(center_x, center_y, 108, angle + 0.12)], 2, scale_color(ICE, 0.6), 0.5)
    layer, layer_draw = new_layer()
    draw_donut_cat(layer_draw, center_x, center_y, 92)
    rotated = layer.rotate(-360 * progress, resample=Image.BICUBIC, center=(center_x * SCALE, center_y * SCALE))
    frame.paste_front_layer(rotated)
    return frame.render()


def scene_thermometer(progress):
    heat = 0.5 - 0.5 * math.cos(TAU * progress)
    frame = Frame(lerp_color((4, 8, 22), (24, 6, 12), heat), lerp_color((2, 14, 36), (42, 12, 8), heat))
    accent = lerp_color(CYAN, ORANGE, heat)
    ground_y = 330

    neon_line(frame, [(90, ground_y), (430, ground_y)], 2.5, accent)
    tube_x, tube_top, tube_bottom = 56, 110, 345
    rectangle(frame.base_draw, tube_x - 9, tube_top, tube_x + 9, tube_bottom, fill=(10, 14, 26))
    level = lerp(tube_bottom, tube_top + 10, heat)
    rectangle(frame.base_draw, tube_x - 5, level, tube_x + 5, tube_bottom + 4, fill=accent)
    rectangle(frame.glow_draw, tube_x - 5, level, tube_x + 5, tube_bottom + 4, fill=accent)
    ellipse(frame.base_draw, tube_x, tube_bottom + 14, 17, 17, fill=accent)
    ellipse(frame.glow_draw, tube_x, tube_bottom + 14, 17, 17, fill=accent)
    for tick_index in range(6):
        tick_y = lerp(tube_bottom - 10, tube_top + 15, tick_index / 5)
        line(frame.base_draw, [(tube_x + 13, tick_y), (tube_x + 22, tick_y)], 1.5, (120, 140, 170))

    draw = frame.front_draw
    length = lerp(78, 128, heat)
    height = lerp(42, 17, heat)
    body_x = 280
    # Back legs stretch out behind when hot.
    if heat > 0.35:
        leg_reach = (heat - 0.35) / 0.65
        ellipse(draw, body_x + length + 10 * leg_reach, ground_y - 8, 8 + 22 * leg_reach, 8, fill=FUR)
    tail_points = [
        (body_x + length * 0.85 + step / 39 * lerp(30, 90, heat),
         ground_y - 10 - math.sin(step / 39 * math.pi) * lerp(22, 4, heat))
        for step in range(40)
    ]
    draw_tail(draw, tail_points, 15, 11)
    head_x = body_x - length + lerp(18, -12, heat)
    head_y = ground_y - lerp(40, 27, heat)
    openness = 0.0 if heat < 0.45 else lerp(0.2, 0.55, (heat - 0.45) / 0.55)
    draw_loaf(draw, body_x, ground_y, length, height, 34, head_x, head_y, openness=openness, tongue=heat > 0.65)

    if heat < 0.35:
        draw_zzz(frame, head_x - 20, head_y - 52, progress, opacity=1 - heat / 0.35)
    if heat > 0.6:
        intensity = (heat - 0.6) / 0.4
        drop_fall = (progress * 6) % 1
        drop_x, drop_y = head_x + 38, head_y - 30 + drop_fall * 22
        ellipse(draw, drop_x, drop_y, 4, 6, fill=(120, 200, 255, int(230 * intensity)))
        for wave_index in range(3):
            wave_y = ground_y - height * 2 - 26 - wave_index * 16
            wave = [
                (wave_x, wave_y + 4 * math.sin(wave_x * 0.08 + TAU * progress * 4 + wave_index))
                for wave_x in range(int(body_x - 70), int(body_x + 70), 4)
            ]
            line(frame.glow_draw, wave, 2, scale_color(ORANGE, intensity * 0.8))
    return frame.render()


def blink_openness(progress, blink_centers, half_width=0.03):
    openness = 1.0
    for center in blink_centers:
        distance = abs(progress - center) / half_width
        openness = min(openness, max(0.0, min(1.0, distance)))
    return openness


def scene_peeking(progress):
    frame = Frame((4, 7, 18), (2, 14, 34))
    draw_stars(frame, progress, seed=4, count=40)
    draw_neon_octagon(frame, 170, 240, 120, TAU * progress, glow_amount=0.5)
    draw_fan(frame, 170, 240, 92, progress * TAU / 7 * 3, glow_amount=0.5)
    neon_line(frame, [(462, 60), (462, 420)], 3, CYAN)

    if progress < 0.2:
        emerge = ease_in_out(progress / 0.2)
    elif progress < 0.8:
        emerge = 1.0
    else:
        emerge = 1 - ease_in_out((progress - 0.8) / 0.2)
    head_x = lerp(570, 405, emerge)
    head_y = 235
    look_window = max(0.0, min(1.0, (progress - 0.25) / 0.5))
    pupil_shift = -1.0 * math.sin(math.pi * look_window)
    openness = blink_openness(progress, (0.45, 0.62))

    layer, layer_draw = new_layer()
    draw_cat_head(layer_draw, head_x, head_y, 64, openness=openness, pupil_shift=pupil_shift)
    tilted = layer.rotate(24, resample=Image.BICUBIC, center=(head_x * SCALE, head_y * SCALE))
    frame.paste_front_layer(tilted)
    paw_x = lerp(520, 455, emerge)
    for paw_y in (300, 160):
        ellipse(frame.front_draw, paw_x, paw_y, 20, 15, fill=FUR_LIGHT)
        for toe in (-1, 0, 1):
            line(frame.front_draw, [(paw_x - 12, paw_y + toe * 6), (paw_x - 2, paw_y + toe * 6)], 1.2, FUR_DARK)
    return frame.render()


def draw_planet(frame, center_x, center_y, radius, progress):
    ring_tilt = -0.32
    ring_points = [rotate_point(center_x + radius * 1.6 * math.cos(angle), center_y + radius * 0.3 * math.sin(angle), center_x, center_y, ring_tilt)
                   for angle in np.linspace(0, TAU, 120)]
    back_half, front_half = ring_points[60:], ring_points[:61]
    neon_line(frame, back_half, 4, scale_color(ICE, 0.55), 0.4)

    layer, layer_draw = new_layer()
    ellipse(layer_draw, center_x, center_y, radius, radius, fill=(26, 70, 170))
    for band_index in range(5):
        band_y = center_y - radius * 0.8 + band_index * radius * 0.4
        ellipse(layer_draw, center_x, band_y, radius * 1.1, radius * 0.07, fill=(40, 110, 210))
    generator = np.random.default_rng(5)
    for _ in range(6):
        latitude = generator.uniform(-0.7, 0.7)
        offset = generator.uniform(0, TAU)
        spot_size = generator.uniform(0.08, 0.16) * radius
        longitude = TAU * progress + offset
        if math.cos(longitude) <= 0:
            continue
        spot_x = center_x + radius * math.cos(latitude) * math.sin(longitude)
        spot_y = center_y + radius * math.sin(latitude)
        ellipse(layer_draw, spot_x, spot_y, spot_size * math.cos(longitude), spot_size * 0.7, fill=(70, 160, 240))
    mask = Image.new("L", (CANVAS, CANVAS), 0)
    ellipse(ImageDraw.Draw(mask), center_x, center_y, radius, radius, fill=255)
    shadow = Image.new("L", (CANVAS, CANVAS), 0)
    ellipse(ImageDraw.Draw(shadow), center_x + radius * 0.45, center_y + radius * 0.35, radius * 1.05, radius * 1.05, fill=120)
    planet = Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0))
    planet.paste(layer, (0, 0), mask)
    night_side = Image.new("RGBA", (CANVAS, CANVAS), (4, 8, 20, 255))
    night_side.putalpha(ImageChops.multiply(shadow, mask))
    planet.alpha_composite(night_side)
    frame.base.alpha_composite(planet)
    neon_ring(frame, center_x, center_y, radius, 1.5, scale_color(CYAN, 0.8), 0.7)
    neon_line(frame, front_half, 4, ICE, 0.6)


def scene_space(progress):
    frame = Frame((2, 3, 12), (6, 10, 30))
    draw_stars(frame, progress, seed=6, count=90)
    draw_planet(frame, 120, 375, 95, progress)

    float_x = 300 + 14 * math.sin(TAU * progress)
    float_y = 190 + 10 * math.sin(TAU * progress * 2)
    sway = 9 * math.sin(TAU * progress)
    layer, draw = new_layer()
    tail_points = [
        (float_x + 28 + step / 39 * 46, float_y + 82 - step / 39 * 30 + math.sin(TAU * progress * 2 + step / 39 * 3) * 9 * step / 39)
        for step in range(40)
    ]
    draw_tail(draw, tail_points, 13, 10)
    ellipse(draw, float_x, float_y + 70, 36, 40, fill=(222, 228, 240))
    ellipse(draw, float_x - 38, float_y + 58, 13, 10, fill=(222, 228, 240))
    ellipse(draw, float_x + 38, float_y + 58, 13, 10, fill=(222, 228, 240))
    ellipse(draw, float_x - 15, float_y + 108, 11, 9, fill=(200, 206, 220))
    ellipse(draw, float_x + 15, float_y + 108, 11, 9, fill=(200, 206, 220))
    rectangle(draw, float_x - 14, float_y + 66, float_x + 14, float_y + 82, fill=(40, 120, 230))
    ellipse(draw, float_x, float_y + 74, 3, 3, fill=(170, 240, 255))
    ellipse(draw, float_x, float_y - 2, 58, 58, fill=(70, 120, 180, 90))
    draw_cat_head(draw, float_x, float_y + 4, 32, openness=blink_openness(progress, (0.7,)), pupil_shift=0.4)
    arc(draw, float_x, float_y - 2, 58, 58, 0, 360, 3, (190, 230, 255))
    arc(draw, float_x, float_y - 2, 46, 46, 200, 250, 4, (235, 248, 255))
    ellipse(draw, float_x, float_y + 52, 34, 9, fill=(180, 190, 210))
    rotated = layer.rotate(sway, resample=Image.BICUBIC, center=(float_x * SCALE, (float_y + 30) * SCALE))
    frame.paste_front_layer(rotated)
    return frame.render()


def draw_fluffy_body(draw, center_x, center_y, radius_x, radius_y, seed):
    generator = np.random.default_rng(seed)
    ellipse(draw, center_x, center_y, radius_x, radius_y, fill=FUR)
    for _ in range(520):
        angle = generator.uniform(0, math.pi)
        edge_x, edge_y = center_x + radius_x * math.cos(angle), center_y + radius_y * math.sin(angle)
        strand = generator.uniform(3, 8)
        lean = generator.uniform(-0.6, 0.6)
        color = FUR_LIGHT if generator.random() < 0.4 else FUR
        end = (edge_x + strand * math.cos(angle + lean), edge_y + strand * math.sin(angle + lean))
        line(draw, [(edge_x - math.cos(angle) * 6, edge_y - math.sin(angle) * 6), end], 3, color)
    for _ in range(140):
        inside_x = center_x + generator.uniform(-0.9, 0.9) * radius_x
        inside_y = center_y + generator.uniform(-0.3, 0.85) * radius_y
        if ((inside_x - center_x) / radius_x) ** 2 + ((inside_y - center_y) / radius_y) ** 2 > 0.8:
            continue
        color = FUR_DARK if generator.random() < 0.5 else FUR_LIGHT
        line(draw, [(inside_x, inside_y), (inside_x + generator.uniform(-6, 6), inside_y + generator.uniform(4, 9))], 1.6, color)


def scene_roof(progress):
    frame = Frame((16, 26, 42), (4, 6, 14))
    rectangle(frame.base_draw, 30, 218, 450, 490, fill=(5, 7, 15))
    rotation = progress * TAU / 7 * 3
    for fan_x in (145, 335):
        draw_neon_octagon(frame, fan_x, 355, 92, TAU * progress + fan_x)
        draw_fan(frame, fan_x, 355, 72, rotation)
    neon_line(frame, [(260, 248), (330, 228)], 3, (235, 240, 255), 0.6)
    line(frame.base_draw, [(30, 218), (450, 218)], 3, (150, 170, 196))

    breath = 1 + 0.03 * math.sin(TAU * progress)
    layer, draw = new_layer()
    draw_fluffy_body(draw, 230, 95, 320, 120 * breath, seed=8)
    for toe_offset in (-9, 0, 9):
        ellipse(draw, 95 + toe_offset, 236, 7, 8, fill=FUR_LIGHT)
    ellipse(draw, 95, 222, 20, 16, fill=FUR_LIGHT)
    for toe_offset in (-9, 0, 9):
        ellipse(draw, 95 + toe_offset, 233, 3, 2.5, fill=INNER_EAR)
    tail_points = []
    for step in range(40):
        along = step / 39
        swing = math.sin(TAU * progress - along * 1.6) * 26 * along ** 1.4
        tail_points.append((392 + along * 10 + swing, 205 + along * 125))
    draw_tail(draw, tail_points, 30, 22, tip_share=0.2)
    frame.paste_front_layer(layer)
    return frame.render()


def draw_city(frame, progress):
    draw = frame.base_draw
    rectangle(draw, 50, 50, 430, 360, fill=(10, 16, 40))
    generator = np.random.default_rng(9)
    left = 50
    while left < 430:
        width = generator.uniform(30, 70)
        top = generator.uniform(150, 280)
        rectangle(draw, left, top, min(left + width, 430), 360, fill=(15, 22, 48))
        for window_y in np.arange(top + 10, 350, 14):
            for window_x in np.arange(left + 6, min(left + width, 430) - 6, 12):
                if generator.random() < 0.35:
                    flicker = 0.6 + 0.4 * math.sin(TAU * progress * int(generator.integers(1, 3)) + generator.uniform(0, TAU))
                    color = (255, 190, 110) if generator.random() < 0.15 else (70, 160, 255)
                    rectangle(draw, window_x, window_y, window_x + 5, window_y + 6, fill=scale_color(color, flicker * 0.75))
        left += width + generator.uniform(2, 8)
    sign_on = 0.15 if 0.3 < progress < 0.34 or 0.37 < progress < 0.39 else 1.0
    sign = [(100, 115), (205, 115), (205, 150), (100, 150), (100, 115)]
    neon_line(frame, sign, 3, scale_color(MAGENTA, sign_on))
    neon_line(frame, [(120, 132), (185, 132)], 3, scale_color(CYAN, sign_on))


def draw_rain(frame, progress):
    generator = np.random.default_rng(10)
    for _ in range(80):
        start_x = generator.uniform(40, 440)
        start_y = generator.uniform(0, 310)
        length = generator.uniform(10, 22)
        rain_y = 50 + (start_y + progress * 620) % 310
        rain_x = start_x - (rain_y - 50) * 0.15
        if not 50 < rain_x < 430:
            continue
        end_y = min(rain_y + length, 360)
        line(frame.base_draw, [(rain_x, rain_y), (rain_x - (end_y - rain_y) * 0.15, end_y)], 1.1, (110, 160, 220))


def scene_night_window(progress):
    frame = Frame((6, 8, 20), (3, 4, 12))
    draw_city(frame, progress)
    draw_rain(frame, progress)
    frame_color = (26, 30, 46)
    rectangle(frame.base_draw, 42, 42, 438, 368, outline=frame_color, width=12)
    line(frame.base_draw, [(240, 48), (240, 362)], 8, frame_color)
    line(frame.base_draw, [(48, 205), (432, 205)], 8, frame_color)
    rectangle(frame.base_draw, 20, 362, 460, 392, fill=(36, 40, 58))
    line(frame.base_draw, [(20, 362), (460, 362)], 2, (90, 100, 130))

    cat_x, base_y, size = 240, 368, 52
    head_tilt = 0.25 * math.sin(TAU * progress)
    for glow_radius in (0.8,):
        ellipse(frame.glow_draw, cat_x, base_y - size * 1.1, size * 0.72 * glow_radius + 6, size * 0.85, fill=(10, 40, 80))
        ellipse(frame.glow_draw, cat_x, base_y - size * 2.0, size * 0.6, size * 0.52, fill=(30, 10, 50))
    tail_points = []
    for step in range(40):
        along = step / 39
        swing = math.sin(TAU * progress - along * 1.8) * 18 * along ** 1.3
        tail_points.append((cat_x + size * 0.55 + swing, base_y - 6 + along * 95))
    draw_cat_back(frame.front_draw, cat_x, base_y, size, head_tilt=head_tilt, tail_points=tail_points)
    head_x = cat_x + head_tilt * size * 0.18
    arc(frame.front_draw, head_x, base_y - size * 2.0, size * 0.6, size * 0.52, 200, 300, 2, (120, 90, 200))
    arc(frame.front_draw, cat_x, base_y - size * 1.1, size * 0.72, size * 0.85, 190, 250, 2, (70, 150, 230))
    return frame.render()


def scene_tail_clock(progress):
    frame = Frame((4, 7, 18), (2, 14, 34))
    pivot_x, pivot_y = 240, -30
    swing = 0.55

    points = [(pivot_x, pivot_y)]
    step_length = 330 / 40
    direction = math.pi / 2
    for step in range(40):
        along = step / 39
        bend = swing * math.sin(TAU * progress - along * 1.3)
        direction = math.pi / 2 + bend
        last_x, last_y = points[-1]
        points.append((last_x + math.cos(direction) * step_length, last_y + math.sin(direction) * step_length))
    tip_x, tip_y = points[-1]
    tip_angle = math.atan2(tip_x - pivot_x, tip_y - pivot_y)

    dial_radius = 352
    for tick_index in range(13):
        tick_angle = lerp(-0.62, 0.62, tick_index / 12)
        closeness = math.exp(-((tick_angle - tip_angle) / 0.09) ** 2)
        brightness = 0.25 + 0.75 * closeness
        inner = (pivot_x + math.sin(tick_angle) * (dial_radius - 12), pivot_y + math.cos(tick_angle) * (dial_radius - 12))
        outer = (pivot_x + math.sin(tick_angle) * (dial_radius + 8 + 8 * closeness), pivot_y + math.cos(tick_angle) * (dial_radius + 8 + 8 * closeness))
        neon_line(frame, [inner, outer], 3, scale_color(CYAN, brightness))
    dial = [(pivot_x + math.sin(angle) * (dial_radius - 22), pivot_y + math.cos(angle) * (dial_radius - 22)) for angle in np.linspace(-0.66, 0.66, 60)]
    line(frame.base_draw, dial, 1.5, (30, 60, 110))

    generator = np.random.default_rng(11)
    draw = frame.front_draw
    for index, (point_x, point_y) in enumerate(points):
        fluff = lerp(26, 19, index / 40) + generator.uniform(0, 3)
        ellipse(draw, point_x, point_y, fluff, fluff, fill=FUR_DARK)
    draw_tail(draw, points, 46, 34, tip_share=0.12)
    for index in range(4, 36, 7):
        point_x, point_y = points[index]
        radius = lerp(23, 17, index / 40)
        arc(draw, point_x, point_y - 2, radius * 0.9, radius * 0.5, 30, 150, 3, FUR_LIGHT)
    return frame.render()


SCENES = {
    "1-chasing-the-fan": (scene_chasing_fan, 60),
    "2-eros-is-the-fan": (scene_donut_fan, 120),
    "3-thermometer-cat": (scene_thermometer, 120),
    "4-peeking": (scene_peeking, 90),
    "5-space-eros": (scene_space, 120),
    "6-on-the-roof": (scene_roof, 90),
    "7-night-watch": (scene_night_window, 90),
    "8-tail-clock": (scene_tail_clock, 60),
}


def render_scene(job):
    name, output_root = job
    scene_function, frame_count = SCENES[name]
    frame_directory = Path(output_root) / name
    frame_directory.mkdir(parents=True, exist_ok=True)
    for frame_index in range(frame_count):
        scene_function(frame_index / frame_count).save(frame_directory / f"frame_{frame_index:03d}.png")
    return name


if __name__ == "__main__":
    output_root = sys.argv[1]
    selected = sys.argv[2:] or list(SCENES)
    with Pool(min(8, len(selected))) as pool:
        for finished in pool.imap_unordered(render_scene, [(name, output_root) for name in selected]):
            print("done", finished, flush=True)


def preview(output_path):
    tiles = [function(0.6 if "thermo" not in name else 0.5) for name, (function, _) in SCENES.items()]
    sheet = Image.new("RGB", (OUTPUT_SIZE * 4, OUTPUT_SIZE * 2))
    for index, tile in enumerate(tiles):
        sheet.paste(tile, ((index % 4) * OUTPUT_SIZE, (index // 4) * OUTPUT_SIZE))
    sheet.save(output_path)
