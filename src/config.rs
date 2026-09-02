//! Game constants, ported verbatim from the original Pascal source
//! (`reference/stuntcopter/StuntCopter.pas`). Keeping them in one place makes
//! the port auditable against the original and easy to calibrate.
//!
//! Some constants are defined ahead of the gameplay code that will consume them,
//! so the whole ported ruleset lives here from the start.
#![allow(dead_code)]

/// Logical playfield width — the classic 512-pixel Macintosh screen.
pub const LOGICAL_W: i32 = 512;
/// Logical playfield height — the classic Macintosh screen.
pub const LOGICAL_H: i32 = 342;

// --- Simulation timing -----------------------------------------------------
//
// The original had no fixed frame rate: its loop ran as fast as the CPU allowed
// (`SpeedTrapOn` defaulted to false; the optional `Delay` only *slowed* faster
// Macs). On period hardware with QuickDraw blits that was well under 60 Hz, so
// we tick a fixed simulation at `TICK_HZ` and calibrate the feel by playtest.
// All velocities below are integer pixels-per-tick, exactly as in the original.

/// Fixed simulation rate. THE key fidelity knob — calibrate against the original.
pub const TICK_HZ: f32 = 30.0;
/// Seconds per simulation tick.
pub const TICK_PERIOD: f32 = 1.0 / TICK_HZ;
/// Largest frame delta we will ever integrate, in seconds. Caps catch-up after a
/// backgrounded tab / minimized window so the accumulator can't spiral.
pub const MAX_FRAME_TIME: f32 = 0.25;

// --- Copter control (mouse-as-joystick with inertia) -----------------------

/// Region the pointer maps within, `(left, top, right, bottom)`. Pointer
/// position here maps linearly into [`DELTA_RECT`] to give the requested velocity.
pub const MOUSE_RECT: (i32, i32, i32, i32) = (210, 134, 302, 206);
/// Requested-velocity range `(min_h, min_v, max_h, max_v)`, px/tick. Note the
/// asymmetric vertical range: the copter rises faster than it can climb.
pub const DELTA_RECT: (i32, i32, i32, i32) = (-4, -3, 4, 4);

/// Copter sprite size in pixels.
pub const COPTER_W: i32 = 74;
pub const COPTER_H: i32 = 26;
/// Copter spawn position (top-left).
pub const COPTER_START: (i32, i32) = (212, 110);

// --- Wagon -----------------------------------------------------------------

pub const WAGON_W: i32 = 73;
pub const WAGON_H: i32 = 22;
/// Slowest wagon speed (level 1), px/tick. Ramps 1->3 across levels.
pub const WAGON_SPEED_START: i32 = 1;

// --- Stuntman --------------------------------------------------------------

/// Fastest fall (level 1), px/tick. Ramps 4->1 across later levels (slower fall
/// = harder timing).
pub const GRAVITY_START: i32 = 4;
pub const MAN_W: i32 = 14;
pub const MAN_H: i32 = 16;
/// The hanging man's offset from the copter's top-left (`CoptRect.left+36`,
/// `top+23`).
pub const MAN_HANG_OFFSET: (i32, i32) = (36, 23);

// --- Progression -----------------------------------------------------------

/// Men per level; all must land to advance.
pub const MEN_PER_LEVEL: i32 = 5;
/// Lightest to heaviest gravity words, indexed by `gravity` (1..=4).
pub const GRAVITY_WORDS: [&str; 4] = ["OH BOY", "FLYING", "NORMAL", "HEAVY"];
/// Slowest to fastest wagon words, indexed by `wagon speed` (1..=3).
pub const WAGON_WORDS: [&str; 3] = ["WALK", "TROT", "GALLOP"];

// --- Layout: HUD at the bottom, ground line just above it ------------------

/// Height of the bottom scorebox HUD panel.
pub const SCOREBOX_H: i32 = 50;
/// Y of the top of the scorebox HUD.
pub const SCOREBOX_TOP: i32 = LOGICAL_H - SCOREBOX_H;
/// The ground line the wagon rolls along (`ScoreBoxRect.top - 4`).
pub const GROUND_Y: i32 = SCOREBOX_TOP - 4;
/// Copter may not descend below this bottom edge (`WagonRect.top - 10`).
pub const COPTER_BOTTOM_LIMIT: i32 = (GROUND_Y - WAGON_H) - 10;

// --- HUD reconstruction: a full-width gray strip of white section boxes ------
// `(x, width)` per section; every box spans `HUD_TOP..LOGICAL_H`.

pub const HUD_TOP: i32 = SCOREBOX_TOP;
pub const HUD_H: i32 = LOGICAL_H - SCOREBOX_TOP;

pub const HUD_FLIP_L: (i32, i32) = (2, 52);
pub const HUD_YOKE_BOX: (i32, i32) = (56, 52);
pub const HUD_MEN_BOX: (i32, i32) = (110, 92);
pub const HUD_SCORE_BOX: (i32, i32) = (204, 138);
pub const HUD_PANEL_BOX: (i32, i32) = (344, 116);
pub const HUD_FLIP_R: (i32, i32) = (462, 48);

/// Numeral cell width for the packed score digits.
pub const HUD_DIGIT_DX: i32 = 21;
/// Crosshair arm length inside the yoke box.
pub const HUD_YOKE_RADIUS: f32 = 15.0;
