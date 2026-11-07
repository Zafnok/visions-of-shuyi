---
id: "1252"
title: The battle map offers the debug view's grid channel
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: low
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1252 — The battle map offers the debug view's grid channel

## Context

Engine 2315 (Zafnok/shuyi-engine#284, engine ADR-0084), passed by the pin
move of 2026-11-07, added the debug view: screens hand gizmos to the
engine through `Screen::debug_draw`, the debug menu's "Debug view" entry
turns channels on, and the grid kit's `grid` channel
(`shuyi_ui::grid::debug::layout_gizmos`) outlines the cells, the map's
bounds, the cursor's cell, each unit's cell and index, and the edges the
map goes on past. The game's battle map is drawn by the kit's grid skins
(1167), so it can offer the channel with one call; today the game's
debug menu shows "Debug view" with no channel to turn on.

Engine 3209 did the same for Hello, Grid and the grid template.

## Nick input

None. Debug builds only; nothing a player sees changes.

## Scope

**In:**
- `BattleScreen::debug_draw` (and the world map's, if it draws on the
  kit's grid) calls `layout_gizmos` with the scene and grid it paints.
- A Harness test: with the channel on, `frame-png --debug grid` of a
  battle lists the gizmos; with it off, none; the state hash is the same
  either way.

**Out (do not do):** channels of the game's own (threat areas, AI
targets): a ticket each if wanted.

## Implementation steps

Expected size: about 15 production lines, judged by Hello, Grid's
`debug_draw` (engine 3209).

1. Read engine ADR-0084 and `grid/debug.rs`'s module docs.
2. `debug_draw` on the battle screen, from the `MapScene` and `Grid` its
   skin paints.
3. The test, and a `cargo shuyi frame-png --debug grid` picture in the
   PR.

## Acceptance criteria

- [ ] In a debug build, the debug menu's "Debug view" offers `grid` on the battle map, and turning it on outlines the cells, the cursor's cell and each unit's (Harness test).
- [ ] With debug tools off, nothing is built (the engine's rule; a test).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the Harness test above.

## Completion notes

