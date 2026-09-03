//! The game world.
//!
//! What it is: the mutable simulation state and the single fixed-timestep `tick`
//! that advances it — the copter, wagon, cloud, and stuntman, plus score, lives,
//! and level progression. It orchestrates the stuntman state machine and applies
//! outcomes.
//!
//! What it is not: rendering, input sampling, or timing — the caller's job.
//! `tick` is deterministic given the same intents.

use crate::cloud::Cloud;
use crate::config::{COPTER_H, GROUND_Y, MAN_H, MAN_HANG_OFFSET, MEN_PER_LEVEL, WAGON_H};
use crate::copter::Copter;
use crate::event::{Event, EventSink};
use crate::geom::Pos;
use crate::intents::Intents;
use crate::level::Progression;
use crate::rng::Rng;
use crate::score::{Height, Points, Score};
use crate::stuntman::{classify, Faller, Flip, Outcome, Splat, Stuntman, Wreck};
use crate::wagon::Wagon;

/// Ticks between cloud scroll steps — the original scrolls it on 1 of its 3 loop
/// phases, i.e. one pixel every three ticks.
const CLOUD_TICKS: u8 = 3;

/// How long the "LEVEL n" banner lingers after clearing a level (~2s at 30 Hz,
/// the original's `LevelTimer := TickCount + 120` at 60 Hz).
const LEVEL_BANNER_TICKS: u16 = 60;

/// A completed animation asks to retire the man; an unfinished one does nothing.
fn finished_or_nothing(finished: bool) -> Next {
    if finished {
        Next::FinishMan
    } else {
        Next::Nothing
    }
}

/// The wreck a failed outcome leaves, or `None` for a safe landing.
fn wreck_of(outcome: Outcome) -> Option<Wreck> {
    match outcome {
        Outcome::Landed => None,
        Outcome::Splat => Some(Wreck::Ground),
        Outcome::HitDriver => Some(Wreck::Driver),
        Outcome::HitHorse => Some(Wreck::Horse),
    }
}

pub struct World {
    pub copter: Copter,
    pub wagon: Wagon,
    pub cloud: Cloud,
    pub stuntman: Stuntman,
    pub progression: Progression,
    pub score: Score,
    pub hiscore: Score,
    pub men_left: i32,
    pub good_jumps: i32,
    /// Per-man outcome for this level (`None` until the man is resolved).
    pub results: [Option<Outcome>; MEN_PER_LEVEL as usize],
    /// Set when the last man is spent without clearing the level — the scene
    /// layer switches to the game-over screen.
    pub over: bool,
    /// Ticks left on the "LEVEL n" banner after clearing a level; 0 when hidden.
    pub level_banner: u16,
    phase: u8,
    /// Deterministic wind/cloud randomness; injected so runs are reproducible.
    rng: Rng,
    /// Debug/testing: when `Some(x)`, the wagon is frozen at `x` instead of
    /// rolling, so reproducible landing scenarios can be set up.
    wagon_pinned: Option<i32>,
    /// Debug/testing: when `Some(pos)`, the copter is held there (rotor still
    /// spinning) instead of flying, to line a drop up exactly.
    copter_pinned: Option<Pos>,
}

impl Default for World {
    fn default() -> Self {
        Self {
            copter: Copter::default(),
            wagon: Wagon::default(),
            cloud: Cloud::default(),
            stuntman: Stuntman::default(),
            progression: Progression::default(),
            score: Score::default(),
            hiscore: Score::default(),
            men_left: MEN_PER_LEVEL,
            good_jumps: 0,
            results: [None; MEN_PER_LEVEL as usize],
            over: false,
            level_banner: 0,
            phase: 0,
            rng: Rng::default(),
            wagon_pinned: None,
            copter_pinned: None,
        }
    }
}

/// A snapshot of the interpolatable positions, taken before each tick so the
/// renderer can smooth motion between the fixed 30 Hz ticks and the display's
/// refresh rate.
#[derive(Clone, Copy)]
pub struct RenderState {
    pub copter: Pos,
    pub wagon_x: i32,
    pub cloud: Pos,
    pub faller: Option<Pos>,
}

