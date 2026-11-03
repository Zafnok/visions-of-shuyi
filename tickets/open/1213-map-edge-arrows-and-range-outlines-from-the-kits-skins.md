---
id: "1213"
title: "Map edge arrows and the danger zone's outline from the kit's skins"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: blocked
blocked_by: ["engine#3203"]
nick_input: none
completed:
---

# 1213 — Map edge arrows and the danger zone's outline from the kit's skins

## Context

The game/engine boundary audit of 2026-11-03 found that 0449's arrows
at the map's edges (`docs/design/look-and-feel.md`, *Arrows at the map's
edges*) are painted by the game. The kit's skins don't paint
`MapScene::more`, so the game paints it from its unit painters'
`paint_over`. That code is `map_view/edges.rs`, `GlyphUnits::paint_over`
and `edge_arrow` in `map_view/glyph.rs`, and `SpriteUnits::paint_over`,
`EDGE_GAP`, `edge_src` and `edge_dest` in `map_view/sprite.rs`. The
Playdate skin draws the danger zone as its outline in `OneBitUnits::paint_under`
(`map_view/playdate/marks.rs`, `danger_edges`). None of this names a game
concept.

[engine#3203](https://github.com/Zafnok/shuyi-engine/blob/main/tickets/open/3203-the-kits-skins-paint-edge-arrows-and-a-range-as-its-outline.md)
adds the painting to the kit: `GlyphStyle.edges` (`EdgeArrows`, `EdgeShow`),
`SpriteSkin::with_edge_marks` (`EdgeMarks`) and `GlyphStyle.outline`
(`RangeOutline`). This ticket moves the game onto them. Needed first.

## Nick input

None. The looks don't change: Nick's B2 fade (full to 35% and back once
a second, in the cursor's colour) and the dashed danger edge
(`docs/design/playdate.md`) stay as the game's settings. Tests and the
snapshots prove it.

## Scope

**In:**
- The engine pin past engine#3203's merge.
- The kit's options set on `GLYPH`, `ONE_BIT` and the sprite skin. The
  game's arrow and outline painters are deleted.

**Either order with 1178:** if 1178 is done, `OneBitUnits::paint_over`
paints the Playdate's arrows. Delete it and set `ONE_BIT`'s `edges` to
the look 1178 recorded in `docs/design/playdate.md` (steady:
`EdgeShow::Steady`; blinking: `EdgeShow::Blink { period_ms: 1000 }`), in
the palette's ink 1178 used. If not, set `ONE_BIT`'s `edges: None` (no
arrows, as today), and 1178 sets it.

**Either order with 1076:** if 1076 is done and drew the exit area as
an outline in `map_view/playdate/marks.rs`, add `range::EXIT` to
`ONE_BIT`'s `RangeOutline` (if its thickness or ink differs from the
danger edge's, keep its painter and say why in the Completion notes). If
it chose another mark, or isn't done, nothing to do (1076 can use the
outline option).

**Either order with 1232:** if 1232 is done, it deleted other tests in
`map_view/sprite.rs` and `glyph/tests.rs` and left the three edge-arrow
tests here: change or delete them as step 6 says; if not, nothing to do
(1232 leaves the edge-arrow tests alone).

**Either order with 1233:** if 1233 is done, its `map_view*` row says
"moves with 1213": write it as moved in step 7; if not, edit the rows as
step 7 says, and 1233 leaves them.

**Out (do not do):**
- When the battle screen sets `more`, and the camera (0449's, 0072's).
- Any change to how the arrows or the outline look.

## Implementation steps

1. Move the engine pin past engine#3203's merge (`cargo xtask
   engine-pin`, the `engine-pin` skill's review of the PRs it passes),
   unless an open `engine-pin` PR has done so.
2. `map_view/glyph.rs`: `GLYPH` is styled with `edges: Some(EdgeArrows
   { glyphs: ['→', '↓', '←', '↑'], color: UiColor::Cursor.name(), show:
   EdgeShow::Fade { period_ms: PULSE_MS, faintest: FAINTEST } })` on top
   of `GlyphStyle::PLAIN`. `PULSE_MS` and `FAINTEST` move here from
   `edges.rs`, still *Tunable*. Delete `GlyphUnits::paint_over` and `edge_arrow`.
3. `map_view/sprite.rs`: `MapSkin::sprite(tileset, edge_marks:
   Option<ImageId>)` calls `with_edge_marks(EdgeMarks::new(id,
   EDGE_BOXES, 1, <GLYPH's show>))` when it has the image. `EDGE_BOXES`
   holds `edge_src`'s four boxes in `Dir::ALL` order. `map_view.rs`
   `skin_named` passes `content.images.id(EDGE_MARKS_PATH)`. Delete
   `SpriteUnits::paint_over`, `EDGE_GAP`, `edge_src` and `edge_dest`.
4. `map_view/playdate.rs`: `ONE_BIT` gets `outline: Some(RangeOutline {
   ranges: &[range::DANGER], px: DANGER_EDGE })` and keeps `skip`.
   `DANGER_EDGE` moves here. Delete `OneBitUnits::paint_under`,
   `marks.rs` and `marks/tests.rs`. Set `edges` as the 1178 clause says.
5. Delete `map_view/edges.rs` and its `mod`. Fix the module docs that
   name it.
6. Tests: keep one test per skin that the game's settings reach the
   frame. That is the glyph arrow's cell in the cursor's colour at 0 and
   500 ms, the sprite arrow from `EDGE_MARKS_PATH` above the HP bar, and
   the danger edge's rectangles. Delete the placement and fade tests.
   The engine has them now.
7. `docs/engine/boundary.md`: the `map_view/glyph*`, `map_view/sprite*`
   and `map_view/playdate*` rows say the kit paints the arrows and the
   outline.

Expected size: about 140 production lines removed and 30 added (three
style settings, the boxes, the image passed in). About 250 test lines
removed. Measure with `cargo xtask code-lines`.

## Acceptance criteria

- [ ] `crates/ui/src/map_view/edges.rs` and `map_view/playdate/marks.rs`
      are gone, and `grep -rn "paint_over" crates/ui/src/map_view` finds
      no edge arrows.
- [ ] Every map snapshot (glyph, sprite, sprite units, Playdate) is
      unchanged, or changes only in the order of the arrows' sprite items.
- [ ] The kept tests of step 6 pass.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: step 6's tests through `MapSkin::paint`.
- Property: each skin's paints-only-the-area property with `more` set
  still holds.
- Snapshot: the existing map snapshots, unchanged.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
