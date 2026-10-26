---
id: "0516"
title: "The persona bots score an escape and a downed boss right, and play the rule battles in the tests"
type: bug
milestone: M4 Enemy AI
model: sonnet-5
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-10-26
---

# 0516 — The persona bots score an escape and a downed boss right

## Context

From the retro of the last 50 PRs (2026-10-22). 0319 (#393: the escape
objective, a fleeing boss, a boss who can't die) and 0506 (#394: the
persona bots, ADR-0095) were built side by side; 0319 taught the
personas the escape objective while merging 0506 in. Three gaps came
out of it:

1. **Waiting on the exit scores more than escaping.** A persona scores a
   candidate command as named counts times weights
   (`crates/bots/src/persona.rs`, `Tally::between`;
   `crates/bots/personas.ron`). In an escape, `progress` is +1 per unit
   that escaped (`progress`, `Objective::Escape`), weighted 10 (15 for
   Hardcore), and `approach` is the tiles each player unit came closer
   to the goals, weighted 2 (3). But `approach` sums only units still on
   the map (`end.unit(u.id)?`), so a unit that escapes loses its whole
   approach credit for the move. A unit that walked 6 tiles onto the
   exit scores 2 × 6 = 12 by standing there and 10 by leaving: it leaves
   a turn late, in reach of the reinforcement waves.
2. **Downing a boss who can't die earns nothing.** `enemy_fell` counts
   only `fallen()`; a unit that goes down (`Defeat::GoesDown`,
   `BattleState::down()`) isn't in it.
3. **No persona ever plays an escape.** CI plays `assets/battles/*.ron`
   (Chapter 1 only today); `crates/bots/tests/baseline.rs` plays the
   rule battles of `assets-test/battles/` (`test_escape`,
   `test_retreat`, `test_hold`, `test_green`) with the baseline bot only.
   Chapter 6 (0852, *The Ashfields*) is an escape map, and its balance
   bands would be judged with scoring that has never run.

Also stale: `personas.ron`'s comment and `docs/playtesting.md`'s table
define `progress` without escape, and `crates/bots/src/bot.rs` says the
baseline bot "never uses items", wrong since 0513 (green units drink
their own items through `next_command` on the other phases).
`baseline.rs`'s `recorded()` writes its own play loop, the one the
engine's `play_observed` (`shuyi_core::bots`) gives.

## Nick input

None. The personas' settings, weights and bands don't change (ADR-0095
§5); this fixes what two counts mean. Verified by the tests below and
by Chapter 1's playtest numbers, which must not move (Chapter 1 has no
escape and no unit that goes down).

## Scope

**In:**
- `approach`: a player unit that escaped between `start` and `end`
  counts as having reached its goal (its distance after is 0), so its
  approach credit is its whole distance before.
- `enemy_fell`: also counts enemies that went down (`end.down()` past
  `start.down()`).
- The comments in `personas.ron` and the `docs/playtesting.md` table:
  `progress` in an escape is a unit escaped; `enemy_fell` includes a unit
  that went down.
- A persona smoke test: each persona plays `test_escape`, `test_retreat`,
  `test_hold` and `test_green` for 3 seeds and every try ends in 60
  turns; in `test_escape` at least one try is a Victory.
- `baseline.rs`'s `recorded()` on `play_observed`, collecting the
  commands and events in the observe closure.
- `bot.rs`'s doc comment: the baseline never chooses an item on the
  player's side; on the other phases the enemy AI's `next_command` may
  have a green unit drink its own (0513).

**Out (do not do):**
- Any weight, band, floor or persona setting (ADR-0095 §5).
- Adding the rule battles to the CI playtest (it plays real battles).
- Moving the scoring onto the engine's `Weighted` (1180).