/// A pending stuntman transition, computed while the state is borrowed and
/// applied afterwards so self-mutation never overlaps the borrow.
#[derive(Clone, Copy)]
enum Next {
    Nothing,
    StartFall,
    /// Landed with this outcome, drop height, and impact x.
    Land(Outcome, Height, i32),
    /// The celebration or crash animation has run its course.
    FinishMan,
}

impl World {
    /// Advance the simulation by exactly one fixed tick, reporting domain events
    /// to `sink`. Pass [`crate::NoSink`] to ignore them at zero cost, or an
    /// [`crate::EventLog`] to collect them.
    pub fn tick(&mut self, intents: &Intents, sink: &mut dyn EventSink) {
        match self.copter_pinned {
            Some(pos) => {
                self.copter.pos = pos;
                self.copter.animate();
            }
            None => self.copter.tick(intents.req),
        }
        match self.wagon_pinned {
            Some(x) => self.wagon.x = x,
            None => self.wagon.tick(self.progression.wagon.px()),
        }
        self.phase = (self.phase + 1) % CLOUD_TICKS;
        if self.phase == 0 {
            self.cloud.tick(&mut self.rng);
        }
        self.level_banner = self.level_banner.saturating_sub(1);
        self.step_stuntman(intents.drop, sink);
    }

    /// Idle animation for the attract screen: the copter hovers (rotor spins),
    /// the wagon strolls, and clouds drift — no gameplay.
    pub fn attract_tick(&mut self) {
        self.copter.animate();
        self.wagon.tick(self.progression.wagon.px());
        self.phase = (self.phase + 1) % CLOUD_TICKS;
        if self.phase == 0 {
            self.cloud.tick(&mut self.rng);
        }
    }

    /// Start a fresh game, keeping the running high score and continuing the RNG
    /// stream so successive games don't replay identical wind.
    pub fn begin(&mut self) {
        let (hiscore, rng) = (self.hiscore, self.rng);
        *self = Self::default();
        self.hiscore = hiscore;
        self.rng = rng;
    }

    /// Reseed the wind/cloud RNG — the presentation layer calls this once at
    /// startup with a fresh seed so a session isn't deterministic, while tests
    /// keep the fixed default seed for reproducibility.
    pub fn reseed(&mut self, seed: u64) {
        self.rng = Rng::new(seed);
    }

    /// Debug/testing: freeze the wagon at `Some(x)`, or let it roll again with
    /// `None`. Handy for reproducible landing scenarios, in tests or in a manual
    /// session.
    pub fn pin_wagon(&mut self, x: Option<i32>) {
        self.wagon_pinned = x;
    }

    /// Whether the wagon is currently pinned (for the debug HUD).
    #[must_use]
    pub fn wagon_pinned(&self) -> bool {
        self.wagon_pinned.is_some()
    }

    /// Debug/testing: hold the copter at `Some(pos)`, or let it fly with `None`.
    /// Combined with [`World::pin_wagon`], this lines up a drop exactly.
    pub fn pin_copter(&mut self, pos: Option<Pos>) {
        self.copter_pinned = pos;
    }

    /// The pinned copter position, if any (so the debug UI can nudge it).
    #[must_use]
    pub fn copter_pinned(&self) -> Option<Pos> {
        self.copter_pinned
    }

    /// The hanging man's top-left, tracking the copter.
    #[must_use]
    pub fn hang_pos(&self) -> Pos {
        self.copter.pos + MAN_HANG_OFFSET
    }

    /// Current copter height above the ground, as shown in the HUD.
    #[must_use]
    pub fn height(&self) -> Height {
        Height::new((GROUND_Y - (self.copter.pos.y + COPTER_H)).max(0))
    }

    /// Zero-based index of the man currently in play (0..=4).
    #[must_use]
    pub fn current_man(&self) -> usize {
        (MEN_PER_LEVEL - self.men_left).clamp(0, MEN_PER_LEVEL - 1) as usize
    }

    /// Snapshot of interpolatable positions for smooth rendering.
    #[must_use]
    pub fn render_state(&self) -> RenderState {
        RenderState {
            copter: self.copter.pos,
            wagon_x: self.wagon.x,
            cloud: self.cloud.pos,
            faller: match &self.stuntman {
                Stuntman::Falling(f) => Some(f.pos),
                _ => None,
            },
        }
    }

