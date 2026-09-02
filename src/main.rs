#![doc = include_str!("../README.md")]

mod assets;
mod atlas;
mod canvas;
mod config;
mod copter;
mod draw;
mod input;
mod theme;
mod wagon;
mod world;

use assets::Assets;
use atlas::Sprite;
use canvas::Canvas;
use config::{GROUND_Y, SCOREBOX_TOP, TICK_PERIOD, WAGON_H};
use macroquad::prelude::*;
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
