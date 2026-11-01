---
id: "0521"
title: "AI considers art range and Cascade damage when choosing an attack"
type: bug
milestone: M4 Enemy AI
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["0325", "0329", "0139", "0330"]
nick_input: none
completed:
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

- [ ] The range-2 fixtures fail before the fix and select a valid Long
      Thrust, Hurl or Moonwave afterward with no unrelated long-range arm.
- [ ] Unaffordable arts and ordinary/noncombatant enemies remain excluded.
- [ ] With 100% hit and no crits, Cascade's expected damage/kill result
      matches actual resolution, including early death.
- [ ] A hit/miss/hit branch increments Cascade on landed hits only;
      a one-strike attack gains no Cascade bonus.
- [ ] The same state gives the same valid command; weights and plain
      attacks with no new effects are unchanged. Any baseline changes
      are measured and explained in the PR.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: range-bound fixtures in `crates/core/src/ai/tests.rs` and direct
  `Odds` cases compared with combat's supported strike inputs.
- Property: existing valid-command/determinism properties and bounded
  Cascade damage comparisons; do not mirror the implementation formula.
- Integration: existing persona/rule-battle baselines, with changed
  commands reported. No new screenshots are required for AI behavior.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, baseline changes, follow-ups and anything for 1064.)*
