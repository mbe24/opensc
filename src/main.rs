#![doc = include_str!("../README.md")]

mod assets;
mod audio;
mod canvas;
mod config;
#[cfg(feature = "debug-controls")]
mod debug;
mod draw;
mod font;
mod hud;
mod input;
mod screen;
mod theme;

use std::fmt::Write;

use assets::Assets;
use audio::{Ambient, Audio};
use canvas::Canvas;
use config::{GROUND_Y, MAN_H, TICK_PERIOD, WAGON_H, WAGON_W};
use macroquad::prelude::*;
use stuntcopter_sim::stuntman::{Splat, Stuntman, Wreck};
use stuntcopter_sim::{Level, RenderState, Sprite, World};

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
    // Handle touches directly (see `input`): a left-half joystick steers and a
    // right-half tap drops. So don't also synthesize mouse events from touch.
    simulate_mouse_with_touch(false);

    let assets = Assets::load();
    let mut audio = Audio::load().await;
    let canvas = Canvas::new();
    let mut world = World::default();
    // A per-session seed so wind isn't identical every run (tests keep the fixed
    // default seed). `date::now()` is available on native and web alike.
    world.reseed((miniquad::date::now() * 1000.0) as u64);
    let mut prev = world.render_state();
    let mut scene = Scene::Attract;
    let mut accumulator = 0.0f32;
    // The drop edge can land on a render frame that runs no sim tick (half of
    // them at 60 Hz over a 30 Hz sim); latch it here so no press is ever lost,
    // and clear it once a tick or a scene change consumes it.
    let mut drop_pending = false;
    let mut input = input::Input::default();
    // Collect each frame's simulation events; audio reacts to them, and the debug
    // build also displays them.
    let mut sink = stuntcopter_sim::EventLog::default();
    #[cfg(feature = "debug-controls")]
    let mut debug = debug::Debug::default();

    loop {
        // Clamp the frame delta so a backgrounded tab can't make the accumulator
        // spiral into hundreds of catch-up ticks on return.
        accumulator += get_frame_time().min(config::MAX_FRAME_TIME);

        #[cfg(feature = "debug-controls")]
        debug.update(&mut world);
        #[cfg(feature = "debug-controls")]
        let mouse_steer = debug.mouse_steer();
        #[cfg(not(feature = "debug-controls"))]
        let mouse_steer = true;
        sink.clear();

        let mut intents = input.gather(&canvas, mouse_steer);
        drop_pending |= intents.drop;
        // On the menu screens (attract/paused/game-over) a tap anywhere confirms,
        // so mobile players don't have to find the drop half to press BEGIN.
        let confirm = drop_pending || screen::any_tap() || is_key_pressed(KeyCode::Enter);
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
                    drop_pending = false; // don't drop the first man on the start click
                }
            }
            Scene::Playing => {
                while accumulator >= TICK_PERIOD {
                    prev = world.render_state();
                    intents.drop = drop_pending;
                    world.tick(&intents, &mut sink);
                    drop_pending = false; // consumed by exactly one tick
                    accumulator -= TICK_PERIOD;
                    if world.over {
                        scene = Scene::GameOver;
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
                    drop_pending = false; // the resume click is not a drop
                } else if exit {
                    scene = Scene::Attract;
                }
            }
            Scene::GameOver => {
                // The final board stays frozen until the player starts anew.
                accumulator = 0.0;
                if confirm {
                    world.begin();
                    prev = world.render_state();
                    scene = Scene::Playing;
                    drop_pending = false;
                }
            }
        }

        // React to the frame's events (one-shots) and state (the drone loop).
        audio.handle_events(sink.events(), world.progression.level);
        audio.set_ambient(Ambient::Copter, matches!(scene, Scene::Playing));
        #[cfg(feature = "debug-controls")]
        debug.record(sink.events());

        // Interpolate render positions between the last two ticks so motion is
        // smooth at the display's refresh rate, not just the 30 Hz sim rate.
        let alpha = (accumulator / TICK_PERIOD).clamp(0.0, 1.0);

        canvas.begin();
        draw_frame(&assets, &world, &scene, prev, alpha, input.touch_seen());
        #[cfg(feature = "debug-controls")]
        debug.draw_hint(&assets, &world);
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
    GameOver,
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

/// Draw the whole scene for one frame: the world, then the scene's overlay
/// (title, pause, game-over, or the level banner).
fn draw_frame(
    assets: &Assets,
    world: &World,
    scene: &Scene,
    prev: RenderState,
    alpha: f32,
    touch: bool,
) {
    draw_scene(
        assets,
        world,
        prev,
        alpha,
        !matches!(scene, Scene::Attract),
        matches!(scene, Scene::Playing),
    );
    match scene {
        Scene::Attract => draw_attract(assets, touch),
        Scene::Paused => draw_paused(assets),
        Scene::GameOver => draw_game_over(assets),
        Scene::Playing => {
            if world.level_banner > 0 {
                draw_level_banner(assets, world.progression.level);
            }
        }
    }
}

