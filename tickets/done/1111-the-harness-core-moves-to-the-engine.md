---
id: "1111"
title: "The Harness core (key and pad scripts, waits, audio capture, snapshots) moves to the engine"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: high
status: done
blocked_by: ["1110"]
nick_input: none
completed: 2026-10-17
---

# 1111 — The Harness core moves to the engine

## Context

The Harness (`ui/src/harness.rs`, ADR-0017) is what lets tests and
agents drive the game headlessly. About 750 of its 1,124 lines are
generic (key, pad and wait scripts in 1/60 s frames, audio capture, the
music simulation, `top_screen`, `snapshot`); the rest is wired to this
game (`flow()`, `battle()`, `map_scene()`, `cursor_tile()`, the test
chapter, `harness.rs:20-28,542`). `docs/engine/boundary.md`, coupling 10.

## Nick input

None.

## Scope

**In:**
- `shuyi_ui::harness::Harness<G, A>` wrapping `shuyi_ui::Game<G, A>`
  with the generic API, behind a `harness` feature (as today, ADR-0017).
- `trpg-ui`'s `Harness` becomes a thin wrapper adding `flow()`,
  `battle()`, `map_scene()`, `cursor_tile()` and the test chapter start.
- The engine's E-class tests in `ui/tests/it` that only need the generic
  Harness (key bindings, controller, rebind buttons, layout picker,
  sprite test; `docs/engine/boundary.md`, *ui*) are rewritten against a
  toy game and move; the game keeps a copy only where it tests its own
  screens.
- `frame-png`'s use of the Harness goes through the generic part.

**Playdate:** the Playdate port (1025's tickets, 1027–1053) builds
`core`, `content` and `ui` without `std` and adds a second front end,
`crates/playdate`. If those tickets have landed, code this ticket moves
keeps its default `std` feature and stays in the `no-std` CI build, and
`cargo xtask playdate` keeps building (ADR-0070 §6).

**Out (do not do):** the CLI contract (1129); MCP (1133).

## Implementation steps

1. Split `harness.rs`; wrap.
2. Write the toy game used by engine tests in
   `engine/crates/ui/tests/toy/` (two screens, a menu, a sprite).
3. Move the tests that fit.

## Acceptance criteria

- [x] `cargo test -p shuyi-ui` runs Harness tests against the toy game.
- [x] Every game Harness test passes unchanged.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the toy game's Harness tests.

## Completion notes

- **`shuyi_ui::harness::Harness<G: GameData>`** (the `harness` feature)
  wraps `Game<G>`: `keys`, `type_text`, `hold`, `pad` (with `+` chords),
  `hold_pad`, `wait`, `frame` (one frame of raw events, now public), the
  audio recorded (`audio_requests`, `sounds`, `voices`,
  `last_frame_audio`, `music_commands`, `clear_audio`), what the last
  frame told the platform (`volumes`, `fullscreen`), the music player's
  stand-in (`music_load_delay`, `without_music`, `music_clock`),
  `use_pad`, `top_screen`, `screens`, `screen::<T>`/`screen_mut`,
  `snapshot`, `snapshot_as`, `at_prompt`, `ctx`/`ctx_mut`, `game`,
  `into_storage`. `FRAME_DT` and `MAX_FRAMES` are the engine's; the
  game re-exports them.
- **`trpg_ui::harness::Harness`** is a newtype over it (`Core`) that
  derefs to it, keeping its constructors (`new`, `with_layout(_and)`,
  `at_prompt*`, `with_storage`, `with_screen`, `from_game`,
  `with_shipped_story`, the test chapter) and lookups (`flow`,
  `battle`, `map_scene`, `map_text`, `cursor_tile`, `unit_at`,
  `tints_at`, `path`, `with_map_skin`, `with_tips`, `with_lessons`).
  The scripts are forwarded rather than reached through `Deref`, because
  tests chain them with the game's methods
  (`h.use_pad(kind).with_tips()`) and pass arguments that read the
  Harness (`h.wait(2.25 - time(&h))`), which a `DerefMut` call can't
  borrow. No game test changed; `frame-png` and the Playdate test use it
  as before, through the generic part for the scripts and the frame.
- **The toy game** is the one 1109 wrote for the unit tests
  (`engine/crates/ui/src/toy.rs`), now public behind a `toy` feature (as
  `shuyi_assets::keymap::sample` is behind `sample-actions`), with what
  the Harness tests need: one image (`card.png`), a layout `Picker` the
  first key opens, a `Gallery` debug tool the Debug key opens (two
  sprites, one clipped, and a help line naming the Cancel key), `hooks`,
  `game`, `game_on` and `launch` (a later launch from a storage).
- **Engine tests** (`engine/crates/ui/tests/it`, 42 tests, one program):
  `scripts` (frames, typing, repeat, pad chords, waits, panics, screens,
  quit, the prompt, the storage across launches), `audio` (requests,
  sounds, voices, volumes, the music clock, its load delay, silence and a
  switch of track), `key_bindings` (rebinding at once, fixed Escape,
  per-layout keys across a restart), `controller` (buttons act, the
  device and pad kind, help naming the button, `snapshot_as`),
  `layout_picker` (first key, no keyboard, either hand, can't cancel,
  kept) and `sprites` (the Debug key, one snapshot line per sprite).
  The music player's stand-in has its unit tests in `src/harness/tests.rs`.
- **Deviations from the ticket:**
  - `Harness<G>`, not `Harness<G, A>`: since 1109 `Game<G>` takes the
    action type from `G::Action`.
  - The toy game is in `src/toy.rs` behind a feature, not in
    `tests/toy/`: 1109 had already written it there for the unit tests,
    and one toy for both kinds of test beats two copies.
  - No game test moved. The E tests named (key bindings, controller,
    rebind buttons, layout picker, sprite test) all test this game's
    screens (its title, layout picker, Key bindings screen and debug
    tools), so the game keeps them and the acceptance criterion keeps
    them unchanged; what they check of the engine is now also checked on
    the toy. Of the game's own Harness unit tests, the generic ones
    (scripts, audio, the music player's stand-in) moved to the engine;
    those of the title's music clock, the map scene and the start-up
    stayed.
- **Playdate:** the engine's `harness` and `toy` modules use only `alloc`
  and `core`; the `no_std` builds and `cargo xtask playdate`'s crates
  build as before (the Harness is never in them).
- No gameplay rules decided; nothing changes for the player.
