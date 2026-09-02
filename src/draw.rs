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
