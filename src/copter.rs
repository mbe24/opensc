//! The player's copter.
//!
//! What it is: position, velocity and rotor-animation state, plus the faithful
//! inertia model — the copter accelerates one pixel/tick toward the requested
//! velocity, so reversing direction takes several ticks. Ported from
//! `AnimateOneLoop` in the original.

use crate::atlas::Sprite;
use crate::config::{COPTER_BOTTOM_LIMIT, COPTER_H, COPTER_START, COPTER_W, LOGICAL_W};

/// Copter can drift this far past a screen edge before wrapping (`BorderRect`).
const WRAP_LEFT: i32 = -76;
const WRAP_RIGHT: i32 = 510;
/// Highest the copter may climb (`BorderRect` top).
const TOP_LIMIT: i32 = -4;

pub struct Copter {
    /// Top-left position, logical pixels.
    pub x: i32,
    pub y: i32,
    /// Current velocity, px/tick.
    dh: i32,
    dv: i32,
    /// Rotor animation frame, 0..3.
    frame: u8,
}

impl Default for Copter {
    fn default() -> Self {
        Self {
            x: COPTER_START.0,
            y: COPTER_START.1,
            dh: 0,
            dv: 0,
            frame: 0,
        }
    }
}

impl Copter {
    /// Advance one tick toward the requested velocity, then move and confine.
    pub fn tick(&mut self, req_dh: i32, req_dv: i32) {
        // Inertia: step velocity one pixel/tick toward the request.
        self.dh += (req_dh - self.dh).signum();
        self.dv += (req_dv - self.dv).signum();

        self.x += self.dh;
        self.y += self.dv;

        self.confine();
        self.frame = (self.frame + 1) % 3;
    }

    /// Wrap horizontally at the far edges; clamp vertically to the flight band.
    fn confine(&mut self) {
        if self.x > WRAP_RIGHT {
            self.x = -COPTER_W;
        } else if self.x < WRAP_LEFT {
            self.x = LOGICAL_W - 2;
        }

        if self.y < TOP_LIMIT {
            self.y = TOP_LIMIT;
        } else if self.y + COPTER_H > COPTER_BOTTOM_LIMIT {
            self.y = COPTER_BOTTOM_LIMIT - COPTER_H;
        }
    }

    /// Current rotor sprite.
    #[must_use]
    pub fn sprite(&self) -> Sprite {
        match self.frame {
            0 => Sprite::Copter1,
            1 => Sprite::Copter2,
            _ => Sprite::Copter3,
        }
    }
}
