//! Input.
//!
//! What it is: turns raw per-frame device state into a device-agnostic
//! [`Intents`] value — the requested copter velocity plus edge-triggered
//! actions. Mouse-as-joystick is primary (faithful proportional control);
//! keyboard overrides it while any steer key is held.
//!
//! What it is not: game logic — it decides *what the player asked for*, never
//! what happens. Edge actions (drop) are latched here and drained once per tick
//! by the caller so a fast or slow frame can't lose or double them.

use macroquad::prelude::*;

use crate::canvas::Canvas;
use crate::config::{DELTA_RECT, MOUSE_RECT};

/// What the player is asking for this frame.
pub struct Intents {
    /// Requested copter velocity, px/tick, within [`DELTA_RECT`]. The copter
    /// accelerates toward this; it is not applied directly.
    pub req_dh: i32,
    pub req_dv: i32,
    /// Edge-triggered: the player pressed drop this frame.
    pub drop: bool,
}

/// Sample all input sources for this frame.
#[must_use]
pub fn gather(canvas: &Canvas) -> Intents {
    let (req_dh, req_dv) = match keyboard_steer() {
        Some(kb) => kb,
        None => pointer_steer(canvas),
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

/// Proportional steering from the pointer: its position within [`MOUSE_RECT`]
/// maps linearly onto [`DELTA_RECT`], exactly like the original's `MapPt`.
fn pointer_steer(canvas: &Canvas) -> (i32, i32) {
    let (mouse_x, mouse_y) = mouse_position();
    let p = canvas.screen_to_canvas(vec2(mouse_x, mouse_y));
    let (left, top, right, bottom) = MOUSE_RECT;
    let (min_h, min_v, max_h, max_v) = DELTA_RECT;
    (
        map_clamp(p.x, left, right, min_h, max_h),
        map_clamp(p.y, top, bottom, min_v, max_v),
    )
}

/// Map `v` from input range `[in0, in1]` onto output range `[out0, out1]`,
/// clamping first. Integer division truncates toward zero, matching Pascal.
fn map_clamp(v: f32, in0: i32, in1: i32, out0: i32, out1: i32) -> i32 {
    let v = (v as i32).clamp(in0, in1);
    out0 + (v - in0) * (out1 - out0) / (in1 - in0)
}
