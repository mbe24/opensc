//! Domain events emitted by the simulation, and the sink they flow into.
//!
//! What it is: [`World::tick`] reports the meaningful things that happen — a man
//! dropped, landed, splatted, a level cleared, the game ended — to an
//! [`EventSink`] the caller passes in. The presentation layer reacts to them
//! (sound, one-shot effects) and tests assert on them, so behaviour can be
//! checked without timing or rendering.
//!
//! Why a sink rather than a stored buffer: the caller decides whether events are
//! collected at all. A normal build passes [`NoSink`] (zero-sized; the `emit`
//! calls compile to nothing), so there is no per-tick bookkeeping cost. The
//! debug UI and tests pass an [`EventLog`].
//!
//! What it is not: a general logging or tracing facility. Events carry only game
//! facts and hold no platform types — richer observability (console, `tracing`)
//! belongs in the UI layer, bridged from these events.
//!
//! [`World::tick`]: crate::world::World::tick

use crate::score::{Height, Level, Points, Score};
use crate::stuntman::{Outcome, Wreck};

/// A destination for [`Event`]s produced during a tick.
pub trait EventSink {
    fn emit(&mut self, event: Event);
}

/// A sink that discards every event. Zero-sized, so with it the simulation does
/// no event work at all — the default for a normal game build.
pub struct NoSink;

impl EventSink for NoSink {
    #[inline]
    fn emit(&mut self, _event: Event) {}
}

/// A sink that collects events in order, for the debug UI and tests.
#[derive(Default)]
pub struct EventLog {
    events: Vec<Event>,
}

impl EventLog {
    /// The collected events, oldest first.
    #[must_use]
    pub fn events(&self) -> &[Event] {
        &self.events
    }

    /// Discard the collected events.
    pub fn clear(&mut self) {
        self.events.clear();
    }
}

impl EventSink for EventLog {
    fn emit(&mut self, event: Event) {
        self.events.push(event);
    }
}

/// Something the simulation did on a tick, in the order it happened.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    /// The stuntman let go of the copter and began to fall.
    Dropped,
    /// A drop resolved: it was classified and scored. `points` is the score
    /// gained (0 unless landed) and `height` is the drop height. This is the
    /// *cause* — the celebration/crash it triggers announces itself separately.
    Resolved {
        outcome: Outcome,
        points: Points,
        height: Height,
    },
    /// The success backflip animation began (emitted as the state is entered, so
    /// it proves the celebration actually started, not merely that a man landed).
    CelebrationStarted,
    /// The crash animation began, leaving `wreck` behind when it finishes.
    CrashStarted { wreck: Wreck },
    /// A man's turn fully ended (celebration or crash animation finished), with
    /// `men_left` still to play this level.
    ManRetired { men_left: i32 },
    /// All five men landed; the game advanced to `level`.
    LevelCleared { level: Level },
    /// The game ended, with this final `score`.
    GameEnded { score: Score },
}
