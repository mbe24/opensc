//! # StuntCopter simulation
//!
//! The platform-agnostic game model: state, rules, and the fixed-timestep
//! [`World::tick`]. It has **no dependency on macroquad** (or any windowing,
//! rendering, or timing library), so it can be driven headless in tests and the
//! presentation layer can be swapped without touching game logic.
//!
//! The presentation layer (the `stuntcopter` binary) owns the clock, samples
//! input into [`Intents`], calls [`World::tick`] at a fixed rate, and renders
//! [`World::render_state`] plus the public world fields. Which sprite to draw is
//! named by [`Sprite`]; where it sits in the texture is [`Sprite::rect`], a pure
//! [`SrcRect`] the renderer maps onto its own coordinates.

pub mod cloud;
pub mod config;
pub mod copter;
pub mod event;
pub mod intents;
pub mod level;
pub mod rng;
pub mod sprite;
pub mod stuntman;
pub mod wagon;
pub mod world;

pub use event::{Event, EventLog, EventSink, NoSink};
pub use intents::Intents;
pub use sprite::{Sprite, SrcRect};
pub use world::{RenderState, World};
