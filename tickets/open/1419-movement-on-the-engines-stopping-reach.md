---
id: "1419"
title: "Movement's reach and drawn-path check on the engine's reach with stopping tiles and walk cost"
type: feature
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: blocked
blocked_by: ["engine#2151"]
nick_input: none
completed:
---

# 1419 — Movement on the engine's stopping reach

## Context

The third engine-friction sweep (1397) found that
`crates/core/src/movement.rs` builds its own `Reach` around the
engine's `path::Reach`, to add the tiles a move may enter but not leave
(next to an enemy holding the Hold the Ford stance). It also checks a
drawn path with its own `path_cost` and `PathError`. That is about 130
code lines against about 40.

Engine ticket 2151 adds `path::reachable_with_stops`,
`Reach::is_stop`, `path::walk_cost` and `StepError` to the tactics kit.
The game keeps:
- which tiles stop;
- its closed tiles (hostile units, blocked tiles);
- its step costs by movement type;
- the budget check with its own `PathError::OverBudget`.

## Nick input

None.

## Scope

**In:**
- Move the pin past engine 2151 (`engine-change` skill, §7) if it isn't
  there yet.
- `movement::search` calls `reachable_with_stops` with the game's
  closures; the game's `Reach` becomes the engine's, or a thin newtype
  if the game needs `stoppable`.
- `path_cost` calls `walk_cost`, then maps `StepError` onto the game's
  `PathError` (whose variants the UI matches) and adds the budget
  check.

**Either order with 1180:** if 1180 is done, rebase on its threat
calls, which take the engine's `Reach` and fit this change. If not,
make this change alone. 1180's `attack_tiles` and `threat_*` items
take a `&Reach` either way.

**Out (do not do):**
- The threat planner (1420).
- Any change to which tiles a unit can reach or to the order of the
  paths: recorded battles must replay the same.

## Implementation steps

1. Write the engine-friction estimate first: about 100 lines removed and
   30 added.
2. Move the pin and rewrite `search` and `path_cost` on the engine's
   functions.
3. Delete the marked code (`// engine-friction: engine#2151`).
4. Run the movement tests, the Hold the Ford tests, and the replay tests
   of recorded battles (`trpg-bots` records, `crates/ui` act tests).

## Acceptance criteria

- [ ] `grep -rn "engine#2151" crates` finds nothing.
- [ ] Every movement and `PathError` test passes unchanged.
- [ ] Every recorded battle replays to the same state hashes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit/replay: the existing ones.

## Completion notes

*(Filled in by the session that completes the ticket.)*
