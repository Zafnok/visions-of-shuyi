---
id: "1112"
title: "Generic screens and widgets move to the engine: menus, help, key bindings, layout picker, options shell, debug menu, dialogue player, text"
type: infra
milestone: "Engine: carve"
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: ["1111"]
nick_input: none
completed: 2026-10-17
---

# 1112 — Generic screens and widgets move to the engine

## Context

`docs/engine/boundary.md`, *ui*: these are generic with this game's rows
or details mixed in:

- `widgets/` (menu, help, wrap);
- `screens/key_bindings` (only its `ROWS` are the game's; 1106 already
  made them injected);
- `screens/layout_picker` (layouts are data after 1106);
- `screens/options` (a generic shell with this game's rows);
- `debug*` (the menu, glyph sampler, portrait viewer, backdrop camera are
  generic; promote/reclass, skin switch, test cinematic and scene preview
  are the game's);
- `dialogue.rs` and `screens/dialogue` (the player state machine and the
  box are generic; the lead and cast are the game's);
- `words.rs` (text by key; the names of classes and items are the
  game's);
- `tips.rs` (`{Action}` fill and the seen-set are generic; placing the
  tip on the battle map is the game's);
- `audio.rs` (assumes four menu cue ids exist), `storage.rs`.

Each screen already follows ADR-0054 (logic, view, skin) or is moved so
it does.

## Nick input

None. Nothing a player sees changes.

## Scope

**In:**
- Move the generic parts to `shuyi-ui::{widgets, screens, debug,
  dialogue, text, audio, storage}`; the game injects rows, entries,
  cue ids (`AudioConfig { menu_move, menu_select, menu_cancel,
  cursor_move }`) and the lead/cast hooks into dialogue
  (`DialogueHooks`).
- The debug menu takes entries from the game (`Vec<DebugEntry<G, A>>`).
- The options screen takes rows from the game: each row a label key and
  a typed value (`Toggle`, `Choice`, `Volume`, `Action`).

**Playdate:** the Playdate port (1025's tickets, 1027–1053) builds
`core`, `content` and `ui` without `std` and adds a second front end,
`crates/playdate`. If those tickets have landed, code this ticket moves
keeps its default `std` feature and stays in the `no-std` CI build, and
`cargo xtask playdate` keeps building (ADR-0070 §6). The Playdate paints of moved screens (1038–1046) stay the game's skins over them.

**Out (do not do):** cinematics (1113); the map view (1115).

## Implementation steps

1. Move one module at a time, each with its tests; inject what is the
   game's; keep aliases.
2. Snapshots must not change. If one does, the move changed a look: fix
   the move, never accept the snapshot.

## Acceptance criteria

- [x] The toy game (1111) uses the engine's menu, options shell, key
      bindings screen and dialogue box in a Harness test.
- [x] No snapshot changes; every test passes; `check-keys` and
      `check-text` pass.
- [x] `docs/engine/boundary.md` rows updated.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Integration: toy-game Harness tests for each moved screen.

## Completion notes

- **`shuyi-ui` gains** (ADR-0087):
  - `widgets`: the menu (with its view and glyph skin) and word wrap.
  - `screens`: `key_bindings` (the game gives its rows, `KeyRow<A>`),
    `layout_picker` (the game gives its legend: `LegendKeys::{Cursor,
    All, First}` and a text key per line), `options` (the game gives its
    rows, see below), `centre_x`, `print_centred`, `text_keys_help`.
  - `debug`: `DebugMenuScreen::new(ctx, Vec<DebugEntry<G>>)` (an entry is
    a label, fixed or read from the context, and a boxed closure that keeps
    its own state), split into a view and a skin; the glyph sampler (the
    game adds its sample panels); the Debug key's hint and its painter.
  - `dialogue`: `DialoguePlayer<H: DialogueHooks>` (the game's own lines,
    what they set, and its tokens; `Plain` is a game with none) and the
    text box (`TextBox`: pages, typing, the fast-forward and the blinking
    marker; `TextView`, `SkipView`, `WaitView`; a glyph skin at a
    `BoxPlace`).
  - `tips`: the tips seen (`TipsSeen`, `reset_tips`).
  - `audio`: `MenuSound` and `AudioConfig` from `GameData::audio_config`,
    held by the `AudioQueue`, so `ctx.audio.menu(…)` keeps its shape
    (`cursor_tick()` for the map cursor's tick).
  - `EngineColor` gains `Focus`, `Background`, `Accent` and `Bright`; the
    game names them `panel_border_focus`, `black`, `player`, `white`, so
    every cell keeps its color.
  - `text::key_text`: `GameData::key_text` now fills `{Cursor}` too by
    default (the generic part of the tips' fill); `Ctx::text_with_keys`
    fills text with another keymap's names (the Key bindings screen's,
    opened with).
- **The options shell**: `OptionsScreen::new(rows)`, `rows` a
  `fn(&Ctx<G>) -> Vec<OptionRow<G>>` built each frame. A row is a label
  key (which names it) and a `RowValue::{Toggle, Choice, Volume, Action}`;
  an action returns `RowAction::{Open, Ask(Question), Refuse(message)}`.
  The engine's settings come as ready rows. A row marked `unsaved` (this
  game's Tutorial level) is kept or put back on leaving like a setting.
  The game's `Row` enum stays, built into these rows (`options::rows`),
  and turns into its key for `view.row(Row::X)`.
- **The game keeps** aliases and thin builders: `key_bindings::screen`,
  `layout_picker::{first, change, LEGEND}`, `options::{screen, rows}`,
  `debug::{menu, sampler, glyph_sampler}`, and its `DialoguePlayer` as a
  newtype over the engine's with the old API, so the dialogue screen, the
  log, the talk shot and their tests didn't change.
- **Tests**: each moved module's unit tests moved with it, ported to the
  toy game (its keymap is this game's less the Playdate's actions; its
  English, `toy/en.ron`, has the screens' words). New engine tests for the
  dialogue player (with a toy `@lamp` line), the text box and its skin,
  the debug menu and sampler. The toy gained a `Lobby` (an engine menu)
  and a `Talk` (a scene in the engine's text box), its option rows, Key
  bindings rows and legend; `tests/it/screens.rs` drives the menu, the
  options (keep, a typed volume discarded, the layout through the
  picker), the Key bindings screen (a rebound key works after leaving)
  and the text box through the Harness. The game keeps the tests of its
  own rows, legend and tools. Engine: 603 unit and 48 integration tests;
  game `trpg-ui`: 1,412 and 356. No snapshot file changed.
- **Deviations from the ticket:**
  - `AudioConfig`'s fields are `move_cue`, `select_cue`, `cancel_cue`,
    `denied_cue` and `cursor_cue`, not `menu_move`…: those are this
    game's cue ids, which `check-engine-boundary` refuses in `engine/`.
    For the same reason the engine's defaults are `ui_move`… There is a
    fifth, Denied's, which this game sets to `menu_cancel` (until 0427).
  - `DebugEntry<G>`, one parameter: since 1109 the actions come from
    `G::Action`.
  - Not moved: the dialogue *screen* (portraits, the place behind the
    talk, openings and the log), which waits on the map view (1115) and
    cinematics (1113); the portrait viewer (this game's portrait frame,
    `portrait.rs`), the scene camera (the map scene) and the sprite test
    (this game's test card; the toy's gallery already covers sprites);
    typing a name (`name_entry`, this game's grid and rules). `words.rs`
    and `storage.rs` had nothing generic left (1109 moved text by key and
    storage).
- `cargo xtask check-keys`' list of keyboard-picture files still names the
  game's layout picker files; the picture is the engine's now, which the
  check doesn't scan (its comment says so).
- **Not checked locally:** `cargo xtask playdate` (the SDK's `pdc.exe`
  crashes on this machine, as in 1108 and 1109; CI's `playdate` job runs
  it) and mutation testing (CI).

No gameplay rules were decided; nothing a player sees changes.
