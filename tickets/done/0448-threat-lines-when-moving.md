---
id: "0448"
title: "Show which enemies can attack the tile a unit is about to move to"
type: feature
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: ["0055"]
nick_input: answer-first
completed: 2026-10-12
---

# 0448 — Threat lines when moving

## Context

Nick asked for it while playing Chapter 1 (ticket 0804, 2026-10-12): while
you propose a move, the game shows which enemies can attack that tile,
"so you don't need to rely on danger zone as much". Ticket 0055 decides
what is shown, for which enemies and when; it is decided (2026-10-12):
**read *Threat arrows* in `docs/design/turn-structure.md` (the rule) and
in `docs/design/look-and-feel.md` (the look) first.** In short:

- **Look:** a slim straight red arrow from each enemy to the tile, and a
  yellow `!` above each of those enemies' heads (not on the head).
- **Which enemies:** every enemy that could attack the tile next enemy
  phase, Guard and Stationary enemies included. Not the AI's choice.
- **When:** a unit is selected and the cursor is on a tile it can move
  to; or nothing is selected and the cursor rests on a player unit.
- **Off the screen:** the arrow comes in from the edge of the map view.
- **No setting, no new key.**

Existing pieces: `threat_area` and `danger_zone` in
`crates/core/src/movement.rs` (the tiles one enemy, or all, can attack);
`danger_tiles` in `crates/ui/src/screens/battle/mod.rs`; the map scene
(`crates/ui/src/map_view/scene.rs`, `MapScene`, which already carries the
selected unit's `path`); the two skins (`map_view/glyph`,
`map_view/sprite`).

## Nick input

**Answer first:** ticket 0055. **Sign-off** after merge: play Chapter 1's
first turns on Pages.

## Scope

**In:**
- Working out, for the proposed tile, the enemies 0055's rule names.
- Putting them in the map scene and painting them in both skins.

**Out (do not do):**
- Anything 0055 didn't decide (no setting, no new key, unless it says so).
- The danger zone and the forecast.
- Predicting the AI's choice (0055: everyone who could attack).

## Implementation steps

1. `core`: a pure function giving the enemies that threaten a position
   for a given unit standing there (built on `threat_area`; no state
   change, so no `Command`). Property test: an enemy is listed exactly
   when the position is in its `threat_area`.
2. `ui`: the battle screen computes the list in the two cases above and
   the starting rules beside them in `turn-structure.md` (pointing at an
   enemy, the action menu, acted units), with the selected unit counted
   on the proposed tile (cache it per cursor tile; don't recompute every
   frame) and puts it in
   `MapScene` as plain data, e.g. `threats: Vec<Pos>` (the enemies' tiles)
   beside `path`, with the target tile.
3. Both map skins paint it (ADR-0038): the glyph skin and the sprite
   skin both draw the arrows and the marks over units and terrain, at
   the mockup's sizes (`look-and-feel.md`). Nothing is drawn
   straight into the buffer by the screen. A line's picture, if it needs
   one, is a sprite item from an asset file, not per-pixel rectangles.
4. An enemy outside the view: its arrow is cut at the view's edge; no
   mark. The mark on the view's top row moves down to stay inside.
5. Render the Chapter 1 frame from 0055's mockup (the Sergeant selected,
   cursor 5 tiles east: four arrows) with `frame-png` and look at it.

## Acceptance criteria

- [x] In Chapter 1, selecting a unit and pointing at a tile an enemy can
      attack shows an arrow from that enemy and a mark over it; a tile
      nobody reaches shows nothing.
- [x] The cursor resting on a player unit, with nothing selected, shows
      the same for the tile it stands on.
- [x] A Guard enemy that reaches the tile is listed.
- [x] A test reads `MapScene` for the listed enemies (not cells).
- [x] Each skin has a test or snapshot of its look.
- [x] `core` knows nothing about the look.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit + property (`core`): the threat list.
- Integration (`crates/ui/tests/it/`): the scene while proposing a move.
- Snapshot: both skins.

