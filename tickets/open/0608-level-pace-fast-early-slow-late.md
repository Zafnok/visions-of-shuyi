---
id: "0608"
title: "Level pace: about 2 levels per battle early, 1 every 2 battles late"
type: tuning
milestone: M5 Progression
model: opus-5.5
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0608 — Level pace: about 2 levels per battle early, 1 every 2 battles late

## Context

Ticket 0013 (2026-10-22) set the level cap at 99 for Act 1, and Nick
revised the level pace (`docs/design/progression.md`, *Character level and
unit EXP*):

> "99 after 50 battles seems too easy it should probably avg 1 level per
> battle, with early lvs easier to hit (maybe 2 per battle) and higher lvs
> harder to hit (2 battles per lv)"

Today every EXP award (`crates/core/src/progression.rs`, `exp_for_combat`
and the fixed awards) is sized for **2 levels per battle at every level**
(the formulas are FE GBA's, doubled). So a unit reaches 99 in about 50
battles. This ticket makes high levels slower.

## Nick input

None. The numbers are *tunable* starting values; verified by tests. How the
pace feels goes on the Act 1 playtest's checklist (1064).

## Scope

**In:**
- A data table that scales every unit EXP award by the unit's character
  level.
- Tests that pin the resulting pace.
- `progression.md`: replace "Until 0608 lands…" with the table.

**Out (do not do):**
- Class points (CP) pace: unchanged.
- The level cap (stays 99) and "100 EXP = 1 level" (the EXP bar stays
  0–99).
- Weapon EXP.

## Implementation steps

1. Add to `assets/data/classes.ron` (next to `level_cap`) an
   `exp_pace: [(from_level, percent)]` list, strictly increasing levels,
   first entry at level 1. Starting values (*tunable*):
   `[(1, 100), (40, 50), (70, 25)]`: about 2 levels per battle to Lv 40,
   1 per battle to Lv 70, then 1 every 2 battles. That reaches Lv 99 after
   about 110 battles (20 + 30 + about 60), about 0.9 levels per battle on
   average, close to Nick's "avg 1". Load and validate it in
   `trpg-content` like `level_cap`.
2. In `core`, apply it where unit EXP is granted (`grant_exp` or its
   callers): `award = max(1, award * percent / 100)` with the percent for
   the unit's character level **before** the award. The allied EXP pool
   (green units) is scaled per receiving unit, at that unit's level.
   Keep the clamp so one award still gives at most one level.
3. Update `progression.md`: the *Pace* bullet gets the table and the
   example; the EXP formulas heading notes the scaling.

## Acceptance criteria

- [ ] A test: a unit that earns a typical battle's EXP (state the award set
      in the test, e.g. 200 EXP before scaling) gains about 2 levels per
      battle at Lv 10, about 1 at Lv 50 and about 0.5 at Lv 80, and reaches
      Lv 99 in 100–120 battles.
- [ ] Awards never fall below 1 EXP; the 100-per-award limit still holds.
- [ ] Chapter 1 (levels 1–5) gives exactly the same EXP as before (its
      levels are all in the 100% band).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the pace test above; the band lookup at each boundary (39/40,
  69/70).
- Property: a higher level never gets a bigger scaled award for the same
  event.
- Content validation: `exp_pace` sorted, starts at level 1, percents
  1–100.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
