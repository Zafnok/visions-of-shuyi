# Engine boundary: what is engine, what is game

Audit of 2026-10-14 (ticket 1101, at `a852d164`) for
[ADR-0063](../adr/0063-an-engine-and-a-game-one-boundary-now-two-repositories-later.md).
Each module is one of:

- **E**: engine. Any 2D game would want it unchanged.
- **K**: tactics kit. Another grid or turn-based game would want it.
- **G**: game. Visions of Shuyi's rules, data, screens or story.
- **M**: mixed. The note says where to cut.

Line counts are rough. For `ui`, `app` and `xtask` they are whole files
including inline tests (`wc -l`). For `core` and `content` they are code
lines without tests. Keep this file current as modules move: the ticket
that moves a module changes its row.

## Summary

| Crate | Engine | Tactics kit | Game | Notes |
| ----- | ------ | ----------- | ---- | ----- |
| `core` (14.1k code lines) | ~0.5k (3%) | ~1.5k (11%) | ~12.1k (86%) | Has no simulation trait. The K code is typed over the game's types, so it is rewritten generically, not moved. |
| `content` (15.6k code lines) | ~7.3k (47%) | ~0.7k (4%) | ~7.6k (49%) | Almost every loader is `load()` (reads the bundle) plus a pure `from_source(file, text)`. That is the seam. |
| `ui` (78.5k lines) | ~21.8k (28%) | ~6.9k (9%) | ~49.8k (63%) | M files: 28.5k lines, cut by the notes below. |
| `app` (3.6k lines) | ~3.3k | — | ~0.3k | Thin. The game's leaks are listed below. |
| `xtask` (18.3k lines) | ~8.4k | ~1.2k | ~4.4k | Plus 4.3k M. Repository tools (`ticket-lint`, `clean-merged-targets`) stay with us. |
| `bots` (0.7k lines) | — | ~0.2k shape | ~0.5k | Done (1116): the bot trait and play loop are `shuyi_core::bots`. |

**Total that goes to the engine:** roughly a third of the codebase. That
is about 60k of about 180k lines, counting tests.

## The couplings that block a second game

These are the things ADR-0063 §6 makes generic before moving anything.

1. **`Action` is a closed enum.** It lives at `content/src/keymap.rs:338-479`,
   along with `ALL`, `name`, `is_repeatable`, `is_required` and
   `is_rebindable`. It runs through `Keymap`, `InputState`, `FrameInput`,
   `PlayerKeys` (`ui/src/input/bindings.rs:43,113`), `HelpKeys`, tips and the
   rebinding screen's `ROWS` (`ui/src/screens/key_bindings.rs:56-77`). `Layout`
   is a closed 2-variant enum (`keymap.rs:518-548`).
   **Fix:** an `ActionSet` trait, plus engine-owned `UiAction`s for widgets.
   Layouts become named data. **Done:** 1106 made input generic over
   `ActionSet`; 1107 moved it to `shuyi_assets::keymap` and `shuyi_ui::input`.
2. **`Ctx` is wired to the game** (`ui/src/screen.rs:224-327`). It holds
   `Content`, `campaign_mode`, `mode_switch`, `map_skin` and `lead`.
   `Ctx::embedded` calls `trpg_content::load_embedded` (`:448`).
   **Fix:** `Ctx<G>`, which holds the engine services and a `game: G`.
   **Done (1109):** `shuyi_ui::Ctx<G: GameData>`; the game's part is
   `trpg_ui::screen::Shuyi` and `Ctx` its alias (ADR-0080).
3. **`Game` builds this game's screens** (`ui/src/game.rs`). `start()` builds
   `TitleScreen` (`:103-118`) and `start_on_scene` opens a dialogue
   (`:128-170`). `Game` itself opens the layout picker (`:257,:304`) and the
   debug menu (`:297-307`).
   **Fix:** the game hands `Game` its root screen and its hooks.
   **Done (1109):** `shuyi_ui::Game<G>` with `GameHooks` (`first_key`,
   `debug`); `trpg_ui::game::{start, start_on_scene}` build them.
4. **The console size is fixed:** 100×32 cells of 8×16 pixels
   (`ui/src/console.rs:4-10`). Fifteen files in `ui`, `app` and `xtask` use
   these numbers, along with `pixel_bounds` (`glyph_buffer.rs:477`) and
   `Backdrop::clip_px`.
   **Fix:** a `ConsoleSpec` that the game passes at start-up.
   **Done:** 1030 added `ConsoleSpec` in place; 1108 moved it to
   `shuyi_ui::frame::console` with no sizes of its own (only
   `ConsoleSpec::text`, 8 × 16-pixel cells). The game's two sizes are
   `ConsoleSpec::PC` and `ConsoleSpec::PLAYDATE` through its
   `trpg_ui::console::ConsoleSizes` trait.
