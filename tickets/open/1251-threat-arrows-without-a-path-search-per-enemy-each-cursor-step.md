---
id: "1251"
title: Threat arrows without a path search per enemy each cursor step
type: bug
milestone: Later
model: opus-5.5 / gpt-6.1-sol
effort: high
status: in-progress
blocked_by: ["1247"]
nick_input: none
completed:
---

# 1251 — Threat arrows without a path search per enemy each cursor step

## Context

1247 brought the unit menu's frame of `cargo shuyi bench`'s
`battle-attack` scenario down from 16.8 million instructions to under its
p99 (`docs/targets.md`). The scenario's costliest frame is now **frame
11, 2.70 million on the desktop and 3.41 million on the Playdate
(frame 13)**, against the Playdate's budget of 2 million. That frame is
a cursor step while the rider is selected (each action of a scenario is
two frames).

A count made with `trpg_core::movement::searches` (the `search-count`
feature, 1247) in *The Company Ledger* shows what each step does:

| Step (bench's `battle-attack`) | `reachable` searches |
| --- | ---: |
| cursor moves with no unit selected | 0–28 |
| Confirm on the rider | 1 |
| each cursor move while she is selected | **14** |
| Confirm on the bridge, and the unit menu opening | 1 |

The 14 are the threat arrows (`docs/design/turn-structure.md`, *Threat
arrows*): on each cursor move the screen asks
`threat_enemies(state, unit, tile)` (`crates/ui/src/screens/battle/mod.rs`),
which runs `trpg_core::movement::threats_at`, and that function searches
every hostile unit's moves again with the selected unit placed on the
cursor's tile. `KeptThreat` keeps the answer for one tile only, so every
new tile costs a full search per enemy. The same work is very likely
what puts `battle-cursor-sweep` at 2.27 M (desktop) and 3.11 M
(Playdate), which 1248 names as "the cursor's reach".

## Nick input

None. No rule or look changes: the same arrows on the same tiles.

## Scope

**In:**
- Work out the threat arrows for a selected unit with far fewer
  searches per cursor move, with exactly `threats_at`'s answer for
  every tile. One way: for the selection, search each hostile unit's
  moves once with the selected unit off its tile, and keep them. For a
  cursor tile, reuse an enemy's kept search when its paths never go
  through that tile, and search again only when they do. The selected
  unit on that tile closes it to hostile units, and that is the only
  thing that changes. The rule is the core's (`movement.rs`); the
  screen keeps the result for the selection's life.
- `battle-attack`'s and `battle-cursor-sweep`'s costliest frames measured
  again by CI's `bench` job, and the numbers in `docs/targets.md`.

**Out (do not do):**
- The unit menu (1247, done).
- The one-bit drawing cost (1248).
- Changing which enemies count as a threat, or how the arrows look.
- The danger zone (`danger_zone`), unless the same kept searches serve
  it without changing its answer.

## Implementation steps

1. Write a test in `crates/ui` that counts `trpg_core::movement::searches()`
   (the `search-count` feature is on in `trpg-ui`'s dev-dependencies)
   for each cursor move while a unit is selected in *The Company Ledger*,
   with the bench's steps (`bench/battle-attack.bench.ron`, as
   `opening_the_unit_menu_searches_the_move_once` in
   `crates/ui/src/screens/battle/art_tests.rs` does). It fails today
   (14 a move).
2. In `crates/core/src/movement.rs`, add a planner beside `threats_at`.
   It is made once for a selected unit and answers `threats_at` for any
   tile, reusing its kept searches as described under **In**. A
   property test (proptest, as `movement/tests.rs` does) checks that it
   gives `threats_at`'s answer for random maps, units and tiles.
3. Keep it in the battle screen for the selection's life (beside
   `KeptThreat`, which it may replace), and use it where
   `threat_enemies` is called for the selected unit.
4. Push; read CI's `bench` table; put the numbers in `docs/targets.md`
   (*The numbers*, and the *Over budget* list).

## Acceptance criteria

- [ ] A cursor move while a unit is selected runs at most 2 path
      searches in the test of step 1 (most moves: none).
- [ ] The planner's answers equal `threats_at`'s (property test).
- [ ] Every existing threat-arrow test and snapshot passes unchanged.
- [ ] `battle-attack`'s costliest frame on the colour targets is under
      2,000,000 instructions (the `bench` job's table, in
      `docs/targets.md`), or the completion notes name what is left and
      its ticket.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the search count per cursor move.
- Property: the planner equals `threats_at`.
- Snapshot: unchanged threat arrows.

## Completion notes

- **Core.** `movement::ThreatPlanner` is made once for a unit: it
  searches every hostile unit's moves with that unit off the map and
  keeps them with their threat areas. The unit standing on a tile only
  takes moves away (no step enters its tile; with a stance that stops
  enemies, none goes on from a tile next to it), so a hostile unit's
  threat then is a part of the kept one. For a tile, a hostile unit is
  settled without a search when the tile is outside its kept threat
  area (no threat), when its moves never reach the tile and the unit
  holds no such stance (the kept answer), or when a tile in range has a
  kept path clear of the unit (a threat). Otherwise it is searched again
  with the unit on the tile, as `threats_at` does. Units that never move
  are checked by distance, as before. `threat_area`'s set-building moved
  to `area_of`, shared by both.
- **UI.** The battle screen keeps the planner beside `KeptThreat` (which
  stays, keyed by unit and tile) and drops both when the battle changes.
  A new unit makes a new planner: hovering a unit still costs one search
  per enemy, as before; another tile for the same unit costs almost
  nothing.
- **Tests.** `a_cursor_move_while_a_unit_is_selected_hardly_searches`
  plays the bench's moves in *The Company Ledger*: before, each of the
  five moves ran 14 searches; now `[0, 0, 0, 2, 2]`. The two are the
  bridge tiles, where some enemies' only paths to her go through her
  tile. `the_planner_answers_as_threats_at_does` (proptest: random maps,
  units, Mov, ranges, stances, units that never move, blocked tiles,
  tiles off the map) compares every answer with `threats_at`. Random
  maps rarely build the shapes where a path through the unit or past a
  stance matters, so hand-made scenes check each case and its search
  count. Every existing threat test and snapshot passes unchanged.
