//! Level progression.
//!
//! What it is: the current level and its derived difficulty — wagon speed and
//! gravity — as bounded newtypes so the exact ramp can only advance the one
//! faithful way. Ported from `ResetManHanging`'s level-up block.
//!
//! What it is not: the men/score bookkeeping (that lives in [`crate::world`]).

use crate::config::{GRAVITY_WORDS, WAGON_WORDS};

/// Wagon speed, px/tick, `1..=3` (WALK / TROT / GALLOP).
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct WagonSpeed(i32);

/// Gravity, px/tick of fall, `1..=4` (OH BOY / FLYING / NORMAL / HEAVY). Higher
/// = faster fall = easier timing, so it *decreases* at later levels.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Gravity(i32);

impl WagonSpeed {
    const MAX: i32 = 3;
    #[must_use]
    pub fn px(self) -> i32 {
        self.0
    }
    #[must_use]
    pub fn word(self) -> &'static str {
        WAGON_WORDS[(self.0 - 1) as usize]
    }
}

impl Gravity {
    #[must_use]
    pub fn px(self) -> i32 {
        self.0
    }
    #[must_use]
    pub fn word(self) -> &'static str {
        GRAVITY_WORDS[(self.0 - 1) as usize]
    }
}

pub struct Progression {
    pub level: i32,
    pub wagon: WagonSpeed,
    pub gravity: Gravity,
}

impl Default for Progression {
    fn default() -> Self {
        Self {
            level: 1,
            wagon: WagonSpeed(1),
            gravity: Gravity(4),
        }
    }
}

impl Progression {
    /// Advance one level. The order is exact and load-bearing: gravity only eases
    /// *after* wagon speed has already reached its max on a previous level, since
    /// the guard reads the pre-increment wagon speed (`ResetManHanging`).
    pub fn level_up(&mut self) {
        self.level += 1;
        if self.wagon.0 == WagonSpeed::MAX && self.gravity.0 > 1 {
            self.gravity.0 -= 1;
        }
        if self.wagon.0 < WagonSpeed::MAX {
            self.wagon.0 += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Progression;

    #[test]
    fn ramp_matches_the_original() {
        // (level, wagon_px, gravity_px) after each level-up, from the source table.
        let expected = [
            (2, 2, 4),
            (3, 3, 4),
            (4, 3, 3),
            (5, 3, 2),
            (6, 3, 1),
            (7, 3, 1),
        ];
        let mut p = Progression::default();
        assert_eq!((p.level, p.wagon.px(), p.gravity.px()), (1, 1, 4));
        for &(level, wagon, gravity) in &expected {
            p.level_up();
            assert_eq!(
                (p.level, p.wagon.px(), p.gravity.px()),
                (level, wagon, gravity)
            );
        }
    }
}
