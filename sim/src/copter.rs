//! The player's copter.
//!
//! What it is: position, velocity and rotor-animation state, plus the faithful
//! inertia model — the copter accelerates one pixel/tick toward the requested
//! velocity, so reversing direction takes several ticks. Ported from
//! `AnimateOneLoop` in the original.

use crate::config::{COPTER_BOTTOM_LIMIT, COPTER_H, COPTER_START, COPTER_W, LOGICAL_W};
use crate::geom::{Pos, Vel};
use crate::sprite::Sprite;

/// Copter can drift this far past a screen edge before wrapping (`BorderRect`).
const WRAP_LEFT: i32 = -76;
const WRAP_RIGHT: i32 = 510;
/// Highest the copter may climb (`BorderRect` top).
const TOP_LIMIT: i32 = -4;

pub struct Copter {
    /// Top-left position, logical pixels.
    pub pos: Pos,
    /// Current velocity, px/tick.
    vel: Vel,
    /// Rotor animation frame, 0..3.
    frame: u8,
}

impl Default for Copter {
    fn default() -> Self {
        Self {
            pos: COPTER_START,
            vel: Vel::default(),
            frame: 0,
        }
    }
}

impl Copter {
    /// Advance one tick toward the requested velocity, then move and confine.
    pub fn tick(&mut self, req: Vel) {
        // Inertia: step velocity one pixel/tick toward the request.
        self.vel.dh += (req.dh - self.vel.dh).signum();
        self.vel.dv += (req.dv - self.vel.dv).signum();

        self.pos += self.vel;

        self.confine();
        self.frame = (self.frame + 1) % 3;
    }

    /// Wrap horizontally at the far edges; clamp vertically to the flight band.
    fn confine(&mut self) {
        if self.pos.x > WRAP_RIGHT {
            self.pos.x = -COPTER_W;
        } else if self.pos.x < WRAP_LEFT {
            self.pos.x = LOGICAL_W - 2;
        }

        if self.pos.y < TOP_LIMIT {
            self.pos.y = TOP_LIMIT;
        } else if self.pos.y + COPTER_H > COPTER_BOTTOM_LIMIT {
            self.pos.y = COPTER_BOTTOM_LIMIT - COPTER_H;
        }
    }

    /// Advance the rotor animation without moving (attract-screen idle).
    pub fn animate(&mut self) {
        self.frame = (self.frame + 1) % 3;
    }

    /// Current velocity (px/tick), for the HUD yoke indicator.
    #[must_use]
    pub fn velocity(&self) -> Vel {
        self.vel
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
