---
id: "0523"
title: Hardcore stops stalling in front of a guarded chokepoint
type: bug
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: ["0520"]
nick_input: none
completed: 2026-11-03
---

# 0523 — Hardcore stops stalling in front of a guarded chokepoint

## Context

The report's check "Hardcore is faster than Normal"
(`docs/design/playtest-bots.md`, *Hardcore is faster than Normal*) says a
miss is "a problem with the bots, not the map". On Chapter 2 (Kell's Ford,
autobalanced by 0520 with the campaign army) it passes on seeds 1–100
(median 4 turns against Normal's 5) and fails on seeds 101–200 (5 against
4), with every band met on both.

0520 traced the cause. Hardcore's turns over its clean wins are bimodal:
a third finish on turn 3 (Rue burns Hendry down, Res 1), and a long tail
runs to turns 12 and 13, which Normal doesn't have (seeds 101–200:
Hardcore 3: 13, 4: 10, 5: 10, 6: 13, 7: 6, 8+: 11; Normal 3: 22, 4: 15,
5: 5, 6: 10, 7+: 7). Seed 116, traced command by command: by its turn 4
Hardcore has met the village's minimum and stands on the south bank; from
turn 4 to turn 10 **no unit moves**, it ends its turn seven times in a
row (Maud waits at 2 HP the whole time), and it only crosses the ford on
turns 11–13. Hendry is `Stationary` and his guards `Guard`, so nothing
comes to it.

Why (*0520's reading*, to confirm): every step towards the ford puts a
unit in reach of the guards and Hendry's Pack Hunt (`at_risk` −3000,
`exposed` −10 per HP), while ending the turn costs nothing: `turns`
(−200) counts only in a longer search than one turn's plan, and
`approach` (+3 a tile) is far too small to outweigh one unit at risk.
ADR-0095 fixed the same fault once ("careful personas ended their turns
for 50 turns in a row"); it is back on a map where the enemy holds a
chokepoint. On Chapter 1, a rout where the enemy comes to the army,
caution costs no turns, so Hardcore is faster there.

The stall also moves Hardcore's success rate (a stalling army loses
houses and wins later), so the later chapters' tuning (0849–0854, all
autobalanced on Hardcore's band) should come after this fix.

## Nick input

None. The personas' settings stay as they are; this is a fault of how a
plan is scored, like ADR-0095's. Verified by the bots' reports; anything
left to judge by playing goes on 1064's checklist.

## Scope

