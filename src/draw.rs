//! Sprite drawing helpers.
//!
//! What it is: thin wrappers over `draw_texture_ex` that draw atlas sprites in
//! **logical canvas coordinates** (0..`LOGICAL_W`, 0..`LOGICAL_H`). Call these
//! between [`crate::canvas::Canvas::begin`] and `end`.
//!
//! What it is not: screen/window drawing — that scaling belongs to `canvas`.

use macroquad::prelude::*;

use crate::assets::Assets;
use crate::atlas::Sprite;
use crate::font;

/// Draw `text` with the bitmap font, top-left at `(x, y)`, tinted `color`.
pub fn text(assets: &Assets, text: &str, x: f32, y: f32, color: Color) {
    let mut cx = x;
    for c in text.chars() {
        if let Some(src) = font::glyph(c) {
            draw_texture_ex(
                &assets.font,
                cx,
                y,
                color,
                DrawTextureParams {
                    source: Some(src),
                    ..Default::default()
                },
            );
        }
        cx += (font::advance(c) + 1) as f32;
    }
}

/// Width in pixels the bitmap font would render `text` at.
#[must_use]
pub fn text_width(text: &str) -> i32 {
    text.chars().map(|c| font::advance(c) + 1).sum()
}

/// Draw `which` at logical top-left `(x, y)`, tinted `color`, at native size.
pub fn sprite(assets: &Assets, which: Sprite, x: f32, y: f32, color: Color) {
    draw_texture_ex(
        &assets.atlas,
        x,
        y,
        color,
        DrawTextureParams {
            source: Some(which.rect()),
            ..Default::default()
        },
    );
}
