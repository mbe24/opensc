//! Input.
//!
//! What it is: samples the real devices into the simulation's device-agnostic
//! [`Intents`] — requested copter velocity plus the drop edge. Three schemes,
//! chosen automatically per frame:
//! - **keyboard** — arrows/WASD steer (full deflection), Space drops; overrides
//!   the others while a steer key is held;
//! - **touch** — a floating joystick on the left ~60% of the screen (wherever
//!   the thumb lands is neutral; drag to fly) and a tap on the right to drop,
//!   so steering and dropping never collide;
//! - **mouse** — the desktop pointer's offset from the canvas centre.
//!
//! What it is not: game logic. The drop edge is latched by the caller and
//! drained once per tick so a fast or slow frame can't lose or double it.

use macroquad::prelude::*;
use stuntcopter_sim::{Intents, Vel};

use crate::canvas::Canvas;
use crate::config::{DELTA_RECT, LOGICAL_H, LOGICAL_W};
use crate::screen::{self, Finger, Physical, Point};

/// Fraction around neutral that reads as no input, so hovering is stable.
const DEAD_ZONE: f32 = 0.10;
/// Thumb travel for full joystick deflection, as a fraction of screen height.
/// Larger = less sensitive (more travel needed for full speed).
const STICK_RADIUS_FRAC: f32 = 0.30;
/// Fraction of the screen width given to the steering half; the rest taps to
/// drop. Steering gets the majority so it never feels cramped.
const STEER_FRACTION: f32 = 0.6;

/// Input sampler. Holds the little state the touch joystick needs across frames.
#[derive(Default)]
pub struct Input {
    /// The active left-half steering touch, if any.
    stick: Option<Stick>,
    /// Set once any touch is seen, so a touch device never falls back to the
    /// (stale) mouse position when no finger is down.
    touch_seen: bool,
}

/// A floating joystick: `anchor` is where the thumb first landed (neutral); the
/// current position comes from the live touch each frame. In physical pixels, to
/// match the touch positions it's compared against.
#[derive(Clone, Copy)]
struct Stick {
    id: u64,
    anchor: Point<Physical>,
}

impl Input {
    /// Sample all input sources for this frame. `mouse_steer` off (a debug aid)
    /// disables mouse steering on desktop.
    pub fn gather(&mut self, canvas: &Canvas, mouse_steer: bool) -> Intents {
        let drop = is_mouse_button_pressed(MouseButton::Left) || is_key_pressed(KeyCode::Space);

        // Keyboard overrides everything while a steer key is held.
        if let Some(req) = keyboard_steer() {
            self.stick = None;
            return Intents { req, drop };
        }

        let mut fingers = screen::fingers().peekable();
        if fingers.peek().is_some() {
            self.touch_seen = true;
            return self.touch_intents(fingers);
        }
        self.stick = None;

        // A touch device with nothing pressed hovers — never chase the stale mouse.
        if self.touch_seen {
            return Intents::default();
        }

        // Desktop: mouse-as-joystick from the pointer's offset to the canvas centre.
        let req = if mouse_steer {
            let p = canvas.to_canvas(screen::mouse());
            velocity(axis(p.x(), LOGICAL_W as f32), axis(p.y(), LOGICAL_H as f32))
        } else {
            Vel::default()
        };
        Intents { req, drop }
    }

    /// Whether a touch has ever been seen (so the UI can show touch hints).
    #[must_use]
    pub fn touch_seen(&self) -> bool {
        self.touch_seen
    }

    fn touch_intents(&mut self, fingers: impl Iterator<Item = Finger>) -> Intents {
        // Everything here is in physical pixels — the space the touch positions
        // come in. The window size is logical, so convert it once; skipping this
        // is what once shrank the steer zone to `STEER_FRACTION / dpi` of the
        // width and made the stick `dpi`× too sensitive on high-DPI phones. The
        // type system now rejects mixing the two spaces.
        let win = screen::window().to_physical(screen::dpi());
        let divider = win.w() * STEER_FRACTION;
        let radius = (win.h() * STICK_RADIUS_FRAC).max(1.0);
        let mut req = Vel::default();
        let mut steering = false;
        let mut drop = false;
        for f in fingers {
            if f.at.x() < divider {
                steering = true;
                // Floating joystick: keep the anchor from when this touch began.
                let anchor = match self.stick {
                    Some(s) if s.id == f.id => s.anchor,
                    _ => f.at,
                };
                self.stick = Some(Stick { id: f.id, anchor });
                let d = f.at - anchor;
                req = velocity(axis_norm(d.x() / radius), axis_norm(d.y() / radius));
            } else if matches!(f.phase, TouchPhase::Started) {
                drop = true;
            }
        }
        if !steering {
            self.stick = None;
        }
        Intents { req, drop }
    }
}

/// Full-deflection steering from the keyboard, or `None` if no steer key is held.
fn keyboard_steer() -> Option<Vel> {
    let held = |keys: &[KeyCode]| keys.iter().any(|&k| is_key_down(k));

    let mut dir_h = 0;
    let mut dir_v = 0;
    let mut active = false;
    if held(&[KeyCode::Left, KeyCode::A]) {
        dir_h = DELTA_RECT.min.dh;
        active = true;
    }
    if held(&[KeyCode::Right, KeyCode::D]) {
        dir_h = DELTA_RECT.max.dh;
        active = true;
    }
    if held(&[KeyCode::Up, KeyCode::W]) {
        dir_v = DELTA_RECT.min.dv;
        active = true;
    }
    if held(&[KeyCode::Down, KeyCode::S]) {
        dir_v = DELTA_RECT.max.dv;
        active = true;
    }
    active.then_some(Vel::new(dir_h, dir_v))
}

/// Map normalized axes in `[-1, 1]` to a requested velocity within [`DELTA_RECT`].
/// The vertical range is asymmetric (the copter rises faster than it climbs), so
/// each direction scales against its own extreme and the centre is a true hover.
fn velocity(nx: f32, ny: f32) -> Vel {
    let req_h = (nx * DELTA_RECT.max.dh as f32).round() as i32;
    let req_v = if ny < 0.0 {
        (ny * DELTA_RECT.min.dv.unsigned_abs() as f32).round() as i32
    } else {
        (ny * DELTA_RECT.max.dv as f32).round() as i32
    };
    Vel::new(
        req_h.clamp(DELTA_RECT.min.dh, DELTA_RECT.max.dh),
        req_v.clamp(DELTA_RECT.min.dv, DELTA_RECT.max.dv),
    )
}

/// Normalize a coordinate to `[-1, 1]` about the centre of `size`, with a dead
/// zone (see [`axis_norm`]).
fn axis(coord: f32, size: f32) -> f32 {
    axis_norm((coord - size / 2.0) / (size / 2.0))
}

/// Apply the centre dead zone to an already-normalized value, re-expanding the
/// remainder so the extreme still reaches ±1.
fn axis_norm(n: f32) -> f32 {
    let n = n.clamp(-1.0, 1.0);
    if n.abs() < DEAD_ZONE {
        0.0
    } else {
        n.signum() * (n.abs() - DEAD_ZONE) / (1.0 - DEAD_ZONE)
    }
}
