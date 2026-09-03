//! The palette. The original is 1-bit black ink on a white screen; sprites in
//! the atlas are white silhouettes on transparency, so drawing them tinted
//! [`INK`] reproduces the authentic look while leaving room to recolour later.

use macroquad::color::Color;
use macroquad::color_u8;

/// The sky / paper the game is drawn on.
pub const SKY: Color = Color::new(1.0, 1.0, 1.0, 1.0);
/// Sprite ink.
pub const INK: Color = Color::new(0.0, 0.0, 0.0, 1.0);
/// Letterbox bars around the virtual canvas.
pub const BARS: Color = color_u8!(24, 24, 24, 255);
