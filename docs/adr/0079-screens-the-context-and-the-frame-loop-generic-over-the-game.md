# ADR-0079: Screens, the context and the frame loop are the engine's, generic over the game

- **Status:** Accepted
- **Date:** 2026-10-17
- **Related tickets:** 1109, 1110, 1111, 1112
- **Amends:** ADR-0017 (where `Screen`, `Transition`, `ScreenStack`, `Ctx`
  and `Game` live, and what `Game` does on its own) and ADR-0050 (which
  settings are the engine's; the saved record and its format are
  unchanged). ADR-0069's profiles move with `Ctx`, unchanged.

## Context

`Screen`, `Transition`, `FrameInput`, `ScreenStack` and the frame loop
(`Game`, ADR-0017) were generic in shape, but every screen took a `Ctx`
holding this game's `Content`, campaign mode, map skin and lead, and `Game`
built this game's title screen, opened the layout picker on the first key
and the debug menu on `Debug` (`docs/engine/boundary.md`, couplings 2 and
3). ADR-0063 §6 asks for the generic part to be made generic in place, then
moved, with game code keeping its names through aliases.

Three Rust rules shape how that can look:

- An inherent method can only be added in the crate that defines the type.
  Once `Ctx` is the engine's, the game cannot add `ctx.words()` to it.
- `impl Deref for shuyi_ui::Ctx<Shuyi>` is an orphan (a foreign trait for a
  foreign type) and isn't allowed; neither is a blanket impl of the
  engine's `Screen` for the game's own screen trait.
- A trait can't be aliased on stable Rust.

## Decision

1. **`shuyi_ui::Ctx<G: GameData>`** holds the engine's services and
   `pub game: G`: the palette (built for `G::Colors`, and the theme in the
   settings), the console, the active keymap, the device pressed last, the
   layout and the player's bindings, the warnings, the profile storage and
   the profiles, the settings (previewed and stored), debug tools, idle
   motion, the audio queue and the music clock, voices (on, volume, the
   playable clips), the music seed and picks, the clock, the key prompt,
   whether there is a keyboard, and the language. Its methods are the old
   `Ctx`'s that need only these: text by key (`text`, `text_with`,
   `effective_lang`), `help_keys`, layouts and bindings, settings,
   profiles, voices, `pick_music`. `Ctx::new(game)` fails with the names of
   the colors the game's palette lacks.
2. **`GameData`** is what the engine reads from the game's part, through
   `ctx.game`:
   - associated types: `Action: ActionSet`, `Colors: ColorSet`,
     `Voice: ClipVariant`, `Settings: GameSettings`;
   - data: `lang()`, `keymap_def()`, `audio_manifest()`, `palette_def()`,
     `console()`, `theme_def(theme)`;
   - hooks with defaults: `debug_lang` (none), `key_text` (an action's
     key), `voices` (none: a manifest is refused), `apply_settings`
     (nothing), `blank_cell` (text on panel), `keys_before_profiles`
     (none), `old_layout_key` (none); and `voice_variant`, required.

   The data stays in the game's `Content`, validated as one, and the
   engine reads it by reference: nothing is copied or loaded twice.
3. **Screens:** `shuyi_ui::Screen<G>`, `Transition<G>`, `ScreenStack<G>`;
   `update` takes `&mut Ctx<G>` and `&FrameInput<G::Action>`. The action
   type is `G`'s associated type, not a second parameter, so a screen and
   its game can't disagree about it.
4. **The frame loop:** `shuyi_ui::Game<G>`, with `RawInputEvent` and
   `FrameOutput<'a, V>` (`V` the voice variant). The game gives it the
   screens it starts on (`Game::new(ctx, root, hooks)` or `with_stack`)
   and its **`GameHooks`**: `first_key`, the screen the first key press
   opens while no layout is chosen on a platform with a keyboard (by name
   and constructor, `ScreenHook`), and `debug`, the screen the
   `UiAction::Debug` action opens with debug tools on, except over one of
   `debug_screens`. `GameHooks::default()` has neither. The music fade
   comes from the audio manifest, which the engine already reads for the
   music clock.
