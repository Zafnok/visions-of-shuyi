# ADR-0083: A skin set in the context picks the Playdate's skins; its map skin paints the shared scene in one-bit inks

- **Status:** Accepted
- **Date:** 2026-10-17
- **Related tickets:** 1037 (this), 1038 and later (the Playdate's screen
  skins), 0067 (the bought art in one bit), 1033 (the front end)
- **Extends:** ADR-0038 (graphics are a skin), ADR-0054 (a screen's view
  and its skin), ADR-0070 §4 (the Playdate's look is new skins), ADR-0074
  (the one-bit inks)

## Context

ADR-0070 §4 makes the Playdate's look new skins: a one-bit map skin, and a
one-bit `paint` for each screen's view, "chosen where `draw` chooses
today's glyph skin". The map skin lives in `Ctx::game.map_skin`
(`Rc<dyn MapSkin>`), but the settings choose it: `Shuyi::apply_settings`
puts in the skin of the player's map look (glyphs or the bought art's
pictures) whenever the settings load or that setting changes, so a
Playdate skin put there by the front end would be replaced the moment the
player's settings are read. And a screen's `draw` has nothing to ask
which look it is painting.

## Decision

- **`SkinSet`** (`trpg_ui::screen`): `Glyph` (the PC's, the default) or
  `Playdate`. It is a field of the game's part of the context,
  `Ctx::game.skin_set`. A screen's `draw` reads it to pick its skin (1038
  onwards); nothing in a screen's logic, `core`, the bots or play records
  reads it (ADR-0038).
- **`map_view::skin_for(set, look, content)`** is the one place the map
  skin is chosen: on `Glyph` the player's map look decides
  (`skin_for_look`, unchanged); on `Playdate` it is the `PlaydateSkin`
  whatever the look. `apply_settings` goes through it, so the settings
  never swap the Playdate's map skin out.
- **`ShuyiCtx::use_playdate_skins`** sets the set and the map skin. The
  front end calls it before loading the player's settings
  (`crates/playdate/src/start.rs`), and `cargo xtask frame-png
  --playdate` calls it, so its pictures are the device's.
- **The `PlaydateSkin`** (`map_view/playdate*`) paints the same
  `MapScene` as every skin, nothing added to the scene. It reuses the
  glyph skin's two-cell tile layout (`Layout::fitted`), terrain glyphs,
  HP bar lengths, cursor arms, path and threat arrows, and paints them in
  ADR-0074's inks: cell backgrounds for tiles and range fills,
  `BlackEdged` glyphs, `Under` rectangles for an acted unit's outline and
  the danger zone's dashed edge, `Invert` rectangles for the cursor and
  anything picked out. A tile is 16 × 16 pixels; the map is centred with a
  cell clear at each side (24 × 15 tiles on the 50 × 15 screen). It can't
  zoom.
- **Its inks are palette names** in `assets/data/palettes/playdate.ron`
  (`map_player_tile`, `map_enemy_letters`, `map_threat_edge`, …, as
  ADR-0074 §3 allows), each with the design's ink as the fallback when a
  palette lacks it, so the skin draws the same with the PC's palette in a
  test.
- **A second Playdate map skin** (0067's bought art in one bit, if Nick
  wants it) is a new arm in `skin_for` (for instance by the map look, as
  on the PC) and leaves `SkinSet` as it is.
- **Seeing the map alone:** until 1038 lays the battle screen out for the
  Playdate, its map area is the PC's and is cut off at 50 × 15.
  `cargo xtask frame-png --map` paints the battle's map over the whole
  screen with the map skin (`BattleScreen::scene_of_size`,
  `Harness::map_scene_in`), for pictures of the skin.

## Consequences

- The PC is unchanged: its set is `Glyph`, its skins and snapshots are as
  before.
- A Playdate screen skin is a `match ctx.game.skin_set` in that screen's
  `draw`, beside its glyph `paint`; the view stays the same data.
- Cursor corners are flipped, not painted: where the glyph skin's two arms
  of a corner share a pixel, the Playdate's gives it to one arm, or a
  double flip would hide the corner.
- One more place to keep in step: a new mark in `MapScene` needs paint
  code in the Playdate skin too (ADR-0038 §3 already says every skin).

## Alternatives considered

- **The front end sets `map_skin` and nothing else**: the first settings
  load replaces it with the glyph skin.
- **A "Playdate" map look in the settings**: the map look is the player's
  choice on the PC; the Playdate's look is the device's, not a choice,
  and a PC player could pick it.
- **Screens find the look from the console size or the palette**: implicit,
  and a Playdate-sized test console or a light theme would be mistaken for
  the device.
- **A second `MapScene` with ink-level marks** (fills, outlines): ADR-0038
  keeps one scene for every skin; everything the Playdate needs is in it.
