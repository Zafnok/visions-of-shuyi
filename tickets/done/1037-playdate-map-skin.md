---
id: "1037"
title: "Playdate: the one-bit map skin (yours black, foes grey, acted outlined, ranges as dots and lines), and choosing the Playdate skins"
type: feature
milestone: Post–Chapter 1
model: opus-5.5
effort: high
status: done
blocked_by: ["1032", "1033"]
nick_input: sign-off
completed: 2026-10-17
---

# 1037 — The Playdate map skin

## Context

The battle map is a skin (ADR-0038): the battle screen builds a
`MapScene` (`crates/ui/src/map_view/scene.rs`) and `ctx.map_skin`
(`Rc<dyn MapSkin>`, `map_view/skin.rs`) paints it; the glyph skin is in
`map_view/glyph*`. The Playdate look is a new map skin
(ADR-0070 §4), painted with 1032's one-bit effects, as decided in
`docs/design/playdate.md` (pictures: `docs/design/playdate/q1-a-8x16.png`,
`q3r2-e-grey.png`):

- Tiles stay **two 8 × 16 cells wide** (16 × 16 px), the same glyphs as
  the PC (`..`, `♣♣`, `≈≈`, `^^`, `╦╦`, …), black on a **white** ground.
  The map fills the screen: the design says 24 × 15 tiles at once
  (Chapter 1's 24 × 16 map scrolls by one row with the cursor).
- **Your units:** the two letters **white in a solid black tile**.
- **Enemies:** the two letters on a **grey tile** (50% checker), each
  letter with a white edge.
- **A unit that has acted:** its tile becomes an **outline** with black
  letters.
- **HP:** the line along the bottom of the tile, as long as the HP left.
- **Move range:** light dots; **attack range:** diagonal lines; **danger
  zone:** a dashed line along its edge, showing over the ranges. Terrain
  glyphs on a marked tile keep a white edge.
- **Cursor:** the PC's corner marks, drawn by flipping the pixels they
  cover. Threat arrows and their `!` marks: black with a white edge.
- *Claude's starting rules* in the design doc (allies and neutrals wait
  for a later chapter) stand.

## Nick input

