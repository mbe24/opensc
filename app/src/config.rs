//! Presentation constants: the pixel-exact HUD/ScoreBox layout, plus a
//! re-export of the simulation's gameplay constants so UI code has one import
//! site (`crate::config`) for both.
//!
//! Layout offsets are ported from `OneTimeGameStuff` (portRect = 512x342). The
//! gameplay geometry/timing lives in [`stuntcopter_sim::config`] and is
//! re-exported below.
#![allow(dead_code)]

pub use stuntcopter_sim::config::*;
use stuntcopter_sim::Rect;

// --- Layout: exact from OneTimeGameStuff (portRect = 512x342) ---------------

/// Top of the dkGray HUD band (`ScoreBoxRect.top - 3`).
pub const HUD_BAND_TOP: i32 = 286;

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
pub const HUD_YOKE_WIN: Rect = Rect::new(4, 4, 43, 43);
/// Height number (left-aligned), and centered wagon/gravity word rows.
pub const HUD_HEIGHT: (i32, i32) = (339, 2);
pub const HUD_WAGON_ROW: i32 = 19;
pub const HUD_GRAV_ROW: i32 = 36;
/// Value column (right of the panel divider), for centering the status words.
pub const HUD_VALUE_X0: i32 = 325;
pub const HUD_VALUE_X1: i32 = SB_W - 2;

/// Flip boxes at the screen edges (`FlipFrame[1/2]`).
pub const FLIP_BOX_L: Rect = Rect::new(11, 289, 40, 46);
pub const FLIP_BOX_R: Rect = Rect::new(460, 289, 40, 46);
