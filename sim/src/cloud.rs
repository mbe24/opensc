//! A drifting cloud.
//!
//! What it is: a single cloud scrolling right-to-left, respawning off the right
//! edge at a random height with the next of three cloud sprites. A stuntman
//! falling through it catches a gust (random horizontal jitter). Ported from
//! `StartNewCloud` and the cloud case of `AnimateOneLoop`.

use crate::config::LOGICAL_W;
use crate::rng::Rng;
use crate::sprite::Sprite;

pub struct Cloud {
    pub x: i32,
    pub y: i32,
    frame: u8,
}

impl Default for Cloud {
    fn default() -> Self {
        Self {
            x: LOGICAL_W,
            y: 30,
            frame: 0,
        }
    }
}

impl Cloud {
    /// Drift one pixel left; respawn (at a fresh random height) once fully off
    /// the left edge.
    pub fn tick(&mut self, rng: &mut Rng) {
        self.x -= 1;
        if self.x + self.width() < 0 {
            self.respawn(rng);
        }
    }

    fn respawn(&mut self, rng: &mut Rng) {
        self.frame = (self.frame + 1) % 3;
        self.x = LOGICAL_W;
        self.y = rng.range(8, 120);
    }

    #[must_use]
    pub fn sprite(&self) -> Sprite {
        match self.frame {
            0 => Sprite::CloudLeft,
            1 => Sprite::CloudBottom,
            _ => Sprite::CloudRight,
        }
    }

    fn width(&self) -> i32 {
        self.sprite().rect().w as i32
    }

    /// Whether logical point `(px, py)` lies over the cloud's bounding box.
    #[must_use]
    pub fn covers(&self, px: i32, py: i32) -> bool {
        let r = self.sprite().rect();
        px >= self.x && px < self.x + r.w as i32 && py >= self.y && py < self.y + r.h as i32
    }
}