**Sign-off:** Nick looks at the PR's `frame-png --playdate` pictures of
Harrowby Ford (browsing, a unit's ranges, the danger zone, an acted unit)
and, if he has time, the Simulator or his Playdate. He compares them to the
0061 mockup he picked.

## Scope

**In:**
- **Choosing the Playdate's skins:** the front end (1033) puts the
  Playdate map skin in `ctx.map_skin`; add a `Ctx` value naming the skin
  set (e.g. `SkinSet::Glyph` / `SkinSet::Playdate`) that the screen skins
  of 1038 onwards will read in their `draw`. PC unchanged (it stays
  `Glyph`).
- **`PlaydateSkin`** (`crates/ui/src/map_view/playdate.rs` and
  submodules), implementing `MapSkin`: `view_tiles` for a 50 × 15 area,
  painting terrain, units, HP lines, ranges, danger zone, path, cursor,
  threat arrows and `!` marks with 1032's inks and items, as above. It
  paints the same `MapScene` the glyph skin does; nothing in `MapScene`,
  `core`, the bots or play records changes (ADR-0038). If a mark needs
  data the scene lacks, add it to the scene for both skins.
- **Look tests live with the skin** (ADR-0038: only tests of a look read
  cells and pixels): one per mark above, through 1032's renderer, and
  snapshot PNG/text of Harrowby Ford's opening view.
- `cargo xtask frame-png --playdate` can render a battle map scene (a
  test battle or Chapter 1's first battle at turn 1) so the PR shows it.

**Out (do not do):**
- The battle screen's layout, pop-ups and menus (1038): the battle's map
  area is still the PC layout's until then, so in the Simulator the map
  is cut off at 50 × 15. That is expected.
- Animations beyond what the glyph skin does (walks, the danger-zone
  toggle): same timing as the glyph skin.
- Map zoom (`MapZoom`): the Playdate skin can't zoom (`at_zoom` → `None`).
- Any change to the PC's map skins or look.

**Carve (1115):** either order works. If 1115 moved the grid map scene
and skins to the engine, the Playdate skin is the game's skin over the
engine's scene, as the glyph skin is.

**Bought tiles and sprites (0067):** either order works. 0067 asks Nick
whether the Playdate map uses the bought tiles and unit sprites in
one-bit instead of glyphs. If it does, they come as a second Playdate
map skin in their own ticket, and this glyph skin stays for builds
without the bought art; so keep `SkinSet` and the front end's choice of
map skin open to a second Playdate map skin, and build this one as
written.

## Implementation steps

1. Read `docs/design/playdate.md`, ADR-0038, 1032's ADR,
   `map_view/skin.rs`, `map_view/glyph.rs` and `glyph/`.
2. `SkinSet` in `Ctx`; the front end sets it and the map skin.
3. `PlaydateSkin`: terrain and units first, then ranges, danger, cursor,
   path and threat marks, each with its look test.
4. The `frame-png` scene and the PR pictures.

## Acceptance criteria

- [x] `frame-png --playdate` pictures of Harrowby Ford (browsing, ranges,
      danger zone, an acted unit) are in the PR.
- [x] Each mark in *Context* has a look test with the skin.
- [x] No `MapScene` change breaks the glyph skin's tests; no PC snapshot
      changes.
- [x] Nick's sign-off comment is in the completion notes (or "no comment
      yet": sign-off never blocks the PR).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit (look, with the skin): each mark; the opening view snapshot.
- Unit: `view_tiles` for the 50 × 15 area.

## Completion notes

**Done.**

- **`SkinSet`** (`Glyph` / `Playdate`) in `Ctx::game.skin_set`, and
  `map_view::skin_for(set, look, content)`, the one place the map skin is
  chosen: the settings' map look on the PC, the `PlaydateSkin` on the
  Playdate whatever the look (so loading the player's settings no longer
  swaps it out). `ShuyiCtx::use_playdate_skins` sets both; the front end
  (`crates/playdate/src/start.rs`) and `frame-png --playdate` call it.
  ADR-0083 records it, and how 0067's second Playdate map skin would slot
  in.
- **`PlaydateSkin`** (`crates/ui/src/map_view/playdate.rs`, `units.rs`,
  `marks.rs`): 24 × 15 tiles on the 50 × 15 screen, centred with a cell
  clear each side; terrain glyphs black on white; your units white in a
  black tile, enemies white-edged on grey, acted units outlined; HP line;
  move range dots, attack range lines, danger zone a 2 px dashed edge;
  white-edged terrain glyphs on a range; the PC's corner marks as flips;
  the shared path and threat arrows (black, white edge). Its inks are
  new `map_*` names in `assets/data/palettes/playdate.ron`, falling back
  to the design's ink. Nothing in `MapScene`, `core`, the bots or play
  records changed.
- **Look tests** with the skin, through the one-bit renderer: each mark,
  the 50 × 15 `view_tiles`, a paints-only-its-area property test, and a
  braille text snapshot of Harrowby Ford's opening view
  (`map_view/playdate/snapshots/`).
- **`cargo xtask frame-png --map`** paints the battle's map alone over the
  whole screen (`BattleScreen::scene_of_size`, `Harness::map_scene_in`),
  since the battle screen keeps the PC's layout until 1038. The PR's
  pictures are `docs/design/playdate/1037-*.png`, made with it.

**Deviations.** The glyph skin's cursor arms share a pixel at each corner;
flipped twice it would vanish, so the Playdate's gives it to one arm only.
The glyph skin's `Layout` gained `fitted` (no pixel of the PC changes).

***Claude's starting rules*** (in `docs/design/playdate.md`; Nick can
veto): the map centred with a cell each side; ranges don't show on units'
tiles; the heal range is the attack range's lines; timed effects (buffs,
debuffs) have no mark on the Playdate map; allies and neutral units are
their letters alone (their HP line as yours); the danger edge is 2 px inside the zone and absent at
the screen's edge; a battle note's unit, a struck unit and a changed tile
are flipped for the first half of their flash; a falling unit vanishes at
the middle of its fall; the cursor doesn't pulse, and the tile-glow cursor
option flips the whole tile.

**Follow-ups:** none.

**Nick's sign-off (2026-10-17):** on the HP line: "lets look at the hp can
we dither or use a box outline or smth?" Shown four options
(`docs/design/playdate/1037-hp-options.png`): "lets have A for enemy, but C
for ally. This gives more contrast for readability". Built: enemies keep
the black line with the HP lost white; your units' line is in their
letters' colour with the HP lost dithered grey (`docs/design/playdate.md`,
*The HP line*). The Harrowby Ford snapshot now plays the real game to the
battle (the first one built the battle's start state, before its opening
events bring most units in).
