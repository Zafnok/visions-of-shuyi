---
id: "1110"
title: "shuyi-app: the platform shell and run loop; trpg-app becomes a small binary that picks the game"
type: infra
milestone: "Engine: carve"
model: opus-5.5
effort: high
status: done
blocked_by: ["1109"]
nick_input: none
completed: 2026-10-17
---

# 1110 — shuyi-app: the platform shell and run loop

## Context

`crates/app` is already almost all platform (`docs/engine/boundary.md`,
*app*): rendering, keys, controllers (gilrs, the web pad plugin),
storage (files, `localStorage`), audio (sounds, music decoded on a worker
thread, voice), and the main loop. Its game leaks: the binary name, the
window title and icon, exe resource strings, storage names
`tactical-rpg` (`storage/native.rs:15`, `storage/web.rs:11`),
`Ctx::embedded`, the start screen, the release check on test files, and
`args.rs` (`--scene`, `--lead`).

## Nick input

None. The game must look, sound and save exactly as before.

## Scope

**In:**
- New crate `engine/crates/app` (package `shuyi-app`) with
  `pub fn run<G, A>(config: AppConfig, make: impl FnOnce(&Platform) ->
  shuyi_ui::Game<G, A>)`. `AppConfig`: window title, size, icon bytes,
  storage name (native folder and web prefix), the font atlas, extra
  checks to run at start (the game's release check).
- Move `render.rs`, `keys.rs`, `pads*`, `storage/`, `audio*` and the loop
  from `main.rs`. The web JS plugin names `trpg_pad_*` become
  `shuyi_pad_*` (update `web/` and `xtask web` in the same PR).
- `trpg-app/src/main.rs` keeps the `#[macroquad::main]` entry (macroquad
  needs it in the binary), builds `AppConfig` and calls `run`. `args.rs`
  stays the game's.
- The storage name stays `tactical-rpg` for this game so saves are
  found (test: the path is unchanged).

**Playdate:** if a Playdate front end exists by then (`crates/playdate`,
1031 and 1033 of 1025's build tickets, Nick 2026-10-15), it must keep
building (`cargo xtask playdate`, the CI `playdate` job). ADR-0070 §6
keeps it a **second front end beside `shuyi-app`** (no `std`, no threads,
a C entry point and a callback loop): what the two share is
`Game::frame` (input events and `dt` in, a frame out), so keep that
platform-neutral, and keep the "no keyboard" start setting 1033 adds.
Say in the completion notes if anything else turned out shareable.

**Out (do not do):** new platform features (mouse, touch: 1121); a
headless runner (the Harness covers it).

## Implementation steps

1. Create the crate; move modules; parameterize the leaks.
2. Keep `#[macroquad::main]` in the binary; the engine's `run` is
   `async` and called from it.
3. Native and web builds; play the title, a battle and a save/load by
   hand, and through the browser pane on the web build.

## Acceptance criteria

- [x] `trpg-app/src/main.rs` is under 120 lines.
- [x] Saves written before the change load after it, native and web
      (same folder and prefix).
- [x] `cargo build -p trpg-app --target wasm32-unknown-unknown` and the
      native build pass; the web build runs in the browser pane.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: storage path and prefix from `AppConfig`.
- Manual: native and web smoke test (listed in the PR).

## Completion notes

Done (Opus 5.5, high, as the ticket names).

- **`shuyi-app`** (`engine/crates/app`): `render.rs`, `keys.rs`, `pads*`,
  `storage/` and `audio*` moved with `git mv`, unchanged in behavior; the
  loop, `window_conf`, the icon decoding and the error screen came out of
  `main.rs`. `run(config, make)` is `async` and called from the game's
  `#[macroquad::main]`. ADR-0081 records the shape.
- **Deviations from the ticket's sketch:**
  - `run<G>` has one type parameter: since 1109 `Game<G>` takes its
    action type from `G`.
  - `make` returns `Result<Game<G>, String>` (an `Err` is shown instead
    of the game: a wrong command line, a broken asset) and takes
    `&Platform`, whose `setup(ctx)` adds the storage, the voices, idle
    motion, the key prompt and the music seed.
  - `AppConfig` is generic over the game's data: the font atlas and the
    image table are read from it with `fn(&G) -> &…` (the orphan rule
    rules out a trait the game would implement), the atlas picture by
    path with the game's file reader. The sound files are the engine's
    audio `Files` (which names the music folder), the voices a
    `VoiceFiles` (folder, language, manifest).
  - "Extra checks to run at start": the game's release check on test
    files stays a compile-time assertion in `main.rs` (stronger than a
    check at start); `make`'s `Err` is where a game runs checks at
    start.
  - The console's background is now "the blank cell's background"
    (`GameData::blank_cell`), which for this game is its black, as before.
  - `--allow-undefined` for the web stays in the game's `build.rs`: a link
    argument only reaches the binary from its own build script. Noted
    in 1117, which moves `xtask web` and is the time to give the engine
    its own web shell.
  - The JS plugin is `shuyi_gamepad` with `shuyi_pad_*` functions; its
    version went from 2 to 3 with the rename.
  - `trpg-app` now inherits the workspace lints (`unsafe_code = forbid`):
    its only `unsafe` (the web pads) moved. `shuyi-app` copies them with
    `deny`, as `crates/app` did; the xtask test follows.
  - Coverage, mutation testing and Sonar leave `engine/crates/app` out,
    as they always left `crates/app` out (a window, a sound device and
    controllers no test has).
  - Two hand-run measurements in `native_music.rs` now take their files
    from `MUSIC_DIR` / `VOICE_CLIP` instead of the game's folders.
- **Tests:** engine: the storage folder and web prefix from a name, the
  error text wrapping, a broken icon, the voice player on a test variant,
  the manifest read (55 tests). Game (`crates/app/src/tests.rs`): saves
  stay in `%APPDATA%/tactical-rpg` and under `tactical-rpg/`, the window
  has its icon, the test voice folder still gives the game its three
  clips, the config reads the content's font and images.
- **Manual smoke test.** Native (debug): starts, the storage smoke check
  passes against the existing `%APPDATA%/tactical-rpg`, runs 20 s
  without a problem; `--bogus` shows the error screen with the usage line.
  Web (release + debug tools, browser pane): the title and its key
  prompt, the layout picker on the first key, New Game → lead → the
  opening dialogue → the first battle under control (the cursor lesson
  completed), the map menu's Suspend wrote `tactical-rpg/p1-suspend`; after
  a page reload the title skipped the layout picker and offered
  Continue, which resumed the battle. Every key was under `tactical-rpg/`.
  The voice manifest was fetched from `voice/en/voice.ron` as before.
- **Playdate:** unchanged and still builds (its clippy gates). What it
  shares with `shuyi-app` is `Game::frame`, as ADR-0070 §6 says. Turned
  out shareable later, not done here: the audio player behind its
  `Backend` trait (the music fades, voice preloading and late-clip rules)
  needs only `alloc` collections, so a Playdate with sound could reuse
  it.
- **Gameplay rules decided:** none.
- **Follow-ups:** none new; a note added to 1117 (above).
