---
id: "0523"
title: Hardcore stops stalling in front of a guarded chokepoint
type: bug
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["0520"]
nick_input: none
completed:
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

- [ ] The new test of a guarded choke passes, and failed before the fix.
- [ ] `cargo xtask playtest ch02 --bot all` on seeds 1–100 and 101–200:
      `hardcore · turns median … lower ✓` on both, every band ✓.
- [ ] `cargo xtask playtest ch01 --bot all`: every band still ✓ and
      Hardcore still faster.
- [ ] `personas.ron` is unchanged.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: a persona at a guarded choke advances.
- Integration: the playtest reports above.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
