//! The virtual canvas.
//!
//! What it is: a fixed `LOGICAL_W`×`LOGICAL_H` render target that the whole game
//! draws into, then scaled to fit the real window (aspect-preserving, so it
//! fills the available space with letterbox bars only on the longer axis) and
//! sampled nearest-neighbour to keep the 1-bit pixel art crisp. Gives every
//! platform a single coordinate space, from 4K down to a phone.
//!
//! What it is not: a scene graph or camera system — it only owns the offscreen
//! target and the screen<->canvas mapping.

use macroquad::prelude::*;

use crate::config::{LOGICAL_H, LOGICAL_W};
use crate::screen::{Canvas as CanvasSpace, Logical, Point};

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
    /// centered at the largest scale that fits, so it fills the available space.
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

    /// Map a logical window-space point (e.g. the mouse) into canvas
    /// coordinates, clamped to the playfield. Recomputed per call, so window
    /// resizes are free. The typed spaces make the units unmistakable: a
    /// physical touch position won't type-check here without converting first.
    #[allow(clippy::unused_self)]
    #[must_use]
    pub fn to_canvas(&self, p: Point<Logical>) -> Point<CanvasSpace> {
        let l = Layout::compute();
        Point::new(
            ((p.x() - l.offset_x) / l.scale).clamp(0.0, LOGICAL_W as f32),
            ((p.y() - l.offset_y) / l.scale).clamp(0.0, LOGICAL_H as f32),
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
        // Largest aspect-preserving scale that fits: fills one axis, letterboxes
        // the other. Fractional is fine — nearest sampling keeps pixels crisp.
        let scale = (sw / LOGICAL_W as f32)
            .min(sh / LOGICAL_H as f32)
            .max(f32::EPSILON);
        Self {
            scale,
            offset_x: (sw - LOGICAL_W as f32 * scale) * 0.5,
            offset_y: (sh - LOGICAL_H as f32 * scale) * 0.5,
        }
    }
}