5. **Settings:** `shuyi_ui::settings::Settings<S: GameSettings>` holds the
   engine's (`text_speed`, `fullscreen`, `theme: S::Theme`, `music_volume`,
   `sound_volume`, the layout) and `pub game: S`. The saved record stays
   one record, and its form is the game's: `GameSettings::to_ron` /
   `from_ron`. The theme's type is the game's (`S::Theme`), so its names in
   the saved record and its values in code don't change.
6. **Storage, profiles, audio and placeholders move with `Ctx`:**
   `shuyi_ui::{storage, profiles, audio, text}`. `AudioRequest<V>` and
   `AudioQueue<V>` take the voice variant; the menu sounds' cue ids stay
   the game's (`MenuSound`, until 1112's `AudioConfig`). `profiles::load`
   takes the keys a game kept before it had profiles.
7. **The game keeps its names.** `trpg_ui::screen` defines
   `pub struct Shuyi { content, campaign_mode, campaign_tutorial,
   campaign_lessons, mode_switch, map_skin, tips_enabled, lessons, lead }`
   and the aliases `Ctx = shuyi_ui::Ctx<Shuyi>`, `FrameInput`,
   `Transition`, `ScreenStack`, `Game`, `FrameOutput`, `Settings`,
   `AudioRequest`, `AudioQueue`. What needs both parts is a trait the game
   implements for its `Ctx`, in the crate's prelude: `ShuyiCtx`
   (`Ctx::embedded()`, `from_content`, `words`, `language`,
   `set_map_skin`); likewise `MenuSounds` for `ctx.audio.menu(…)`. Game
   screens are `impl Screen<Shuyi> for …`. Starting is
   `trpg_ui::game::{start, start_on_scene, new}`, which build the hooks.

## Consequences

- A second game builds a context and a frame loop from `shuyi-ui` alone:
  the engine's tests run a toy game of two screens (push, pop, quit) and
  the whole context on a toy `GameData`.
- Game code changed field paths (`ctx.game.content`,
  `settings.game.map_zoom`), the `impl` line of each screen, and
  `Game::start` → `game::start`; no screen's logic or look changed, and no
  snapshot did.
- A file outside `trpg-ui` that calls `Ctx::embedded()` or `ctx.words()`
  imports `ShuyiCtx`. Inside the crate the prelude brings it.
- The engine names a few text keys the game's language files must have:
  `keys.not_mapped`, `keys.arrows`, `profile.numbered` and the text
  speeds' names.
- Saved data keeps its form: the settings record (a fixture written by
  `main` loads and is written again byte for byte), the profiles list, the
  key bindings.
- `GameData` will grow as more moves (1112's audio config, dialogue
  hooks). Each hook has a default where most games want one.

## Alternatives considered

- **The engine's `Ctx` owns copies of the text, keymap file, audio
  manifest and palettes**, and the game's `Content` loses them. No trait
  for the data, but `Content` is validated as a whole (dialogue against
  text, the palette against the themes), and moving fields out of it
  touches every loader and the hundreds of reads of `content.lang` and
  `content.audio`. Reading through `GameData` keeps one copy.
- **`Ctx` as a field of the game's context** (`struct Ctx { engine:
  shuyi_ui::Ctx, content, … }`). Screens would keep `ctx.content`, but the
  engine's `Screen` and `Game` couldn't name the game's context without a
  generic over the whole context type, which is `Ctx<G>` with more steps,
  and every engine service would be `ctx.engine.…`.
- **A game-side `Screen` trait with an adapter.** Rust's orphan rules
  forbid a blanket impl of the engine's trait; a wrapper type would make
  every `Box::new(screen)` a `Box::new(Local(screen))` and break
  `ScreenStack::find`.
- **`Screen<C>` over the context type** (the ticket's other suggestion):
  the same number of changes, but nothing ties the screen's action type to
  its context's; an associated type on `G` does.
- **The settings record written by the engine with the game's fields
  flattened in** (`#[serde(flatten)]`): RON writes a flattened struct as a
  map, which would change the saved form.