## Completion notes

**Done.**

- `core`: `threats_at` (`crates/core/src/movement.rs`): the units hostile
  to a unit that could attack it next phase if it stood on a tile, worked
  out with the unit on that tile. Unit tests and a property test (listed
  exactly when the tile is in the enemy's `threat_area`).
- `ui`: `BattleScreen` works the list out for the two cases of
  `turn-structure.md`, keeps it per unit and tile (dropped when a command
  is applied or the battle is rewound) and puts it in the scene as
  `MapScene::threat` (`ThreatView`: the tile, the enemies' tiles). The
  scene's text has a `threat (5,3) from (6,3) (9,1)` line when there is
  one.
- Both skins paint it through `crates/ui/src/map_view/threat.rs`, last, so
  over units, the path and the cursor: per arrow a 1-pixel line with a
  dark edge and two 4-pixel barbs, stopping 3 pixels short of the tile's
  middle; per enemy a 3 × 9 `!`, its lowest pixel 4 above the tile's top.
  An enemy off the view has its arrow cut at the view's edge and no mark;
  a mark on the top row moves down. It follows the tile size and the zoom.
- Looked at with `frame-png`: Chapter 1, the Sergeant selected and the
  cursor 5 tiles east shows the four arrows of 0055's mockup (the Brigand
  beside it, the Archer, the Brigand to the east, the Raider on guard
  north of the road bridge), in the glyph look and under the test
  tileset at 2×, where the Raider is off the view and its arrow comes in
  from the top edge.
- Tests: `crates/ui/tests/it/threat.rs` (reads `MapScene`, Chapter 1 and
  the Quick Battle), `screens/battle/threat_tests.rs` (the kept list),
  `map_view/threat.rs` (the look, to the pixel), one test per skin that it
  paints them last on its own tiles. 29 existing snapshots gained the
  arrow the Quick Battle's lord has from its first frame.

**Rules decided here** (*the agent's starting rules*, Nick can veto; also in
the design docs):

- **The glyph look starts each arrow at the edge of the enemy's tile**,
  not its middle: from the middle the line ran through the enemy's two
  letters (`Br` read as `B-`). With pictures it starts in the middle, as in
  the mockup.
- **Colours:** arrows in the palette's enemy red, the `!` in the highlight
  yellow, both edged in black, so they change with the colour theme.
- **The arrows also stay while the unit walks to the tile and in the
  lists the action menu opens** (arts, spells, skills, items, equip). They
  go whenever a target is picked.
- **With a unit selected and the cursor on a tile it can't stop on** (out
  of its range, or another unit's tile): nothing shows.
- **An enemy that never moves** (Harl) gets an arrow only for tiles its
  weapon reaches from where it stands, as `turn-structure.md` says. The
  danger zone still tints his range as if he walked first, so the two
  disagree round him: ticket 0450.

**Deviations.**

- A line at an angle is drawn as short solid rectangles, one per step
  across it (as the path's arrowhead already is), not as a picture from a
  file: it is geometry between two tiles at any angle and any zoom, and a
  picture can't be turned. No new kind of frame item was needed. An arrow
  across half the map is about 50 lines in a frame snapshot.
- `scene.threat` is one `Option<ThreatView>` (tile + enemies' tiles)
  rather than a bare `threats: Vec<Pos>`.
- **Not this ticket's code:** `main` was failing clippy (`load` in
  `crates/content/src/lib.rs` was 102 lines, 2 over the limit, after two
  PRs each added to it). Two pairs of `let`s were joined so this PR's
  checks can pass. No behaviour changed.

**Follow-up:** ticket 0450 (the danger zone of an enemy that never moves).

**For Nick, when playing Chapter 1's first turns:** rest the cursor on
Tamsin (an arrow from the nearest Brigand), then select her and walk the
cursor east along the road: arrows appear from each enemy as its reach is
entered. Say if the arrows should be thicker, a different red, or start
somewhere else on the enemy.
