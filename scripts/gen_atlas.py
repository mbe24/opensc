"""Generate the game atlas + typed Rust sprite table from the decoded sheet.

Each sprite is extracted from the decoded FullPaint sheet, recoloured to a
white silhouette on transparency (so the game can TINT it to any colour at draw
time), and PACKED into a fresh atlas with a transparent gutter around every
sprite. The gutter is essential: without it, nearest-neighbour sampling at a
sprite's edge bleeds a sliver of the neighbouring sprite (visible as blinking
lines during animation).
"""

import json, os, re
import numpy as np
from PIL import Image

REF = r"D:\training\opensc\reference\stuntcopter\png"
ASSETS = r"D:\training\opensc\assets"
SRC = r"D:\training\opensc\src"
os.makedirs(ASSETS, exist_ok=True)

# Where the artwork sits inside the full decoded sheet.
OX, OY = 60, 25
PAD = 2          # transparent gutter around every sprite, in pixels
ATLAS_W = 512    # wide enough for the widest sprite (scorebox, ~388px)

sheet = Image.open(os.path.join(REF, "shapes_full.png")).convert("L")
meta = json.load(open(os.path.join(REF, "sprites.json")))


def edge_divider_mask(ink):
    """Pixels belonging to detached full-span divider lines at the cell edges.

    The FullPaint sheet has layout lines between rows/columns that fall inside a
    sprite's rect but aren't part of the sprite (separated by a blank gap). A
    divider is a near-full row/column with a blank line just inward. We peel such
    lines (and any blank lines) from each edge until we reach real content, so a
    line touching the sprite is never mistaken for a divider.
    """
    h, w = ink.shape
    clear = np.zeros_like(ink)

    def peel(is_divider, is_blank, mark):
        i = 0
        while i < max(h, w) - 1:
            if is_divider(i):
                mark(i)
            elif is_blank(i):
                pass
            else:
                break
            i += 1

    peel(lambda i: ink[i].mean() >= 0.5 and ink[i + 1].mean() <= 0.1,
         lambda i: ink[i].mean() == 0, lambda i: clear.__setitem__((i, slice(None)), True))
    peel(lambda i: ink[h - 1 - i].mean() >= 0.5 and ink[h - 2 - i].mean() <= 0.1,
         lambda i: ink[h - 1 - i].mean() == 0, lambda i: clear.__setitem__((h - 1 - i, slice(None)), True))
    peel(lambda i: ink[:, i].mean() >= 0.5 and ink[:, i + 1].mean() <= 0.1,
         lambda i: ink[:, i].mean() == 0, lambda i: clear.__setitem__((slice(None), i), True))
    peel(lambda i: ink[:, w - 1 - i].mean() >= 0.5 and ink[:, w - 2 - i].mean() <= 0.1,
         lambda i: ink[:, w - 1 - i].mean() == 0, lambda i: clear.__setitem__((slice(None), w - 1 - i), True))
    return clear


def remove_border_specks(ink, max_size=3):
    """Erase tiny ink blobs that touch the cell border — 1-2px bleed from the
    neighbouring sprite in the tightly-packed source sheet. Interior detail
    (e.g. cloud shading dots) never touches the border, so it is preserved."""
    from collections import deque

    h, w = ink.shape
    seen = np.zeros_like(ink)
    for sy in range(h):
        for sx in range(w):
            if not ink[sy, sx] or seen[sy, sx]:
                continue
            queue, comp, touches = deque([(sy, sx)]), [], False
            seen[sy, sx] = True
            while queue:
                y, x = queue.popleft()
                comp.append((y, x))
                if y in (0, h - 1) or x in (0, w - 1):
                    touches = True
                for dy, dx in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                    ny, nx = y + dy, x + dx
                    if 0 <= ny < h and 0 <= nx < w and ink[ny, nx] and not seen[ny, nx]:
                        seen[ny, nx] = True
                        queue.append((ny, nx))
            if touches and len(comp) <= max_size:
                for y, x in comp:
                    ink[y, x] = False
    return ink


