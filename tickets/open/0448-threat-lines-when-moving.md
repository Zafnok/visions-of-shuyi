---
id: "0448"
title: "Show which enemies can attack the tile a unit is about to move to"
type: feature
milestone: M3 Battle UI
model: opus-5.5
effort: high
status: todo
blocked_by: ["0055"]
nick_input: answer-first
completed:
---

# 0448 — Threat lines when moving

## Context

Nick asked for it while playing Chapter 1 (ticket 0804, 2026-10-12): while
you propose a move, the game shows which enemies can attack that tile,
"so you don't need to rely on danger zone as much". Ticket 0055 decides
what is shown, for which enemies and when; **read its answers in
`docs/design/turn-structure.md` and `look-and-feel.md` first.** The steps
below are the shape of the work; 0055 fills in the rule and the look.

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
- Predicting the AI's choice, unless 0055 picked that rule; if it did,
  say in the PR how it stays true to what the AI then does.

## Implementation steps

1. `core`: a pure function giving the enemies that threaten a position
   for a given unit standing there (built on `threat_area`; no state
   change, so no `Command`). Property test: an enemy is listed exactly
   when the position is in its `threat_area`.
2. `ui`: the battle screen computes the list when 0055's "when" holds
   (cache it per cursor tile; don't recompute every frame) and puts it in
   `MapScene` as plain data, e.g. `threats: Vec<Pos>` (the enemies' tiles)
   beside `path`, with the target tile.
3. Both map skins paint it (ADR-0038): the glyph skin and the sprite
   skin each in their own way, as 0055's look says. Nothing is drawn
   straight into the buffer by the screen. A line's picture, if it needs
   one, is a sprite item from an asset file, not per-pixel rectangles.
4. Enemies outside the view: as 0055 decided.

## Acceptance criteria

- [ ] In Chapter 1, selecting a unit and pointing at a tile an enemy can
      attack shows that enemy as 0055 decided; a tile nobody reaches shows
      nothing.
- [ ] A test reads `MapScene` for the listed enemies (not cells).
- [ ] Each skin has a test or snapshot of its look.
- [ ] `core` knows nothing about the look.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit + property (`core`): the threat list.
- Integration (`crates/ui/tests/it/`): the scene while proposing a move.
- Snapshot: both skins.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
