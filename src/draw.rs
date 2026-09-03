//! Sprite drawing helpers.
//!
//! What it is: thin wrappers over `draw_texture_ex` that draw atlas sprites in
//! **logical canvas coordinates** (0..`LOGICAL_W`, 0..`LOGICAL_H`). Call these
//! between [`crate::canvas::Canvas::begin`] and `end`.
//!
//! What it is not: screen/window drawing — that scaling belongs to `canvas`.

use std::fmt::Write;

use macroquad::prelude::*;
use stuntcopter_sim::Sprite;

use crate::assets::Assets;
use crate::font;

/// A fixed-capacity stack string for short UI labels (scores, "LEVEL n"), so
/// building them each frame allocates nothing. `write!` into it, then
/// [`StackStr::as_str`]. Intended for short ASCII labels; a write past `N` is
/// truncated rather than panicking.
pub struct StackStr<const N: usize> {
    buf: [u8; N],
    len: usize,
}

impl<const N: usize> StackStr<N> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            buf: [0; N],
            len: 0,
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        // Only whole &str chunks of ASCII labels are written, so this is valid.
        std::str::from_utf8(&self.buf[..self.len]).unwrap_or("")
    }
}

impl<const N: usize> Default for StackStr<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> Write for StackStr<N> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        let end = (self.len + s.len()).min(N);
        let take = end - self.len;
        self.buf[self.len..end].copy_from_slice(&s.as_bytes()[..take]);
        self.len = end;
        Ok(())
    }
}

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

/// Draw `text` scaled by `scale` (integer-crisp), top-left at `(x, y)`.
pub fn text_scaled(assets: &Assets, text: &str, x: f32, y: f32, scale: f32, color: Color) {
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
                    dest_size: Some(vec2(src.w * scale, src.h * scale)),
                    ..Default::default()
                },
            );
        }
        cx += (font::advance(c) + 1) as f32 * scale;
    }
}

/// Fill a rounded rectangle (classic Mac push-button shape): a plus of two
/// rectangles with a disc tucked into each corner.
pub fn round_rect(x: f32, y: f32, w: f32, h: f32, radius: f32, color: Color) {
    draw_rectangle(x + radius, y, w - 2.0 * radius, h, color);
    draw_rectangle(x, y + radius, w, h - 2.0 * radius, color);
    for (cx, cy) in [
        (x + radius, y + radius),
        (x + w - radius, y + radius),
        (x + radius, y + h - radius),
        (x + w - radius, y + h - radius),
    ] {
        draw_circle(cx, cy, radius, color);
    }
}

/// Draw `which` at logical top-left `(x, y)`, tinted `color`, at native size.
pub fn sprite(assets: &Assets, which: Sprite, x: f32, y: f32, color: Color) {
    let r = which.rect();
    draw_texture_ex(
        &assets.atlas,
        x,
        y,
        color,
        DrawTextureParams {
            source: Some(Rect::new(r.x, r.y, r.w, r.h)),
            ..Default::default()
        },
    );
}
