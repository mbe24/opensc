//! Game assets, compiled straight into the binary with `include_bytes!`.
//!
//! Embedding (rather than `load_texture(...).await`) keeps native and web builds
//! byte-identical, needs no filesystem or HTTP fetch, and sidesteps asset-path
//! problems under the GitHub Pages `/<repo>/` subpath.

use macroquad::prelude::*;

use crate::config::{HUD_H, LOGICAL_W};

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
        }
    }
}

/// A full-width HUD-strip checkerboard of black/transparent pixels — drawn once
/// behind the HUD, it reads as the original's 50% gray dither over the sky.
fn hud_dither() -> Texture2D {
    let (w, h) = (LOGICAL_W as u16, HUD_H as u16);
    let mut img = Image::gen_image_color(w, h, Color::new(0.0, 0.0, 0.0, 0.0));
    // 25% coverage — a light gray that keeps black content readable once the
    // whole canvas is magnified.
    for y in 0..u32::from(h) {
        for x in 0..u32::from(w) {
            if x % 2 == 0 && y % 2 == 0 {
                img.set_pixel(x, y, BLACK);
            }
        }
    }
    let tex = Texture2D::from_image(&img);
    tex.set_filter(FilterMode::Nearest);
    tex
}
