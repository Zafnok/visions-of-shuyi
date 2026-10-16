---
id: "1114"
title: "Tactics kit in shuyi-core: a TurnRules trait, grid and pathfinding over a cost function, the RNG, replay and save versioning"
type: infra
milestone: "Engine: carve"
model: opus-5.5
effort: xhigh
status: todo
blocked_by: ["1102"]
nick_input: none
completed:
---

# 1114 — Tactics kit in shuyi-core

## Context

`docs/engine/boundary.md`, couplings 7 and 8, and *core*. `core` has no
simulation trait: `Command`, `Event`, `CommandError` and `BattleState`
are concrete, named directly by `history.rs`, `legal.rs`, `ai.rs`,
`save.rs`, the bots and the cinematic combat shot. Pathfinding
(`movement.rs:334-515`) takes this game's map, tables and units, and a
second Dijkstra sits in `ai.rs:755`. The generic pieces are small: the
RNG (`rng.rs`), `Pos`/`Dir`/`Grid` (`geom.rs`), `TileSet` and
`Reach::path_to` (`movement.rs:58-152`), and the save header's version
rule (`save.rs`).

ADR-0063 §6: this code is **rewritten generically**, not moved; the
game's turn loop stays the game's and implements the trait.

This touches only `core`, so it can run in parallel with 1103–1113 (it
waits for the boundary gate only; it waited for 0804 too until playtest
round 3, 2026-10-16, when Nick put the carve first and made 0804's round
4 wait for it).

## Nick input

None. No rule changes: every battle plays identically.

## Scope

**In:**
- New crate `engine/crates/core` (package `shuyi-core`): pure, no I/O,
  `#![forbid(unsafe_code)]`, depends on `serde` only.
- `shuyi_core::rng`: `SimRng` (PCG32), `RandomSource`, `ScriptedRng`
  (moved).
- `shuyi_core::grid`: `Pos`, `Dir`, `Grid<T>`, `TileSet`, Manhattan and
  neighbours (moved).
- `shuyi_core::path`: `reachable(start, budget, cost: impl Fn(Pos) ->
  Option<u32>) -> Reach` and `path_to`, Dijkstra with the deterministic
  `(cost, y, x)` tie-break and `Dir::ALL` order; `flow_field` for the AI's
  use. The game's `movement.rs` and `ai.rs:755` call it with their cost
  closures; the duplicate Dijkstra goes.
- `shuyi_core::turn`: `trait TurnRules { type Command; type Event; type
  Error; type Outcome; fn check(&self, &Self::Command) -> Result<(),
  Self::Error>; fn apply(&mut self, Self::Command) ->
  Result<Vec<Self::Event>, Self::Error>; fn legal_commands(&self) ->
  Vec<Self::Command>; fn outcome(&self) -> Option<Self::Outcome>; }`.
  `BattleState` implements it in an `impl` block that calls the
  existing methods.
- `shuyi_core::history`: replay, `state_at`, rewind over `TurnRules +
  Clone` (the rewind *charges* stay the game's).
- `shuyi_core::save`: the version header and the refuse-don't-migrate
  rule (ADR-0039); the game's `SaveFile` payload stays.

**Playdate:** the Playdate port (1025's tickets, 1027–1053) builds
`core`, `content` and `ui` without `std` and adds a second front end,
`crates/playdate`. If those tickets have landed, code this ticket moves
keeps its default `std` feature and stays in the `no-std` CI build, and
`cargo xtask playdate` keeps building (ADR-0070 §6). 1051 (the AI plans once per phase) changes the AI wherever it lives.

**Out (do not do):** the battle's turn loop, phases, factions, triggers
(they stay the game's; a generic phase model can come with a second
tactics example); bots (1116); the map scene (1115).

## Implementation steps

1. Create the crate; move `rng` and `geom` with their tests.
2. Write `path` with property tests (on random grids, its distances
   equal a brute-force BFS with costs; ties break as documented).
3. Switch `movement.rs` and `ai.rs` to it; every movement, AI and
   battle test must pass unchanged, and the bots' baseline
   (`bots/tests/baseline.rs`) must give the same results.
4. Add `TurnRules`; implement for `BattleState`; move `history` to use
   it; golden save tests and `tests/it/replay.rs` must pass unchanged.

## Acceptance criteria

- [ ] `shuyi-core` tests pass alone, including the property tests.
- [ ] Every `core`, `content`, `ui` and `bots` test passes unchanged; no
      snapshot changes; the golden saves load.
- [ ] Only one Dijkstra remains in the workspace (`grep`).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Property: path distances vs brute force; determinism of ties.
- Integration: replay and golden save tests (unchanged).

## Completion notes
