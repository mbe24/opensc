# StuntCopter sprite assets

Decoded from `StuntCopter.shapes` — a **FullPaint (MacPaint-compatible) document**
recovered from Duane Blehm's original 1986/87 source (the `blehm` repo could not be
checked out on Windows because sibling files contain `:` in their names).

## Format
- 512-byte MacPaint header, then PackBits-compressed 1-bit bitmap, 576×720.
- The stream decoded exactly to EOF; content occupies a 426×261 region that matches
  the game's offscreen bitmap (`OffRight=426`, `OffBottom=261` in `StuntCopter.pas`).
- A set ink bit = black. In `shapes_full.png` this region begins at pixel (60, 25).

## Files
- `shapes_full.png` — full decoded 576×720 canvas (master reference).
- `sprites/` — 52 individual sprites, faithful 1-bit crops (white background).
- `sprites_alpha/` — the game sprites (copter/wagon/flip/man/cross/driver/horse/cloud)
  with white made transparent (RGBA), ready to draw in macroquad.
- `sprites.json` — each sprite's offscreen rect + size.
- `contact_sheet.png` — labeled overview of every sprite.
- `verify_overlay.png` — crop boxes drawn over the sheet (alignment check).

## Sprite map (from `CreateOffScreenRects` in StuntCopter.pas)
| Group | Count | Cell | Notes |
|---|---|---|---|
| `copter_1..3` | 3 | 74×26 | rotor animation frames |
| `wagon_1..3` | 3 | 73×22 | horse-drawn wagon frames |
| `flip_01..14` | 14 | 32×41 | stuntman flip animation (frame 15 = frame 1) |
| `man_hang` | 1 | 14×16 | hanging from copter |
| `man_drop1..5` | 5 | 14×16 | falling poses |
| `man_thumbup` | 1 | 14×16 | success |
| `man_splat1..6` | 6 | 14×16 | crash/splat frames |
| `man_thumbdown` | 1 | 14×16 | failure |
| `num_0..9` | 10 | 21×15 | score font (white digit on hatched bg) |
| `cross` | 1 | 81×81 | control cross / yoke (mouse-to-copter map) |
| `man_in_wagon` | 1 | 28×10 | safe landing |
| `driver` | 1 | 40×22 | shown when driver is hit |
| `horse_dead` | 1 | 29×22 | shown when horse is hit |
| `scorebox` | 1 | — | HUD panel: score/hiscore/height/wagon/gravity |
| `cloud_left/bottom/right` | 3 | — | scenery |

Grid sprites use exact source rects; `scorebox`/`cloud_*` are tightened to ink.
The score numerals sit on the scorebox's diagonal-hatch background (kept as drawn).
