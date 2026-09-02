//! The HUD: live scorebox data drawn over the panel sprite.
//!
//! What it is: score/hiscore numerals, the height number, the wagon/gravity
//! words, and the lives row (per-man thumbs + a box around the man in play).
//! Positions come from `config`, matching the original's `OneTimeGameStuff`.
//!
//! What it is not: the panel graphic itself (a sprite) or any game logic.

use macroquad::prelude::*;

use crate::assets::Assets;
use crate::atlas::Sprite;
use crate::config::{
    GRAVITY_WORDS, HUD_DIGIT_DX, HUD_GRAV_POS, HUD_HEIGHT_POS, HUD_HISCORE_DY, HUD_MAN_DX,
    HUD_MAN_POS, HUD_SCORE_POS, HUD_THUMB_DOWN_DY, HUD_THUMB_UP_DY, HUD_WAGON_POS, HUD_YOKE_CENTER,
    HUD_YOKE_RADIUS, MAN_H, MAN_W, SCOREBOX_TOP, WAGON_WORDS,
};
use crate::draw;
use crate::theme::INK;
use crate::world::World;

pub fn draw(assets: &Assets, world: &World) {
    let top = SCOREBOX_TOP;
    let (sx, sy) = HUD_SCORE_POS;
    draw_digits(assets, world.score, sx, sy + top);
    draw_digits(assets, world.hiscore, sx, sy + HUD_HISCORE_DY + top);
    draw_status(assets, world, top);
    draw_lives(assets, world, top);
    draw_yoke(world, top);
}

/// The far-left yoke: a crosshair that drifts with the copter's velocity, like
/// the original's mouse-tracking control indicator.
fn draw_yoke(world: &World, top: i32) {
    let (vx, vy) = world.copter.velocity();
    let cx = (HUD_YOKE_CENTER.0 + vx) as f32;
    let cy = (HUD_YOKE_CENTER.1 + vy + top) as f32;
    let r = HUD_YOKE_RADIUS;
    draw_line(cx - r, cy, cx + r, cy, 1.0, INK);
    draw_line(cx, cy - r, cx, cy + r, 1.0, INK);
    draw_circle_lines(cx, cy, 3.0, 1.0, INK);
}

/// Draw a score as six numeral shapes; the one's digit is always 0, exactly as
/// the original (`DrawScoreIntoBox`), so an internal 510 reads as `005100`.
fn draw_digits(assets: &Assets, score: i32, x: i32, y: i32) {
    let digits = [
        (score / 10000) % 10,
        (score / 1000) % 10,
        (score / 100) % 10,
        (score / 10) % 10,
        score % 10,
        0,
    ];
    for (i, &d) in digits.iter().enumerate() {
        let dx = x + i as i32 * HUD_DIGIT_DX;
        draw::sprite(assets, digit_sprite(d), dx as f32, y as f32, INK);
    }
}

fn draw_status(assets: &Assets, world: &World, top: i32) {
    let wagon = WAGON_WORDS[(world.wagon.speed - 1).clamp(0, 2) as usize];
    let gravity = GRAVITY_WORDS[(world.gravity - 1).clamp(0, 3) as usize];
    text(assets, &world.height().to_string(), HUD_HEIGHT_POS, top);
    text(assets, wagon, HUD_WAGON_POS, top);
    text(assets, gravity, HUD_GRAV_POS, top);
}

/// Thumbs for finished men plus a box around the one currently in play.
fn draw_lives(assets: &Assets, world: &World, top: i32) {
    let (mx, my) = HUD_MAN_POS;
    for (i, result) in world.results.iter().enumerate() {
        let x = (mx + i as i32 * HUD_MAN_DX) as f32;
        match result {
            Some(true) => {
                draw::sprite(
                    assets,
                    Sprite::ManThumbup,
                    x,
                    (my + HUD_THUMB_UP_DY + top) as f32,
                    INK,
                );
            }
            Some(false) => {
                draw::sprite(
                    assets,
                    Sprite::ManThumbdown,
                    x,
                    (my + HUD_THUMB_DOWN_DY + top) as f32,
                    INK,
                );
            }
            None => {}
        }
    }
    // The man in play: an inverted cell (black box, white man), as in the original.
    let hx = (mx + world.current_man() as i32 * HUD_MAN_DX) as f32;
    let hy = (my + top) as f32;
    draw_rectangle(hx - 1.0, hy - 1.0, MAN_W as f32 + 2.0, MAN_H as f32, INK);
    draw::sprite(assets, Sprite::ManHang, hx, hy, WHITE);
}

/// Sharp bitmap-font text. `pos.1` is a baseline-ish y; the font blits by cell
/// top, so shift up to sit the glyphs on that line.
fn text(assets: &Assets, s: &str, pos: (i32, i32), top: i32) {
    draw::text(assets, s, pos.0 as f32, (pos.1 + top - 11) as f32, INK);
}

fn digit_sprite(d: i32) -> Sprite {
    match d {
        0 => Sprite::Num0,
        1 => Sprite::Num1,
        2 => Sprite::Num2,
        3 => Sprite::Num3,
        4 => Sprite::Num4,
        5 => Sprite::Num5,
        6 => Sprite::Num6,
        7 => Sprite::Num7,
        8 => Sprite::Num8,
        _ => Sprite::Num9,
    }
}
