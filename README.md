# OpenSC

[![CI](https://github.com/mbe24/opensc/actions/workflows/ci.yml/badge.svg)](https://github.com/mbe24/opensc/actions/workflows/ci.yml)
[![Live demo](https://img.shields.io/badge/demo-GitHub%20Pages-89b4fa.svg)](https://mbe24.github.io/opensc/)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-orange.svg)](#license)
![Rust](https://img.shields.io/badge/Rust-dea584.svg)
![macroquad](https://img.shields.io/badge/macroquad-0.4-f74c00.svg)
![WebAssembly](https://img.shields.io/badge/WebAssembly-654ff0.svg)

**OpenSC** is a faithful, modern Rust port of Duane Blehm's 1986 Macintosh game
*StuntCopter* — fly the helicopter, drop the stuntman, and land him in the moving
hay wagon far below. It runs **natively** on Windows, macOS, and Linux, and in
the **browser** via WebAssembly.

It's a preservation port done properly: the rules and feel are reconstructed
from the original Pascal source and its art from the original Mac resources, but
the code is modern, type-safe Rust with a clean separation between the game and
how it's shown — small enough to read, faithful enough to trust.

- 🎮 **Faithful gameplay** — proportional inertia flight, land all five to
  advance, gravity that ramps with the level, the back-flip celebration, and the
  splat / hit-the-driver / hit-the-horse fates — all matched to the original.
- 🧩 **Clean sim / UI split** — the whole game lives in a `sim` crate with **zero
  macroquad dependency**: deterministic, headless-testable, and swappable behind
  any renderer. The `src` layer is the thin macroquad presentation.
- 🔊 **Procedural sound** — the ~10 Hz rotor drone, the sawtooth splat, and the
  building four-note fanfare are **synthesized at runtime** (no audio files),
  played in reaction to simulation events.
- 🖼️ **Authentic art** — sprites are decoded straight from the original
  QuickDraw **PICT resources**, drawn through an integer-scaled virtual canvas
  that stays crisp from a 4K monitor to a phone.
- 🌐 **Runs everywhere** — one codebase, native **and** WebAssembly (no
  wasm-bindgen); the web build is a self-contained static `dist/`.
- 🛠️ **Deterministic test harness** — a seedable RNG, a domain event log, and
  feature-gated debug controls (pin the wagon/copter, watch the event stream) so
  behaviour is verifiable without catching animation frames.

**▶ Live demo: <https://mbe24.github.io/opensc/>** (drag/point to steer, click or
Space to drop; on mobile, drag and tap).

## How it works

The game is a **pure simulation** advanced at a fixed 30 Hz. Rendering reads the
world **state** (interpolated between ticks); **events** are a separate, optional
side-channel that only drives audio and the debug log — the renderer never
consumes them:

```text
 devices ──▶ Intents ──▶ World::tick (30 Hz) ──▶ world state ──▶ render (interpolated)
 (mouse/kbd/touch)                │
                                  └─▶ events ──┬─▶ audio (drone + one-shots)
                                               └─▶ debug event log
```

- The **sim** (`sim/`) owns all state and rules. Its `tick` mutates state (what
  the renderer draws) and, as a side-channel, reports typed **events**
  (`Dropped`, `CelebrationStarted`, `Resolved`, `LevelCleared`, …) to a caller-
  supplied `EventSink`. It knows nothing about rendering, input, timing, or audio.
- The **presentation** (`src/`) runs the loop: a clamped fixed-timestep
  accumulator with **render interpolation** for smooth motion; the renderer reads
  `render_state()` plus the public world fields. Audio and the debug overlay are
  the only consumers of the event sink — pass `NoSink` and the sim does no event
  work at all.
- Because the sim is platform-free, the same logic is unit-tested headlessly and
  could be dropped behind a different renderer without change.

## Repository layout

```text
opensc/
├── sim/          # platform-agnostic game simulation — no macroquad
│   └── src/      #   world, level, copter, wagon, cloud, stuntman, rng, event, sprite
├── src/          # macroquad presentation — main loop, draw, hud, canvas, input, audio
│   └── audio/    #   pure waveform synthesis (synth) + the macroquad player
├── assets/       # generated sprite atlas + bitmap font (embedded at build time)
├── reference/    # Duane Blehm's original 1986 source & resources (public domain — see its README)
├── scripts/      # asset generation (PICT decode) and the Node web build/serve
└── web/          # the WebAssembly page shell + the vendored macroquad loader
```

## Build & run

Native:

```sh
cargo run
```

Web — builds a self-contained static `dist/` (serve over http, not `file://`):

```sh
node scripts/build-web.mjs
node scripts/serve.mjs        # then open http://localhost:8099
```

Or with Docker:

```sh
docker build -t opensc .
docker run --rm -p 8080:80 opensc   # http://localhost:8080
```

## Development

The workspace splits cleanly, so most work is fast and headless:

```sh
cargo test --workspace                 # sim logic + synth (no window needed)
cargo clippy --workspace --all-targets # lint gate (clippy::all denied)
cargo check --target wasm32-unknown-unknown
```

The in-game debug/test controls compile only behind a feature, so they never
ship in a normal build:

```sh
FEATURES=debug-controls node scripts/build-web.mjs
```

`M` toggles mouse steering, `K`/`P` pin the wagon/copter (arrow keys nudge the
pinned copter), and `L` lines up a guaranteed landing. CI runs the format,
clippy, test, and wasm checks on every push.

## Credits & attribution

*StuntCopter* was created by **Duane Blehm** (1986–1987). After his death, his
family released his games into the **public domain**. `reference/` holds his
original source and resources, kept verbatim as this port's source of truth.

## License

The port — everything **except** `reference/` — is licensed under either of
[MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at your option, © Mikael
Beyene.

`reference/` is Duane Blehm's public-domain work and is not covered by that
license (see [`reference/README.md`](reference/README.md)).