    fn step_stuntman(&mut self, drop: bool, sink: &mut dyn EventSink) {
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
                if self.cloud.covers(faller.pos) {
                    faller.fall(1);
                    faller.gust(self.rng.range(-2, 3));
                } else {
                    faller.fall(self.progression.gravity.px());
                }
                if faller.pos.y + MAN_H > GROUND_Y - WAGON_H {
                    let outcome = classify(faller.pos.x, self.wagon.x);
                    Next::Land(outcome, faller.height_of_drop, faller.pos.x)
                } else {
                    Next::Nothing
                }
            }
            Stuntman::Celebrating(ref mut flip) => {
                flip.advance();
                finished_or_nothing(flip.finished())
            }
            Stuntman::Crashing(ref mut splat) => {
                splat.advance();
                finished_or_nothing(splat.finished())
            }
        };
        self.apply(next, sink);
    }

    fn apply(&mut self, next: Next, sink: &mut dyn EventSink) {
        match next {
            Next::Nothing => {}
            Next::StartFall => {
                let pos = self.hang_pos();
                let height = Height::new(GROUND_Y - (self.copter.pos.y + COPTER_H));
                self.stuntman = Stuntman::Falling(Faller::new(pos, height));
                sink.emit(Event::Dropped);
            }
            Next::Land(outcome, height, x) => {
                let points = self.score_outcome(outcome, height);
                sink.emit(Event::Resolved {
                    outcome,
                    points,
                    height,
                });
                // Announce the animation from the transition that starts it, so
                // each event is caused by the thing it names.
                self.stuntman = match wreck_of(outcome) {
                    None => {
                        sink.emit(Event::CelebrationStarted);
                        Stuntman::Celebrating(Flip::new())
                    }
                    Some(wreck) => {
                        sink.emit(Event::CrashStarted { wreck });
                        Stuntman::Crashing(Splat::new(wreck, x))
                    }
                };
            }
            Next::FinishMan => self.finish_man(sink),
        }
    }

    /// Apply an outcome's scoring/bookkeeping and return the points gained.
    fn score_outcome(&mut self, outcome: Outcome, height: Height) -> Points {
        // Record this man's outcome before `men_left` is touched below.
        self.results[self.current_man()] = Some(outcome);
        match outcome {
            Outcome::Landed => {
                let points = self.progression.level * height;
                self.score += points;
                self.good_jumps += 1;
                self.hiscore = self.hiscore.max(self.score);
                points
            }
            // A hit driver or horse ends the game (original sets MenLeft := 1).
            Outcome::HitDriver | Outcome::HitHorse => {
                self.men_left = 1;
                Points::default()
            }
            Outcome::Splat => Points::default(),
        }
    }

    /// Retire the man whose animation just finished: hang the next one, roll into
    /// the next level, or end the game. On game over the final tableau (the wreck
    /// or the man safe in the wagon) is left in place for the game-over screen.
    fn finish_man(&mut self, sink: &mut dyn EventSink) {
        self.men_left -= 1;
        sink.emit(Event::ManRetired {
            men_left: self.men_left,
        });
        if self.men_left > 0 {
            self.stuntman = Stuntman::Hanging;
        } else if self.good_jumps >= MEN_PER_LEVEL {
            self.advance_level(sink);
        } else {
            self.game_over(sink);
        }
    }

    fn advance_level(&mut self, sink: &mut dyn EventSink) {
        self.progression.level_up();
        self.men_left = MEN_PER_LEVEL;
        self.good_jumps = 0;
        self.results = [None; MEN_PER_LEVEL as usize];
        self.stuntman = Stuntman::Hanging;
        self.level_banner = LEVEL_BANNER_TICKS;
        sink.emit(Event::LevelCleared {
            level: self.progression.level,
        });
    }

    fn game_over(&mut self, sink: &mut dyn EventSink) {
        self.hiscore = self.hiscore.max(self.score);
        self.over = true;
        sink.emit(Event::GameEnded { score: self.score });
    }
}

