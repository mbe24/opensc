//! Game audio: the presentation layer's reaction to simulation events and state.
//!
//! What it is: a small macroquad-backed player. It preloads every sound once at
//! startup (generated procedurally, so there are no asset files), then each
//! frame the caller hands it the tick's [`Event`]s (one-shots: splat, fanfare)
//! and the ambient state (the looping copter drone). The pure waveform synthesis
//! lives in [`synth`]; only this file touches macroquad.
//!
//! What it is not: game logic — it only reacts. The simulation neither knows nor
//! cares that audio exists.

mod synth;

use macroquad::audio::{load_sound_from_bytes, play_sound, stop_sound, PlaySoundParams, Sound};
use stuntcopter_sim::stuntman::Outcome;
use stuntcopter_sim::Event;

const DRONE_VOLUME: f32 = 0.25;
const SPLAT_VOLUME: f32 = 0.55;
const FANFARE_VOLUME: f32 = 0.5;

/// A continuously looping sound tied to game state (not to a one-off event).
#[derive(Clone, Copy)]
pub enum Ambient {
    Copter,
}

/// The preloaded sounds and the state needed to drive the loops.
pub struct Audio {
    drone: Sound,
    splat: Sound,
    /// One fanfare per level (index `level - 1`, clamped to [`synth::MAX_LEVEL`]).
    fanfares: Vec<Sound>,
    copter_on: bool,
}

impl Audio {
    /// Generate and load every sound. Async because macroquad decodes on load.
    pub async fn load() -> Self {
        let mut fanfares = Vec::with_capacity(synth::MAX_LEVEL as usize);
        for level in 1..=synth::MAX_LEVEL {
            fanfares.push(decode(&synth::fanfare(level)).await);
        }
        Self {
            drone: decode(&synth::drone()).await,
            splat: decode(&synth::splat()).await,
            fanfares,
            copter_on: false,
        }
    }

    /// Play the one-shots for this tick's events. `level` pitches the fanfare and
    /// is read from current game state, never carried by the event.
    pub fn handle_events(&self, events: &[Event], level: i32) {
        for event in events {
            match *event {
                Event::CelebrationStarted => {
                    let idx = (level.clamp(1, synth::MAX_LEVEL) - 1) as usize;
                    play_once(&self.fanfares[idx], FANFARE_VOLUME);
                }
                Event::Resolved { outcome, .. } if is_crash(outcome) => {
                    play_once(&self.splat, SPLAT_VOLUME);
                }
                _ => {}
            }
        }
    }

    /// Start or stop an ambient loop to match game state. Idempotent: it only
    /// acts on a transition, so calling it every frame is fine.
    pub fn set_ambient(&mut self, which: Ambient, active: bool) {
        match which {
            Ambient::Copter => {
                if active && !self.copter_on {
                    play_sound(
                        &self.drone,
                        PlaySoundParams {
                            looped: true,
                            volume: DRONE_VOLUME,
                        },
                    );
                } else if !active && self.copter_on {
                    stop_sound(&self.drone);
                }
                self.copter_on = active;
            }
        }
    }
}

/// Whether an outcome is a crash (any non-hay result), which gets the splat.
fn is_crash(outcome: Outcome) -> bool {
    matches!(
        outcome,
        Outcome::Splat | Outcome::HitDriver | Outcome::HitHorse
    )
}

fn play_once(sound: &Sound, volume: f32) {
    play_sound(
        sound,
        PlaySoundParams {
            looped: false,
            volume,
        },
    );
}

/// Decode a generated WAV buffer into a playable sound.
async fn decode(wav: &[u8]) -> Sound {
    load_sound_from_bytes(wav)
        .await
        .expect("generated WAV should always decode")
}