5. **The frame imports content types.** `ImageId` and `Light` come in at
   `glyph_buffer.rs:24`, and there is `Paint::Shaded(Light)` (`:172`).
   `UiColor` is a closed list that includes tactics colors
   (`color.rs:119-144`), and content requires it (`palette.rs:65-90`,
   `REQUIRED_COLORS`).
   **Fix:** named palette colors, with the required set given by the game.
   **Partly done in 1104:** `ImageId`, `Light` and `PaletteDef` are
   `shuyi_assets` types (the frame still imports them through
   `trpg_content`), and the palette takes its required names from the game.
   **Done in 1108:** the frame imports them from `shuyi_assets`. The
   engine's `Palette` is built for a game's `ColorSet` (`UiColor` here) and
   adds six `EngineColor`s for the engine's widgets (`text`, `text_dim`,
   `panel`, `border`, `highlight`, `warning`); the game's set says which of
   its names each one is (`panel_bg`, `panel_border`, `text_highlight`,
   `hp_low`), so its palette files keep their names.
6. **The map scene carries battle units.** See `UnitView`
   (`ui/src/map_view/scene.rs:131-168`). `RangeKind` is Danger, Move, Attack
   or Heal. Skins take `trpg_core::Pos` and the whole `Ctx`. Generic code
   reaches into `screens::battle` for walk speeds (`map_view/skin.rs:11`,
   `sprite.rs:34`), unit colors (`glyph/units.rs:13`, `sprite/units.rs:27`)
   and `MAP_VIEW` (`tips.rs:13`).
   **Fix:** the kit's scene is generic over the game's piece view.
7. **There is no simulation trait in `core`.** `Command` (`battle.rs:673`,
   enum), `Event` (45 variants), `CommandError` (52) and `BattleState` are
   named directly by `history.rs:30-39`, `legal.rs:61`, `ai.rs:119`,
   `save.rs:26`, the bots and the cinematic combat shot.
   **Fix:** a `TurnRules` trait that `BattleState` implements.
   History, replay, the bots and lockstep networking are then written
   against the trait.
   **Done (1114):** `shuyi_core::TurnRules` (check, apply, legal commands,
   outcome), implemented by `BattleState` with its own methods. History,
   replay and rewind are `shuyi_core::history::History` over it.
   **Done (1116):** the bots: `shuyi_core::bots` (`Bot`, `play`, a
   seeded `RandomBot`) over `TurnRules` plus `Sides` (whose move, which
   turn), which `BattleState` implements with its phase and turn.
8. **Pathfinding is not generic.** `reachable`, `path_cost`, `threat_area` and
   `danger_zone` take the game's map, tables and units
   (`core/src/movement.rs:334-515`). There is a second Dijkstra in
   `ai.rs:755`, and `movement` and `ai` import each other.
   **Fix:** the kit's Dijkstra over a cost closure, which keeps the
   `(cost, y, x)` tie-break. The game calls it.
   **Done (1114):** `shuyi_core::path` (`reachable`, `Reach::path_to`,
   `flow_field`) is the one Dijkstra; `movement.rs` and the AI's flow
   field call it with their cost closures.
9. **The bundle is global and fixed at compile time**
   (`content/src/bundle.rs:20-92`). The folders are written into
   `include_dir!` and a static `LAYERS`. Two cargo features and `build.rs`
   are tied to them. There is no runtime (disk or mod) layer.
   **Fix:** `Bundle::new(&[Layer])`. The game registers its layers once.
   Runtime layers are for mods (1011) and hot reload (engine ADR-0004).
   **Done in 1103** but for runtime layers: `shuyi_assets::Bundle::new(Vec<Layer>)`,
   installed once from `trpg_content::layers()`.
10. **The `Harness` is wired to the flow, the battle and the test chapter**
    (`ui/src/harness.rs:20-28,542`). Its driver core is generic: about 750
    lines of key, pad and wait scripts, the audio and music simulation, and
    snapshots.

## `core` (trpg-core)

