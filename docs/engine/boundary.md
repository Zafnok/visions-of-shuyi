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
   Layouts become named data.
2. **`Ctx` is wired to the game** (`ui/src/screen.rs:224-327`). It holds
   `Content`, `campaign_mode`, `mode_switch`, `map_skin` and `lead`.
   `Ctx::embedded` calls `trpg_content::load_embedded` (`:448`).
   **Fix:** `Ctx<G>`, which holds the engine services and a `game: G`.
3. **`Game` builds this game's screens** (`ui/src/game.rs`). `start()` builds
   `TitleScreen` (`:103-118`) and `start_on_scene` opens a dialogue
   (`:128-170`). `Game` itself opens the layout picker (`:257,:304`) and the
   debug menu (`:297-307`).
   **Fix:** the game hands `Game` its root screen and its hooks.
4. **The console size is fixed:** 100×32 cells of 8×16 pixels
   (`ui/src/console.rs:4-10`). Fifteen files in `ui`, `app` and `xtask` use
   these numbers, along with `pixel_bounds` (`glyph_buffer.rs:477`) and
   `Backdrop::clip_px`.
   **Fix:** a `ConsoleSpec` that the game passes at start-up.
5. **The frame imports content types.** `ImageId` and `Light` come in at
   `glyph_buffer.rs:24`, and there is `Paint::Shaded(Light)` (`:172`).
   `UiColor` is a closed list that includes tactics colors
   (`color.rs:119-144`), and content requires it (`palette.rs:65-90`,
   `REQUIRED_COLORS`).
   **Fix:** named palette colors, with the required set given by the game.
   **Partly done in 1104:** `ImageId`, `Light` and `PaletteDef` are
   `shuyi_assets` types (the frame still imports them through
   `trpg_content`), and the palette takes its required names from the game.
   `UiColor` is 1108.
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
| `lead.rs` | M | `split_tokens` (`{token}` text) is E. The lead is G. |
| `ai.rs`, `legal.rs`, `campaign.rs`, `prep.rs`, `unit.rs`, `class.rs`, `stats.rs`, `combat.rs`, `item.rs`, `weapon.rs`, `magic.rs`, `spell.rs`, `skill.rs`, `art.rs`, `progression*`, `shop.rs`, `support.rs`, `battle/{arts,skills,supports}.rs` | G | This game's rules. `legal.rs` is `BattleState`'s `TurnRules::legal_commands` (1114); the AI's flow field calls the kit's `flow_field`. |

## `content` (trpg-content)

Modules that do not import `trpg_core` at all are the first set to move:
`audio`, `bundle`, `credits`, `dialogue/line_id`, `error`, `font`, `image`,
`keymap`, `lang`, `lighting`, `palette`, `portrait`, `ron_loader`, `tip`,
`title`. `bundle`, `error` and `ron_loader` moved in 1103; `audio`,
`credits`, `font`, `image`, `lighting` and `palette`'s generic part
(with `ink`) in 1104.

