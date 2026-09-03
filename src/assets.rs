//! Game assets, compiled straight into the binary with `include_bytes!`.
//!
//! Embedding (rather than `load_texture(...).await`) keeps native and web builds
//! byte-identical, needs no filesystem or HTTP fetch, and sidesteps asset-path
//! problems under the GitHub Pages `/<repo>/` subpath.

use macroquad::prelude::*;

use crate::config::{HUD_BAND_TOP, HUD_YOKE_WIN, LOGICAL_H, LOGICAL_W};

/// The packed atlas: every sprite as a white silhouette on transparency, so it
/// can be tinted to any colour at draw time (see [`crate::atlas::Sprite`]).
const ATLAS_PNG: &[u8] = include_bytes!("../assets/atlas.png");

/// The 1-bit bitmap font atlas (see [`crate::font`]).
const FONT_PNG: &[u8] = include_bytes!("../assets/font.png");

/// Handles to every loaded asset. Built once at startup and passed by reference.
pub struct Assets {
    pub atlas: Texture2D,
    pub font: Texture2D,
    /// A 1-px-checkerboard gray tile, tinted and tiled for the HUD panel.
    pub dither: Texture2D,
    /// A 50% checkerboard sized to the yoke window, covering the yoke while no
    /// game is underway (the original's `FillRect(YokeErase, Gray)`).
    pub yoke_gray: Texture2D,
}

impl Assets {
    /// Decode the embedded PNGs. Synchronous and infallible for embedded bytes;
    /// returns textures ready to draw with crisp nearest-neighbour scaling.
    #[must_use]
    pub fn load() -> Self {
        let atlas = Texture2D::from_file_with_format(ATLAS_PNG, Some(ImageFormat::Png));
        let font = Texture2D::from_file_with_format(FONT_PNG, Some(ImageFormat::Png));
        for t in [&atlas, &font] {
            t.set_filter(FilterMode::Nearest);
        }
        Self {
            atlas,
            font,
            dither: hud_dither(),
            yoke_gray: yoke_gray(),
        }
    }
}

/// A 50% black/transparent checkerboard exactly the size of the yoke window, so
/// it drops in crisp and phase-aligned with no scaling.
fn yoke_gray() -> Texture2D {
    let (_, _, w, h) = HUD_YOKE_WIN;
    let mut img = Image::gen_image_color(w as u16, h as u16, Color::new(0.0, 0.0, 0.0, 0.0));
    for y in 0..u32::from(h as u16) {
        for x in 0..u32::from(w as u16) {
            if (x + y) % 2 == 0 {
                img.set_pixel(x, y, BLACK);
            }
        }
    }
    let tex = Texture2D::from_image(&img);
    tex.set_filter(FilterMode::Nearest);
    tex
}

/// A full-width HUD-strip fill in the classic QuickDraw `dkGray` pattern (75%
/// black), matching the original's `FillRect(scoreBox background, dkGray)`. It
/// shows through in the margins beside the opaque ScoreBox panel.
fn hud_dither() -> Texture2D {
    // QuickDraw dkGray: rows alternate 0x77 / 0xDD, six of eight bits set.
    const DK_GRAY: [u8; 8] = [0x77, 0xDD, 0x77, 0xDD, 0x77, 0xDD, 0x77, 0xDD];
    let (w, h) = (LOGICAL_W as u16, (LOGICAL_H - HUD_BAND_TOP) as u16);
    let mut img = Image::gen_image_color(w, h, Color::new(0.0, 0.0, 0.0, 0.0));
    for y in 0..u32::from(h) {
        for x in 0..u32::from(w) {
            let set = (DK_GRAY[(y % 8) as usize] >> (7 - x % 8)) & 1;
            if set == 1 {
                img.set_pixel(x, y, BLACK);
            }
        }
    }
    let tex = Texture2D::from_image(&img);
    tex.set_filter(FilterMode::Nearest);
    tex
}
