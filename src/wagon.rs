//! The horse-drawn wagon the stuntman aims for.
//!
//! What it is: a wagon that rolls right along the ground at the level's speed,
//! wrapping around off the right edge, cycling three animation frames. Ported
//! from the wagon block of `AnimateOneLoop`.

use crate::atlas::Sprite;
use crate::config::{LOGICAL_W, WAGON_W};

/// Distance to jump back when wrapping off the right edge (`OffSetRect(..,-582,..)`).
const WRAP_STRIDE: i32 = LOGICAL_W + WAGON_W - 3;

/// Ticks each trot frame is held. Cycling all three every tick (at 30 Hz) reads
/// as a flicker, so we slow the animation to a calmer trot.
const ANIM_TICKS: u8 = 3;

#[derive(Default)]
pub struct Wagon {
    /// Left edge, logical pixels.
    pub x: i32,
    /// Animation phase, 0..(`ANIM_TICKS` * 3).
    phase: u8,
}

impl Wagon {
    /// Roll one tick at `speed` px, wrapping off the right edge.
    pub fn tick(&mut self, speed: i32) {
        self.phase = (self.phase + 1) % (ANIM_TICKS * 3);
        if self.x > LOGICAL_W {
            self.x -= WRAP_STRIDE;
        } else {
            self.x += speed;
        }
    }

    /// Current wagon sprite.
    #[must_use]
    pub fn sprite(&self) -> Sprite {
        match self.phase / ANIM_TICKS {
            0 => Sprite::Wagon1,
            1 => Sprite::Wagon2,
            _ => Sprite::Wagon3,
        }
    }
}
