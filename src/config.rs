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

// --- Layout: exact from OneTimeGameStuff (portRect = 512x342) ---------------

/// The ground line / wagon baseline (`ScoreBoxRect.top - 4`), full width.
pub const GROUND_Y: i32 = 285;
/// Top of the dkGray HUD band (`ScoreBoxRect.top - 3`).
pub const HUD_BAND_TOP: i32 = 286;
/// Copter may not descend below this bottom edge (`WagonRect.top - 10`).
pub const COPTER_BOTTOM_LIMIT: i32 = (GROUND_Y - WAGON_H) - 10;

/// The ScoreBox panel (PICT #130): centered, `(x, y, w, h)`.
pub const SB_X: i32 = 62;
pub const SB_Y: i32 = 289;
pub const SB_W: i32 = 387;
pub const SB_H: i32 = 51;

// --- ScoreBox sub-rects, offsets relative to (SB_X, SB_Y) -------------------

/// First life man; step [`HUD_MAN_DX`] per column, thumb rows below.
pub const HUD_MAN0: (i32, i32) = (54, 0);
pub const HUD_MAN_DX: i32 = 15;
pub const HUD_THUMBUP_DY: i32 = 17;
pub const HUD_THUMBDOWN_DY: i32 = 34;
/// First score digit; step [`HUD_DIGIT_DX`] per digit, hiscore [`HUD_HISCORE_DY`] below.
pub const HUD_NUM0: (i32, i32) = (135, 10);
pub const HUD_DIGIT_DX: i32 = 21;
pub const HUD_HISCORE_DY: i32 = 25;
/// The yoke window (`YokeErase`), where the crosshair moves.
pub const HUD_YOKE_WIN: (i32, i32, i32, i32) = (4, 4, 43, 43);
/// Height number (left-aligned), and centered wagon/gravity word rows.
pub const HUD_HEIGHT: (i32, i32) = (339, 2);
pub const HUD_WAGON_ROW: i32 = 19;
pub const HUD_GRAV_ROW: i32 = 36;
/// Value column (right of the panel divider), for centering the status words.
pub const HUD_VALUE_X0: i32 = 325;
pub const HUD_VALUE_X1: i32 = SB_W - 2;

/// Flip boxes at the screen edges, `(x, y, w, h)` (`FlipFrame[1/2]`).
pub const FLIP_BOX_L: (i32, i32, i32, i32) = (11, 289, 40, 46);
pub const FLIP_BOX_R: (i32, i32, i32, i32) = (460, 289, 40, 46);
