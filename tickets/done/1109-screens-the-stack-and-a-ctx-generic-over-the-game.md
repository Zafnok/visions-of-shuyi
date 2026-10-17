---
id: "1109"
title: "Screens, the screen stack and the frame driver move to the engine, with a Ctx generic over the game"
type: infra
milestone: "Engine: carve"
model: opus-5.5
effort: xhigh
status: done
blocked_by: ["1108"]
nick_input: none
completed: 2026-10-17
---

# 1109 — Screens, the stack, the driver; `Ctx<G>`

## Context

`docs/engine/boundary.md`, couplings 2 and 3. `Screen`, `Transition`,
`FrameInput` and `ScreenStack` (`ui/src/screen.rs`, ADR-0017) are
generic in shape, but every screen takes a `Ctx` that holds this game's
`Content`, campaign mode, map skin and lead (`screen.rs:224-327`), and
`Game` (`ui/src/game.rs`) builds this game's title screen, opens the
layout picker on the first key and the debug menu on `Action::Debug`.

ADR-0063 §6: game code keeps its names through aliases, so 40,000 lines
of game screens change field paths, not signatures.

## Nick input

None.

## Scope

**In:**
- `shuyi_ui::Ctx<G>`: the engine's services (text by key and language,
  keys and help names, settings the engine owns, the audio queue and
  music clock, voice, storage) plus `pub game: G`. `trpg-ui` defines
  `pub struct Shuyi { content, campaign_mode, mode_switch, map_skin,
  lead, … }` and `pub type Ctx = shuyi_ui::Ctx<Shuyi>`; it implements
  `Deref`-free accessors only where many call sites need them.
- `shuyi_ui::screen`: `Screen<G, A>` (or `Screen<C>` with `C = Ctx<G>`),
  `Transition`, `FrameInput<A>`, `ScreenStack`.
- `shuyi_ui::Game<G, A>`: the frame loop and `RawInputEvent` /
  `FrameOutput`; the game gives a root screen and a `GameHooks` value
  (first-key hook, debug hook, fade time). `trpg-ui` keeps `start()` and
  `start_on_scene()` as functions that build the hooks and root.
