---
id: "0610"
title: "EXP for a level gap: a steeper slope, the kill bonus never below 7"
type: feature
milestone: M5 Progression
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: blocked
blocked_by: ["0081", "0527", "0531"]
nick_input: none
completed:
---

# 0610 — EXP for a level gap: a steeper slope, the kill bonus never below 7

## Context

Nick played the Act 1 build (Chapters 1 to 4) on 2026-11-08:

> "Mages (i.e. Rue) are too necessary ... armored knights are literally
> impossible to kill without mages so she gets too much combat and thus
> snowballs / On the snowball point, we should penalize it, reduce XP
> depending on lv disparity without going to 0. Fire Emblem has a system
> like this"

**The EXP rule today** (`crates/core/src/progression.rs`,
`exp_for_combat` ~:100; design `docs/design/progression.md`, *EXP
formulas* ~:257–300, FE GBA's shape doubled). With `d = target level −
unit level`:

- no damage: `2`;
- damage: `clamp(2 × ((31 + d) / 3), 2, 100)`;
- kill: `clamp(2 × ((31 + d) / 3 + max(0, 20 + 3d)) + 40 if boss, 2, 100)`.

Then `paced_exp` (~:124) scales it by the unit's own level (`exp_pace`,
`assets/data/classes.ron:31`: 100% to Lv 39, 50% from 40, 25% from 70)
and never gives less than 1. So a gap already costs EXP, gently: a kill
of an equal-level enemy gives 60; 3 levels below the unit, 2 × (9 + 11)
= 40; 5 below, 26; 10 below, 14. In the recorded playthroughs Rue is
Lv 8 at Chapter 3 against Lv 6 Hounds (a kill gives her 46) and Lv 12 at
Chapter 4 against Lv 7 (26) (`docs/playtesting.md` ~:310–330).

**Nick's answer (0081, 2026-11-21; `progression.md`, *EXP formulas*):**
option C, the steeper slope with the kill bonus never below 7, and no
level offset for a promoted unit. With `d = target level − unit level`
(shown levels; promotion adds nothing):

- `hit = (31 + 2d) / 3` (`/` toward zero; the gap counts double);
- no damage: `2` (unchanged);
- damage: `clamp(2 × hit, 2, 100)`;
- kill: `clamp(2 × (max(0, hit) + max(7, 20 + 5d)) + 40 if boss, 2, 100)`
  (`hit` stops at 0 inside the kill formula: the agent's starting rule,
  so a kill always gives at least 14 before the pace);
- `paced_exp` then scales it as today.

Worked examples (before the pace; the old value in brackets): hit at
equal levels 20 (20); hit 5 levels up 26 (24); kill at equal levels 60
(60); kill 1 up 72 (66); kill 5 up 100 (94); Rue Lv 8 kills a Lv 6
guard 38 (46), hits it 18 (18); kill 3 below 30 (40); Rue Lv 12 kills a
Lv 7 guard 28 (26), hits it 14 (16); kill 10 below 20 (14), hit 10
below 6 (14); boss at equal levels 100 (100); missed 2 (2).

## Nick input

None: 0081 answered it. Verified by unit and property tests and the
bots; what is left to judge by playing goes on the Act 1 playtest's
checklist (1064).

## Scope

**In:** the formula above, inside `exp_for_combat`, with its constants
at the top of `progression.rs` (~:50–60: the gap's weight in `hit` (2),
the kill bonus's slope (5) and floor (7)) and its doc comment. No new
data table and no change where `battle.rs` makes the award (~:3805, the
Line Pierce kill ~:3832): they already call `exp_for_combat`.
- `progression.md` already has the rule and its examples (0081); change
  it only if building finds a number there wrong, and say so in the PR.
  - Whatever shows EXP to the player (the results screen, a level-up
    line) needs no change: it reads the event's amount.
  - **Re-balance:** levels change, so level-up rolls move and the
    recorded playthroughs may no longer win. Record them again on
    purpose in order (`crates/ui/tests/it/ch01_winnable.rs` to
    `ch04_winnable.rs`; each header says how) and run the bots on
    Chapters 1 to 4, both seed ranges, inside their bands; the army's
    levels at each chapter (the arrival army, `docs/playtesting.md`)
    before and after go in the PR.

**Out (do not do):**
- Class points, heal and tile-cast EXP, the unused-rewind bonus, unless
  0081 says the cut applies to them.
- The level pace (`exp_pace`) itself.
- **0527** (the bots' per-unit report) and **0531** come first
  (`blocked_by`): 0527's per-unit EXP shares before and after go in the
  PR.
- **0333** (armour), **0870** (Joss), **0528** (the AI): either order;
  whichever lands second re-records the playthroughs and re-runs the
  bands with the first in.

## Implementation steps

1. Read 0081's answer and `progression.rs` (`exp_for_combat`,
   `paced_exp`, the constants), its tests
   (`crates/core/src/progression/tests.rs` ~:190–220, ~:800), and where
   `battle.rs` awards EXP.
2. Build the rule in the shape above; keep every number in a constant or
   the data, and `core` pure (ADR-0004).
3. Update the existing exact tests (~:204–217) to the new numbers, on
   purpose, and add the property tests below.
4. Docs.
5. Re-record the playthroughs, run the bots, record the tables.

## Acceptance criteria

- [ ] `exp_for_combat` gives the formula above; every worked example
      above is an exact unit test.
- [ ] A unit test: a Lv 8 unit killing a Lv 6 enemy gets **38**, a Lv 12
      unit killing a Lv 7 enemy **28**.
- [ ] Property tests (below) pass.
- [ ] The winnable tests pass with re-recorded playthroughs and the bots
      are in their bands on Chapters 1 to 4 (tables in the PR).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Property (`crates/core/src/progression/tests.rs`, proptest): for every
  unit level, target level, result and boss flag, the award after pacing
  is **never 0**; for a fixed target level and result, the award **never
  rises as the unit's level rises** (monotone in the gap); a kill never
  gives less than a hit at the same levels; no award is over
  `MAX_AWARD`; a kill before the pace is never under 14.
- Unit: the worked examples above (the design doc's table).
- Integration: the winnable tests of Chapters 1 to 4; the bots' runs.

## Completion notes

*(Filled in by the session that completes the ticket.)*
