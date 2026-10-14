---
id: "1122"
title: "Collision and movement kit: tile collision, boxes, pixel-stepping movers, triggers, a top-down controller"
type: feature
milestone: "Engine: real time"
model: opus-5.5
effort: high
status: todo
blocked_by: ["1119"]
nick_input: none
completed:
---

# 1122 — Collision and movement kit

## Context

ADR-0065 §4. Free movement and platformers need collision that is exact
under replay, so it uses `Fx` (1119), not floats. Movers step one pixel
at a time and stop at what they hit (the TowerFall and Celeste way). That
way they never tunnel through a wall and never overlap something they
then have to be pushed out of.

## Nick input

None.

## Scope

**In:**
- `engine_core::collide`:
  - `TileCollision` (solid, empty, one-way-up), built from a `Grid` and a
    rule per tile id;
  - `Aabb`;
  - a uniform-grid broad phase;
  - `Mover::move_x` / `move_y`, which keep the fractional remainder, step
    one pixel at a time, and return what they hit;
  - `Trigger` enter and leave events.
- `engine_core::controllers::top_down`: 8-way and analog movement with
  acceleration, top speed and friction, all as data (RON).
- Property tests:
  - a mover never ends inside a solid tile or box;
  - moving by `a` then by `b` equals moving by `a + b` when nothing is
    in the way;
  - the same inputs give the same positions.

**Out (do not do):** slopes; the platformer controller (1127); rigid
bodies (1128).

## Implementation steps

1. Write the types and the movers with unit tests on hand-drawn maps.
2. Add the property tests.
3. Use them in the tick example from 1119: the dot walks a little room.

## Acceptance criteria

- [ ] The property tests pass with 10,000 cases.
- [ ] The example room's replay hash is stable across runs and builds.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit and property, as above.

## Completion notes
