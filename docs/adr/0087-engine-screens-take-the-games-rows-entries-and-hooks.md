# ADR-0087: The engine's screens and widgets take the game's rows, entries and hooks

- **Status:** Accepted
- **Date:** 2026-10-17
- **Related tickets:** 1112
- **Extends:** ADR-0054 (logic, view, skin), ADR-0080 (`Ctx<G>`,
  `GameData`). ADR-0072's options (previewed, then kept or put back) are
  the engine's options screen's, unchanged. ADR-0026's menu sounds keep their meaning; which cues they
  are becomes the game's.

## Context

`docs/engine/boundary.md` lists screens and widgets that are generic with
this game's rows or details mixed in: the menu widget and word wrap, the
Key bindings screen (only its rows are the game's since 1106), the layout
picker (its legend), the Options screen (a shell with this game's rows),
the debug menu (some tools are the game's), the dialogue player and text
box (the lead and the cast are the game's) and the one-time tips' seen
list. The menu played four cue ids that are this game's (`menu_move`…).

Each screen already said what it shows as a view and had a glyph skin
paint it (ADR-0054); each skin painted with the game's colors
(`UiColor::PanelBorderFocus`, `Black`, `Player`, …).

The engine boundary check (ADR-0063 §4) fails on any of the game's ids in
`engine/`, sound cue ids among them. Mutation testing runs each mutant
against the tests of the crate it is in, so code moved to the engine needs
engine tests; the game's tests no longer count for it.

## Decision

1. **Data in, not callbacks on a trait, where the game's part is a list.**
   - Key bindings: `KeyBindingsScreen::new(ctx, rows: &'static [KeyRow<A>])`.
   - Layout picker: `new(ctx, legend)` / `change(ctx, legend)`, a legend
     being lines of `LegendKeys::{Cursor, All(a), First(a)}` and a text key.
   - Options: `OptionsScreen::new(rows: fn(&Ctx<G>) -> Vec<OptionRow<G>>)`.
     A row is a label key (which also names it: the focus, `view.row(key)`)
     and a typed value: `Toggle { get, set }`, `Choice { values, get, set }`
     (by index into the text keys of the values offered now), `Volume`
     (0–`MAX_VOLUME`, slider and number box) or `Action { value, confirm,
     help }`, whose `confirm` returns `RowAction::{Open(screen),
     Ask(Question), Refuse(message)}`. Setters return whether anything
     changed, which decides the sound. A row marked `unsaved` isn't a
     setting: the screen takes its value when it opens and puts it back on
     Discard, so leaving asks about it like a setting. The engine's own
     settings come as ready-made rows (`text_speed`, `fullscreen`,
     `music_volume`, `sound_volume`, `restore_defaults`). Rows are built
     each frame, so which show can depend on the context.
   - Debug menu: `DebugMenuScreen::new(ctx, Vec<DebugEntry<G>>)`; an entry
     is a label (fixed, or read from the context: "Map skin: glyph") and a
     boxed `FnMut(&mut Ctx<G>) -> Transition<G>`, which keeps its own state
     (the voice test's count). A tool that opens nothing leaves the menu on
     it with its labels read again.
2. **A trait where the game changes how something plays.** The dialogue
   player is `DialoguePlayer<H: DialogueHooks>`: `H::Directive` (the game's
   own lines, the `.dlg` format's `Directive` of ticket 1105), `H::Stage`
   (what those lines have set: a place, an opening) and `fill` (the game's
   tokens). `Plain` is a game
   with none. The game wraps the player in a newtype with its old API, so
   its dialogue screen, log, cinematic talk shot and tests are unchanged.
3. **The text box, not the dialogue screen.** The engine has the
   typewriter (`TextBox`: pages, typing at the text speed, the
   fast-forward, the blinking marker), its view (`TextView`, `SkipView`,
   `WaitView`) and its glyph skin (`text_box::glyph::paint` at a `BoxPlace`
   the game gives). The dialogue screen stays the game's while it draws
   places and openings, which wait on the map view (1115) and cinematics
   (1113).
4. **Menu sounds by meaning, cues by the game.** `shuyi_ui::audio::MenuSound`
   (Move, Select, Cancel, Denied) and `AudioConfig { move_cue, select_cue,
   cancel_cue, denied_cue, cursor_cue }` from `GameData::audio_config`
   (default `ui_move`, `ui_select`, `ui_cancel`, `ui_cursor`, Denied playing
   `ui_cancel`). The `AudioQueue` holds the config, so `ctx.audio.menu(…)`
   and `Menu::handle_with_sound(action, &mut ctx.audio)` keep their shape.
   This game's `AUDIO` names its own cues.
5. **Skins paint with engine colors.** `EngineColor` gains `Focus`,
   `Background`, `Accent` and `Bright`; the game says which of its colors
   they are (`panel_border_focus`, `black`, `player`, `white`), so every
   cell keeps its color and every snapshot its text.
6. **Tests move with the code.** Each moved module's unit tests are ported
   to the toy game (`shuyi_ui::toy`, whose keymap is this game's less the
   Playdate's actions and whose English has the screens' words). The game
   keeps the tests of its own rows, legend and tools, and its integration
   snapshots, which show that nothing a player sees changed.

## Consequences

- A second game gets the menu, the options, Key bindings and layout
  picker screens, the debug menu, the dialogue player and text box by
  giving lists and a few closures.
- Rows are `fn` pointers (non-capturing closures): no per-row state, but
  `Clone`-free and cheap to rebuild every frame. A row that needs the
  game's state reads it from `ctx.game`.
- Generic screens can't derive `Clone`/`Debug` (the derive would ask the
  game's type for them); they implement them by hand.
- The game's skins of these screens are now the engine's glyph skins. The
  Playdate paints (1038–1046) stay the game's.
- `focus` on the options screen takes the context (it is the first shown
  row until the cursor moves), and its rows are named by text key, which
  the game turns its `Row` enum into (`From<Row> for &'static str`).

## Alternatives considered

- **An `OptionsGame` trait on `GameData`** (rows, unsaved state, hooks):
  one more trait every game implements even for one screen; the `unsaved`
  row flag covers this game's one non-setting row without it.
- **Moving the whole dialogue screen now:** it draws the map scene behind
  the talk and plays openings; moving it before the map view and the
  cinematics would move this game's map types into the engine.
- **Keeping the skins in the game and moving only logic and views:** the
  toy game, and any second game, would get screens with no look.
- **Keeping the game's cue ids as the engine's defaults:** the boundary
  check rightly refuses them.