def remove_isolated_hlines(ink, min_fill=0.7, max_neighbor=0.2):
    """Erase a full-width horizontal line that has (near-)empty rows both above
    and below it -- a detached layout line, not a sprite's connected baseline."""
    h = ink.shape[0]
    for r in range(1, h - 1):
        if (ink[r].mean() >= min_fill
                and ink[r - 1].mean() <= max_neighbor
                and ink[r + 1].mean() <= max_neighbor):
            ink[r] = False
    return ink


def silhouette(rect):
    """Extract an offscreen rect as white-on-transparent RGBA, with detached
    layout divider lines and border-bleed specks erased."""
    x0, y0, x1, y1 = rect
    region = np.array(sheet.crop((x0 + OX, y0 + OY, x1 + OX, y1 + OY)))
    ink = region < 128
    ink &= ~edge_divider_mask(ink)
    ink = remove_isolated_hlines(ink)
    ink = remove_border_specks(ink)
    rgba = np.zeros((*ink.shape, 4), dtype=np.uint8)
    rgba[ink] = (255, 255, 255, 255)
    return Image.fromarray(rgba, "RGBA")


def ident(name):
    """sprite key -> CamelCase enum variant (copter_1 -> Copter1)."""
    return "".join(part.capitalize() for part in re.split(r"[_]+", name))


# Extract every sprite, then shelf-pack tallest-first for a tight atlas.
sprites = []
for name in sorted(meta):
    img = silhouette(meta[name]["offscreen_rect"])
    sprites.append((name, img))
sprites.sort(key=lambda s: (-s[1].height, s[0]))

placements = {}  # name -> (x, y, w, h)
cur_x, cur_y, row_h = PAD, PAD, 0
for name, img in sprites:
    if cur_x + img.width + PAD > ATLAS_W:
        cur_x, cur_y, row_h = PAD, cur_y + row_h + PAD, 0
    placements[name] = (cur_x, cur_y, img.width, img.height)
    cur_x += img.width + PAD
    row_h = max(row_h, img.height)
atlas_h = cur_y + row_h + PAD

atlas = Image.new("RGBA", (ATLAS_W, atlas_h), (0, 0, 0, 0))
for name, img in sprites:
    x, y, _, _ = placements[name]
    atlas.paste(img, (x, y))
atlas.save(os.path.join(ASSETS, "atlas.png"))
print("atlas.png", atlas.size)

# Emit the typed Sprite enum + packed source rects (atlas pixel coords).
variants = [(ident(n), n) for n in sorted(meta)]
lines = [
    "// @generated by scripts/gen_atlas.py from reference/stuntcopter/png/sprites.json",
    "// Do not edit by hand. Regenerate to change sprite rects.",
    "//",
    "// Each rect is a padded source rectangle within `atlas.png` (packed with a",
    "// transparent gutter so nearest-neighbour sampling never bleeds neighbours).",
    "//",
    "// The full sprite set is generated up front; not every variant is drawn yet.",
    "#![allow(dead_code)]",
    "// Generated exhaustive match over every sprite; length is inherent.",
    "#![allow(clippy::too_many_lines)]",
    "",
    "use macroquad::math::Rect;",
    "",
    "/// A sprite region within the packed atlas texture.",
    "#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]",
    "pub enum Sprite {",
]
for var, orig in variants:
    lines.append(f"    /// `{orig}`")
    lines.append(f"    {var},")
lines.append("}")
lines.append("")
lines.append("impl Sprite {")
lines.append("    /// Source rectangle of this sprite within the atlas texture, in pixels.")
lines.append("    #[must_use]")
lines.append("    #[rustfmt::skip]")
lines.append("    pub const fn rect(self) -> Rect {")
lines.append("        match self {")
for var, orig in variants:
    x, y, w, h = placements[orig]
    lines.append(
        f"            Self::{var} => Rect {{ x: {x}.0, y: {y}.0, w: {w}.0, h: {h}.0 }},"
    )
lines.append("        }")
lines.append("    }")
lines.append("}")
lines.append("")

open(os.path.join(SRC, "atlas.rs"), "w", newline="\n").write("\n".join(lines))
print("src/atlas.rs", len(variants), "sprites,", "atlas", ATLAS_W, "x", atlas_h)
