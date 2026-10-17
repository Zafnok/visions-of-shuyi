# ADR-0081: The platform shell is the engine's; the game gives an `AppConfig`

- **Status:** Accepted
- **Date:** 2026-10-17
- **Related tickets:** 1110
- **Amends:** ADR-0004 (where the platform code lives: `engine/crates/app`,
  `shuyi-app`, as well as the game's binary)

## Context

`crates/app` was almost all platform (`docs/engine/boundary.md`, *app*):
the window, blitting, the keyboard, controllers, sounds, music and voices,
storage and the main loop. What tied it to this game was small: the
window's title and icon, the storage's name `tactical-rpg`, the asset
bundle and the font and images in `Content`, the sound and voice folders,
`Ctx::embedded`, the start screen and its command line, the release check
on the test files and the exe's resource strings.

Two constraints shape the split:

- `#[macroquad::main]` has to be on the binary's `main`, and its
  `window_conf` is a plain function.
- The orphan rule: a trait of `shuyi-app` can't be implemented for the
  game's `Shuyi` outside `trpg-ui` (which must not depend on macroquad), so
  the shell can't ask the game's data through a trait of its own.

## Decision

1. **`shuyi-app`** (`engine/crates/app`) holds the renderer, the keyboard,
   the controllers (gilrs; the web's `web/gamepad.js`, renamed to the
   `shuyi_gamepad` plugin with `shuyi_pad_*` functions), the audio player,
   storage and the loop, unchanged in behavior. Like the Playdate's, its
   lints copy the workspace's with `unsafe_code = "deny"` (the web pads'
   imports); the game's binary now inherits the workspace's `forbid`.
2. **The game's binary keeps the entry point** and hands over:
   - `shuyi_app::window_conf(&Window)` for `#[macroquad::main(...)]`:
     title, size and the icon as three PNGs;
   - `shuyi_app::run(AppConfig<G>, make).await`, where `AppConfig` says
     the storage's name, the embedded files by path (`fn(&str) ->
     Option<&'static [u8]>`), where the font atlas and image table are in
     the game's data (`fn(&G) -> &…`) and the atlas picture's path, the
     sound files (the engine's audio `Files`, which also names the music
     folder) and the voice folder, language and manifest (`VoiceFiles`,
     `None` for no voices).
3. **`make: FnOnce(&Platform) -> Result<Game<G>, String>`** builds the
   game. The game makes its context and sets what is its own (the console,
   tips, lessons), then `Platform::setup(ctx)` adds the platform's part:
   the storage, the voices the manifest allows, idle motion, a waiting key
   prompt and a music seed from the clock. An `Err` (a wrong command line,
   a broken asset) is shown instead of the game: this is where a game runs
   its checks at start.
4. **The storage's name** is the native folder in the OS's data folder and
   the web prefix (`<name>/`); the game's stays `tactical-rpg` so every
   save is found, and a test says so.
5. **The console's background** is that of the game's blank cell
   (`GameData::blank_cell`), which for this game is its black, as before.
6. **The game's release check on test files stays a compile-time
   assertion** in the binary, rather than a check at start: it stops a
   wrong build from existing at all.
7. The web build's `--allow-undefined` stays in the game's `build.rs`: a
   link argument only reaches a binary from its own package's build
   script.

## Consequences

- A second game's binary is the entry point, an `AppConfig` and a `make`:
  under 100 lines here.
- `Game::frame` stays the platform-neutral heart that the Playdate front
  end shares (ADR-0070 §6); the Playdate keeps its own loop.
- Coverage, mutation testing and Sonar leave `engine/crates/app` out as
  they did `crates/app`: it drives a window, a sound device and
  controllers.
- The voice manifest is read before `make`, so a game reads its command
  line after it; only the order of two log lines changes.
- The JS plugin's version went to 3 with the rename; a page with an old
  `gamepad.js` and a new wasm says so in the console.

## Alternatives considered

- **A trait in `shuyi-app` the game implements** for its font, images and
  stick settings: the orphan rule forbids it outside `trpg-ui`, and
  `trpg-ui` must not depend on a platform crate.
- **New `GameData` hooks** for the font and images: they are the
  platform's needs (the Playdate draws with a one-bit atlas instead), not
  the screens'.
- **`AppConfig` holding a loaded `FontAtlasDef` and `ImageTable`**: the
  game would load and check them twice, once for its `Content` and once
  for the shell.
- **The release check as a function run at start**: weaker than refusing
  to build.
