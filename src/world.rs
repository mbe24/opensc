//! The game world.
//!
//! What it is: the mutable simulation state and the single fixed-timestep
//! `tick` that advances it. It owns the entities and applies player intents.
//!
//! What it is not: rendering, input sampling, or timing — those are the
//! caller's job. `tick` is deterministic given the same intents.

use crate::copter::Copter;
use crate::input::Intents;
use crate::wagon::Wagon;

#[derive(Default)]
pub struct World {
    pub copter: Copter,
    pub wagon: Wagon,
}

impl World {
    /// Advance the simulation by exactly one fixed tick.
    pub fn tick(&mut self, intents: &Intents) {
        self.copter.tick(intents.req_dh, intents.req_dv);
        self.wagon.tick();
        // Drop / stuntman / collision land in the next milestone; `intents.drop`
        // is already latched and drained once per tick by the caller.
    }
}
