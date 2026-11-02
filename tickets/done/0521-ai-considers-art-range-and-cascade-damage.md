---
id: "0521"
title: "AI considers art range and Cascade damage when choosing an attack"
type: bug
milestone: M4 Enemy AI
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: ["0325", "0329", "0139", "0330"]
nick_input: none
completed: 2026-11-01
---

# 0521 — AI considers art range and Cascade damage

## Context

The 2026-11-01 audit found that the combat additions in
[PR 454](https://github.com/Zafnok/visions-of-shuyi/pull/454) (0325) and
[PR 474](https://github.com/Zafnok/visions-of-shuyi/pull/474) (0329)
did not reach two consumers in `crates/core/src/ai.rs`:

- `extras` gives every art `range: 0`. `Arm::reach` and `best_attack`
  prune by the longest weapon/active range across the loadout. A boss
  with only a range-1 weapon cannot consider Long Thrust, Hurl or
  Moonwave at range 2 unless another arm or active happens to widen that
  bound. `legal_commands` was updated; the AI's bound was not.
- `Walk::strike` and `struck_hp` count strikes but ignore
  `SideForecast::cascade` and the number of landed hits. Cascade is
  valued like the ordinary attack, then charged its durability cost.
  The combat resolver already adds its bonus after each landed hit.

The AI's module contract promises range changes and forecast damage are
included. 0511 concerns utility beyond forecast numbers, so it does not
cover these omissions. 0330 settles the shared strike inputs first; use
its final representation instead of creating another static damage copy.
The game owns art semantics; an engine API is not needed for this fix.

## Nick input

None. Restore the existing rules and attack-choice contract using failing
AI regressions. Keep weights unchanged. Changed commands and their reason
go in the PR; the Act 1 playtest (1064) judges anything left about feel.

## Scope

**In:**
- Correct broad reach bounds for affordable usable Combat Arts,
  including their replacement minimum/maximum ranges.
- Count landed hits in Cascade's expected-damage/kill walk and use the
  same per-strike damage rules as resolution where possible.
- Preserve 0139's integer scoring, deterministic tie breaks, costs and
  restrictions on which factions/roles may use arts.

**Out (do not do):**
- New utility weights or changing their values (0511). Either order
  works: if 0511 is already done, preserve its utility branches; otherwise
  it must preserve these range/Cascade tests when extending `Odds`.
- Chapter tuning, bot persona changes, new arts or crit valuation.
- First-hit combat mechanics (0330, required first); changing gameplay
  strike-order rules or adding an engine combat abstraction.

## Implementation steps

**Size:** about 100–180 changed production lines, compared with the
current `Arm`/`Extra` bounds and `Walk`, and at most 1,400 changed lines
total with tests and the archived ticket. Split an independently useful
prerequisite before coding if the final 0330 representation needs more.

1. Read `ai.rs`'s attack-choice contract, the art range and Cascade rules
   in `docs/design/combat-arts.md`, and 0330's completed strike path.
2. Write failing AI fixtures first: a stationary spear boss, axe boss and
   Moonblade wielder with no other long-range arm/active, each facing a
   target exactly two tiles away. Verify a legal art is considered rather
   than a wait or an impossible plain attack.
3. Derive safe technique bounds from the real art and weapon data. Keep
   final legality with the battle's existing validation, including
   affordability and who may use an art.
4. Add failing odds-walk cases for Cascade: guaranteed hits, a branch
   with a miss between hits, and a target that falls before later strikes.
   Correct the walk without replacing integer chances with floats.
5. Share or explicitly test the strike-damage calculation against the
   resolver so a future forecast field cannot silently disappear here.
   Run representative AI and persona baselines and record changes without
   tuning them away. 0512 certifies the campaign only after this fix.

## Acceptance criteria

- [x] The range-2 fixtures fail before the fix and select a valid Long
      Thrust, Hurl or Moonwave afterward with no unrelated long-range arm.
- [x] Unaffordable arts and ordinary/noncombatant enemies remain excluded.
- [x] With 100% hit and no crits, Cascade's expected damage/kill result
      matches actual resolution, including early death.
- [x] A hit/miss/hit branch increments Cascade on landed hits only;
      a one-strike attack gains no Cascade bonus.
- [x] The same state gives the same valid command; weights and plain
      attacks with no new effects are unchanged. Any baseline changes
      are measured and explained in the PR.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: range-bound fixtures in `crates/core/src/ai/tests.rs` and direct
  `Odds` cases compared with combat's supported strike inputs.
- Property: existing valid-command/determinism properties and bounded
  Cascade damage comparisons; do not mirror the implementation formula.
- Integration: existing persona/rule-battle baselines, with changed
  commands reported. No new screenshots are required for AI behavior.

## Completion notes

**Done.** The AI now considers an art that reaches farther than any of the
unit's weapons, and its odds count Cascade by the hits landed on each branch.

- **Reach.** Each technique has its own `reach` (an art's replacement maximum,
  or the arm's maximum plus an active's bonus). The broad bound uses it, and a
  technique is skipped at a distance beyond its own reach. The battle still has
  the final say on who may use what and on the minimum range.
- **Strikes.** `Walk` uses the resolver's own `striker`, `base_damage` and
  `take`, so `struck_hp`, the AI's copy, is gone. A property test averages
  `resolve` over every way the rolls can fall and compares that with the odds,
  so a new forecast field can't silently vanish from the walk.

**Red first.** On the old code the range-2 fixtures waited instead of
attacking, and the Cascade cases counted no bonus (three 50% strikes of 5 gave
7.5 expected damage; it is 9.75).

**Baselines.** No result changed on the seeds compared: every try's line matched
its "before" line. Those were the baseline bot on `ch01` (200 tries) and the
three personas on `ch01`, `ch02` and `act1_random_road` (30 tries each, against
the first 30 of a 100-try run on the old code). The bosses of `ch01` and `ch02`
are at Axe D, below Hurl's rank B, and nobody in these battles has Cascade's
rank A.

**Rules decided.** None. Weights are unchanged.

**Follow-up.** 0477: the danger zone, the threat arrows and the AI's own danger
zone read only weapon and spell ranges, so they don't show what an art or an
active reaches.

**For 1064.** Nothing new from these battles.
