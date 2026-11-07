---
id: "1251"
title: "The battle map offers the debug view's grid channel"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: low
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1251 — The battle map offers the debug view's grid channel

## Context

Engine PR #284 (2315) came in with the pin move of 2026-11-07: the
debug view. A screen hands gizmos to the frame from
`Screen::debug_draw`, in channels turned on from the debug menu's new
"Debug view" entry (saved under `debug_view`, debug builds only), and
`cargo shuyi frame-png --debug <channels>` draws them. The grid kit's
`grid` channel, `shuyi_ui::grid::debug::layout_gizmos(scene, grid,
out)`, outlines the cell lines, the map's bounds, the cursor's cell,
each unit's cell and index, and the edges the map goes on past. Engine
PR #299 (3209) shows the use in Hello, Grid's stage
(`examples/hello-grid/src/screens/stage.rs`, `debug_draw`). Engine
ADR-0085 and the book's *Seeing what the game sees* say how.

The game's battle map is painted by the grid kit (`GridSkin`, through
`crates/ui/src/map_view/skin.rs`), with a camera, two zooms and two
looks, so the channel shows where the skin put each cell and unit:
what the scene camera and the map skin's debug tools check by eye
today. Nothing in the game offers a channel yet, so the menu entry is
empty for it.

## Nick input

None. A debug tool; verified by tests.

## Scope

**In:**
- `BattleScreen`'s `debug_draw` (`crates/ui/src/screens/battle/mod.rs`):
  when `GRID` is on, `layout_gizmos` over the scene the skin paints,
  with the skin's `grid(&scene, area)` for the map's area at the
  current camera and zoom, as Hello, Grid does.
- Tests through the Harness: the channel on and off, both looks.

**Out (do not do):**
- Channels of the game's own (threat areas, AI plans): a ticket each
  if wanted.
- The Playdate's one-bit map skin (no debug menu there).
- Changing the map's look, the debug menu's other entries or the
  scene.

## Implementation steps

Expected size: about 20 production lines, judged by Hello, Grid's
`debug_draw` (9 lines).

1. Read engine ADR-0085, `shuyi_ui::debug::gizmo` and `grid::debug` at
   the pin.
2. Implement `debug_draw` on `BattleScreen`: return unless
   `out.channel_on(GRID)`; build the scene and area the way `draw`
   does (share the code rather than copy it) and call `layout_gizmos`.
3. Tests in `crates/ui/tests/it/` (a new `debug_view.rs` or beside
   `map_skin.rs`): with `grid` on, the gizmos list holds the cursor's
   cell and each unit's cell where the scene has them, for the glyph
   and the sprite looks; with it off, none.

## Acceptance criteria

- [ ] With the `grid` channel on, a battle's gizmos name the cursor's
      cell and every unit's cell (Harness test, both looks).
- [ ] With it off, the battle hands in no gizmos (test).
- [ ] `cargo shuyi frame-png out.png --debug grid` on a battle draws the
      layout over the map.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the Harness tests above.
- Snapshot: only if a test of the look is added; the behaviour tests
  read the gizmos list.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
