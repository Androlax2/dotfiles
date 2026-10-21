#!/usr/bin/env python3
"""Exercise tmux 3.8's border draw entry point without starting a server.

Usage: python3 test-border-junction.py /path/to/tmux/source
The source tree must contain screen-redraw.c and tmux.h, with the patch applied.
Only terminal writes and unrelated style/option lookups are replaced.
"""

import argparse
import os
from pathlib import Path
import re
import subprocess
import tempfile


def extract_function(source, name):
    signature = source.index(f"\n{name}(")
    start = source.rfind("\nstatic ", 0, signature) + 1
    end = source.index("\n}", signature) + 3
    return source[start:end]


parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("source_root", type=Path)
source_root = parser.parse_args().source_root
source = (source_root / "screen-redraw.c").read_text()
header = (source_root / "tmux.h").read_text()
cell_constants = "\n".join(re.findall(r"^#define CELL_\w+ \d+$", header, re.M))
pane_lines = re.search(r"enum pane_lines \{.*?\n};", header, re.S).group()
span_types = source[source.index("enum redraw_span_type {"):source.index("/* A visible line")]
functions = "\n".join(extract_function(source, name) for name in [
    "redraw_data_has_pane",
    "redraw_get_pane_for_border_style",
    "redraw_draw_border_arrow",
    "redraw_draw_border_span",
])

harness = r'''
#include <assert.h>
#include <stdio.h>
#include <string.h>
#include <sys/types.h>
#define TAILQ_ENTRY(type) struct { struct type *next; }
#define TAILQ_HEAD(name, type) struct name { struct type *first; }
#define GRID_ATTR_CHARSET 1
#define GRID_ATTR_REVERSE 2
#define REDRAW_ISOLATES 1
#define REDRAW_STATUS_TOP 4
struct options { const char *clip_top; };
struct window_pane { u_int sx; int xoff, yoff; };
struct window { struct options *options; };
struct tty { int unused; };
struct client { struct tty tty; };
struct redraw_scene { struct client *c; struct window *w; u_int ox, oy; };
struct redraw_draw_ctx {
    struct redraw_scene *scene;
    struct window_pane *active, *marked;
    u_int status_lines;
    int flags;
};
struct grid_cell { int cell_type, attr; struct { char value; } data; };
static struct grid_cell drawn;
static int writes;
static void *options_get(struct options *oo, const char *name) {
    assert(strcmp(name, "@pane-border-clip-top") == 0);
    return (void *)oo->clip_top;
}
const char *options_get_string(struct options *oo, const char *name) {
    assert(options_get(oo, name) != NULL);
    return oo->clip_top;
}
static void redraw_get_default_border_style(struct redraw_draw_ctx *ctx,
    struct grid_cell *gc, enum pane_lines *lines) {
    (void)ctx; *gc = (struct grid_cell){0}; *lines = PANE_LINES_SINGLE;
}
static void window_get_fill_cell(struct window *w, int inside, struct grid_cell *gc) {
    (void)w; (void)inside; gc->cell_type = CELL_NONE;
}
static void window_get_border_cell(struct window_pane *wp, enum pane_lines lines,
    u_int type, struct grid_cell *gc) {
    (void)wp; (void)lines; gc->cell_type = type;
}
static void window_pane_get_border_style(struct window_pane *wp, struct client *c,
    struct grid_cell *gc) { (void)wp; (void)c; *gc = (struct grid_cell){0}; }
static void window_pane_get_border_cell(struct window_pane *wp, u_int type,
    struct grid_cell *gc) { (void)wp; gc->cell_type = type; }
static void utf8_set(void *data, char ch) { (void)data; (void)ch; }
static void tty_cursor(struct tty *tty, u_int x, u_int y) {
    (void)tty; (void)x; (void)y;
}
static void tty_puts(struct tty *tty, const char *text) { (void)tty; (void)text; }
static void tty_cell(struct tty *tty, struct grid_cell *gc, void *context) {
    (void)tty; (void)context; drawn = *gc; writes++;
}
'''