- The engine-owned part of settings (volumes, fullscreen, text speed,
  theme name, layout name) moves; enemy-phase speed, auto end turn,
  cursor style, map look and zoom and combat animations stay the game's
  (ADR-0050's saved record keeps its format: test with a fixture).

**Playdate:** the Playdate port (1025's tickets, 1027–1053) builds
`core`, `content` and `ui` without `std` and adds a second front end,
`crates/playdate`. If those tickets have landed, code this ticket moves
keeps its default `std` feature and stays in the `no-std` CI build, and
`cargo xtask playdate` keeps building (ADR-0070 §6).

**Out (do not do):** the Harness (1111); widgets and generic screens
(1112); `app` (1110).

## Implementation steps

1. Introduce `Ctx<G>` in place with the alias; move game fields into
   `Shuyi`; fix call sites (mechanical: `ctx.content` → `ctx.game.content`).
2. Make `Game` take the root screen and hooks.
3. Split settings; check the saved settings fixture loads unchanged.
4. Move `screen.rs` (generic part) and `game.rs` (loop) to `shuyi-ui`.
5. A test in `shuyi-ui` runs a two-screen toy game (push, pop, quit)
   with a toy `G` and a toy action set.

## Acceptance criteria

- [x] The toy two-screen test passes in `shuyi-ui` alone.
- [x] No snapshot changes; every test passes.
- [x] A settings file saved by the current `main` loads unchanged.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: stack behavior with the toy game.
- Integration: all game Harness tests unchanged.

## Completion notes

- **`shuyi-ui`** gains `ctx` (`Ctx<G: GameData>`), `screen` (`Screen<G>`,
  `Transition<G>`, `ScreenStack<G>`, `FrameInput`), `game` (`Game<G>`,
  `GameHooks`, `ScreenHook`, `RawInputEvent`, `FrameOutput<'a, V>`),
  `settings` (`Settings<S: GameSettings>`, `TextSpeed`, the volumes) and
  `text` (`{name}` filling). `storage`, `profiles` and `audio` moved there
  with `git mv`, because `Ctx` holds them: `AudioRequest<V>` and
  `AudioQueue<V>` take the game's voice variant, `profiles::load` the keys
  a game kept before it had profiles. ADR-0079 records the design.
- **`GameData`** is what the engine reads from the game's part: its
  action, color, voice and settings types, the text, keymap file, audio
  manifest, palette, console and theme palettes, and hooks with defaults
  (`debug_lang`, `key_text`, `voices`, `apply_settings`, `blank_cell`,
  `keys_before_profiles`, `old_layout_key`; `voice_variant` is required).
  The data stays in the game's `Content` and is read by reference.
- **The game** (`trpg_ui::screen`): `pub struct Shuyi { content,
  campaign_mode, campaign_tutorial, campaign_lessons, mode_switch,
  map_skin, tips_enabled, lessons, lead }` implements `GameData`; `Ctx`,
  `FrameInput`, `Transition`, `ScreenStack`, `Game`, `FrameOutput`,
  `Settings`, `AudioRequest` and `AudioQueue` are aliases. Call sites
  changed field paths only (`ctx.game.content`, `settings.game.map_zoom`),
  done from the compiler's error spans.
- **Deviations from the ticket:**
  - One type parameter, not two: `Screen<G>` and `Game<G>` take the
    action type from `G::Action`, so a screen and its game can't disagree.
  - The fade time isn't a hook: the engine reads the audio manifest for
    the music clock already, so it takes the fade from there.
  - Rust can't add inherent methods to the engine's `Ctx` from the game,
    nor alias a trait: what needs both parts is the `ShuyiCtx` trait
    (`Ctx::embedded()`, `from_content`, `words`, `language`,
    `set_map_skin`), in the crate's prelude, and `MenuSounds` for
    `ctx.audio.menu(…)`. Screens are `impl Screen<Shuyi> for …` (31
    impls). `Game::start`/`start_on_scene` became
    `trpg_ui::game::{start, start_on_scene}`, and `Game::new(ctx, root)`
    `game::new(ctx, root)`, which add the game's hooks.
  - The theme in the engine's settings is the game's type
    (`GameSettings::Theme`) rather than a string, so its values and its
    saved names don't change. Anim speed (not listed by the ticket) stays
    the game's with the other battle settings.
  - `Ctx::shown_layout` returns an `Option` (the engine has no default
    layout); the Key bindings screen falls back to the right-handed one as
    before. Two debug-panic messages no longer name the game's files
    (`text key not in the English text`, `audio request for a cue not in
    the audio manifest`); their tests say so.
- **Settings record:** the game keeps its `SettingsFile` as the
  `GameSettings` impl, so the form is unchanged. Fixtures written by
  `main` at `6677932e` (every setting changed, and the defaults:
  `crates/ui/src/settings/*_before_1109.ron`) load to the same settings and
  are written again byte for byte.
- **Tests:** the engine's run on a toy game (`shuyi_ui::toy`, tests only,
  excluded from mutation testing like the sample actions): the two-screen
  game (push, pop, quit), the stack, the frame loop (first-key and debug
  hooks, key prompt, device, clock, audio and the fade, music clock,
  volumes), and the context (text and the debug language, help keys,
  music picks, voices, layouts, bindings, settings preview/keep/restore,
  themes, profiles, and the trait's defaults). The game's tests of its
  `Ctx` and `Game` stay in the game, on its content: 1,516 unit and 343
  integration tests pass unchanged, no snapshot changed.
- `shuyi-ui` gains a `storage-conformance` feature (`trpg-ui`'s turns it
  on); `trpg-ui` depends on `shuyi-assets` directly (the text table's
  type). `profiles::fits_every_profile` is new (the game's test that its
  old keys fit a profile's key).
- `trpg-ui` drops `once_cell` and `thiserror` (they went with `Ctx` and
  `storage`).
- **Not checked locally:** `cargo xtask playdate` builds the Rust, but the
  SDK's `pdc.exe` crashes (0xc0000005) while packaging on this machine, as
  in 1108. CI's `playdate` job runs it. Mutation testing runs in CI.

No gameplay rules were decided. Nick input: none.
