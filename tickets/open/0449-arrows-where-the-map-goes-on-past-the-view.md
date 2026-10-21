---
id: "0449"
title: "Little arrows at the view's edges where the map goes on past it"
type: feature
milestone: M3 Battle UI
model: opus-5.5
effort: medium
status: todo
blocked_by: ["1177"]
nick_input: decision
completed:
---

# 0449 — Arrows where the map goes on past the view

## Context

Nick, 2026-10-12, right after ticket 0439 (the 1× / 2× zoom) was built:

> "also when the map overflows the viewport, there should be little arrows
> that show you can move the map, and I guess it would just happen when you
> move the cursor past the viewport it moves"

Since 0439 a new player starts at 2×, where the view shows 17×15 tiles of
the bought art. Chapter 1's map is 24×16, so part of it is off screen (the
fort in the top right at the start), and nothing on screen says so.

Nick asked again in Chapter 1 playtest round 3 (0804, 2026-10-16): "we
need the map scroll arrows to ensure players know when it's cut off from
the whole picture. Otherwise 2x is ok to start on." 0804's round 4 waits
for this ticket; like the rest of round 3's tickets it waits for the
engine carve (1118) and the Playdate port (1050), Nick's priority.

**Re-gated 2026-10-21 (0129):** the carve is done and 1050 was folded
into the Act 1 playtest (1064), so neither 1118 nor 1050 is in
`blocked_by` any more.

**The second half of his sentence is how the game already works.** The
camera follows the cursor (`Camera::follow`, `crates/ui/src/screens/battle/
camera.rs`): the view scrolls once the cursor comes within 3 tiles of its
edge (`Camera::MARGIN`), and stops at the map's edge. This ticket doesn't
change scrolling; it only adds the arrows. Check it against what he meant
when showing him the mockups (below).

Rules this must follow:

- ADR-0038 (graphics are a skin): the arrows are something shown on the
  battle map, so they go in the map scene and every map skin paints them.
  The battle screen never draws them into the buffer.
- ADR-0018 and `docs/design/look-and-feel.md` for the glyph look.