**In:**
- Reproduce: Chapter 2, `--bot hardcore --seed 116 --runs 1` wins on
  turn 13; trace it (0520 traced it with a throwaway test calling
  `shuyi_core::bots::play_observed` on `BattleEntrant::persona` and
  printing each Player phase's units and commands).
- Fix the stall in how the personas plan or score
  (`crates/bots/src/persona.rs`): a persona that ends its turn without
  any unit acting or moving, while no enemy can reach it and the battle
  needs it to advance (a boss fight, a seize, a rout with an enemy that
  won't come), must weigh the turn it gives away. Options to compare:
  counting `turns` when a plan ends its turn; a cost for a turn passed
  with nothing done; scoring `approach` by the safest tile on the way.
  Pick by the numbers, and say why in an ADR amending ADR-0095.
- Re-run `cargo xtask playtest ch01 --bot all` and `ch02` (seeds 1–100
  and 101–200); re-autobalance `ch02` if a band moves, re-recording
  `ch02_winning.ron` if the Hounds change (its test says how).

**Out (do not do):**
- The personas' numbers in `personas.ron` (fixed; only 0508 calibrates
  them against Nick's play).
- The bands, the check itself, or Chapters 3–8.
- What a persona knows (no foresight of the enemy's turn).

## Implementation steps

1. Trace seed 116 and one Normal try of the same length to confirm the
   reading above (which candidate wins each step while the army stands).
2. Write a unit test in `crates/bots/src/persona.rs`'s tests: a small
   battle where a `Stationary` boss guards a choke and every approach
   tile is at risk; the persona must not end its turn with no move for
   more than a turn or two. It fails before the fix.
3. Fix it (Scope); the test passes; the proptest that every persona
   plays random battles without a refused command still passes.
4. Re-run the playtests and the autobalance (Scope); record the numbers
   in the Completion notes and in `docs/playtesting.md`.

## Acceptance criteria

- [x] The new test of a guarded choke passes, and failed before the fix.
- [ ] `cargo xtask playtest ch02 --bot all` on seeds 1–100 and 101–200:
      `hardcore · turns median … lower ✓` on both, every band ✓.
      *Lower ✓ on both; Normal and Hardcore ✓; Casual ⚠ (75, 64) and
      the autobalance found no setting: 0524.*
- [ ] `cargo xtask playtest ch01 --bot all`: every band still ✓ and
      Hardcore still faster. *Faster ✓; Hardcore 76 ⚠ on seeds 1–100, 70
      ✓ on 101–200 (`main`: 68 and 77): unmoved over 200 tries, 0524.*
- [x] `personas.ron` is unchanged.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: a persona at a guarded choke advances.
- Integration: the playtest reports above.

## Completion notes

**Done.** Hardcore no longer stands in front of Hendry's ford: the stall
was the order's cap, not the scoring, and the order changed (ADR-0111,
amending ADR-0095 §2).

- **The trace (step 1)** (seed 116, a throwaway test calling
  `play_observed` on the campaign army, scoring every candidate at each
  turn passed with nothing done): turns 4–9 end with nobody moving, as
  0520 saw. Every candidate the persona weighed scored below ending the
  turn (−150 to −5170), **but Hollis stepping into the ford scored +122**
  (not at risk, his counter worth more than his danger) and Rue's advance
  +12. They were places 12 and 16 of the order; Hardcore tries 12, the
  12th being `EndPhase`. Places 1–9 were Tamsin's, the lead's and Aske's
  attacks on each of their three targets (Hendry and two guards), all
  blows they couldn't survive. 0520's reading (ending a turn costs
  nothing) is true but not the cause: nothing better was ever weighed.
- **The fix** (`crates/bots/src/persona.rs`, `order` and the new
  `split_attacks`): each unit's best attack first (its best target); its
  attacks on its other targets after every unit's advance and hold, before
  the other ways of striking. 11 production lines.
- **The test** `hardcore_walks_its_sturdy_unit_up_a_guarded_ford`: a ford
  with a `Stationary` boss and two bowmen at its north end, three frail
  units who can each strike all three (and would fall to the reply), and
  a unit that can't be hurt a turn behind. It failed before the fix (no
  win in three turns: nine attacks and two doomed advances filled the 12
  places, and the sturdy unit's advance was cut) and passes after. It
  also fails under candidates 1 and 2.
- **The three candidates, by the numbers** (Hardcore on Chapter 2, seeds
  101–200, wins with nobody dying):

| Change | Succeeds | Wins by turn (3 / 4 / 5 / 6 / 7 / 8+) | Median | Normal's | Faster |
| ------ | -------- | -------------------------------------- | ------ | -------- | ------ |
| none (`main`) | 57 | 13 / 10 / 10 / 13 / 6 / 11 | 5 | 4 | ✗ |
| 1. `turns` when a plan ends its turn | 61 | 13 / 15 / 13 / 11 / 6 / 6 | 5 | 4 | ✗ |
| 2. `turns` for a turn passed with nothing done | 62 | 13 / 10 / 15 / 15 / 6 / 5 | 5 | 4 | ✗ |
| 3. `approach` by the safest tile | not run: every unit already stood on it (its hold was waiting in place, 0) | | | | |
| **the order** | 60 | 11 / 9 / 21 / 10 / 7 / 7 | 5 | 6 | ✓ |

  1 and 2 change Hardcore alone (Normal's `turns` is 0, so Normal stays
  at 4) and shorten the tail only by a side effect: a unit waiting where
  it stands beats a costly end of turn, which takes its attacks off the
  list and lets Hollis's advance in.

- **The playtests with the fix** (100 tries a range; clean = won with
  nobody dying):

| Battle, seeds | Casual (band) | Normal | Hardcore | Hardcore turns vs Normal |
| ------------- | ------------- | ------ | -------- | ------------------------ |
| ch02, 1–100 (`main`: 47 / 52 / 55, 4 vs 5 ✓) | **75 ⚠** (30–55) | 64 ✓ | 62 ✓ | 5 vs 6 ✓ |
| ch02, 101–200 (`main`: 50 / 61 / 57, 5 vs 4 ✗) | **64 ⚠** | 67 ✓ | 60 ✓ | 5 vs 6.5 ✓ |
| ch01, 1–100 (`main` today: 47 / 65 / 68, 7 vs 9 ✓) | 48 ✓ | 53 ✓ | **76 ⚠** (50–75) | 8 vs 9 ✓ |
| ch01, 101–200 (`main` today: 50 / 53 / **77 ⚠**, 8 vs 9 ✓) | 49 ✓ | 61 ✓ | 70 ✓ | 8 vs 9 ✓ |

  No try passes a turn with nobody moving: traced seeds 105 and 126 (8
  turns, the longest wins left) fight every turn; the 12–13 turn tail is
  gone (longest clean win: 8).
- **Chapter 1 doesn't move**: Hardcore succeeds in 145 of 200 tries on
  `main` and 146 with the fix. It sits at the top of its band either way,
  so which 100 seeds you run decides ✓ or ⚠ (`main` itself reads 77 ⚠ on
  101–200). Follow-up 0524.
- **Chapter 2's Casual band moved, and the autobalance couldn't bring it
  back.** With the order fixed, Casual (its success: a win) wins about as
  often as Normal wins with nobody dying. `cargo shuyi autobalance --stage
  ch02 --seed 1 --turns 60` ran out of its 100 evaluations (4.6 h) with
  Casual 58 ⚠, Normal 65, Hardcore 69, and Hardcore no faster (5 vs 5);
  60 more evaluations from there (2.7 h) found nothing better; the racing
  raiders at level 7 by hand gave Casual 70 and 72. None of it was
  applied: `ch02.ron` and `ch02_winning.ron` are unchanged, since the
  search's best fails the check this ticket is for. Follow-up 0524
  re-tunes Chapters 1 and 2 (Chapter 2's army comes from Chapter 1).

**Deviations:** the ticket's candidates were all scoring changes; the
trace showed the order's cap, so the fix is in the order (still "how the
personas plan", `persona.rs`). Candidate 3 wasn't run, for the reason in
the table. `personas.ron` is unchanged. Two acceptance boxes are not met
(above): Chapter 2's Casual band, and Chapter 1's Hardcore band on seeds
1–100 (at its edge on `main` too); both go to 0524.

**Follow-up tickets:** 0524 (Chapters 1 and 2 back inside their bands
with the new order).

**the agent's starting rules:** none (no gameplay rule; the bots' order
is dev tooling).

**Left to judge by playing (1064):** nothing new (no battle changed).

Engine friction: estimate 15, measured 11 (0.7×), under 2×.
