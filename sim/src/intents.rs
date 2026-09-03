//! Player intents — the simulation's device-agnostic input.
//!
//! What it is: what the player is asking for on a given tick, decoupled from how
//! it was expressed (mouse, keyboard, touch). The presentation layer samples the
//! real devices and fills this in; the simulation only ever sees intents.

use crate::geom::Vel;

/// What the player is asking for this tick.
#[derive(Clone, Copy, Default)]
pub struct Intents {
    /// Requested copter velocity, px/tick, within `config::DELTA_RECT`. The copter
    /// accelerates toward this; it is not applied directly.
    pub req: Vel,
    /// Edge-triggered: the player pressed drop this tick.
    pub drop: bool,
}
