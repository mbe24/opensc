//! Debug / test controls, compiled only under the `debug-controls` feature so
//! they can never reach a normal player build (nor bloat it).
//!
//! Keys:
//! - `M` toggles mouse steering (keyboard-only when off);
//! - `K` pins/unpins the wagon at its current x;
//! - `P` pins/unpins the copter at its current position — while pinned, the
//!   arrow keys nudge it a few pixels at a time;
//! - `L` lines up a guaranteed landing: wagon centered and copter set directly
//!   above the hay, so a single drop celebrates (hold Shift for a low drop).
//!
//! Two readouts sit at the top: the **left** lists the active debug/UI modes
//! (a statement about this harness), the **right** lists the most recent
//! **simulation** events (a statement about the game model), so you can see what
//! the sim did without catching an animation frame.

use std::collections::VecDeque;

use macroquad::prelude::*;
use stuntcopter_sim::{Event, Pos, World};

use crate::assets::Assets;
use crate::config;
use crate::{draw, font, theme};

/// Nudge step for the pinned copter, in logical pixels per key press.
const NUDGE: i32 = 4;
/// How many recent events the on-screen log keeps.
const LOG_CAP: usize = 8;
/// Where the man lands inside the hay for the "line up a landing" preset — 14px
/// into the wagon, well inside the `< 34` success band.
const HAY_INSET: i32 = 14;
/// Height the copter is placed at for the preset, high enough for a full drop.
const PRESET_COPTER_Y: i32 = 40;
/// A lower placement (Shift+L) for a near-instant drop, handy when watching the
/// celebration rather than the fall.
const PRESET_COPTER_Y_LOW: i32 = 210;

/// Debug-session state that isn't part of the game model.
pub struct Debug {
    mouse_steer: bool,
    /// The most recent simulation events, newest last.
    log: VecDeque<Event>,
}

impl Default for Debug {
    fn default() -> Self {
        Self {
            mouse_steer: true,
            log: VecDeque::new(),
        }
    }
}

impl Debug {
    /// Whether the mouse currently steers the copter (off = keyboard only).
    pub fn mouse_steer(&self) -> bool {
        self.mouse_steer
    }

    /// Apply this frame's debug key presses to `world` and to local state.
    pub fn update(&mut self, world: &mut World) {
        if is_key_pressed(KeyCode::M) {
            self.mouse_steer = !self.mouse_steer;
        }
        if is_key_pressed(KeyCode::K) {
            let pin = (!world.wagon_pinned()).then_some(world.wagon.x);
            world.pin_wagon(pin);
        }
        if is_key_pressed(KeyCode::P) {
            let pin = world.copter_pinned().is_none().then_some(world.copter.pos);
            world.pin_copter(pin);
        }
        if is_key_pressed(KeyCode::L) {
            // Line up a guaranteed hay landing: wagon centered, copter directly
            // above the hay. Positions are set exactly, not nudged. Hold Shift
            // for a low placement (near-instant drop) to watch the celebration.
            let low = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
            let y = if low {
                PRESET_COPTER_Y_LOW
            } else {
                PRESET_COPTER_Y
            };
            let wagon_x = (config::LOGICAL_W - config::WAGON_W) / 2;
            world.pin_wagon(Some(wagon_x));
            world.pin_copter(Some(Pos::new(
                wagon_x + HAY_INSET - config::MAN_HANG_OFFSET.dh,
                y,
            )));
        }
        if let Some(mut pos) = world.copter_pinned() {
            pos.x += NUDGE
                * (i32::from(is_key_pressed(KeyCode::Right))
                    - i32::from(is_key_pressed(KeyCode::Left)));
            pos.y += NUDGE
                * (i32::from(is_key_pressed(KeyCode::Down))
                    - i32::from(is_key_pressed(KeyCode::Up)));
            world.pin_copter(Some(pos));
        }
    }

    /// Record this frame's simulation events into the rolling on-screen log.
    pub fn record(&mut self, events: &[Event]) {
        for &event in events {
            if self.log.len() == LOG_CAP {
                self.log.pop_front();
            }
            self.log.push_back(event);
        }
    }

    /// Draw the active UI modes (left) and the simulation event log (right).
    pub fn draw_hint(&self, assets: &Assets, world: &World) {
        // UI/debug modes — a statement about this harness — top-left.
        let mut y = 2.0;
        for (on, label) in [
            (!self.mouse_steer, "KEYBOARD"),
            (world.wagon_pinned(), "WAGON PINNED"),
            (world.copter_pinned().is_some(), "COPTER PINNED"),
        ] {
            if on {
                draw::text(assets, label, 2.0, y, theme::INK);
                y += font::CELL_H + 1.0;
            }
        }

        // Simulation events — a statement about the game model — top-right,
        // right-aligned, newest last.
        let mut y = 2.0;
        for event in &self.log {
            let text = label(*event);
            let x = (config::LOGICAL_W - draw::text_width(&text) - 2) as f32;
            draw::text(assets, &text, x, y, theme::INK);
            y += font::CELL_H + 1.0;
        }
    }
}

/// A short past-tense label for an event — events describe what happened, so the
/// log reads uniformly as a list of facts.
fn label(event: Event) -> String {
    use stuntcopter_sim::stuntman::Outcome;
    match event {
        Event::Dropped => "dropped".to_owned(),
        Event::Resolved {
            outcome, points, ..
        } => match outcome {
            Outcome::Landed => format!("landed +{points}"),
            Outcome::Splat => "splatted".to_owned(),
            Outcome::HitDriver => "hit the driver".to_owned(),
            Outcome::HitHorse => "hit the horse".to_owned(),
        },
        Event::CelebrationStarted => "celebration started".to_owned(),
        Event::CrashStarted { wreck } => format!("crash started: {wreck:?}"),
        Event::ManRetired { men_left } => format!("man retired ({men_left} left)"),
        Event::LevelCleared { level } => format!("level {level} cleared"),
        Event::GameEnded { score } => format!("game ended ({score})"),
    }
}