#[cfg(test)]
// The tests build a default world, then poke a few fields to set up a scenario;
// a full struct literal would be far noisier than the targeted reassignments.
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::World;
    use crate::event::{Event, EventLog, NoSink};
    use crate::geom::{Pos, Vel};
    use crate::intents::Intents;
    use crate::stuntman::{Flip, Outcome, Splat, Stuntman, Wreck};

    const IDLE: Intents = Intents {
        req: Vel::new(0, 0),
        drop: false,
    };

    const DROP: Intents = Intents {
        req: Vel::new(0, 0),
        drop: true,
    };

    /// Tick until `stuntman` leaves its current animated variant (or we give up).
    fn run_until_transition(world: &mut World) {
        for _ in 0..500 {
            match world.stuntman {
                Stuntman::Celebrating(_) | Stuntman::Crashing(_) if !world.over => {
                    world.tick(&IDLE, &mut NoSink);
                }
                _ => break,
            }
        }
    }

    #[test]
    fn a_finished_celebration_hangs_the_next_man() {
        let mut world = World::default();
        world.stuntman = Stuntman::Celebrating(Flip::new());
        run_until_transition(&mut world);
        assert!(matches!(world.stuntman, Stuntman::Hanging));
        assert_eq!(world.men_left, super::MEN_PER_LEVEL - 1);
        assert!(!world.over);
    }

    #[test]
    fn clearing_a_level_ramps_difficulty_and_shows_the_banner() {
        let mut world = World::default();
        world.good_jumps = super::MEN_PER_LEVEL; // all five landed
        world.men_left = 1; // on the last man
        world.stuntman = Stuntman::Celebrating(Flip::new());
        run_until_transition(&mut world);
        assert_eq!(world.progression.level.get(), 2);
        assert!(world.level_banner > 0);
        assert!(!world.over);
        assert!(matches!(world.stuntman, Stuntman::Hanging));
    }

    #[test]
    fn a_hay_landing_triggers_the_celebration() {
        let mut world = World::default();
        let wagon_x = 200;
        world.pin_wagon(Some(wagon_x));
        // The man's x is copter.x + MAN_HANG_OFFSET.dh; aim it 14px into the hay
        // (well inside the `< 34` success band), and drop from high up.
        let copter_x = wagon_x + 14 - super::MAN_HANG_OFFSET.dh;
        world.pin_copter(Some(Pos::new(copter_x, 40)));

        world.tick(&DROP, &mut NoSink);
        assert!(matches!(world.stuntman, Stuntman::Falling(_)));

        for _ in 0..500 {
            world.tick(&IDLE, &mut NoSink);
            if !matches!(world.stuntman, Stuntman::Falling(_)) {
                break;
            }
        }
        assert!(
            matches!(world.stuntman, Stuntman::Celebrating(_)),
            "a drop into the hay celebrates"
        );
        assert_eq!(world.good_jumps, 1);
        assert!(world.score.get() > 0);
    }

    #[test]
    fn the_event_log_records_a_drop_and_a_landing() {
        let mut world = World::default();
        let wagon_x = 200;
        world.pin_wagon(Some(wagon_x));
        world.pin_copter(Some(Pos::new(wagon_x + 14 - super::MAN_HANG_OFFSET.dh, 40)));

        // An EventLog collects what the tick reports — no timing or rendering.
        let mut log = EventLog::default();
        world.tick(&DROP, &mut log);
        assert_eq!(log.events(), [Event::Dropped]);

        // Drive to the landing and confirm the celebration announces itself: the
        // resolution (cause) and the backflip starting (effect), in that order.
        let mut resolved = false;
        let mut celebrated = false;
        for _ in 0..500 {
            log.clear();
            world.tick(&IDLE, &mut log);
            for &event in log.events() {
                match event {
                    Event::Resolved {
                        outcome: Outcome::Landed,
                        points,
                        ..
                    } if points.get() > 0 => {
                        resolved = true;
                    }
                    Event::CelebrationStarted => {
                        assert!(resolved, "the landing resolves before it celebrates");
                        celebrated = true;
                    }
                    _ => {}
                }
            }
            if celebrated {
                break;
            }
        }
        assert!(celebrated, "a hay landing starts the backflip celebration");
    }

    #[test]
    fn striking_the_driver_ends_the_game_and_leaves_the_wreck() {
        let mut world = World::default();
        world.men_left = 1; // the driver hit set MenLeft := 1
        world.stuntman = Stuntman::Crashing(Splat::new(Wreck::Driver, 100));
        run_until_transition(&mut world);
        assert!(world.over);
        // The wreck is left in place for the game-over screen, not cleared.
        assert!(matches!(world.stuntman, Stuntman::Crashing(_)));
    }
}