**The scene and the skins' order are the engine's: engine#2304** (in
[Shuyi Engine's tickets](https://github.com/Zafnok/shuyi-engine/tree/main/tickets)).
Since the carve (1115) `MapScene` is the tactics kit's generic scene
(the game's `MapScene` is it with the game's units), and the glyph and
sprite skins are the kit's, which call the game's `UnitPainter` between
their layers. 2304 gives the scene the sides the map goes on past
(`more: MapEdges`, set by a helper from the view's origin and size
against the map's), lists it in `to_text`, and adds
`UnitPainter::paint_over`, painted over the units and under the cursor.
This ticket is the game's half: Nick's decision, setting the sides on
the battle screen, and painting the arrows he picks in that layer.

## Nick input

**Decision, asked at the start** (`ask-nick` skill, with rendered mockups:
Nick judges pictures, expect more than one round). Render Chapter 1's first
turn at 2× with the bought art (`cargo xtask frame-png`, built with
`--features private-assets`; the key route is in 0439's screenshots) with
each option drawn in, and Quick Battle under the glyph look on a map wider
than its view for the glyph version. Ask:

1. **What the arrows look like and where they sit.** Offer at least:
   one small arrow in the middle of each edge the map goes on past (like
   the scroll arrows of Advance Wars' map view and Wargroove's overview);
   an arrow in each such edge that pulses slowly (like Fire Emblem's
   blinking cursor arrows on its menus); a thin bright line along the
   whole edge instead of an arrow. Say honestly what each covers: an arrow
   sits on top of a tile at the edge, and a unit may stand there.
2. **Whether they show all the time or only while browsing** (hidden
   during the enemy's turn, a fight's playback and menus).
3. **Whether the scrolling is what he meant**: show him that the view
   already moves when the cursor nears its edge, and ask if that is it.

Record the answers in `docs/design/look-and-feel.md` (a new bullet under
*Battle map: bought tiles and unit sprites*, after *Zoom*, and the glyph
look's version where the glyph map is described).

No check after merge (Nick, 2026-10-21: he checks only design and the
full Act 1 playtest, 1064). Verification is by tests and `cargo shuyi
frame-png` pictures in the PR; what is left to judge by playing goes
on 1064's checklist (the debug menu's test tileset at 2×,
where Quick Battle's 14-tile map is wider than the 11-tile view).

## Scope

**In:**
- The map scene says on which of its four sides the map goes on past the
  view.
- Every map skin paints the arrows Nick picked there: the glyph skin, the
  sprite skin, and the sprite skin on glyph terrain.

**Out (do not do):**
- Changing how or when the view scrolls (`Camera::follow`, its margin),
  unless Nick's answer to question 3 asks for it: then write a ticket for
  that and leave it out of this one.
- A minimap, a "whole map" overview key, scrolling with the mouse.
- The cursor's look on bright tiles (0444), the players' outline (0447),
  the help bar's toggles line (0445): each either order works; none of
  them touches the view's edges.
- Zoom levels or the zoom key (0439, done).

## Implementation steps

1. **Pin**: 1177 moves the pin past engine#2304 (and adapts whatever
   the build needed); check `shuyi-engine.rev` has it, and do only what
   1177 left of the rest of this step.
   Add the new field to every place in the game that lists the scene's
   fields: `every_field_is_listed` and `features` in
   `crates/ui/src/map_view/sprite.rs` tests, the glyph skin's equivalent
   (`crates/ui/src/map_view/glyph/tests.rs`) and `any_scene`.
2. **Battle screen** (`BattleScreen::scene`,
   `crates/ui/src/screens/battle/mod.rs`): set the scene's `more` with
   engine#2304's helper from the scene's origin and size against the
   map's width and height. A map smaller than the view has none. If Nick
   answered "only while browsing", clear it in the other modes
   (`Mode::cursor_free` is the browsing test).
3. **Skins**: paint the arrows in the game's unit painters'
   `paint_over` (engine#2304), inside the area they are given, on top of
   the tiles and units, under the cursor.
   - Sprite skin (`crates/ui/src/map_view/sprite.rs`): as sprite items
     from an asset file (ADR-0038: never per-pixel rectangles). Add the
     arrow pictures to an image generated by an `xtask` like
     `effect_marks.rs` does for the effect arrows (our own drawing, so no
     license row beyond what that file already has), and scale them with
     `Grid::zoomed` so they double at 2× like everything else (0439).
   - Glyph skin (`crates/ui/src/map_view/glyph.rs`): arrow glyphs the
     font already has (`↑ ↓ ← →`, used for the D-pad's names), in the
     colour Nick picked from the palette, on the edge tiles' cells.
   - The mixed skin (sprite units on glyph terrain) uses the glyph skin's.
4. If Nick picked a pulse, drive it from the scene's `clock_ms` (as the
   effect arrows' bounce is), never from a clock in the skin.
5. Render both looks with `cargo xtask frame-png` and **look at them**: an
   arrow never covers a unit's HP bar or the cursor, and nothing is drawn
   outside the map area.

## Acceptance criteria

- [ ] Nick's answers are in `look-and-feel.md` with his words.
- [ ] On a map wider than the view, the scene's `more` is right at the
      left edge, in the middle and at the right edge of the map, and the
      same down the map (a scene assertion on the battle screen, at 1× and
      2× of the test tileset and under the glyph skin on a map bigger than
      35×30).
- [ ] A map that fits in the view has no arrows (Quick Battle under the
      glyph skin: its snapshot is unchanged).
- [ ] `every_scene_feature_is_painted` covers the new field under every
      skin.
- [ ] `the_skin_never_changes_the_game` still passes.
- [ ] Snapshots of Quick Battle on the test tileset at 2× with the arrows
      (the 0439 snapshots change: read them before accepting).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: which sides are set for a camera at each corner and in the middle
  of a big map, and none for a small one; each skin's arrows (where, which
  picture or glyph, their size at 2×).
- Property: whatever the scene and the area, the skins still paint
  nothing outside the area (the existing `…_paints_only_the_area` tests,
  with the new field generated by `any_scene`).
- Snapshot / integration: as listed; one Harness test that scrolls the
  cursor across a wide map and reads `h.map_scene()` for the sides.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
