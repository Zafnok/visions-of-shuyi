---
id: "1037"
title: "Playdate: the one-bit map skin (yours black, foes grey, acted outlined, ranges as dots and lines), and choosing the Playdate skins"
type: feature
milestone: Post–Chapter 1
model: opus-5.5
effort: high
status: todo
blocked_by: ["1032", "1033"]
nick_input: sign-off
completed:
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

- [ ] `frame-png --playdate` pictures of Harrowby Ford (browsing, ranges,
      danger zone, an acted unit) are in the PR.
- [ ] Each mark in *Context* has a look test with the skin.
- [ ] No `MapScene` change breaks the glyph skin's tests; no PC snapshot
      changes.
- [ ] Nick's sign-off comment is in the completion notes (or "no comment
      yet": sign-off never blocks the PR).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit (look, with the skin): each mark; the opening view snapshot.
- Unit: `view_tiles` for the 50 × 15 area.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