| Module | Side | Note |
| ------ | ---- | ---- |
| `rng.rs` | E | Moved to `shuyi_core::rng` (1114): own PCG32 `SimRng`, `RandomSource`, `ScriptedRng`. `trpg_core::rng` names it. |
| `geom.rs` | E | Moved to `shuyi_core::grid` (1114) with `TileSet`: `Pos`, `Dir` (4-way), `Grid<T>`. `trpg_core::geom` names it. |
| `terrain.rs` | K | Move costs per movement type are K. The `defense`/`avoid`/`heal_percent` fields are G. |
| `movement.rs` | G on K | Done (1114): `TileSet` moved to the kit's grid, and `reachable` is the kit's Dijkstra (`shuyi_core::path`) with this game's cost closure; its `Reach` adds where a unit may stop. `path_cost`, attack tiles, threat and the danger zone are this game's rules of units and weapons over it. |
| `map.rs` | M | `BattleMap` grid is K. `TileFeature::{Shop,Chest}` is G. |
| `history.rs` | M | Done (1114): replay, `state_at` and rewind are `shuyi_core::history::History` over `TurnRules`. `BattleHistory` adds the rewind charges (G) and keeps the save's format. |
| `save.rs` | M | Done (1114): the version header and the refuse-don't-migrate rule are `shuyi_core::save` (`SaveHeader<F: SaveFormat>`). `SaveFile` payload is G. |
| `battle.rs` | M | About 450 lines of turn skeleton (phases, turns, objectives, the validate→plan→execute pattern) are K in shape. The skeleton calls game rules inline, so it stays in the game and implements `TurnRules` (1114). The rest (2,800) is G. |
| `battle/triggers.rs` | M | Areas, `Who`, `TurnStart`, enters-area, talk and fired-once are K (about 250 lines). The others are G. |
| `lead.rs` | M → cut | 1105: `split_tokens` and `Part` (`{token}` text) are `shuyi_core::text`, and `CharacterId` (`unit.rs`) is `shuyi_core::CharacterId`, the dialogue's speaker id too; both re-exported. The lead (`LEAD_ID`, pronouns, gendered portraits, its tokens) is G. |
| `ai.rs`, `legal.rs`, `campaign.rs`, `prep.rs`, `unit.rs`, `class.rs`, `stats.rs`, `combat.rs`, `item.rs`, `weapon.rs`, `magic.rs`, `spell.rs`, `skill.rs`, `art.rs`, `progression*`, `shop.rs`, `support.rs`, `battle/{arts,skills,supports}.rs` | G | This game's rules. `legal.rs` is `BattleState`'s `TurnRules::legal_commands` (1114); the AI's flow field calls the kit's `flow_field`. |

## `content` (trpg-content)

Modules that do not import `trpg_core` at all are the first set to move:
`audio`, `bundle`, `credits`, `dialogue/line_id`, `error`, `font`, `image`,
`keymap`, `lang`, `lighting`, `palette`, `portrait`, `ron_loader`, `tip`,
`title`. `bundle`, `error` and `ron_loader` moved in 1103; `audio`,
`credits`, `font`, `image`, `lighting` and `palette`'s generic part
(with `ink`) in 1104; `lang`, `names`, the dialogue format, `voice` and
`tip`'s generic parts in 1105; `keymap`'s file format in 1107.

