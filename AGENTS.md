# opensc

opensc is a faithful, multiplatform port of Duane Blehm's 1986 Macintosh game
*StuntCopter*, built in Rust on [macroquad](https://macroquad.rs). A single
binary crate (`stuntcopter`) runs natively on Windows/macOS/Linux and compiles
to WebAssembly for the browser (hosted on GitHub Pages).

## Project status — PRE-RELEASE, ZERO USERS

opensc has **not shipped**. There are **no users and no saved data in the wild**
(the only persisted state is a local high score). Design and review accordingly:

- **No data migration is ever required.** There is no existing state to preserve.
  "Migrate existing saves", "coexistence with the old format", and "don't break
  existing data" are **non-issues** — do not raise them as blockers.
- **Breaking changes are free.** Save/high-score storage, the sprite atlas layout,
  asset formats, and module/crate boundaries may change without back-compat,
  deprecation, or a migration path, whenever there's a clear reason. Prefer the
  clean design over a compatible one.
- A clean breaking cutover is the default answer to "old vs new"; dual-path /
  back-compat machinery is almost never warranted pre-release.

(When this changes — first real release — update this section.)

## Faithfulness

This is a *port*, not a reimagining. The original Pascal source and decoded
assets live in `reference/`; the distilled ruleset and the constants in
`config.rs` are the port's source of truth. When a mechanic is in question,
check it against `reference/stuntcopter/StuntCopter.pas` rather than guessing.
Deliberate departures from the original (controls, resolution independence,
pacing) are fine, but call them out as departures.

## Commits

Use Conventional Commits with a scope on the component you touched. Imperative
mood, lowercase start, no trailing period.

- Structure: `type(scope): summary`
- Types: `feat`, `fix`, `chore`, `build`, `docs`, `refactor`, `test`, `perf`
- Scope: the component, e.g. `game`, `render`, `input`, `hud`, `assets`,
  `audio`, `config`, `web`, `ci`, `docs`
- Example: `fix(render): flip the virtual canvas so the scene isn't upside-down`
- Do NOT append a `Co-Authored-By:` / agent-attribution trailer (or any
  Claude/session line) to commit messages.

## Validation before committing

This repo builds cargo natively — no WSL2/Docker needed. After a series of
commits, run:

- `cargo fmt --check`
- `cargo clippy --all-targets`
- `cargo test`
- `cargo check --target wasm32-unknown-unknown` — keep the web target green; it
  is easy to break with native-only code (filesystem, threads, `Instant`).

## Code style

Prefer functions under ~100 lines. A longer one should be split into named
phases or helper functions — unless the logic is irreducibly coupled (shared
mutable state that can't be cleanly threaded across a call boundary), in which
case say why in a brief comment. This is a guideline, not a hard gate:
readability matters more than the exact line count.

Reach for the type system to make errors impossible rather than merely caught:
newtype domain values that would otherwise be interchangeable primitives, so a
wrong-argument slip is a compile error — a `Tick` is not a pixel count, a
requested velocity is not a position, a `Sprite` is not a raw atlas `Rect`.
Introduce a newtype when a real mix-up is *possible*, not speculatively; don't
carry abstraction ahead of a second consumer that justifies it.

Game state is an integer simulation (pixels-per-tick, matching the original);
cast to `f32` only at the drawing boundary. The clippy `cast_*` lints are
relaxed in `Cargo.toml` for exactly that boundary — everywhere else, hold the
code to a `clippy::pedantic`-clean bar even though pedantic is only a warning.