| Module | Side | Note |
| ------ | ---- | ---- |
| `bundle.rs`, `build.rs` | E → moved | 1103: the layer logic is `shuyi_assets::bundle` (`Bundle`, `Layer`, `install`). `bundle.rs` keeps the game's folders (`include_dir!`, `layers()`) and the old free functions over the installed bundle; `build.rs` and the two features stay the game's. |
| `error.rs`, `ron_loader.rs` | E → moved | 1103: `shuyi_assets::{error, ron}` (`ContentError`, `parse_ron<T>`, `load_ron` over a `Bundle`). Both files are re-exports now; `load_ron(path)` reads the game's bundle. |
| `font.rs`, `image.rs`, `lighting.rs`, `audio.rs`, `credits.rs` | E → moved | 1104: `shuyi_assets::{font, image, lighting, audio, credits}`. The parsers take what names the game's files as arguments: the bundle (`ImageTable::load`), the required glyphs and atlas image's name, the default light, the audio `Files` (with the sound folder's shown names and the music folder), the credits' `Places` (audio manifest and private folder). Each content module keeps its paths, `load()` and a wrapper with its old arguments, and re-exports the rest. `credits.rs`' bought-art check (ADR-0051) moved with it. |
| `lang.rs`, `names.rs` | E | Text by key, packs, stale or missing text. `Lang` holds the game-built `DataText` (`lang.rs:175`). Cut that. |
| `dialogue/parse.rs`, `dialogue/line_id.rs` | E | The `.dlg` parser needs only a character-id newtype. |
| `portrait.rs`, `title.rs`, `tileset/look.rs` | E | The five required expressions and the 256 px frame become parameters. |
| `palette.rs`, `ink.rs` | M → cut | 1104: `PaletteDef`, hex, `theme_problems` (a theme has the default's names) and the Playdate's `Ink`s are `shuyi_assets::{palette, ink}`; `PaletteDef::from_source` takes the required names. `Theme`, `REQUIRED_COLORS`, `load_themes` and the palette files' paths stay. |
| `keymap.rs` | M | Keys, chords, buttons, repeat and sticks are E (about 740 lines). `Action` and `Layout` are G. |
| `lang/lines.rs`, `lang/data.rs` | M | Per-line translation is E. `ByGender`, and the table lookups for classes and items, are G. |
| `dialogue.rs`, `dialogue/check.rs`, `dialogue/presence.rs` | M | Scenes, steps, choices, music lines, `@if` and the on-screen checks are E. Lead rules, box sizes and roster checks are G. |
| `voice.rs`, `tip.rs` | M | Clip manifest and tip schema are E. M/F variants and the `TipTrigger` enum are G. |
| `map.rs` | M | Done (1116): the `.map` format (reader, printer, legend, look and light) is `shuyi_assets::grid_map`, with a `MapFeature` hook. Shop and Chest features, the tile looks, the size limit and the terrain lookup stay here. |
| `tileset.rs` | M | Terrain tiles, looks and walk sheets are E. Unit pictures keyed by class or character are G. |
| `terrain.rs` | K | Its generic `line_of` helper moved to `shuyi_assets::error` (1103); `terrain.rs` re-exports it for the 11 modules that use it. |
| `cinematic.rs` | M | The timeline (shots at ms, music length) is E. `MapPan` is K. `Characters`, `Talk` and `Combat` are G. |
| `lib.rs` | G | The `Content` struct. Its "run every loader, collect every error" pattern is E. |
| `cinematic/{action,combat}.rs`, `battler.rs`, `chapter.rs`, `battle.rs`, `character.rs`, `class.rs`, `item.rs`, `spell.rs`, `skill*`, `art.rs`, `support.rs`, `ai.rs`, `trigger.rs`, `enums.rs` | G | The game's data. |

## `ui` (trpg-ui)

| Module | Side | Note |
| ------ | ---- | ---- |
| `glyph_buffer*`, `snapshot.rs`, `console.rs`, `cinema.rs`, `portrait.rs` | E | See couplings 4 and 5. |
| `storage.rs`, `audio.rs` | E | `audio.rs` assumes four menu cue ids exist. Make them settings. |
| `widgets/` | E | `help.rs:82-90` (select and end-turn helpers) is G. |
| `input/pad.rs` | E | — |
| `debug/sprite_test.rs`, `screens/mod.rs` | E | — |
| `input.rs`, `input/bindings` | M | The fixed Esc→Cancel (`:33`) and `select_action`/`end_turn_accept_actions` (`:319-339`) are G. |
| `screen.rs` | M | `Screen`, `Transition`, `FrameInput` and `ScreenStack` are E. `Ctx` is coupling 2. |
| `game.rs` | M | Frame loop is E. Start-up and debug wiring are G (about 350 lines). |
| `color.rs`, `settings.rs`, `words.rs`, `tips.rs` | M | Engine parts: `Rgb`/`Palette`, volumes/fullscreen/text speed, text by key, `{Action}` fill. The tactics colors, settings and names, and tip placement, are G or K. |
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
| `render.rs`, `keys.rs`, `pads*` | E | `trpg_pad_*` JS names are renamed. |
| `storage/` | E | `APP_DIR_NAME = "tactical-rpg"` (`native.rs:15`) and the web `PREFIX` become settings. |
| `audio*` | E | Uses content's `AudioManifest`, `MUSIC_DIR` and `VOICE_DIR`. |
| `main.rs`, `build.rs` | M | Game parts: window title, icon, `Ctx::embedded`, start screen, the release check on test files, exe strings. |
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
