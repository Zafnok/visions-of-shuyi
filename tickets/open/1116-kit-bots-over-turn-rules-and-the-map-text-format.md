---
id: "1116"
title: "Tactics kit: playtest bots over any TurnRules, and the .map text format"
type: infra
milestone: "Engine: carve"
model: sonnet-5
effort: medium
status: todo
blocked_by: ["1114"]
nick_input: none
completed:
---

# 1116 — Kit: bots over any `TurnRules`; the `.map` format

## Context

The playtest bots (ADR-0033, `crates/bots`) are all this game's today:
they use `BattleState`, `Command`, `AiWeights` and the rest. The shape
is generic and about 200 lines: a `PlayerBot::choose` trait, a
`play_battle` loop, and the measures pattern. After 1114 it can be
written over `TurnRules`.

The `.map` text format (`content/src/map.rs`: a RON header, `---`, one
character per tile, a legend, a look and light) is the kit's. Its Shop
and Chest features (`map.rs:136-157`) are the game's.
`docs/engine/boundary.md`, *bots* and *content*.

## Nick input

None.

## Scope

**In:**
- `engine_core::bots` holds three things:
  - `trait Bot<R: TurnRules> { fn choose(&mut self, &R) -> R::Command }`
  - `play<R>(rules, bots_by_side, limit) -> Report` (outcome, turns,
    commands)
  - a seeded random bot over `legal_commands()`
- `trpg-bots` keeps its bots and measures and uses the generic loop.
- `engine_assets::grid_map` holds the `.map` reader and printer, with a
  hook for the game's tile features. The game keeps Shop and Chest as
  its features. The printer round-trips: parse → print → parse gives
  the same map.

**Out (do not do):** the map editor (1019); bot personas (0506).

## Implementation steps

1. Write the generic loop. Port `trpg-bots`. `bots/tests/baseline.rs`
   gives the same results.
2. Move the `.map` format with a feature hook. Every map in `assets/`
   round-trips (property test over the real maps in the game's tests;
   fixture maps in the engine's).

## Acceptance criteria

- [ ] A toy `TurnRules` game (tic-tac-toe in the engine's tests) is
      played to the end by two random bots, deterministically per seed.
- [ ] The game's bot baseline is unchanged.
- [ ] Every `.map` round-trips.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the loop with the toy game.
- Property: `.map` round trip.

## Completion notes
