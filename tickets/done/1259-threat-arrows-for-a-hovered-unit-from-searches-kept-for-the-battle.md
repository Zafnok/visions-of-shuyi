---
id: "1259"
title: Threat arrows for a hovered unit from searches kept for the battle
type: bug
milestone: Later
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: ["1251"]
nick_input: none
completed: 2026-11-09
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

- [x] The cursor reaching a player unit with none selected runs no path
      search once the battle's areas are kept (test of step 1).
- [x] The battle-wide answers equal `threats_at`'s (property test).
- [x] Every existing threat-arrow test and snapshot passes unchanged.
- [ ] `battle-attack`'s and `battle-cursor-sweep`'s costliest frames on
      the colour targets are under 2,000,000 instructions, or the
      completion notes name what is left and its ticket.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the search count per cursor step.
- Property: the battle-wide answers equal `threats_at`.
- Snapshot: unchanged threat arrows.

## Completion notes

- **The count first.** `the_cursor_reaching_a_player_unit_runs_no_path_search`
  plays `battle-cursor-sweep`'s 48 steps and `battle-attack`'s first
  three in *The Company Ledger*. Before the change, the sweep's last
  three steps, onto units 2, 1 and 4 (its frame 91), ran 14 searches
  each, one per enemy; every other step ran none. Now every step runs
  none.
- **Core.** `movement::ThreatAreas` searches each unit hostile to a
  faction once, with every unit in place, and keeps it as
  `ThreatPlanner` keeps one (`Threat`: its moves and threat area, or its
  tile if it never moves). `threats_to(unit)` answers `threats_at` for
  a unit of that faction on its own tile, with no search, and `None`
  for a unit of another faction. `Threat::in_range` is shared with the
  planner.
- **Deviation: the planner starts from the areas.** With only the
  areas, selecting the rider found her threat arrows already kept from
  the hover, so her planner was made on the first cursor move instead:
  the 14 searches moved from the hover to that move (`[14, 0, 0, 2, 2]`).
  The ticket allows the planner to start from the kept searches if the
  answers stay the same, and they do: a hostile unit's search with
  everyone in place is the very search with the selected unit off the
  map when none of its passable tiles is next to her (or within two
  tiles of her, if she holds a stance that stops enemies), as the
  search never tries the tiles that change. `ThreatPlanner::with_areas`
  reuses those and searches only the others. The battle screen makes
  the selected unit's planner as it is selected, from the kept areas:
  selecting the rider now runs 2 searches (her own moves and the one
  enemy whose moves come next to her) where it ran 15, and the moves
  after it stay `[0, 0, 0, 2, 2]`, so 1251's test passes unchanged.
- **UI.** `BattleScreen` keeps the areas beside `threats` and `planner`
  and drops them where they are dropped (a command, a rewind).
  `planned_threats` answers a unit on its own tile from the areas, in
  any mode, and every other tile from the planner.
- **Tests.** `the_battle_threat_areas_answer_as_threats_at_does`
  (proptest: random maps, units, Mov, ranges, stances, units that
  never move, blocked tiles, either side) compares every unit's answer
  with `threats_at`; `the_planner_answers_as_threats_at_does` now also
  checks planners started from either side's areas. Both passed 20,000
  cases locally. Hand-made scenes count the searches
  (`the_battle_threat_areas_search_each_moving_enemy_once_then_never`,
  `a_planner_started_from_the_battle_threat_areas_searches_only_enemies_near_the_unit`,
  stance included). Every existing threat-arrow test and snapshot
  passes unchanged.
- **The danger zone isn't shared.** `danger_zone` also searches the
  enemies that never move (it shows where they could walk); the arrows
  only measure their reach. Sharing would add those searches to every
  hover, and the danger zone is only worked out while shown, so it
  keeps its own.
- **The unit menu (frame 43).** Callgrind's dump of `battle-attack`'s
  frame 43 on `main` (2,305,584 instructions; the frame before,
  1,205,126): the battle screen's `draw` is 1.81 million, of which the
  engine's `GlyphBuffer::fill_rect` is 1.03 million (`cut_items`
  0.91 million), the menu widget's paint 0.64 million and the map
  skin's 0.57 million; opening the menu (the attack preview, fitting
  the art list) 0.31 million. The threat arrows aren't among them.
  `cut_items` rebuilds every item drawn so far on each `fill_rect`,
  with new `Vec`s, also those the rectangle doesn't meet, and the menu
  fills a rectangle per row. Every frame with the menu open costs about
  2,013,000. Ticket **1260** (an engine change, `engine-change`
  skill), added to 1250's `blocked_by`.
- **Bench:** the numbers of CI's `bench` job on this PR are in
  `docs/targets.md` (*The numbers*, *Over budget*).
- **Friction:** estimate 70, measured 143 (2.0×), accepted: the
  estimate was for the areas alone, before the planner's reuse was
  added; game-side rules and a screen's cache, no engine piece missing
  (`docs/engine/friction.md`).
- No gameplay rule decided; nothing for the Act 1 playtest.
