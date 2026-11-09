---
id: "0610"
title: "EXP for a level gap, as 0081 decided"
type: feature
milestone: M5 Progression
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: blocked
blocked_by: ["0081"]
nick_input: answer-first
completed:
---

# 0610 — EXP for a level gap, as 0081 decided

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

**0081** asks Nick for a bigger cut for a level gap; this ticket builds
it. His one fixed rule: it never goes to 0.

## Nick input

**Answer first:** ticket 0081. Nothing else is asked here. Verified by
unit and property tests and the bots; what is left to judge by playing
goes on the Act 1 playtest's checklist (1064).

## Scope

**In:** what 0081 decided. Where each likely shape goes:

- **A change inside the formula** (a steeper slope for `d < 0`, a
  different kill bonus): `exp_for_combat`, its constants at the top of
  `progression.rs` (~:50–60), with its doc comment.
- **A multiplier by the gap, applied after the formula** (a table like
  `exp_pace`: e.g. "unit 3+ levels above: × 1/2"): a new table in
  `classes.ron` beside `exp_pace`, loaded by `trpg_content::class` into
  `ClassTable` (as `exp_percent` is, `crates/core/src/class.rs` ~:189),
  applied where the award is made (`battle.rs` ~:3805 and the Line Pierce
  kill ~:3832), keeping a floor so no award is 0.
- **A catch-up bonus for a unit below the army** (if 0081 picks one):
  the same place as the multiplier, the other way.
- In every case:
  - `progression.md` *EXP formulas*: the new rule, its examples table
    redone (the cases there plus Rue at Chapters 3 and 4 above), and the
    sentence in *Pace* that says what it does to the per-battle pace.
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
- **0527** (the bots' per-unit report): either order. If it is done
  first, its per-unit EXP shares before and after go in the PR; if not,
  the arrival army's levels are enough.
- **0333** (armour), **0866** (Joss), **0526** (the AI): either order;
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

- [ ] The rule is what 0081 decided, and `progression.md` shows it with
      this game's numbers (Rue at Chapters 3 and 4).
- [ ] A unit test: a Lv 8 unit killing a Lv 6 enemy gets the award
      0081's rule gives.
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
  `MAX_AWARD`.
- Unit: the worked examples in the design doc.
- Integration: the winnable tests of Chapters 1 to 4; the bots' runs.

## Completion notes

*(Filled in by the session that completes the ticket.)*
