#![doc = include_str!("../README.md")]

mod assets;
mod atlas;
mod canvas;
mod cloud;
mod config;
mod copter;
mod draw;
mod font;
mod hud;
mod input;
mod level;
mod stuntman;
mod theme;
mod wagon;
mod world;

use assets::Assets;
use atlas::Sprite;
use canvas::Canvas;
use config::{GROUND_Y, MAN_H, TICK_PERIOD, WAGON_H};
use macroquad::prelude::*;
use stuntman::{Outcome, Stuntman};
use world::{RenderState, World};

fn window_conf() -> Conf {
    Conf {
        window_title: "OpenSC — StuntCopter".to_owned(),
        window_width: config::LOGICAL_W * 2,
        window_height: config::LOGICAL_H * 2,
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    install_panic_logging();
    // A touch must not also fire a synthetic mouse event, or every tap would be
    // processed twice.
    simulate_mouse_with_touch(false);

    let assets = Assets::load();
    let canvas = Canvas::new();
    let mut world = World::default();
    let mut prev = world.render_state();
    let mut scene = Scene::Attract;
    let mut accumulator = 0.0f32;

    loop {
        // Clamp the frame delta so a backgrounded tab can't make the accumulator
        // spiral into hundreds of catch-up ticks on return.
        accumulator += get_frame_time().min(config::MAX_FRAME_TIME);

        let mut intents = input::gather(&canvas);
        let confirm = intents.drop || is_key_pressed(KeyCode::Enter);
        let exit = is_key_pressed(KeyCode::Backspace) || is_key_pressed(KeyCode::Escape);

        match scene {
            Scene::Attract => {
                while accumulator >= TICK_PERIOD {
                    prev = world.render_state();
                    world.attract_tick();
                    accumulator -= TICK_PERIOD;
                }
                if confirm {
                    world.begin();
                    prev = world.render_state();
                    scene = Scene::Playing;
                }
            }
            Scene::Playing => {
                // Drain the edge-triggered drop after the first tick so a
                // multi-tick frame can't fire it twice.
                while accumulator >= TICK_PERIOD {
                    prev = world.render_state();
                    world.tick(&intents);
                    intents.drop = false;
                    accumulator -= TICK_PERIOD;
                    if world.over {
                        scene = Scene::Attract;
                        break;
                    }
                }
                if matches!(scene, Scene::Playing) && exit {
                    scene = Scene::Paused;
                }
            }
            Scene::Paused => {
                accumulator = 0.0; // time doesn't pass while paused
                if confirm {
                    scene = Scene::Playing;
                } else if exit {
                    scene = Scene::Attract;
                }
            }
        }

        // Interpolate render positions between the last two ticks so motion is
        // smooth at the display's refresh rate, not just the 30 Hz sim rate.
        let alpha = (accumulator / TICK_PERIOD).clamp(0.0, 1.0);

        canvas.begin();
        draw_scene(
            &assets,
            &world,
            prev,
            alpha,
            !matches!(scene, Scene::Attract),
            matches!(scene, Scene::Playing),
        );
        match scene {
            Scene::Attract => draw_attract(&assets),
            Scene::Paused => draw_paused(&assets),
            Scene::Playing => {}
        }
        canvas.end();
        canvas.present(theme::BARS);
        next_frame().await;
    }
}

/// The top-level game scene.
enum Scene {
    Attract,
    Playing,
    Paused,
}

/// Interpolate an integer coordinate between the previous and current tick,
/// rounded to a whole pixel (crisp), snapping across wraps/teleports so a
/// wrapping sprite never slides across the screen.
fn lerp(prev: i32, cur: i32, alpha: f32) -> f32 {
    if (cur - prev).abs() > 32 {
        cur as f32
    } else {
        (prev as f32 + (cur - prev) as f32 * alpha).round()
    }
}

/// Draw one frame, interpolating positions between ticks for smooth motion.
/// `show_man` is false on the attract screen, where no stuntman is present;
/// `playing` gates the live yoke crosshair (gray-covered otherwise).
fn draw_scene(
    assets: &Assets,
    world: &World,
    prev: RenderState,
    alpha: f32,
    show_man: bool,
    playing: bool,
) {
    clear_background(theme::SKY);
    draw_line(
        0.0,
        GROUND_Y as f32,
        config::LOGICAL_W as f32,
        GROUND_Y as f32,
        1.0,
        theme::INK,
    );

    let cur = world.render_state();
    let cloud = (
        lerp(prev.cloud.0, cur.cloud.0, alpha),
        lerp(prev.cloud.1, cur.cloud.1, alpha),
    );
    let wagon_x = lerp(prev.wagon_x, cur.wagon_x, alpha);
    let copter = (
        lerp(prev.copter.0, cur.copter.0, alpha),
        lerp(prev.copter.1, cur.copter.1, alpha),
    );

    draw::sprite(assets, world.cloud.sprite(), cloud.0, cloud.1, theme::INK);
    draw::sprite(
        assets,
        world.wagon.sprite(),
        wagon_x,
        (GROUND_Y - WAGON_H) as f32,
        theme::INK,
    );
    draw::sprite(
        assets,
        world.copter.sprite(),
        copter.0,
        copter.1,
        theme::INK,
    );
    if show_man {
        draw_stuntman(assets, world, prev, alpha, copter, wagon_x);
    }
    hud::draw(assets, world, playing);
}

/// The attract screen: the bold title/credit and a rounded BEGIN button drawn
/// over the idle scene, matching the original's wait-for-BEGIN layout.
fn draw_attract(assets: &Assets) {
    text_center(assets, "StuntCopter", 44.0, 2.0);
    text_center(assets, "by Duane Blehm", 74.0, 1.0);

    // Authentic button geometry from `CreateWindow`: 80x26, centered
    // horizontally, top at v=165.
    let (bw, bh) = (80.0, 26.0);
    let bx = (config::LOGICAL_W as f32 - bw) / 2.0;
    let by = 165.0;
    let r = 8.0;
    draw::round_rect(bx, by, bw, bh, r, theme::INK);
    draw::round_rect(bx + 1.0, by + 1.0, bw - 2.0, bh - 2.0, r - 1.0, theme::SKY);
    let tw = draw::text_width("BEGIN") as f32;
    draw::text(
        assets,
        "BEGIN",
        bx + (bw - tw) / 2.0,
        by + (bh - font::CELL_H) / 2.0,
        theme::INK,
    );
}

/// The pause overlay.
fn draw_paused(assets: &Assets) {
    text_center(assets, "PAUSED", 110.0, 2.0);
    text_center(assets, "SPACE resumes   ESC ends", 140.0, 1.0);
}

/// Draw `s` horizontally centered on the canvas at top `y`, scaled by `scale`.
fn text_center(assets: &Assets, s: &str, y: f32, scale: f32) {
    let w = draw::text_width(s) as f32 * scale;
    draw::text_scaled(
        assets,
        s,
        (config::LOGICAL_W as f32 - w) / 2.0,
        y,
        scale,
        theme::INK,
    );
}

/// Draw the stuntman for the current phase, tracking the interpolated copter/wagon.
fn draw_stuntman(
    assets: &Assets,
    world: &World,
    prev: RenderState,
    alpha: f32,
    copter: (f32, f32),
    wagon_x: f32,
) {
    let ink = theme::INK;
    let wagon_y = (GROUND_Y - WAGON_H) as f32;
    match &world.stuntman {
        Stuntman::Hanging => {
            let x = copter.0 + config::MAN_HANG_OFFSET.0 as f32;
            let y = copter.1 + config::MAN_HANG_OFFSET.1 as f32;
            draw::sprite(assets, Sprite::ManHang, x, y, ink);
        }
        Stuntman::Falling(faller) => {
            let (x, y) = match prev.faller {
                Some(p) => (lerp(p.0, faller.x, alpha), lerp(p.1, faller.y, alpha)),
                None => (faller.x as f32, faller.y as f32),
            };
            draw::sprite(assets, faller.sprite(), x, y, ink);
        }
        Stuntman::Held(held) => match held.outcome {
            Outcome::Landed => {
                draw::sprite(
                    assets,
                    Sprite::ManInWagon,
                    wagon_x + 30.0,
                    wagon_y - 6.0,
                    ink,
                );
            }
            Outcome::Splat => {
                let y = (GROUND_Y - MAN_H) as f32;
                draw::sprite(assets, Sprite::ManSplat1, held.x as f32, y, ink);
            }
            Outcome::HitDriver => {
                draw::sprite(assets, Sprite::Driver, wagon_x + 30.0, wagon_y, ink);
            }
            Outcome::HitHorse => {
                draw::sprite(assets, Sprite::HorseDead, wagon_x + 45.0, wagon_y, ink);
            }
        },
    }
}

/// Route panics to the browser console on web (via macroquad/miniquad's
/// re-exported `error!`, which needs no wasm-bindgen) and to stderr on native.
/// Without this, a web panic is a silent blank canvas.
fn install_panic_logging() {
    std::panic::set_hook(Box::new(|info| {
        #[cfg(target_arch = "wasm32")]
        macroquad::miniquad::error!("{info}");
        #[cfg(not(target_arch = "wasm32"))]
        eprintln!("{info}");
    }));
}
