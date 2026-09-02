//! The game world.
//!
//! What it is: the mutable simulation state and the single fixed-timestep `tick`
//! that advances it — the copter, wagon, cloud, and stuntman, plus score, lives,
//! and level progression. It orchestrates the stuntman state machine and applies
//! outcomes.
//!
//! What it is not: rendering, input sampling, or timing — the caller's job.
//! `tick` is deterministic given the same intents.

use macroquad::rand::gen_range;

use crate::cloud::Cloud;
use crate::config::{COPTER_H, GROUND_Y, MAN_H, MAN_HANG_OFFSET, MEN_PER_LEVEL, WAGON_H};
use crate::copter::Copter;
use crate::input::Intents;
use crate::level::Progression;
use crate::stuntman::{classify, Faller, Held, Outcome, Stuntman, HOLD_TICKS};
use crate::wagon::Wagon;

/// Ticks between cloud scroll steps — the original scrolls it on 1 of its 3 loop
/// phases, i.e. one pixel every three ticks.
const CLOUD_TICKS: u8 = 3;

pub struct World {
    pub copter: Copter,
    pub wagon: Wagon,
    pub cloud: Cloud,
    pub stuntman: Stuntman,
    pub progression: Progression,
    pub score: i32,
    pub hiscore: i32,
    pub men_left: i32,
    pub good_jumps: i32,
    /// Per-man result for this level: `Some(true)` landed, `Some(false)` failed.
    pub results: [Option<bool>; MEN_PER_LEVEL as usize],
    phase: u8,
}

impl Default for World {
    fn default() -> Self {
        Self {
            copter: Copter::default(),
            wagon: Wagon::default(),
            cloud: Cloud::default(),
            stuntman: Stuntman::default(),
            progression: Progression::default(),
            score: 0,
            hiscore: 0,
            men_left: MEN_PER_LEVEL,
            good_jumps: 0,
            results: [None; MEN_PER_LEVEL as usize],
            phase: 0,
        }
    }
}

/// A pending stuntman transition, computed while the state is borrowed and
/// applied afterwards so self-mutation never overlaps the borrow.
#[derive(Clone, Copy)]
enum Next {
    Nothing,
    StartFall,
    Land(Outcome, i32),
    ResetMan,
}

impl World {
    /// Advance the simulation by exactly one fixed tick.
    pub fn tick(&mut self, intents: &Intents) {
        self.copter.tick(intents.req_dh, intents.req_dv);
        self.wagon.tick(self.progression.wagon.px());
        self.phase = (self.phase + 1) % CLOUD_TICKS;
        if self.phase == 0 {
            self.cloud.tick();
        }
        self.step_stuntman(intents.drop);
    }

    /// The hanging man's top-left, tracking the copter.
    #[must_use]
    pub fn hang_pos(&self) -> (i32, i32) {
        (
            self.copter.x + MAN_HANG_OFFSET.0,
            self.copter.y + MAN_HANG_OFFSET.1,
        )
    }

    /// Current copter height above the ground, as shown in the HUD.
    #[must_use]
    pub fn height(&self) -> i32 {
        (GROUND_Y - (self.copter.y + COPTER_H)).max(0)
    }

    /// Zero-based index of the man currently in play (0..=4).
    #[must_use]
    pub fn current_man(&self) -> usize {
        (MEN_PER_LEVEL - self.men_left).clamp(0, MEN_PER_LEVEL - 1) as usize
    }

    fn step_stuntman(&mut self, drop: bool) {
        let next = match self.stuntman {
            Stuntman::Hanging => {
                if drop {
                    Next::StartFall
                } else {
                    Next::Nothing
                }
            }
            Stuntman::Falling(ref mut faller) => {
                // Behind a cloud: fall a fixed 1px with a random wind gust;
                // otherwise fall at gravity.
                if self.cloud.covers(faller.x, faller.y) {
                    faller.fall(1);
                    faller.gust(gen_range(-2, 3));
                } else {
                    faller.fall(self.progression.gravity.px());
                }
                if faller.y + MAN_H > GROUND_Y - WAGON_H {
                    Next::Land(classify(faller.x, self.wagon.x), faller.height_of_drop)
                } else {
                    Next::Nothing
                }
            }
            Stuntman::Held(ref mut held) => {
                if held.timer == 0 {
                    Next::ResetMan
                } else {
                    held.timer -= 1;
                    Next::Nothing
                }
            }
        };
        self.apply(next);
    }

    fn apply(&mut self, next: Next) {
        match next {
            Next::Nothing => {}
            Next::StartFall => {
                let (x, y) = self.hang_pos();
                let height = GROUND_Y - (self.copter.y + COPTER_H);
                self.stuntman = Stuntman::Falling(Faller::new(x, y, height));
            }
            Next::Land(outcome, height) => {
                let x = self.hang_pos().0; // horizontal doesn't change during a fall
                self.score_outcome(outcome, height);
                self.stuntman = Stuntman::Held(Held {
                    outcome,
                    x,
                    timer: HOLD_TICKS,
                });
            }
            Next::ResetMan => self.reset_man(),
        }
    }

    fn score_outcome(&mut self, outcome: Outcome, height: i32) {
        // Record this man's result before `men_left` is touched below.
        self.results[self.current_man()] = Some(matches!(outcome, Outcome::Landed));
        match outcome {
            Outcome::Landed => {
                self.score += self.progression.level * height;
                self.good_jumps += 1;
                self.hiscore = self.hiscore.max(self.score);
            }
            // A hit driver or horse ends the game (original sets MenLeft := 1).
            Outcome::HitDriver | Outcome::HitHorse => self.men_left = 1,
            Outcome::Splat => {}
        }
    }

    fn reset_man(&mut self) {
        self.men_left -= 1;
        if self.men_left > 0 {
            self.stuntman = Stuntman::Hanging;
        } else if self.good_jumps >= MEN_PER_LEVEL {
            self.advance_level();
        } else {
            self.game_over();
        }
    }

    fn advance_level(&mut self) {
        self.progression.level_up();
        self.men_left = MEN_PER_LEVEL;
        self.good_jumps = 0;
        self.results = [None; MEN_PER_LEVEL as usize];
        self.stuntman = Stuntman::Hanging;
    }

    fn game_over(&mut self) {
        let hiscore = self.hiscore.max(self.score);
        *self = Self::default();
        self.hiscore = hiscore;
    }
}
