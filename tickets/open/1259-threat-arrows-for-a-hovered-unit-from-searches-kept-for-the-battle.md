---
id: "1259"
title: Threat arrows for a hovered unit from searches kept for the battle
type: bug
milestone: Later
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["1251"]
nick_input: none
completed:
---

# 1259 — Threat arrows for a hovered unit from searches kept for the battle

## Context

1251 gave the battle screen a `trpg_core::movement::ThreatPlanner`.
While a unit is selected, it answers the threat arrows for each cursor
tile from searches kept for that unit: in *The Company Ledger*, the
bench's moves with the rider selected now cost `[0, 0, 0, 2, 2]` path
searches, down from 14 each. CI's `bench` job on PR #593 (2026-11-09):

| Scenario | On | Most (frame), before 1251 | Most (frame), after |
| --- | --- | ---: | ---: |
| battle-attack | desktop, web | 2,695,971 (11) | 2,306,174 (43) |
| battle-attack | playdate | 3,406,850 (13) | 3,047,225 (1) |
| battle-cursor-sweep | desktop, web | 2,236,860 (91) | 2,272,099 (91) |
| battle-cursor-sweep | playdate | 3,111,370 (91) | 3,046,030 (91) |

What is left, very likely:

- **A hovered unit.** With no unit selected, the arrows show for the
  player unit under the cursor, on its own tile
  (`BattleScreen::threat_target`, `Mode::Idle`). Each new unit makes a
  new planner, which searches every hostile unit's moves (14 here).
  `battle-attack`'s frame 1 on the Playdate is its first cursor move;
  the sweep's frame 91 crosses units. 1251's count showed 0–28 searches
  a step before 1251.
- **The unit menu.** `battle-attack`'s frame 43, where the unit menu
  opens, is now its costliest on the colour targets: 2,306,174, over
  the Playdate's 2,000,000 that 1251 aimed at. 1247 brought it to one
  path search; what else it spends isn't measured.

A unit on its own tile is where the battle has it, so its threats are
the hostile units whose threat area, worked out with every unit in
place, holds that tile: `threats_at(unit, unit.pos)` moves nobody. One
search per hostile unit, kept until the battle changes, then answers
every hovered unit with no search.

## Nick input

None. No rule or look changes: the same arrows on the same tiles.

## Scope

**In:**
- Count the path searches in each of `battle-cursor-sweep`'s steps and
  `battle-attack`'s first steps (a test in `crates/ui`, as 1251's
  `a_cursor_move_while_a_unit_is_selected_hardly_searches` does), to
  confirm the hover is what costs.
- In `crates/core/src/movement.rs`: each hostile unit's threat area
  with every unit in place, worked out once for the battle (the same
  areas `danger_zone` unions), answering `threats_at` for any unit on
  its own tile. Units that never move use the distance rule, as
  `threats_at` does. A property test checks it against `threats_at`.
- The battle screen keeps it until the battle changes, as it keeps
  `KeptThreat` and the planner, and uses it for a unit on its own tile.
  If the danger zone can be built from the same areas with the same
  answer, it may share them.
- A Callgrind look at `battle-attack`'s frame 43 (the unit menu): name
  its largest costs in the completion notes, and write a ticket for
  them if they aren't the arrows.
- The numbers from CI's `bench` job in `docs/targets.md`.

**Out (do not do):**
- The selected unit's planner (1251), unless its searches can start
  from the kept ones with the same answers.
- The one-bit drawing (1248).
- Changing which enemies count as a threat, or how the arrows look.

## Implementation steps

1. Write the count test of the first bullet; it shows a search per
   hostile unit each time the cursor reaches a new player unit.
2. Add the battle-wide areas to `movement.rs` beside `ThreatPlanner`,
   with the property test (`movement/tests.rs`'s `arb_scene`, with
   stances, units that never move, blocked tiles).
3. Keep them in `BattleScreen` (`crates/ui/src/screens/battle/mod.rs`),
   dropped where `threats` and `planner` are dropped, and use them in
   `planned_threats` when the tile is the unit's own.
4. Push; read the `bench` table; update `docs/targets.md` (*The
   numbers* and *Over budget*).

## Acceptance criteria

- [ ] The cursor reaching a player unit with none selected runs no path
      search once the battle's areas are kept (test of step 1).
- [ ] The battle-wide answers equal `threats_at`'s (property test).
- [ ] Every existing threat-arrow test and snapshot passes unchanged.
- [ ] `battle-attack`'s and `battle-cursor-sweep`'s costliest frames on
      the colour targets are under 2,000,000 instructions, or the
      completion notes name what is left and its ticket.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the search count per cursor step.
- Property: the battle-wide answers equal `threats_at`.
- Snapshot: unchanged threat arrows.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
