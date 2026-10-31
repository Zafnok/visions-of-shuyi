---
id: "1114"
title: "Tactics kit in shuyi-core: a TurnRules trait, grid and pathfinding over a cost function, the RNG, replay and save versioning"
type: infra
milestone: "Engine: carve"
model: reasoning
effort: xhigh
status: done
blocked_by: ["1102"]
nick_input: none
completed: 2026-10-16
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

- [x] `shuyi-core` tests pass alone, including the property tests.
- [x] Every `core`, `content`, `ui` and `bots` test passes unchanged; no
      snapshot changes; the golden saves load.
- [x] Only one Dijkstra remains in the workspace (`grep`).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Property: path distances vs brute force; determinism of ties.
- Integration: replay and golden save tests (unchanged).

## Completion notes

**What was done.** The engine's first crate, `engine/crates/core`
(`shuyi-core`): pure, `#![forbid(unsafe_code)]`, `serde` only, a default
`std` feature and a `no_std` + `alloc` build (now in CI's `no-std` job).

- `shuyi_core::rng`: `rng.rs` moved as it was, with its tests.
- `shuyi_core::grid`: `geom.rs` moved, with `TileSet` from `movement.rs`
  and its tests (and the property test's saved seed,
  `proptest-regressions/grid.txt`).
- `shuyi_core::path`: **the one Dijkstra** (a private `search`), with two
  fronts: `reachable(width, height, start, budget, cost) -> Reach`
  (`cost`, `contains`, `tiles`, `path_to`) and `flow_field(width, height,
  goals, cost)`. The `(cost, y, x)` key, `Dir::ALL` order and
  "predecessor changes only when strictly cheaper" are kept and
  documented. Property tests: distances against brute force
  (Bellman–Ford) for both, paths add up, the documented tie-break, and
  flow distances against `reachable` from each tile; the cost function
  panics if asked about a tile outside the grid, so every test also checks
  that it never is.
- `shuyi_core::turn::TurnRules` (check, apply, legal commands, outcome),
  implemented by `BattleState` in `battle.rs` with its own methods.
- `shuyi_core::history`: `History<R: TurnRules>` (points, `state_at`,
  `replay`, `rewind_to`) and `Replayed<R>`, tested over a toy game.
- `shuyi_core::save`: `SaveFormat` (the save's name and version) and
  `SaveHeader<F>`, which reads the version of any save; the refuse,
  don't migrate rule is in its docs.

**The game side.** Its names are unchanged (ADR-0063 §6): `trpg_core::geom`
and `trpg_core::rng` are the engine's modules re-exported under those
names, `movement::TileSet` is the engine's, `Replayed` is
`Replayed<BattleState>`, and `SaveHeader` is `SaveHeader<SaveFile>`.
`movement::Reach` is the kit's reach plus where a unit may stop;
`reachable` passes the kit a cost closure (terrain cost, `None` on a
hostile unit's tile). The AI's flow field passes one too, with a target's
own tile free. `BattleHistory` is `History<BattleState>` plus the rewind
charges, and is saved through a mirror struct with the old fields, so a
save is byte for byte what it was (the golden saves load and write back
unchanged; `SAVE_VERSION` stays 2).

**Deviations from the ticket's sketch.**

- `TurnRules::apply` takes the command by reference
  (`&Self::Command`), like `check`: `BattleState::apply` already does,
  and replaying a stored command list would otherwise clone every
  command.
- `reachable` and `flow_field` take the grid's width and height: the
  result is a grid.
- The save header is generic over a `SaveFormat` trait rather than one
  plain struct: RON checks a struct name when a save names one
  (`SaveFile(version: 7, …)`, which the golden-save tests read), and that
  name is the game's.
- The AI's old flow field settled each tile the first time it left the
  queue; the kit's search relaxes like `reachable`. The distances are the
  same (the property tests and every AI, battle and bot test).
- `battle/skills.rs`'s move-after-attack search stays: it is a
  breadth-first walk of unit steps with no costs, not a Dijkstra.

**Tests.** The movement, AI, battle, replay and golden-save tests and
every `content`, `ui` and `bots` test pass unchanged. Two test files
gained one import each (`Grid` in `movement/tests.rs`, `Event` in
`history/tests.rs`), which they used to get through the module's own
imports. The rng, geom and `TileSet` tests moved with their code. New in
`trpg-core`: the `TurnRules` impl answers like the battle's own methods,
and a rewind to the latest point with no charges left is still "no such
point".

**Same games, a bit faster.** The baseline bot played Chapter 1 with
seeds 1–200 (`cargo xtask playtest ch01 --runs 200 --json`) on `main` and
on this branch: all 200 tries are identical (result, turns, every fall,
items, command count; 25% won). The same 200 battles in release
(`chapter_1_difficulty`, three rounds each) took 1.74–1.88 s here against
2.02–2.13 s on `main`, about 13% less: the kit's search queues fewer
duplicate tiles than the AI's old flow field. `reachable_is_fast` (release)
passes at about 30 µs per 64 × 64 reach.

**Also:** `docs/engine/boundary.md` (the rows of the moved modules,
couplings 7 and 8), Sonar now reads `engine/` too, and the `run-gates`
skill builds `shuyi-core` without `std`.

**Gameplay rules decided:** none. Every battle plays identically.
