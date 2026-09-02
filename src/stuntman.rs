//! The stuntman.
//!
//! What it is: a small state machine — hanging from the copter, in free fall,
//! or holding on the last outcome before the next man. Ported from the
//! `ManStatus` cases of `AnimateOneLoop`. Collision classification lives here as
//! a pure function.
//!
//! What it is not: the score/lives bookkeeping — [`crate::world::World`] drives
//! transitions and owns that state.

use crate::atlas::Sprite;

/// How a drop ended, by the man's horizontal offset from the wagon.
#[derive(Clone, Copy)]
pub enum Outcome {
    Landed,
    Splat,
    HitDriver,
    HitHorse,
}

/// Ticks the outcome sprite is held before the next man (placeholder until the
/// full flip/splat animations land).
pub const HOLD_TICKS: u16 = 24;

/// A man in free fall.
pub struct Faller {
    pub x: i32,
    pub y: i32,
    /// Captured at drop time; scores `level * height_of_drop` on a landing.
    pub height_of_drop: i32,
    frame: u8,
}

impl Faller {
    #[must_use]
    pub fn new(x: i32, y: i32, height_of_drop: i32) -> Self {
        Self {
            x,
            y,
            height_of_drop,
            frame: 0,
        }
    }

    /// Fall `dy` pixels and advance the tumble animation. `dy` is gravity in open
    /// air, but a fixed 1px while falling through a cloud.
    pub fn fall(&mut self, dy: i32) {
        self.y += dy;
        self.frame = (self.frame + 1) % 5;
    }

    /// Nudge horizontally (a wind gust behind a cloud).
    pub fn gust(&mut self, dx: i32) {
        self.x += dx;
    }

    #[must_use]
    pub fn sprite(&self) -> Sprite {
        match self.frame {
            0 => Sprite::ManDrop1,
            1 => Sprite::ManDrop2,
            2 => Sprite::ManDrop3,
            3 => Sprite::ManDrop4,
            _ => Sprite::ManDrop5,
        }
    }
}

/// The outcome sprite shown briefly before the next man.
pub struct Held {
    pub outcome: Outcome,
    pub x: i32,
    pub timer: u16,
}

#[derive(Default)]
pub enum Stuntman {
    #[default]
    Hanging,
    Falling(Faller),
    Held(Held),
}

/// Classify a landing by the man's horizontal offset from the wagon's left edge,
/// exactly as the original (`Where = ManRect.left - WagonRect.left`).
#[must_use]
pub fn classify(man_x: i32, wagon_x: i32) -> Outcome {
    match man_x - wagon_x {
        w if !(-6..=70).contains(&w) => Outcome::Splat,
        w if w < 34 => Outcome::Landed,
        w if w < 45 => Outcome::HitDriver,
        _ => Outcome::HitHorse,
    }
}

#[cfg(test)]
mod tests {
    use super::{classify, Outcome};

    fn kind(offset: i32) -> Outcome {
        classify(offset, 0)
    }

    #[test]
    fn collision_boundaries_match_the_original() {
        assert!(matches!(kind(-7), Outcome::Splat)); // just left of the wagon
        assert!(matches!(kind(-6), Outcome::Landed)); // first hay column
        assert!(matches!(kind(33), Outcome::Landed)); // last hay column
        assert!(matches!(kind(34), Outcome::HitDriver));
        assert!(matches!(kind(44), Outcome::HitDriver));
        assert!(matches!(kind(45), Outcome::HitHorse));
        assert!(matches!(kind(70), Outcome::HitHorse)); // back of the horse
        assert!(matches!(kind(71), Outcome::Splat)); // just past it
    }
}
