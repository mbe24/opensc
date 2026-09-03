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
use stuntcopter_sim::{Sprite, World};

use crate::assets::Assets;
use crate::config::{
    FLIP_BOX_L, FLIP_BOX_R, HUD_BAND_TOP, HUD_DIGIT_DX, HUD_GRAV_ROW, HUD_HEIGHT, HUD_HISCORE_DY,
    HUD_MAN0, HUD_MAN_DX, HUD_NUM0, HUD_THUMBDOWN_DY, HUD_THUMBUP_DY, HUD_VALUE_X0, HUD_VALUE_X1,
    HUD_WAGON_ROW, HUD_YOKE_WIN, MAN_H, MAN_W, MEN_PER_LEVEL, SB_H, SB_W, SB_X, SB_Y,
};
use crate::draw;
use crate::theme::{INK, SKY};
use stuntcopter_sim::stuntman::Stuntman;

pub fn draw(assets: &Assets, world: &World, playing: bool) {
    // dkGray band, then the opaque-white ScoreBox PICT on top (gray shows only in
    // the margins / flip-box areas).
    draw_texture(&assets.dither, 0.0, HUD_BAND_TOP as f32, WHITE);
    draw_rectangle(SB_X as f32, SB_Y as f32, SB_W as f32, SB_H as f32, SKY);
    draw::sprite(assets, Sprite::Scorebox, SB_X as f32, SB_Y as f32, INK);

    draw_lives(assets, world);
    draw_digits(assets, world.score, HUD_NUM0.1);
    draw_digits(assets, world.hiscore, HUD_NUM0.1 + HUD_HISCORE_DY);
    draw_status(assets, world);
    // The yoke tracks the copter only while a game is underway; otherwise it is
    // covered with gray (the original's `FillRect(YokeErase, Gray)`).
    if playing {
        draw_yoke(assets, world);
    } else {
        let (wx, wy, _, _) = HUD_YOKE_WIN;
        draw_texture(
            &assets.yoke_gray,
            (SB_X + wx) as f32,
            (SB_Y + wy) as f32,
            INK,
        );
    }
    draw_flip_boxes(assets, world);
}

/// While a landing is being celebrated, the backflip figure tumbles inside a
/// framed box in each gray margin, on either side of the ScoreBox.
fn draw_flip_boxes(assets: &Assets, world: &World) {
    let Stuntman::Celebrating(flip) = &world.stuntman else {
        return;
    };
    let Some(pose) = flip.pose() else {
        return;
    };
    for (fx, fy, fw, fh) in [FLIP_BOX_L, FLIP_BOX_R] {
        draw_rectangle(fx as f32, fy as f32, fw as f32, fh as f32, SKY);
        draw_rectangle_lines(fx as f32, fy as f32, fw as f32, fh as f32, 1.0, INK);
        draw::sprite(assets, pose, (fx + 4) as f32, (fy + 4) as f32, INK);
    }
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
    // Invert the current man's cell: a black fill bounded to the 14x16 cell (no
    // overshoot into the neighbouring dotted-grid cells), with the man in white.
    let x = (SB_X + HUD_MAN0.0 + world.current_man() as i32 * HUD_MAN_DX) as f32;
    let y = (SB_Y + HUD_MAN0.1) as f32;
    draw_rectangle(x, y, MAN_W as f32, MAN_H as f32, INK);
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

/// The yoke: the original's `OffCross` reticle bitmap, centered in the window and
/// shifted by the copter's velocity, clipped to the window (as the copter moves,
/// the cross slides behind the frame — exactly the original's masked CopyBits).
fn draw_yoke(assets: &Assets, world: &World) {
    let (vx, vy) = world.copter.velocity();
    let (wx, wy, ww, wh) = HUD_YOKE_WIN;
    let (win_l, win_t) = ((SB_X + wx) as f32, (SB_Y + wy) as f32);
    let (win_w, win_h) = (ww as f32, wh as f32);
    let cross = Sprite::Cross.rect();

    // Where the full cross would sit: centered in the window, offset by velocity.
    let cross_l = win_l + (win_w - cross.w) / 2.0 + vx as f32;
    let cross_t = win_t + (win_h - cross.h) / 2.0 + vy as f32;

    // Draw only the part overlapping the window (the rest is clipped by the frame).
    let vis_l = cross_l.max(win_l);
    let vis_t = cross_t.max(win_t);
    let vis_r = (cross_l + cross.w).min(win_l + win_w);
    let vis_b = (cross_t + cross.h).min(win_t + win_h);
    if vis_r <= vis_l || vis_b <= vis_t {
        return;
    }
    let (vw, vh) = (vis_r - vis_l, vis_b - vis_t);
    draw_texture_ex(
        &assets.atlas,
        vis_l,
        vis_t,
        INK,
        DrawTextureParams {
            source: Some(Rect::new(
                cross.x + (vis_l - cross_l),
                cross.y + (vis_t - cross_t),
                vw,
                vh,
            )),
            dest_size: Some(vec2(vw, vh)),
            ..Default::default()
        },
    );
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
