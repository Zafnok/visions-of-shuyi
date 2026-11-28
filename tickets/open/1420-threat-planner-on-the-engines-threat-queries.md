---
id: "1420"
title: "The threat planner and threat areas on the engine's threat queries"
type: feature
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: blocked
blocked_by: ["1180", "1419", "engine#2152"]
nick_input: none
completed:
---

# 1420 — The threat planner on the engine's threat queries

## Context

The third engine-friction sweep (1397) found that `ThreatPlanner` and
`ThreatAreas` (`crates/core/src/movement.rs`, about lines 668–961) are
written in the game and named in no engine ticket. They work out which
hostile units could strike a unit if it stood on a tile. Each hostile
unit is searched once, and searched again only when the unit's new tile
could change its moves. That is about 280 code lines, plus the caches
in `crates/ui/src/screens/battle/threat.rs` that drive them, against
about 60. Engine ticket 2152 adds them to the tactics kit, generic over
the piece and the search.

This ticket is blocked by 1180 and 1419. The planner calls
`threat_area`, `threat_tiles` and `area_of`, which 1180 moves onto the
engine. Its searches are the reach that 1419 moves onto engine 2151.
Doing this ticket after both changes those calls only once.

## Nick input

None.

## Scope

**In:**
- Move the pin past engine 2152 (`engine-change` skill, §7) if it isn't
  there yet.
- `ThreatPlanner` and `ThreatAreas` become the engine's, built with the
  game's closures:
  - the search with a unit off the map (`movement::search`'s `absent`);
  - the ranges;
  - `AiBehavior::Stationary`;
  - the stance that stops enemies.
- `battle/threat.rs` keeps its display and its caches, on the engine's
  types.

**Out (do not do):**
- Any change to which threats are shown.

## Implementation steps

1. Write the engine-friction estimate first: about 250 lines removed and
   50 added.
2. Move the pin and build the engine's planner from the game's
   closures.
3. Delete the marked code (`// engine-friction: engine#2152`).
4. Run the threat tests: `movement` tests, `battle/threat_tests.rs`,
   and the search-count tests (`search-count` feature, 1247 and 1251).
   Each must give the same answers with no more searches.

## Acceptance criteria

- [ ] `grep -rn "engine#2152" crates` finds nothing.
- [ ] The threat tests pass unchanged, with the search counts the same
      or lower.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the existing ones.

## Completion notes

*(Filled in by the session that completes the ticket.)*
