---
id: "0516"
title: "The persona bots score an escape and a downed boss right, and play the rule battles in the tests"
type: bug
milestone: M4 Enemy AI
model: sonnet-5
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
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

- [ ] The step 2 test passes: escaping scores higher than waiting on the
      exit, for every persona.
- [ ] Downing an enemy counts in `enemy_fell` (test on `test_hold`).
- [ ] The persona smoke test passes: every try of the four rule battles
      ends in 60 turns, and `test_escape` is won at least once.
- [ ] Chapter 1's 20-try measures are identical before and after (both
      summaries in the PR).
- [ ] `baseline.rs` has no play loop of its own.
- [ ] `personas.ron`, `docs/playtesting.md` and `bot.rs` say what the
      counts and the baseline do now.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the escape and down tallies (`persona/tests.rs`).
- Integration: the persona smoke test over `assets-test/battles/`.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
