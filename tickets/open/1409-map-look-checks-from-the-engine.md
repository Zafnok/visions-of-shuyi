---
id: "1409"
title: "A map's lights and props checked by the engine: check_lights, check_props and their errors go"
type: feature
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: low
status: blocked
blocked_by: ["engine#2290"]
nick_input: none
completed:
---

# 1409 — A map's lights and props checked by the engine

## Context

The third engine-friction sweep (1397, `docs/engine/friction.md`) found
`crates/content/src/map.rs` lines 162–260 (`check_lights`,
`check_props`, `prop_error`, `light_error`, about 95 production lines)
reading only engine types: `MapLook`, `Lighting`, a tileset's
`Look::props`, `grid_map::header_position`. Engine ticket 2290 adds
`MapLook::light_problem`, `prop_problems` and `grid_map::check_looks`.
Each item it replaces is marked `// engine-friction: engine#2290, moves
(game 1409 deletes it)`.

## Nick input

None.

## Scope

**In:**
- Move the pin past engine 2290 if it isn't (`engine-change` §7).
- The content checks call `check_looks` with the game's maps, lighting
  and tileset looks; the four marked functions go.
- **Either order with 1219:** if 1219 is done, pass the engine's
  terrain looks it now keeps; if not, pass the game's
  `tileset.terrain.looks` (with `DEFAULT_TILES` falling back to
  `terrain.look`, as `check_props` does), and 1219 keeps the call
  working when it changes the type.

**Out (do not do):**
- Other map checks (shops, chests, blocked tiles): game rules, they stay.

## Implementation steps

1. Estimate: about 95 lines removed, 15 added.
2. Move the pin if needed; swap the calls; run the content tests.

## Acceptance criteria

- [ ] `grep -n "engine#2290" crates/` finds nothing.
- [ ] The content tests for an unknown light and an unknown prop pass,
      each error at the same line and column.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- The existing content tests, unchanged but for message wording.

## Completion notes

*(Filled in by the session that completes the ticket.)*
