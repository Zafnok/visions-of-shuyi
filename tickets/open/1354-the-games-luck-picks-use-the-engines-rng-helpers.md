---
id: "1354"
title: The game's luck picks use the engine's RNG helpers, draw for draw as before
type: feature
milestone: M1 Engine
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1354 — The game's luck picks use the engine's RNG helpers

## Context

Engine PR #551 (2114) came in with the pin move of 2026-11-24 (issue
#816). It adds helpers on `RandomSource` (`shuyi_core`, re-exported by
`trpg_core`): `range`, `chance`, `index`, `pick`, `shuffle`,
`weighted_index`, `pick_weighted`, a `Bag`, and `SimRng::stream` and
`SimRng::fork`. Each takes a fixed number of draws (engine ADR-0143), and
`index`, `pick`, `range` and `weighted_index` are each one
`roll_below`, the same draw the game's hand-written picks make.

The game writes these picks by hand, each an index cast from
`roll_below(u32::try_from(len)…)`:

- `crates/core/src/worldmap/skirmish.rs`: `roll` (a value in a closed
  range) and, in `spawn_skirmishes`, the place, the battle from the pool
  and the reward.
- `crates/core/src/progression.rs`, `level_up`: the weighted pick of the
  stat that gains when too few did (`roll_below(total)`, then a running
  sum), which is `weighted_index` over the candidates' growths.
- `crates/bots/src/record.rs`, `choose`'s `pick` closure.
- `crates/bots/src/skirmish.rs`: the template, `map_text`'s ground tile,
  `around`, and the `len` helper they share.

## Nick input

None. Nothing a player sees changes: every pick draws what it drew before.

## Scope

**In:**
- Each pick above on the engine's helper (`pick`, `index`, `range`,
  `weighted_index`), and the game's own helpers that only did that
  (`worldmap::skirmish::roll`, the bots' `len` and `pick`) deleted.
  Expected: about 30 to 40 production lines removed, a few changed.
- Draw for draw as before. One place differs: `spawn_skirmishes` draws
  for the reward even when `rules.rewards` is empty
  (`roll_below(0)`), where `index(0)` draws nothing. Keep that draw
  (an explicit `roll_below` there with a comment, or check the map data:
  if every world map's skirmish rules list at least one reward entry and
  the loader refuses an empty list, `index` is safe; say which in the
  completion notes).

**Out (do not do):**
- Any rule whose draws would change: kept records, sessions, saves and
  the bots' bands stay as they are.
- `SimRng::stream`, `fork` and `Bag`: no rule of the game needs a second
  stream or a bag today.
- The combat and level-up `roll_percent` rolls, which are already the
  engine's.

## Implementation steps

1. Read engine ADR-0143 and `RandomSource`'s helpers in `shuyi_core`'s
   `rng.rs` at the pin.
2. Change `crates/core/src/worldmap/skirmish.rs` (`roll` → `range` over
   `range.0..range.1 + 1`, minding `u32` versus `i32`: keep `roll` if the
   conversion costs more lines than it saves; the three picks →
   `index`/`pick`).
3. Change `level_up` in `crates/core/src/progression.rs` to
   `weighted_index` over the candidates' growths.
4. Change the bots' picks in `crates/bots/src/record.rs` and
   `crates/bots/src/skirmish.rs`.
5. Run the gates; every kept record, session and snapshot passes
   unchanged.

## Acceptance criteria

- [ ] `grep -rn "roll_below(u32::try_from" crates` finds nothing outside tests.
- [ ] No snapshot, session, record or bot band changes in the diff.
- [ ] `cargo test -p trpg-core progression` and `worldmap` pass unchanged.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: none new; the existing level-up tests with `ScriptedRng`
  (`crates/core/src/progression/tests.rs`) and the skirmish spawn tests
  hold the draws. If the empty-rewards draw is kept, one test that
  spawns with no rewards and checks the following draw is unchanged.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
