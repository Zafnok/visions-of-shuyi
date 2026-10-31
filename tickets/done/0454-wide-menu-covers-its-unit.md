---
id: "0454"
title: A menu too wide for either side of its unit covers the unit
type: bug
milestone: M3 Battle UI
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-10-28
---

# 0454 — A menu too wide for either side of its unit covers the unit

## Context

Found while doing ticket 0429. `menu_origin` in
`crates/ui/src/screens/battle/mod.rs` puts a menu one cell right of the
unit's tile, or left of it if it would run past the map view, and clamps
it to the map view's left edge. A wide menu fits neither side and the clamp
puts it over the unit.

Seen in the snapshot `archer_arts_menu_snapshot`
(`crates/ui/src/screens/battle/art_tests.rs`): the archer at (8, 4) of the
Quick Battle map (screen cells 36–37, row 15) opens its `Arts` menu, which
is 46 cells wide (`Pinning Shot  −3 dur  D  out of range`); it doesn't fit
right (39 + 46 > 70) or left (36 − 1 − 46 < 0), so it is drawn from column
0 and covers the archer. The same can happen to any menu over a narrow
map view (the arts menu since 0442, a long spell list, the item menu).

## Nick input

None. Nick no longer checks gameplay tickets after merge (Nick,
2026-10-21: "I will save all my checking for either design (i.e. 00
tickets) or for the full act 1 playtest itself"). Verification is by
tests, `cargo shuyi frame-png` pictures in the PR and the playtest bots
(0506, 0509). What the old sign-off asked goes on the checklist of
Nick's next playtest (1064, the Act 1 playtest, if this lands before
it):

> In Quick Battle, open the archer's `Arts` menu where it
> stands: the archer stays visible.

## Scope

**In:**
- When the menu fits neither beside the tile, place it **below** the
  tile's row (first row of the menu one row under the tile), or **above**
  it if it doesn't fit below, x clamped to the map view. *the agent's
  starting rule* (a placement detail, not a look decision): never cover
  the unit's own tile.
- If it fits nowhere without covering the tile (a menu taller than the
  rows above and below), keep today's placement.

**Out (do not do):**
- Making menus narrower or rewording their lines.
- Covering the target: the forecast doesn't use `menu_origin`.

## Implementation steps

1. Change `menu_origin(tile, (w, h))` as above; keep its signature.
2. Unit-test it with plain `Rect`s: fits right, fits left, fits neither →
   below, no room below → above, no room at all → today's place.
3. Re-run the battle snapshots; review each changed one (the archer arts
   snapshot should now show the archer).

## Acceptance criteria

- [x] Unit tests of `menu_origin` for the five cases.
- [x] `archer_arts_menu_snapshot` shows the archer's `Ar` uncovered.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `menu_origin`.
- Snapshot: the changed battle snapshots, reviewed.

## Completion notes

`menu_origin_in` (`crates/ui/src/screens/battle/glyph.rs`) now tries right, then left (only if it fits without clamping), then below the tile's rows, then above, and keeps the old clamped place only when none fits. Unit test covers the five cases; five snapshots changed (archer arts, arts menu, left-handed arts menu, broken weapon lines, broken-weapon equip menu) and each now leaves its unit uncovered. The Playdate uses the same function and its tests still pass. Gates run: fmt, clippy, `trpg-ui` tests. No starting rules beyond the ticket's. Estimate ~15 lines, measured about that; no friction. Nick's Act 1 playtest checklist item: open the archer's `Arts` menu in Quick Battle, the archer stays visible.
