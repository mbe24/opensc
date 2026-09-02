//! The HUD.
//!
//! What it is: the authentic ScoreBox panel (PICT #130, opaque white with baked
//! frames/labels/men/dotted-grid) drawn over a dkGray band, with the dynamic
//! values overlaid at the exact offsets from `OneTimeGameStuff` — the yoke
//! crosshair, the current-man highlight and thumbs, score/hiscore digits, and
//! the height/wagon/gravity readouts.
//!
//! What it is not: game logic — it only reads [`World`].

use macroquad::prelude::*;

use crate::assets::Assets;
use crate::atlas::Sprite;
use crate::config::{
    HUD_BAND_TOP, HUD_DIGIT_DX, HUD_GRAV_ROW, HUD_HEIGHT, HUD_HISCORE_DY, HUD_MAN0, HUD_MAN_DX,
    HUD_NUM0, HUD_THUMBDOWN_DY, HUD_THUMBUP_DY, HUD_VALUE_X0, HUD_VALUE_X1, HUD_WAGON_ROW,
    HUD_YOKE_WIN, MAN_H, MAN_W, MEN_PER_LEVEL, SB_H, SB_W, SB_X, SB_Y,
};
use crate::draw;
use crate::theme::{INK, SKY};
use crate::world::World;

pub fn draw(assets: &Assets, world: &World) {
    // dkGray band, then the opaque-white ScoreBox PICT on top (gray shows only in
    // the margins / flip-box areas).
    draw_texture(&assets.dither, 0.0, HUD_BAND_TOP as f32, WHITE);
    draw_rectangle(SB_X as f32, SB_Y as f32, SB_W as f32, SB_H as f32, SKY);
    draw::sprite(assets, Sprite::Scorebox, SB_X as f32, SB_Y as f32, INK);

    draw_lives(assets, world);
    draw_digits(assets, world.score, HUD_NUM0.1);
    draw_digits(assets, world.hiscore, HUD_NUM0.1 + HUD_HISCORE_DY);
    draw_status(assets, world);
    draw_yoke(world);
}

/// Invert the current man's cell (baked in the PICT) and draw thumbs for the men
/// already resolved this level.
fn draw_lives(assets: &Assets, world: &World) {
    for i in 0..MEN_PER_LEVEL {
        let x = (SB_X + HUD_MAN0.0 + i * HUD_MAN_DX) as f32;
        match world.results[i as usize] {
            Some(true) => {
                let y = (SB_Y + HUD_MAN0.1 + HUD_THUMBUP_DY) as f32;
                draw::sprite(assets, Sprite::ManThumbup, x, y, INK);
            }
            Some(false) => {
                let y = (SB_Y + HUD_MAN0.1 + HUD_THUMBDOWN_DY) as f32;
                draw::sprite(assets, Sprite::ManThumbdown, x, y, INK);
            }
            None => {}
        }
    }
    let x = (SB_X + HUD_MAN0.0 + world.current_man() as i32 * HUD_MAN_DX) as f32;
    let y = (SB_Y + HUD_MAN0.1) as f32;
    draw_rectangle(x - 1.0, y - 1.0, MAN_W as f32 + 2.0, MAN_H as f32, INK);
    draw::sprite(assets, Sprite::ManHang, x, y, SKY);
}

/// Six numeral sprites (white digit on a dithered cell); the one's digit is
/// always 0, exactly as `DrawScoreIntoBox`.
fn draw_digits(assets: &Assets, score: i32, y_rel: i32) {
    let digits = [
        (score / 10000) % 10,
        (score / 1000) % 10,
        (score / 100) % 10,
        (score / 10) % 10,
        score % 10,
        0,
    ];
    let y = (SB_Y + y_rel) as f32;
    for (i, &d) in digits.iter().enumerate() {
        let x = (SB_X + HUD_NUM0.0 + i as i32 * HUD_DIGIT_DX) as f32;
        draw::sprite(assets, digit_sprite(d), x, y, INK);
    }
}

/// Height number (left-aligned), and the centered wagon/gravity words.
fn draw_status(assets: &Assets, world: &World) {
    let wagon = world.progression.wagon.word();
    let gravity = world.progression.gravity.word();
    draw::text(
        assets,
        &world.height().to_string(),
        (SB_X + HUD_HEIGHT.0) as f32,
        (SB_Y + HUD_HEIGHT.1) as f32,
        INK,
    );
    centered(assets, wagon, HUD_WAGON_ROW);
    centered(assets, gravity, HUD_GRAV_ROW);
}

fn centered(assets: &Assets, s: &str, y_rel: i32) {
    let cx = SB_X + (HUD_VALUE_X0 + HUD_VALUE_X1) / 2;
    let x = cx - draw::text_width(s) / 2;
    draw::text(assets, s, x as f32, (SB_Y + y_rel) as f32, INK);
}

/// The yoke: a double-line crosshair with a center hub (the original's `OffCross`
/// reticle), whose arms span the yoke window and shift with the copter's velocity.
fn draw_yoke(world: &World) {
    let (vx, vy) = world.copter.velocity();
    let (wx, wy, ww, wh) = HUD_YOKE_WIN;
    let (l, t) = ((SB_X + wx) as f32, (SB_Y + wy) as f32);
    let (r, b) = (l + ww as f32, t + wh as f32);
    let cx = l + ww as f32 / 2.0 + vx as f32;
    let cy = t + wh as f32 / 2.0 + vy as f32;
    for d in [-1.0, 1.0] {
        draw_line(l, cy + d, r, cy + d, 1.0, INK); // double horizontal arm
        draw_line(cx + d, t, cx + d, b, 1.0, INK); // double vertical arm
    }
    draw_circle_lines(cx, cy, 3.5, 1.0, INK);
    draw_circle(cx, cy, 1.0, INK);
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
