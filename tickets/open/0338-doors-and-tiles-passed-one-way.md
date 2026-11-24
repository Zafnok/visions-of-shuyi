---
id: "0338"
title: "Doors and tiles that are passed only one way"
type: feature
milestone: M2 Core rules
model: opus-5.5 / gpt-6.1-sol
effort: high
status: blocked
blocked_by: ["0097"]
nick_input: answer-first
completed:
---

# 0338 — Doors and tiles that are passed only one way

## Context

Nick (2026-11-22): "doors (one way, not from side only front)". 0097
decides what that means (passed only straight through, or one side only)
and which other directional tiles exist. Today `door` is impassable to
everyone (`assets/data/terrain.ron`); move costs are per tile and
movement type (`crates/core/src/terrain.rs`, `move_cost`), with no
direction. 0491 makes Chapter 4's doorways look like doors (either
order: if 0491 is done first, its door tiles get the new rule here).

## Nick input

**Answer first:** 0097. Nothing to check after merge: verified by tests,
`frame-png` pictures and the bots; what's left to judge in play goes on
1064's checklist.

## Scope

**In:**
- Core: an edge rule — entering or leaving a tile allowed only in the
  directions its data names (per tile kind, and per placement on the map
  for facing). Pathfinding (`crates/core`), ranges, the AI and the bots
  all use it.
- Data: the door's directions in `terrain.ron`; the facing in the map
  format (a door in a north–south wall faces east–west).
- UI: the move range shows it; the terrain window says how it's passed;
  every map skin draws the door facing its way.
- Per 0097: other one-way tiles (ledges) with the same rule.

**Out (do not do):** barricades (0337); keys and locks unless 0097 asks
for them (then file a ticket).

## Implementation steps

1. Core: directional entry in `move_cost`'s caller (the pathfinder), with
   unit and property tests.
2. Content: data and map format, validation (a door's facing must sit in
   a wall line).
3. UI: info text, skins.
4. AI and bots: tests on a staged map in `assets-test/`.

## Acceptance criteria

- [ ] Core tests: a door is entered only from 0097's sides; no path
      enters it from another side.
- [ ] Property: every path the pathfinder returns respects every tile's
      directions.
- [ ] `frame-png` pictures of doors facing both ways in all skins.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit, property (core); bot test on the staged map; view tests.

## Lesser targets

Playdate: one-bit door tiles in two facings. GBA: no target.

## Completion notes