cases = r'''
int main(void) {
    struct window_pane left = {0}, right = {0}, other = {0};
    struct options options = {.clip_top = "on"};
    struct window window = {.options = &options};
    struct client client = {0};
    struct redraw_scene scene = {.c = &client, .w = &window};
    struct redraw_draw_ctx ctx = {.scene = &scene, .active = &right};
    struct redraw_span span = {.data = {.type = REDRAW_SPAN_BORDER,
        .b = {.left_wp = &left, .right_wp = &right, .cell_type = CELL_LRD}}};
    struct test_case {
        const char *name, *option;
        struct window_pane *active, *right_wp;
        u_int y, offset, status_lines;
        int flags, type, expected;
    } cases[] = {
        {"right focus has no left arm", "on", &right, &right, 0, 0, 0, 0, CELL_LRD, CELL_RD},
        {"left focus has no right arm", "on", &left, &right, 0, 0, 0, 0, CELL_LRD, CELL_LD},
        {"unset option preserves tmux", NULL, &right, &right, 0, 0, 0, 0, CELL_LRD, CELL_LRD},
        {"disabled option preserves tmux", "off", &right, &right, 0, 0, 0, 0, CELL_LRD, CELL_LRD},
        {"unfocused junction is unchanged", "on", &other, &right, 0, 0, 0, 0, CELL_LRD, CELL_LRD},
        {"no active pane is unchanged", "on", NULL, &right, 0, 0, 0, 0, CELL_LRD, CELL_LRD},
        {"interior junction is unchanged", "on", &right, &right, 5, 0, 0, 0, CELL_LRD, CELL_LRD},
        {"scrolled viewport is unchanged", "on", &right, &right, 0, 5, 0, 0, CELL_LRD, CELL_LRD},
        {"top status rows offset the junction", "on", &right, &right, 2, 0, 2, REDRAW_STATUS_TOP, CELL_LRD, CELL_RD},
        {"status row is not window top", "on", &right, &right, 0, 0, 2, REDRAW_STATUS_TOP, CELL_LRD, CELL_LRD},
        {"bottom status does not offset", "on", &left, &right, 0, 0, 2, 0, CELL_LRD, CELL_LD},
        {"vertical divider is unchanged", "on", &right, &right, 0, 0, 0, 0, CELL_UD, CELL_UD},
        {"cross junction is unchanged", "on", &right, &right, 0, 0, 0, 0, CELL_LRUD, CELL_LRUD},
        {"outer border is unchanged", "on", &left, NULL, 0, 0, 0, 0, CELL_LRD, CELL_LRD},
    };
    int failures = 0;
    for (u_int index = 0; index < sizeof cases / sizeof cases[0]; index++) {
        struct test_case *test = &cases[index];
        options.clip_top = test->option;
        ctx.active = test->active;
        ctx.flags = test->flags;
        ctx.status_lines = test->status_lines;
        scene.oy = test->offset;
        span.data.b.right_wp = test->right_wp;
        span.data.b.cell_type = test->type;
        writes = 0;
        redraw_draw_border_span(&ctx, &span, 30, test->y, 1);
        if (writes != 1 || drawn.cell_type != test->expected) {
            fprintf(stderr, "FAIL: %s: got cell %d, expected %d\n",
                test->name, drawn.cell_type, test->expected);
            failures++;
        }
    }

    /* A pane under another, its top border on row 10 and its sides on columns 30 and 71. */
    struct window_pane lower = {.xoff = 31, .yoff = 11, .sx = 40}, upper = {0};
    struct redraw_span corner = {.data = {.type = REDRAW_SPAN_BORDER,
        .b = {.top_wp = &upper, .bottom_wp = &lower}}};
    struct corner_case {
        const char *name, *option;
        struct window_pane *active;
        u_int x, y, ox, oy, status_lines;
        int flags, type, expected;
    } corner_cases[] = {
        {"focused top-left corner has no up arm", "on", &lower, 30, 10, 0, 0, 0, 0, CELL_URD, CELL_RD},
        {"focused top-right corner has no up arm", "on", &lower, 71, 10, 0, 0, 0, 0, CELL_ULD, CELL_LD},
        {"unset option preserves the corner", NULL, &lower, 30, 10, 0, 0, 0, 0, CELL_URD, CELL_URD},
        {"disabled option preserves the corner", "off", &lower, 30, 10, 0, 0, 0, 0, CELL_URD, CELL_URD},
        {"unfocused corner is unchanged", "on", &upper, 30, 10, 0, 0, 0, 0, CELL_URD, CELL_URD},
        {"side junction below the top is unchanged", "on", &lower, 30, 15, 0, 0, 0, 0, CELL_URD, CELL_URD},
        {"junction left of the corner is unchanged", "on", &lower, 29, 10, 0, 0, 0, 0, CELL_URD, CELL_URD},
        {"right-hand junction on the left is unchanged", "on", &lower, 30, 10, 0, 0, 0, 0, CELL_ULD, CELL_ULD},
        {"top status rows offset the corner", "on", &lower, 30, 12, 0, 0, 2, REDRAW_STATUS_TOP, CELL_URD, CELL_RD},
        {"scrolled viewport keeps the corner", "on", &lower, 25, 7, 5, 3, 0, 0, CELL_URD, CELL_RD},
        {"cross junction at the corner is unchanged", "on", &lower, 30, 10, 0, 0, 0, 0, CELL_LRUD, CELL_LRUD},
    };
    for (u_int index = 0; index < sizeof corner_cases / sizeof corner_cases[0]; index++) {
        struct corner_case *test = &corner_cases[index];
        options.clip_top = test->option;
        ctx.active = test->active;
        ctx.flags = test->flags;
        ctx.status_lines = test->status_lines;
        scene.ox = test->ox;
        scene.oy = test->oy;
        corner.data.b.cell_type = test->type;
        writes = 0;
        redraw_draw_border_span(&ctx, &corner, test->x, test->y, 1);
        if (writes != 1 || drawn.cell_type != test->expected) {
            fprintf(stderr, "FAIL: %s: got cell %d, expected %d\n",
                test->name, drawn.cell_type, test->expected);
            failures++;
        }
    }
    if (failures) return 1;
    puts("25 border rendering cases passed");
    return 0;
}
'''

with tempfile.TemporaryDirectory(prefix="tmux-border-test-") as temporary:
    test_source = Path(temporary) / "border.c"
    executable = Path(temporary) / "border-test"
    test_source.write_text("\n".join([cell_constants, pane_lines, harness, span_types, functions, cases]))
    subprocess.run([os.environ.get("CC", "cc"), "-Wall", "-Wextra", "-Werror",
                    str(test_source), "-o", str(executable)], check=True)
    subprocess.run([str(executable)], check=True)
