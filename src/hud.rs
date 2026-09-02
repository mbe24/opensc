//! The HUD.
//!
//! What it is: a full-width, gray-dithered strip of white section boxes matching
//! the original scorebox — the yoke, the lives row, score/hiscore, the
//! height/wagon/gravity panel, and the two flip boxes at the ends. Drawn from
//! primitives (not a decoded sprite) so live data and sharp bitmap text land in
//! the right cells.
//!
//! What it is not: game logic — it only reads [`World`].

use macroquad::prelude::*;

use crate::assets::Assets;
use crate::atlas::Sprite;
use crate::config::{
    GRAVITY_WORDS, HUD_DIGIT_DX, HUD_FLIP_L, HUD_FLIP_R, HUD_H, HUD_MEN_BOX, HUD_PANEL_BOX,
    HUD_SCORE_BOX, HUD_TOP, HUD_YOKE_BOX, HUD_YOKE_RADIUS, MAN_H, MAN_W, MEN_PER_LEVEL,
    WAGON_WORDS,
};
use crate::draw;
use crate::theme::{INK, SKY};
use crate::world::World;

const MEN_DX: i32 = 16;

pub fn draw(assets: &Assets, world: &World) {
    // Gray dither behind the whole strip (black checkerboard over the sky).
    draw_texture(&assets.dither, 0.0, HUD_TOP as f32, WHITE);
    // White-filled boxes; the rest stay on the dither.
    for &(x, w) in &[HUD_FLIP_L, HUD_YOKE_BOX, HUD_FLIP_R] {
        box_white(x, w);
    }
    for &(x, w) in &[HUD_MEN_BOX, HUD_SCORE_BOX, HUD_PANEL_BOX] {
        box_frame(x, w);
    }
    draw_yoke(world);
    draw_lives(assets, world);
    draw_score(assets, world);
    draw_panel(assets, world);
}

/// A white section box with a black frame.
fn box_white(x: i32, w: i32) {
    draw_rectangle(x as f32, HUD_TOP as f32, w as f32, HUD_H as f32, SKY);
    box_frame(x, w);
}

/// A section frame only; the dithered strip shows through.
fn box_frame(x: i32, w: i32) {
    draw_rectangle_lines(x as f32, HUD_TOP as f32, w as f32, HUD_H as f32, 2.0, INK);
}

/// The yoke crosshair, drifting with the copter's velocity.
fn draw_yoke(world: &World) {
    let (vx, vy) = world.copter.velocity();
    let (bx, bw) = HUD_YOKE_BOX;
    let cx = bx as f32 + bw as f32 / 2.0 + vx as f32;
    let cy = HUD_TOP as f32 + HUD_H as f32 / 2.0 + vy as f32;
    let r = HUD_YOKE_RADIUS;
    draw_line(cx - r, cy, cx + r, cy, 1.0, INK);
    draw_line(cx, cy - r, cx, cy + r, 1.0, INK);
    draw_circle_lines(cx, cy, 3.0, 1.0, INK);
}

/// The five lives: each man, the one in play inverted, thumbs for finished men.
fn draw_lives(assets: &Assets, world: &World) {
    let (bx, _) = HUD_MEN_BOX;
    let x0 = bx + 7;
    let man_y = HUD_TOP + 2;
    let up_y = HUD_TOP + 18;
    let down_y = HUD_TOP + 33;
    for i in 0..MEN_PER_LEVEL {
        let x = (x0 + i * MEN_DX) as f32;
        if i == world.current_man() as i32 {
            draw_rectangle(
                x - 1.0,
                man_y as f32 - 1.0,
                MAN_W as f32 + 2.0,
                MAN_H as f32,
                INK,
            );
            draw::sprite(assets, Sprite::ManHang, x, man_y as f32, SKY);
        } else {
            draw::sprite(assets, Sprite::ManHang, x, man_y as f32, INK);
        }
        match world.results[i as usize] {
            Some(true) => draw::sprite(assets, Sprite::ManThumbup, x, up_y as f32, INK),
            Some(false) => draw::sprite(assets, Sprite::ManThumbdown, x, down_y as f32, INK),
            None => {}
        }
    }
}

/// Score and hiscore: a centred label over six numeral sprites, twice. The
/// digit rows sit on a white backing so they read cleanly, like the original.
fn draw_score(assets: &Assets, world: &World) {
    let (bx, bw) = HUD_SCORE_BOX;
    let digits_x = bx + (bw - 6 * HUD_DIGIT_DX) / 2;
    let w = (6 * HUD_DIGIT_DX) as f32;
    for (score, y) in [(world.score, HUD_TOP + 10), (world.hiscore, HUD_TOP + 34)] {
        draw_rectangle(digits_x as f32, y as f32, w, 15.0, SKY);
        draw_digits(assets, score, digits_x, y);
    }
    label(assets, "score", bx, bw, HUD_TOP + 1);
    label(assets, "hiscore", bx, bw, HUD_TOP + 25);
}

/// The height/wagon/gravity panel: three rows of label | value with dividers.
fn draw_panel(assets: &Assets, world: &World) {
    let (bx, bw) = HUD_PANEL_BOX;
    let div_x = bx + 54;
    // Values sit in a white column; labels stay on the dither.
    draw_rectangle(
        div_x as f32,
        HUD_TOP as f32,
        (bx + bw - div_x) as f32,
        HUD_H as f32,
        SKY,
    );
    let wagon = WAGON_WORDS[(world.wagon.speed - 1).clamp(0, 2) as usize];
    let gravity = GRAVITY_WORDS[(world.gravity - 1).clamp(0, 3) as usize];
    let rows = [
        ("height", world.height().to_string()),
        ("wagon", wagon.to_owned()),
        ("gravity", gravity.to_owned()),
    ];
    for (i, (name, value)) in rows.iter().enumerate() {
        let ry = HUD_TOP + 1 + i as i32 * (HUD_H / 3);
        draw::text(assets, name, (bx + 4) as f32, ry as f32, INK);
        draw::text(assets, value, (div_x + 5) as f32, ry as f32, INK);
        if i > 0 {
            let y = (ry - 1) as f32;
            draw_line(bx as f32, y, (bx + bw) as f32, y, 1.0, INK);
        }
    }
    draw_line(
        div_x as f32,
        HUD_TOP as f32,
        div_x as f32,
        (HUD_TOP + HUD_H) as f32,
        1.0,
        INK,
    );
}

/// Draw a centred bitmap-font label within `[x, x+w)` at top `y`.
fn label(assets: &Assets, s: &str, x: i32, w: i32, y: i32) {
    let lx = x + (w - draw::text_width(s)) / 2;
    draw::text(assets, s, lx as f32, y as f32, INK);
}

/// Six numeral shapes; the one's digit is always 0 (`DrawScoreIntoBox`).
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
        draw::sprite(
            assets,
            digit_sprite(d),
            (x + i as i32 * HUD_DIGIT_DX) as f32,
            y as f32,
            INK,
        );
    }
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
