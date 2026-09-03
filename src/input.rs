//! Input.
//!
//! What it is: samples the real devices into the simulation's device-agnostic
//! [`Intents`] — the requested copter velocity plus edge-triggered actions.
//! Mouse-as-joystick is primary (faithful proportional control); keyboard
//! overrides it while any steer key is held.
//!
//! What it is not: game logic — it decides *what the player asked for*, never
//! what happens. The drop edge is latched by the caller and drained once per
//! tick so a fast or slow frame can't lose or double it.

use macroquad::prelude::*;
use stuntcopter_sim::Intents;

use crate::canvas::Canvas;
use crate::config::{DELTA_RECT, LOGICAL_H, LOGICAL_W};

/// Fraction of each half-axis around the canvas centre that reads as neutral,
/// so the copter can hover without the pointer being pixel-perfect.
const DEAD_ZONE: f32 = 0.10;

/// Sample all input sources for this frame. When `mouse_steer` is off (a debug
/// aid for reproducible testing), only the keyboard steers.
#[must_use]
pub fn gather(canvas: &Canvas, mouse_steer: bool) -> Intents {
    let (req_dh, req_dv) = match keyboard_steer() {
        Some(kb) => kb,
        None if mouse_steer => pointer_steer(canvas),
        None => (0, 0),
    };
    Intents {
        req_dh,
        req_dv,
        drop: is_mouse_button_pressed(MouseButton::Left) || is_key_pressed(KeyCode::Space),
    }
}

/// Full-deflection steering from the keyboard, or `None` if no steer key is held.
fn keyboard_steer() -> Option<(i32, i32)> {
    let (min_h, min_v, max_h, max_v) = DELTA_RECT;
    let held = |keys: &[KeyCode]| keys.iter().any(|&k| is_key_down(k));

    let mut dir_h = 0;
    let mut dir_v = 0;
    let mut active = false;
    if held(&[KeyCode::Left, KeyCode::A]) {
        dir_h = min_h;
        active = true;
    }
    if held(&[KeyCode::Right, KeyCode::D]) {
        dir_h = max_h;
        active = true;
    }
    if held(&[KeyCode::Up, KeyCode::W]) {
        dir_v = min_v;
        active = true;
    }
    if held(&[KeyCode::Down, KeyCode::S]) {
        dir_v = max_v;
        active = true;
    }
    active.then_some((dir_h, dir_v))
}

/// Proportional steering from the pointer.
///
/// Departure from the original: instead of the tiny faithful [`MOUSE_RECT`]
/// control box (which made anywhere outside it full-deflection), the pointer's
/// offset from the canvas centre drives the requested velocity across the whole
/// canvas, with a dead zone at the centre for stable hovering. The edges reach
/// the [`DELTA_RECT`] extremes.
fn pointer_steer(canvas: &Canvas) -> (i32, i32) {
    let (mouse_x, mouse_y) = mouse_position();
    let p = canvas.screen_to_canvas(vec2(mouse_x, mouse_y));
    let nx = axis(p.x, LOGICAL_W);
    let ny = axis(p.y, LOGICAL_H);
    let (min_h, min_v, max_h, max_v) = DELTA_RECT;

    // Vertical range is asymmetric (rises faster than it climbs); scale up/down
    // against the matching extreme so the centre is always a true hover.
    let req_h = (nx * max_h as f32).round() as i32;
    let req_v = if ny < 0.0 {
        (ny * min_v.unsigned_abs() as f32).round() as i32
    } else {
        (ny * max_v as f32).round() as i32
    };
    (req_h.clamp(min_h, max_h), req_v.clamp(min_v, max_v))
}

/// Normalize a canvas coordinate to `[-1, 1]` about the centre of `size`, with a
/// dead zone: within [`DEAD_ZONE`] of centre returns 0, and the remainder is
/// re-expanded so the edge still reaches ±1.
fn axis(coord: f32, size: i32) -> f32 {
    let half = size as f32 / 2.0;
    let n = ((coord - half) / half).clamp(-1.0, 1.0);
    if n.abs() < DEAD_ZONE {
        0.0
    } else {
        n.signum() * (n.abs() - DEAD_ZONE) / (1.0 - DEAD_ZONE)
    }
}
