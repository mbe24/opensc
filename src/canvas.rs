//! The virtual canvas.
//!
//! What it is: a fixed `LOGICAL_W`×`LOGICAL_H` render target that the whole game
//! draws into, then integer-scaled and letterboxed onto the real window. This
//! keeps the 1-bit pixel art crisp at any resolution (4K down to a phone) and
//! gives every platform a single coordinate space.
//!
//! What it is not: a scene graph or camera system — it only owns the offscreen
//! target and the screen<->canvas mapping.

use macroquad::prelude::*;

use crate::config::{LOGICAL_H, LOGICAL_W};

pub struct Canvas {
    target: RenderTarget,
    camera: Camera2D,
}

impl Default for Canvas {
    fn default() -> Self {
        Self::new()
    }
}

impl Canvas {
    /// Allocate the offscreen target. Requires an initialized macroquad context.
    #[must_use]
    pub fn new() -> Self {
        let target = render_target(LOGICAL_W as u32, LOGICAL_H as u32);
        // Crisp scaling — without this the pixel art blurs when magnified.
        target.texture.set_filter(FilterMode::Nearest);

        let mut camera =
            Camera2D::from_display_rect(Rect::new(0.0, 0.0, LOGICAL_W as f32, LOGICAL_H as f32));
        camera.render_target = Some(target.clone());

        Self { target, camera }
    }

    /// Begin drawing into the canvas, in logical coordinates (y-down, origin
    /// top-left). Pair with [`Canvas::end`].
    pub fn begin(&self) {
        set_camera(&self.camera);
    }

    /// Stop drawing into the canvas; subsequent draws target the window.
    // A canvas operation by design, paired with `begin`; takes `&self` for symmetry.
    #[allow(clippy::unused_self)]
    pub fn end(&self) {
        set_default_camera();
    }

    /// Present the canvas: fill the window with `bars`, then draw the canvas
    /// centered at the largest whole integer scale that fits (fractional only
    /// when the window is smaller than the canvas).
    pub fn present(&self, bars: Color) {
        clear_background(bars);
        let l = Layout::compute();
        draw_texture_ex(
            &self.target.texture,
            l.offset_x,
            l.offset_y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(LOGICAL_W as f32 * l.scale, LOGICAL_H as f32 * l.scale)),
                // Camera2D render targets are stored vertically flipped
                // (macroquad issue #171); flip back when presenting.
                flip_y: true,
                ..Default::default()
            },
        );
    }

    /// Map a window-space point (e.g. the mouse) into logical canvas coordinates,
    /// clamped to the playfield. Recomputed per call, so window resizes are free.
    // Consumed by the input layer (next milestone); a canvas-space operation.
    #[allow(dead_code, clippy::unused_self)]
    #[must_use]
    pub fn screen_to_canvas(&self, screen: Vec2) -> Vec2 {
        let l = Layout::compute();
        vec2(
            ((screen.x - l.offset_x) / l.scale).clamp(0.0, LOGICAL_W as f32),
            ((screen.y - l.offset_y) / l.scale).clamp(0.0, LOGICAL_H as f32),
        )
    }
}

/// Where the canvas lands in the window this frame.
struct Layout {
    scale: f32,
    offset_x: f32,
    offset_y: f32,
}

impl Layout {
    fn compute() -> Self {
        let (sw, sh) = (screen_width(), screen_height());
        let raw = (sw / LOGICAL_W as f32).min(sh / LOGICAL_H as f32);
        // Integer scale when the canvas fits; fractional only on tiny windows.
        let scale = if raw >= 1.0 {
            raw.floor()
        } else {
            raw.max(f32::EPSILON)
        };
        Self {
            scale,
            offset_x: (sw - LOGICAL_W as f32 * scale) * 0.5,
            offset_y: (sh - LOGICAL_H as f32 * scale) * 0.5,
        }
    }
}