/// Draw one frame's world, interpolating positions between ticks for smooth
/// motion. `show_man` is false on the attract screen, where no stuntman is
/// present; `playing` gates the live yoke crosshair (gray-covered otherwise).
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
        lerp(prev.cloud.x, cur.cloud.x, alpha),
        lerp(prev.cloud.y, cur.cloud.y, alpha),
    );
    let wagon_x = lerp(prev.wagon_x, cur.wagon_x, alpha);
    let copter = (
        lerp(prev.copter.x, cur.copter.x, alpha),
        lerp(prev.copter.y, cur.copter.y, alpha),
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
/// over the idle scene, matching the original's wait-for-BEGIN layout. On a touch
/// device it also shows a one-line control hint.
fn draw_attract(assets: &Assets, touch_seen: bool) {
    text_center(assets, "StuntCopter", 44.0, 2.0);
    text_center(assets, "by Duane Blehm", 74.0, 1.0);
    button(assets, "BEGIN", 165.0);
    if touch_seen {
        text_center(assets, "drag left to fly     tap right to drop", 232.0, 1.0);
    }
}

/// The game-over screen: the frozen final board with a title and a fresh BEGIN.
fn draw_game_over(assets: &Assets) {
    text_center(assets, "GAME OVER", 110.0, 2.0);
    button(assets, "BEGIN", 165.0);
}

/// The pause overlay.
fn draw_paused(assets: &Assets) {
    text_center(assets, "PAUSED", 110.0, 2.0);
    text_center(assets, "SPACE resumes   ESC ends", 140.0, 1.0);
}

/// The "LEVEL n" banner shown briefly after clearing a level — the original's
/// LevelButton, drawn at its END/LEVEL slot (v=200).
fn draw_level_banner(assets: &Assets, level: Level) {
    let mut label = draw::StackStr::<16>::new();
    let _ = write!(label, "LEVEL {level}");
    button(assets, label.as_str(), 200.0);
}

/// A classic-Mac rounded push button, 80x26, horizontally centered with its top
/// at `top`, labeled `label`. Matches `CreateWindow`'s `SizeControl(_, 80, 26)`.
fn button(assets: &Assets, label: &str, top: f32) {
    let (bw, bh) = (80.0, 26.0);
    let bx = (config::LOGICAL_W as f32 - bw) / 2.0;
    let r = 8.0;
    draw::round_rect(bx, top, bw, bh, r, theme::INK);
    draw::round_rect(bx + 1.0, top + 1.0, bw - 2.0, bh - 2.0, r - 1.0, theme::SKY);
    let tw = draw::text_width(label) as f32;
    draw::text(
        assets,
        label,
        bx + (bw - tw) / 2.0,
        top + (bh - font::CELL_H) / 2.0,
        theme::INK,
    );
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
            let x = copter.0 + config::MAN_HANG_OFFSET.dh as f32;
            let y = copter.1 + config::MAN_HANG_OFFSET.dv as f32;
            draw::sprite(assets, Sprite::ManHang, x, y, ink);
        }
        Stuntman::Falling(faller) => {
            let (x, y) = match prev.faller {
                Some(p) => (
                    lerp(p.x, faller.pos.x, alpha),
                    lerp(p.y, faller.pos.y, alpha),
                ),
                None => (faller.pos.x as f32, faller.pos.y as f32),
            };
            draw::sprite(assets, faller.sprite(), x, y, ink);
        }
        // The rescued man rides in the wagon; the backflip itself plays in the
        // HUD flip-boxes (see `hud`).
        Stuntman::Celebrating(_) => {
            draw::sprite(assets, Sprite::ManInWagon, wagon_x, wagon_y, ink);
        }
        Stuntman::Crashing(splat) => draw_crash(assets, splat, wagon_x, wagon_y),
    }
}

/// Draw a failed drop: the crumple frames at the point of impact, then the wreck
/// left behind (a dead driver or horse against the wagon, or nothing on a clean
/// miss).
fn draw_crash(assets: &Assets, splat: &Splat, wagon_x: f32, wagon_y: f32) {
    let ink = theme::INK;
    if let Some(frame) = splat.pose() {
        // At ground level for a clean miss; a touch higher when he strikes the
        // driver or horse riding on the wagon.
        let y = match splat.wreck {
            Wreck::Ground => GROUND_Y - MAN_H,
            Wreck::Driver | Wreck::Horse => GROUND_Y - 13 - MAN_H,
        } as f32;
        draw::sprite(assets, frame, splat.x as f32, y, ink);
        return;
    }
    let wreck = match splat.wreck {
        Wreck::Ground => return,
        Wreck::Driver => Sprite::Driver,
        Wreck::Horse => Sprite::HorseDead,
    };
    // Right-aligned to the wagon, where the driver and horse ride.
    let x = wagon_x + (WAGON_W as f32 - wreck.rect().w);
    draw::sprite(assets, wreck, x, wagon_y, ink);
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