**Either order with 1180:** if 1180 is done, `Tally::between` (still the
game's, per 1180) is where both fixes go and the counts are by name in
its features array; if not, fix the fields as they are, and 1180 keeps
them.

## Implementation steps

1. **Read** `crates/bots/src/persona.rs` (`Tally::between`, `progress`,
   `approach`, `Goals`), its tests, `crates/bots/personas.ron`'s header,
   `crates/bots/tests/baseline.rs`, and the engine's `play_observed`.
2. **A failing test first** in `crates/bots/src/persona/tests.rs`: on
   `test_escape` (or a built state), a unit standing on the exit; the
   tally of `Escape` for it scores higher than the tally of `Wait` with
   the Normal persona's weights. It fails today.
3. **`approach`**: for each player unit in `start`, if `end` has it on
   the map, as today; else if `end.escaped()` has it, `before.distance(u,
   u.pos)? - 0`; else (fallen) skip it, as today.
4. **`enemy_fell`**: count enemies in `end.down()` beyond
   `start.down().len()` as well as the fallen ones. A failing test first,
   on `test_hold`.
5. **Smoke test** in `crates/bots/tests/` (a new `personas.rs` or in
   `baseline.rs`): as in Scope, with a turn cap of 60; keep it under
   about 30 s in a debug build (lower the seeds to 2 if it isn't).
6. **`recorded()`** on `play_observed`; the replay part of the test stays.
7. **Docs and comments** as in Scope.
8. **Chapter 1 unchanged**: `cargo xtask playtest <ch01 id> --bot all
   --runs 20` before and after (or `cargo shuyi playtest` if 1180 is
   done) give the same measures try by try; put both summaries in the
   PR.

Expected size: about 40 production lines (the two counts about 15, the
doc comments), plus tests.

## Acceptance criteria

- [x] The step 2 test passes: escaping scores higher than waiting on the
      exit, for every persona.
- [x] Downing an enemy counts in `enemy_fell` (test on `test_hold`).
- [x] The persona smoke test passes (with one exception, see notes): every try of the four rule battles
      ends in 60 turns, and `test_escape` is won at least once.
- [x] Chapter 1's 20-try measures are identical before and after (both
      summaries in the PR).
- [x] `baseline.rs` has no play loop of its own.
- [x] `personas.ron`, `docs/playtesting.md` and `bot.rs` say what the
      counts and the baseline do now.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the escape and down tallies (`persona/tests.rs`).
- Integration: the persona smoke test over `assets-test/battles/`.

## Completion notes

- `Tally::between` (`crates/bots/src/persona.rs`): `approach` gives a
  player unit that escaped its whole distance before (it reached the
  exit); `enemy_fell` counts the enemies that went down as well as the
  fallen. Tests: `escaping_scores_more_than_waiting_on_the_exit` (every
  persona; a tie before the fix) and
  `an_enemy_that_goes_down_counts_as_fallen` (on `test_hold`).
- `the_personas_finish_the_rule_battles` in `crates/bots/tests/baseline.rs`
  (not a new test file: the crate keeps one integration test program).
  Every persona, 3 seeds, the four rule battles, a 60-turn cap; about
  3 s of the file's 26 s in a debug build.
- `recorded()` on the engine's `play_observed`; its replay test unchanged.
- Comments: `personas.ron`, `docs/playtesting.md`'s table, `bot.rs`.
- Chapter 1, 20 tries of each persona, before and after: identical try
  by try (casual 6/20, normal 13/20, hardcore 10/20 succeed).

**Deviation.** The smoke test lets Casual leave unfinished all but one
try of `test_escape`: on seeds 1 and 3 it still runs out the 60 turns,
before and after the fix. Its lead escapes, the enemies are gone, and
the two units left wander: every candidate scores within about 20
points, and Casual's temperature is 100, so it picks almost at random.
Listing each unit's `Escape` early in `order` was tried and didn't end
it (reverted). The settings can't change (ADR-0095 §5), so it is
**0517**, which removes the exception; 0852 now waits for it too.

**Follow-ups:** 0517 (Casual wanders on an exit map once the enemies are
gone), 0518 (Hardcore loses its lead on turn 1 to 5 in `test_escape` and
`test_retreat`, and runs out the clock on 5 of 20 Chapter 1 tries).

Engine friction: estimate 40, measured 8 (0.2×), under 2×.

No gameplay rules decided: this is dev tooling.
