//! Scoring values.
//!
//! What it is: distinct newtypes for the quantities the scoring rule combines, so
//! the one formula that matters — points = level × height-of-drop, accumulated
//! into the score — can't be written with the operands transposed. The algebra
//! is deliberately narrow: only [`Level`] `*` [`Height`] `->` [`Points`] and
//! [`Score`] `+=` [`Points`] are defined, because those are the only meaningful
//! combinations.
//!
//! Each carries [`Display`](std::fmt::Display) so the HUD and event log format it
//! without ceremony, and each is a plain `i32` inside, so the safety is
//! zero-cost.

use std::fmt;

/// Define an `i32`-backed newtype with `new`/`get`/`Display`.
macro_rules! scalar {
    ($(#[$m:meta])* $name:ident) => {
        $(#[$m])*
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
        pub struct $name(i32);

        impl $name {
            #[must_use]
            pub const fn new(v: i32) -> Self {
                Self(v)
            }
            /// The raw value, for arithmetic the domain algebra doesn't cover
            /// (digit splitting, array indexing) and for rendering.
            #[must_use]
            pub const fn get(self) -> i32 {
                self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}

scalar!(
    /// A running or final score, points accumulated over a game.
    Score
);
scalar!(
    /// Points gained from a single drop (0 unless it landed).
    Points
);
scalar!(
    /// The current level, `1..`. Higher levels multiply the points per landing.
    Level
);
scalar!(
    /// A drop height in pixels — how far the man fell, which scores the landing.
    Height
);

impl Level {
    /// The starting level.
    pub const ONE: Self = Self(1);

    /// The next level up.
    #[must_use]
    pub const fn next(self) -> Self {
        Self(self.0 + 1)
    }
}

/// `Level * Height` is the points a landing from that height scores.
impl std::ops::Mul<Height> for Level {
    type Output = Points;
    fn mul(self, h: Height) -> Points {
        Points(self.0 * h.0)
    }
}

/// Accumulate a drop's points into the score.
impl std::ops::AddAssign<Points> for Score {
    fn add_assign(&mut self, p: Points) {
        self.0 += p.0;
    }
}
