//! The stuntman.
//!
//! What it is: the man's state machine over one drop — hanging from the copter,
//! in free fall, celebrating a safe landing, or crashing. Ported from the
//! `ManStatus` cases of `AnimateOneLoop`. Collision classification lives here as
//! a pure function, and each animated state owns its own frame clock.
//!
//! What it is not: the score/lives bookkeeping — [`crate::world::World`] drives
//! transitions and owns that state.

use crate::sprite::Sprite;

/// How a drop ended, by the man's horizontal offset from the wagon.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Landed,
    Splat,
    HitDriver,
    HitHorse,
}

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

/// The success celebration: the 15-step backflip (`OffFlip[1..15]`) that plays
/// in the two HUD flip-boxes, one pose every three ticks, while the rescued man
/// rides in the wagon.
pub struct Flip {
    tick: u16,
}

impl Flip {
    /// Distinct backflip poses; the 15th step loops back to the first.
    const FRAMES: u16 = 15;
    /// Ticks each pose is held.
    const PERIOD: u16 = 3;
    const POSES: [Sprite; 14] = [
        Sprite::Flip01,
        Sprite::Flip02,
        Sprite::Flip03,
        Sprite::Flip04,
        Sprite::Flip05,
        Sprite::Flip06,
        Sprite::Flip07,
        Sprite::Flip08,
        Sprite::Flip09,
        Sprite::Flip10,
        Sprite::Flip11,
        Sprite::Flip12,
        Sprite::Flip13,
        Sprite::Flip14,
    ];

    #[must_use]
    pub fn new() -> Self {
        Self { tick: 0 }
    }

    pub fn advance(&mut self) {
        self.tick += 1;
    }

    #[must_use]
    pub fn finished(&self) -> bool {
        self.tick >= Self::FRAMES * Self::PERIOD
    }

    /// The current backflip pose, or `None` once the celebration is spent (only
    /// the man-in-wagon remains — e.g. left showing on the game-over screen).
    #[must_use]
    pub fn pose(&self) -> Option<Sprite> {
        if self.finished() {
            return None;
        }
        // The 15th step (index 14) loops back to the first pose.
        let i = (self.tick / Self::PERIOD) as usize;
        Some(Self::POSES.get(i).copied().unwrap_or(Sprite::Flip01))
    }
}

impl Default for Flip {
    fn default() -> Self {
        Self::new()
    }
}

/// What a failed drop struck, deciding the wreck it leaves behind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wreck {
    Ground,
    Driver,
    Horse,
}

/// A failed drop: the six-frame crumple (`OffMan[8..13]`) at the point of impact,
/// after which the wreck tableau (a dead driver or horse) is left in place.
pub struct Splat {
    pub wreck: Wreck,
    pub x: i32,
    tick: u16,
}

impl Splat {
    const FRAMES: u16 = 6;
    const PERIOD: u16 = 2;
    const POSES: [Sprite; 6] = [
        Sprite::ManSplat1,
        Sprite::ManSplat2,
        Sprite::ManSplat3,
        Sprite::ManSplat4,
        Sprite::ManSplat5,
        Sprite::ManSplat6,
    ];

    #[must_use]
    pub fn new(wreck: Wreck, x: i32) -> Self {
        Self { wreck, x, tick: 0 }
    }

    pub fn advance(&mut self) {
        self.tick += 1;
    }

    #[must_use]
    pub fn finished(&self) -> bool {
        self.tick >= Self::FRAMES * Self::PERIOD
    }

    /// The current crumple frame, or `None` once the man has come apart and only
    /// the wreck remains.
    #[must_use]
    pub fn pose(&self) -> Option<Sprite> {
        let i = (self.tick / Self::PERIOD) as usize;
        Self::POSES.get(i).copied()
    }
}

#[derive(Default)]
pub enum Stuntman {
    #[default]
    Hanging,
    Falling(Faller),
    Celebrating(Flip),
    Crashing(Splat),
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
    use super::{classify, Flip, Outcome, Splat, Wreck};

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

    #[test]
    fn flip_shows_fifteen_poses_then_finishes() {
        let mut flip = Flip::new();
        let mut poses = 0;
        let mut last = None;
        while !flip.finished() {
            let pose = flip.pose();
            assert!(pose.is_some(), "a pose shows on every tick of the flip");
            if pose != last {
                poses += 1;
                last = pose;
            }
            flip.advance();
        }
        assert_eq!(poses, Flip::FRAMES, "one distinct pose per backflip step");
    }

    #[test]
    fn splat_crumbles_then_leaves_the_wreck() {
        let mut splat = Splat::new(Wreck::Driver, 0);
        // Every crumple frame shows a pose; once spent, only the wreck remains.
        while !splat.finished() {
            assert!(splat.pose().is_some());
            splat.advance();
        }
        assert!(splat.pose().is_none(), "the crumple is spent when finished");
    }
}
