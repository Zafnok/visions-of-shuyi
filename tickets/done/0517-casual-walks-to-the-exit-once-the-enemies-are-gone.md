---
id: "0517"
title: "The Casual persona walks to the exit once the enemies are gone"
type: bug
milestone: M4 Enemy AI
model: opus-5.5
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-10-27
---

# 0517 — The Casual persona walks to the exit once the enemies are gone

## Context

Found by 0516 (the personas' escape and downed-boss scoring, and their
smoke test over `assets-test/battles/`). With 0516's scoring, the Casual
persona still doesn't finish `test_escape` on seeds 1 and 3 in 60 turns:
its lead escapes, the enemies are gone, and the two units left wander
between x 5 and 7 for 50 turns instead of walking the 7 tiles to the
exit (column 13). Normal and Hardcore finish every try.

Why, as far as 0516 saw: once no enemy is left, every candidate command
scores within about 20 points of the others (`approach` is 2 a tile,
`progress` 10 an escape, and nothing else moves), while Casual's
`temperature` is 100 (a pick 100 points below the best is half as
likely; `crates/bots/personas.ron`). So it picks almost at random among
its 10 candidates. Its `Escape` commands also come last in
`trpg_bots::order` (`crates/bots/src/persona.rs`: "the rest in the legal
order"), where the cap of 10 candidates can cut them; 0516 tried listing
each unit's escape early, and it did not end the stall on its own.

A casual player doesn't wander in an empty field; Chapter 6 (0852, an
escape map) would read Casual's band off tries that stall.

The persona settings and weights are fixed (ADR-0095 §5): only the
comparison with Nick's play (0508) may change them, and never for one
battle. So the fix is in how the game hands the persona its choices (the
order, a count that means what a player sees), not in the numbers.

## Nick input

None, unless the only fix found is a setting or a weight: then stop,
say so in the ticket, and it waits for 0508.

## Scope

**In:**
- Find why Casual stalls (play `cargo xtask playtest test_escape --bot
  casual --seed 1 --runs 1` and follow its commands) and fix it in
  `crates/bots/src/persona.rs` without changing `personas.ron`'s numbers.
- In `crates/bots/tests/baseline.rs`, `the_personas_finish_the_rule_battles`:
  drop the Casual-on-`test_escape` exception, so every try of every
  persona ends in 60 turns.

**Out (do not do):**
- Any weight, band, floor or persona setting (ADR-0095 §5).
- Hardcore losing its lead early on `test_escape` and `test_retreat`
  (0518).

## Implementation steps

1. Reproduce: the smoke test with the exception removed fails on
   `test_escape`, casual, seed 1.
2. Look at the candidates Casual gets on a turn with no enemy left
   (`order`'s output) and their scores; check whether an `Escape` and
   the advance towards the exit are in the first 10.
3. Fix it in the order or the scoring. Chapter 1 has no exit: its
   playtest (`cargo xtask playtest ch01 --bot all --runs 20`) must give
   the same measures try by try before and after, unless the fix is
   meant to touch it (then say why in the PR).
4. Remove the exception from the smoke test.

## Acceptance criteria

- [x] `the_personas_finish_the_rule_battles` has no exception and passes.
- [x] `personas.ron` is unchanged.
- [x] Chapter 1's 20-try measures, before and after, in the PR.
- [x] All gates in the `run-gates` skill pass, but `main`'s own red `every_design_cue_is_in_the_manifest` (fixed in another PR).

## Tests required

- Unit: the fixed order or count (`crates/bots/src/persona/tests.rs`).
- Integration: the smoke test without the exception.

## Completion notes

Why Casual stalled: with no enemy left, `order` still filled Casual's 10
candidates with "the rest in the legal order", waits on any tile the
units could reach, most of them no nearer the exit. Every candidate
scored within about 20 points of the others, so at a temperature of 100
Casual picked nearly at random among mostly pointless waits, and its
escapes, listed last, were cut by the cap.

The fix, in `trpg_bots::order` (`crates/bots/src/persona.rs`), no number
in `personas.ron` changed:
- each ready unit's escape is listed once, right after the heals and
  before the advances;
- when no enemy of the player is left on the map, the rest's unit
  actions are left out (a player with nothing to fear doesn't wander):
  the candidates are the AI's pick, the escapes and the advances (and a
  move after acting, and ending the phase).

Verified:
- The smoke test without the exception failed on `test_escape`, casual,
  seeds 1 and 3, before the fix, and passes after.
- Unit test `with_no_enemy_left_order_offers_the_escapes_and_advances_only`.
- Chapter 1 (`cargo xtask playtest ch01 --bot all --runs 20`): every
  try's measures identical before and after (Casual 5/20, Normal 13/20,
  Hardcore 10/20 succeed). Chapter 1 always has its boss on the map until
  it is won, and has no exit, so neither change can touch it.

Engine friction: estimate 15, measured 14 (0.9×), under 2×.

Follow-ups: none. `main` has been red since #448 (0025): the world map
cues are in `audio.md` but not in the manifest or the test's `RESERVED`
list, so `every_design_cue_is_in_the_manifest` fails on every branch;
another session's PR fixes it.

No starting rules: bot tooling only, nothing a player sees.
