//! A drifting cloud.
//!
//! What it is: a single cloud scrolling right-to-left, respawning off the right
//! edge at a random height with the next of three cloud sprites. A stuntman
//! falling through it catches a gust (random horizontal jitter). Ported from
//! `StartNewCloud` and the cloud case of `AnimateOneLoop`.

use crate::config::LOGICAL_W;
use crate::geom::Pos;
use crate::rng::Rng;
use crate::sprite::Sprite;

pub struct Cloud {
    pub pos: Pos,
    frame: u8,
}

impl Default for Cloud {
    fn default() -> Self {
        Self {
            pos: Pos::new(LOGICAL_W, 30),
            frame: 0,
        }
    }
}

impl Cloud {
    /// Drift one pixel left; respawn (at a fresh random height) once fully off
    /// the left edge.
    pub fn tick(&mut self, rng: &mut Rng) {
        self.pos.x -= 1;
        if self.pos.x + self.width() < 0 {
            self.respawn(rng);
        }
    }

    fn respawn(&mut self, rng: &mut Rng) {
        self.frame = (self.frame + 1) % 3;
        self.pos.x = LOGICAL_W;
        self.pos.y = rng.range(8, 120);
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

    /// Whether playfield point `p` lies over the cloud's bounding box.
    #[must_use]
    pub fn covers(&self, p: Pos) -> bool {
        let r = self.sprite().rect();
        p.x >= self.pos.x
            && p.x < self.pos.x + r.w as i32
            && p.y >= self.pos.y
            && p.y < self.pos.y + r.h as i32
    }
}