| Module | Side | Note |
| ------ | ---- | ---- |
| `bundle.rs`, `build.rs` | E → moved | 1103: the layer logic is `shuyi_assets::bundle` (`Bundle`, `Layer`, `install`). `bundle.rs` keeps the game's folders (`include_dir!`, `layers()`) and the old free functions over the installed bundle; `build.rs` and the two features stay the game's. |
| `error.rs`, `ron_loader.rs` | E → moved | 1103: `shuyi_assets::{error, ron}` (`ContentError`, `parse_ron<T>`, `load_ron` over a `Bundle`). Both files are re-exports now; `load_ron(path)` reads the game's bundle. |
| `font.rs`, `image.rs`, `lighting.rs`, `audio.rs`, `credits.rs` | E → moved | 1104: `shuyi_assets::{font, image, lighting, audio, credits}`. The parsers take what names the game's files as arguments: the bundle (`ImageTable::load`), the required glyphs and atlas image's name, the default light, the audio `Files` (with the sound folder's shown names and the music folder), the credits' `Places` (audio manifest and private folder). Each content module keeps its paths, `load()` and a wrapper with its old arguments, and re-exports the rest. `credits.rs`' bought-art check (ADR-0051) moved with it. |
| `lang.rs`, `names.rs` | E → moved | 1105: `shuyi_assets::{lang, names}`: text by key, packs, stale, missing and orphaned text, the English the packs translate (`SourceText`, which the game builds) and the lookups by key. A pack is checked by the game's `PackRules` (its data text's rules, tokens, `DialogueLimits`, line variants). Content keeps `LANG_DIR`, `TEST`, `english_ui()`, `load()`, the wrappers with the old arguments and a `Lang` that derefs to the engine's and adds the game's lookups. `names.rs` keeps `NAMES_PATH` and `load()`. |
| `dialogue/parse.rs`, `dialogue/line_id.rs` | E → moved | 1105: `shuyi_assets::dialogue` (the parser, line ids, captions' keys). A game adds directives with the `Directive` trait (`Step::Directive`). |
| `portrait.rs`, `title.rs`, `tileset/look.rs` | E | The five required expressions and the 256 px frame become parameters. |
| `palette.rs`, `ink.rs` | M → cut | 1104: `PaletteDef`, hex, `theme_problems` (a theme has the default's names) and the Playdate's `Ink`s are `shuyi_assets::{palette, ink}`; `PaletteDef::from_source` takes the required names. `Theme`, `REQUIRED_COLORS`, `load_themes` and the palette files' paths stay. |
| `keymap.rs` | M → cut | 1107: the file format (keys, chords, buttons, named layouts, repeat, sticks, validation; `KeymapDef<A>`) and `ActionSet`/`UiAction` are `shuyi_assets::keymap`, re-exported here (`keymap::format` is the engine's module). `Action`, `KeyGroup`, `RIGHT_HANDED`/`LEFT_HANDED`, `load_keymap` and the game's `keymap.ron` stay, with the tests of this game's keys. The format's tests run on the engine's sample actions (`keymap::sample`). |
| `lang/lines.rs`, `lang/data.rs` | M → cut | 1105: per-line translation is `shuyi_assets::lang`, with `LineText::Variants` (by variant name) for the old `ByGender`; the game names its variants `m` and `f` (`lang::RULES`). `data.rs` keeps the keys of classes, items, tips, lessons, chapters and battle notes (`from_tables`, `Tables`), the tip and lesson limits, and the lookups (`class_name`…). |
| `dialogue.rs`, `dialogue/check.rs`, `dialogue/presence.rs` | M → cut | 1105: scenes, steps, choices, music lines, `@if` (`Present`, `Scene::resolved`) and the on-screen checks are `shuyi_assets::dialogue`, checked against a game's `SceneRules` (limits as `DialogueLimits`, its `Tokens`, cast, expressions, music). Content keeps `@place` and `@opening` (`Stage`, `PlaceLine`, `check_places`, where an opening goes), the lead's rules and tokens (`LeadTokens`), the box sizes, `Cast`/`check_presence`, `load()` and aliases (`Step = Step<Stage>`…). |
| `voice.rs`, `tip.rs` | M → cut | 1105: the clip manifest, cast and staleness are `shuyi_assets::voice` over a game's `ClipVariant`; the tip schema and `{Action}` placeholders are `shuyi_assets::tip` over a game's trigger type. Content keeps `Variant` (M/F) and what a line says of the lead (`spoken_text`), the voice folders, `TipTrigger`, the tip box, the guide and `load()`. |
| `map.rs` | M | Done (1116): the `.map` format (reader, printer, legend, look and light) is `shuyi_assets::grid_map`, with a `MapFeature` hook. Shop and Chest features, the tile looks, the size limit and the terrain lookup stay here. |
| `tileset.rs` | M | Terrain tiles, looks and walk sheets are E. Unit pictures keyed by class or character are G. |
| `terrain.rs` | K | Its generic `line_of` helper moved to `shuyi_assets::error` (1103); `terrain.rs` re-exports it for the 11 modules that use it. |
| `cinematic.rs` | M | The timeline (shots at ms, music length) is E. `MapPan` is K. `Characters`, `Talk` and `Combat` are G. |
| `lib.rs` | G | The `Content` struct. Its "run every loader, collect every error" pattern is E. |
| `cinematic/{action,combat}.rs`, `battler.rs`, `chapter.rs`, `battle.rs`, `character.rs`, `class.rs`, `item.rs`, `spell.rs`, `skill*`, `art.rs`, `support.rs`, `ai.rs`, `trigger.rs`, `enums.rs` | G | The game's data. |

## `ui` (trpg-ui)

| Module | Side | Note |
| ------ | ---- | ---- |
| `glyph_buffer*`, `snapshot.rs`, `console.rs`, `cinema.rs`, `onebit*`, `float.rs` | E → moved | 1108: `shuyi_ui::frame::{glyph_buffer, snapshot, console, cinema, onebit}` and `shuyi_ui::float`, with their tests on the engine's own palette. The game's files re-export them under their old names; `console.rs` keeps the game's sizes (`ConsoleSizes`), `snapshot.rs` and `onebit.rs` the tests on this game's palette and font. |
| `portrait.rs` | M → cut | 1108: fitting a picture to an area of cells at a whole scale (`fit_whole_scale`, `dim_opacity`, `draw_fitted`) is `shuyi_ui::frame::portrait`. The 32 × 16-cell frame and drawing a `Portrait`'s expression stay the game's. |
| `storage.rs`, `audio.rs`, `profiles.rs` | E → moved | 1109: `shuyi_ui::{storage, audio, profiles}`; `AudioRequest` and `AudioQueue` take the game's voice variant, `profiles::load` the keys a game kept before profiles. The game keeps the four menu cue ids (`MenuSound`, until 1112's `AudioConfig`) and its keys from before profiles. |
| `widgets/` | E | `help.rs` is now the game's alias of `HelpKeys` and a re-export of `shuyi_ui::input::help_keys` (1107). |
| `input/` (`keymap`, `state`, `frame`, `bindings`, `pad`, `fixed`, `help_keys`) | E → moved | 1107: `shuyi_ui::input`, with their tests on the sample actions. `trpg-ui::input` re-exports the submodules under their old names. |
| `debug/sprite_test.rs`, `screens/mod.rs` | E | — |
| `input.rs` | G | 1107: what's left is the game's: the aliases with `Action`, `KEY_ROWS`, the text keys and `text_keys_help`. The fixed Esc→Cancel is `Action`'s `fixed_keys` and the split keys are `screens/battle/split_keys.rs` (1106). |
| `screen.rs` | M → cut | 1109: `Screen`, `Transition`, `FrameInput`, `ScreenStack` (`shuyi_ui::screen`) and `Ctx<G>` (`shuyi_ui::ctx`) are the engine's. The game's part (`Shuyi`), its aliases and `ShuyiCtx` stay. |
| `game.rs` | M → cut | 1109: the frame loop is `shuyi_ui::Game<G>`. Start-up (`start`, `start_on_scene`) and the hooks (layout picker, debug menu) stay the game's. |
| `color.rs` | M → cut | 1108: `Rgb`, `Palette`, `ColorSet` and `EngineColor` are `shuyi_ui::frame::color`. `UiColor` (this game's colors, tactics colors among them) and which of them the engine's colors are (`engine_color`) stay. |
| `settings.rs`, `words.rs`, `tips.rs` | M | 1109: the engine's settings (text speed, fullscreen, theme, volumes, layout) are `shuyi_ui::settings`, text by key is `Ctx::text`, and `{name}` filling is `shuyi_ui::text`. The tactics settings (`ShuyiSettings`) and their saved record, the data text (`Words`), `{Cursor}`/`{Select}` and tip placement stay G or K. |
| `dialogue.rs`, `screens/dialogue` | M | The player state machine and box are E. The lead and cast are G. |
| `cinematic*` | M | Timeline and easing are E. Map pan and action are K. Logo, talk, characters and combat shots are G. |
| `harness.rs` | M | See coupling 10. |
| `debug*` | M | The menu, glyph sampler, portrait viewer and backdrop camera are E. Promote/reclass, skin switch and scene preview are G. |
| `screens/{key_bindings,layout_picker,options}` | M | Generic screens with this game's rows and layouts. |
| `map_view.rs`, `map_view/scene.rs`, `map_view/glyph*`, `map_view/sprite*` | M | Tiles, rim, ranges, path, cursor, threat, autotiled ground and light are K. Units, HP bars, effect marks and `GAME_TILESET` are G. |
| `map_view/{skin,grid,corners,path,threat}.rs` | K | Square grid only. `skin.rs:11` imports battle walk speeds. |
| `screens/battle/{camera,cursor,path,walk}.rs` | K | Tile camera, virtual cursor, path steering, walk timing. |
| `screens/battle/*` (the other 42 files) | G | The Fire Emblem loop. |
| `flow.rs`, `save.rs`, `screens/{title,save,preparations,class_change,results,lead_select,credits,game_over,mode_select}` | G | — |

`tests/it` in `ui` totals 9,000 lines:

- E, 1,852 lines: controller, key bindings, rebind buttons, scene camera, sprite test, voice, layout picker.
- K, 840 lines: map skin, map zoom, threat.
- M, 1,105 lines.
- G, 5,203 lines.

Every one of these runs through the game's Harness. The E and K tests move
only once the Harness can drive a toy game (1111).

## `app` (trpg-app)

| Module | Side | Note |
| ------ | ---- | ---- |
| `render.rs`, `keys.rs`, `pads*` | E → moved | 1110: `shuyi-app`. The renderer reads images with the game's file reader; the JS plugin is `shuyi_gamepad` with `shuyi_pad_*` functions. |
| `storage/` | E → moved | 1110: the storage's name is `AppConfig::storage_name` (the native folder, and the web prefix `<name>/`); the game's stays `tactical-rpg`. |
| `audio*` | E → moved | 1110: sounds through the engine's audio `Files` (the music folder's name with them), voices through `AppConfig::voices` (folder, language, manifest); requests generic over the voice variant. |
| `main.rs`, `build.rs` | M → split | 1110: the loop, the window settings, the icon decoding and the error screen are `shuyi_app::{run, window_conf}`. The game's binary keeps macroquad's entry point, the window's title, size and icon, `Ctx::embedded` and its settings, the start screen, the release check on test files (a compile-time check) and `build.rs` (exe strings, the web's `--allow-undefined`). |
| `args.rs` | G | `--scene`, `--lead`. |

macroquad APIs in use: window and loop (`Conf`, fullscreen, DPI,
`next_frame`, frame time), textured quads (`draw_texture_ex`,
`Texture2D::from_image`, `Camera2D` for the backdrop viewport), keyboard
(`get_keys_pressed`/`released`, `is_key_down`, `get_char_pressed`), audio
(`macroquad::audio`), files (`load_file`), coroutines, and logging. Native
music uses quad-snd. Controllers use gilrs plus a web JS plugin.

**What is missing for other kinds of games:** mouse and touch input, and
analog sticks (today they are four digital directions).

## `xtask`

| Subcommand | Side | Ship with the engine? |
| ---------- | ---- | --------------------- |
| `frame-png` | E | **Yes, first.** Headless screenshots of any scripted frame, for agents. |
| `font-atlas`, `check-keys`, `check-text`, `lang-status`, `web`, `private-assets` | E | Yes. Paths become arguments or a config file. |
| `lines`, `check-script` | E | Yes, with the dialogue format. |
| `test-card`, `voice-test-clips` | E | Fixtures for the engine's own tests. |
| `ticket-lint`, `clean-merged-targets` | E | No. These are our repository's workflow. |
| `test-tileset` | K | Kit fixtures. |
| `tileset-import` | M | The Tiled `.tsx` → corner-layers core is K. The bought-pack table is G. |
| `sfx` | M | The synth (tone, burst, pulse, envelope) is E. The sounds are G. |
| `icon` | M | Mostly E. |
| `test-units`, `effect-marks`, `title-art`, `map-sprite-import`, `battler-import`, `portrait-import`, `playtest` | G | — |

## `bots`

Done (1116): the bot trait and the play loop are the kit's
(`shuyi_core::bots`: `Bot<R>`, `play` and `play_observed`, `RandomBot`),
over `TurnRules` and `Sides`. `trpg-bots` keeps the game's bots
(`BaselineBot`; `PlayerBot` is any `Bot<BattleState>`), `play_battle`
(the engine loop with the AI on the other phases) and the measures.

## What other game styles would break (for engine ADR-0003)

- **Integer tile positions everywhere.** Units, triggers, objectives,
  cinematic walks and `TilePoint` all use them. Only one unit may stand on
  a tile. Movement is 4-way with Manhattan ranges.
- **No time step.** Turns and phases only. Replay works only as a command
  log. Real time needs a fixed-step input log.
- **The frame.**
  - Items sit on integer console pixels. Only the backdrop pans by
    fractions.
  - There are two layers (Under, Over) and no sort key. Y-sorting is done
    by the caller.
  - Items are cut where cells are replaced.
  - There is one backdrop, with whole zoom 1–4.
  - There is no world/screen coordinate split, no parallax layers, no
    isometric grid, and no fixed-step loop (`Game::frame` uses variable
    `dt`).
- **Square, orthogonal tiles.** Corner autotiles, `Grid` and the row sort
  all assume them.
