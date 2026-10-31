---
id: "0323"
title: "Extra strikes need a gap or a share of the enemy's speed; ceilings HP 200 / stats 100"
type: feature
milestone: M2 Core rules
model: implementation
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-10-23
---

# 0323 — Extra strikes need a gap or a share of the enemy's speed; ceilings HP 200 / stats 100

## Context

Ticket 0013 (2026-10-22) decided the number scale with Nick
(`docs/design/stats-and-combat.md`, *Nick's words* and *Attack speed and
number of strikes*):

- The scale stays **FE-sized**: no rescale of classes, weapons or terrain.
- **Extra strikes** need a fixed Speed gap **or** a share of the slower
  side's attack speed, **whichever is bigger**:
  `need(k) = max(gap[k], max(0, AS_B) * share[k] / 100)`, with
  `gap = [4, 14, 24]` and `share = [25, 75, 150]` (*tunable*). Today the
  code uses the gaps alone (`CombatRules::strike_thresholds`,
  `crates/core/src/combat.rs`).
- The **hard ceilings** are fixed all game: **HP 200, every other stat 100**
  (today `hard_ceilings: (80, 50, 50, 50, 50, 50, 50)` in
  `assets/data/classes.ron`).
- The **level cap** stays **99** through Act 1 (already the data value; only
  its comment changes).

`StatValue` is already `i32` (0302), so nothing needs widening, and
three-digit numbers already fit every panel.

## Nick input

None. Verified by tests; how the late-game strike counts feel goes on the
Act 1 playtest's checklist (1064).

## Scope

**In:**
- The gap-or-share strike rule in `core`, with its numbers in
  `CombatRules`.
- The new hard ceilings and the level cap comment in `classes.ron`.
- Tests and the design doc's worked examples as tests.

**Out (do not do):**
- The level pace curve (ticket 0608).
- Retuning any class, weapon or terrain number (the scale is unchanged).
- Strike skills ("+1 strike") keep working as they do; don't change them.

## Implementation steps

1. In `crates/core/src/combat.rs`, replace
   `strike_thresholds: Vec<StatValue>` with
   `strike_steps: Vec<StrikeStep>`, where
   `pub struct StrikeStep { pub gap: StatValue, pub share_percent: StatValue }`
   (serde-derived like the rest). Default:
   `[(4, 25), (14, 75), (24, 150)]`. Update the doc comments and the
   module doc (line ~27, which names `[4, 14, 24]`).
2. Change `CombatRules::strikes(diff)` to
   `strikes(diff: StatValue, slower_as: StatValue) -> u8`: a step is reached
   when `diff >= max(step.gap, max(0, slower_as) * step.share_percent / 100)`.
   `max_strikes` uses `strike_steps.len()`.
3. Update the caller (`combat.rs` ~line 476) to pass the other side's
   attack speed, and the tests in `crates/core/src/combat/tests.rs` that
   build `strike_thresholds` (~line 875).
4. In `assets/data/classes.ron`: `hard_ceilings: (200, 100, 100, 100, 100, 100, 100)`;
   change the `level_cap` comment to "99 through Act 1 (ticket 0013); may be
   raised for Acts 2–3". Fix any test or snapshot that hard-codes 80 / 50
   as a ceiling (grep `hard_ceilings`, `80`, `MAX`), only where it means the
   ceiling.
5. `docs/design/stats-and-combat.md` already states the rule; change it only
   if the code forces a different wording.

## Acceptance criteria

- [x] Example 3 of `stats-and-combat.md` (diff 14 → 3 strikes; diff 13 → 2;
      Spd 40 vs 16 → 4) and Example 4 (AS 75 vs 40 → 3 strikes) pass as
      table-driven tests.
- [x] A slower side with attack speed ≤ 0 needs only the gaps (test).
- [x] Every existing combat test still passes with unchanged results for
      Chapter 1's numbers (the shares never beat the gaps at Chapter 1's
      attack speeds).
- [x] `classes.ron` has the new ceilings; content validation passes.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the strike-count table above, including each boundary.
- Property: strikes never decrease as `diff` grows (fixed `slower_as`), and
  never increase as `slower_as` grows (fixed `diff`); always `1..=max_strikes`.
- Snapshot / integration: only existing snapshots, updated if a ceiling
  shows.

## Completion notes

Done as planned. `CombatRules::strike_thresholds` became `strike_steps`
(`StrikeStep { gap, share_percent }`, defaults `(4,25) (14,75) (24,150)`);
`strikes(diff, slower_as)` uses `max(gap, max(0, slower_as) * share / 100)`.
`classes.ron` has the ceilings HP 200 / others 100 and the level cap comment.

- Tests: a table of Examples 3 and 4 with each boundary and AS <= 0, a
  property test (monotone in the gap and in the slower speed, always
  `1..=max_strikes`), and the data-driven steps test.
- Deviation: the class-change screen tests and their snapshots built a unit
  at Def "ceiling - 1" (49 of 50); they now read 99 of 100. The content
  test for ceilings reads 200 / 100.
- Engine friction: under 2x (about 40 estimated, about 45 measured).
- No *the agent's starting rules*. Late-game strike counts go on the Act 1
  playtest checklist (1064).
