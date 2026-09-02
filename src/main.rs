#![doc = include_str!("../README.md")]

mod assets;
mod atlas;
mod canvas;
mod config;
mod copter;
mod draw;
mod hud;
mod input;
mod stuntman;
mod theme;
mod wagon;
mod world;

use assets::Assets;
use atlas::Sprite;
use canvas::Canvas;
use config::{GROUND_Y, MAN_H, SCOREBOX_TOP, TICK_PERIOD, WAGON_H};
use macroquad::prelude::*;
use stuntman::{Outcome, Stuntman};
use world::World;

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
    let mut accumulator = 0.0f32;

    loop {
        // Clamp the frame delta so a backgrounded tab can't make the accumulator
        // spiral into hundreds of catch-up ticks on return.
        accumulator += get_frame_time().min(config::MAX_FRAME_TIME);

        // Sample input once per frame; drain the edge-triggered drop after the
        // first tick so a multi-tick frame can't fire it twice.
        let mut intents = input::gather(&canvas);
        while accumulator >= TICK_PERIOD {
            world.tick(&intents);
            intents.drop = false;
            accumulator -= TICK_PERIOD;
        }

        canvas.begin();
        draw_scene(&assets, &world);
        canvas.end();
        canvas.present(theme::BARS);
        next_frame().await;
    }
}

/// Draw one frame of the world in logical canvas coordinates.
fn draw_scene(assets: &Assets, world: &World) {
    clear_background(theme::SKY);

    draw::sprite(
        assets,
        Sprite::Scorebox,
        0.0,
        SCOREBOX_TOP as f32,
        theme::INK,
    );
    draw::sprite(
        assets,
        world.wagon.sprite(),
        world.wagon.x as f32,
        (GROUND_Y - WAGON_H) as f32,
        theme::INK,
    );
    draw::sprite(
        assets,
        world.copter.sprite(),
        world.copter.x as f32,
        world.copter.y as f32,
        theme::INK,
    );
    draw_stuntman(assets, world);
    hud::draw(assets, world);
}

/// Draw the stuntman for the current phase.
fn draw_stuntman(assets: &Assets, world: &World) {
    let ink = theme::INK;
    let wagon_x = world.wagon.x as f32;
    let wagon_y = (GROUND_Y - WAGON_H) as f32;
    match &world.stuntman {
        Stuntman::Hanging => {
            let (x, y) = world.hang_pos();
            draw::sprite(assets, Sprite::ManHang, x as f32, y as f32, ink);
        }
        Stuntman::Falling(faller) => {
            draw::sprite(
                assets,
                faller.sprite(),
                faller.x as f32,
                faller.y as f32,
                ink,
            );
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
