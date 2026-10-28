---
id: "0318"
title: "Cinder Oath can't be used to counter, either"
type: feature
milestone: M2 Core rules
model: sonnet-5
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-10-28
---

# 0318 — Cinder Oath can't be used to counter

## Context

0056 decided that Rue's Cinder Oath stays range 1 and the enemy can't
counter it; 0317 built that. 0317 also set a *Claude's starting rule*:
"no counter" covers only Rue's own attack, so when an enemy attacks Rue
with Cinder Oath equipped, she still counters with it
(`docs/design/magic.md`, *Personal signature spells*, Cinder Oath).

Chapter 1 playtest round 3 (0804, 2026-10-16), asked whether a
close-range mage spell makes sense now, Nick picked **"She shouldn't
counter with it either"**. So Cinder Oath is an attack Rue chooses on her
turn, never a counter.

**Waits for the engine carve and the Playdate port** (Nick, 2026-10-16),
so `blocked_by` has 1118 and 1050.

**Re-gated 2026-10-21 (0129):** the carve is done and 1050 was folded
into the Act 1 playtest (1064), so neither 1118 nor 1050 is in
`blocked_by` any more.

## Nick input

None. Nick no longer checks gameplay tickets after merge (Nick,
2026-10-21: "I will save all my checking for either design (i.e. 00
tickets) or for the full act 1 playtest itself"). Verification is by
tests, `cargo shuyi frame-png` pictures in the PR and the playtest bots
(0506, 0509). What the old sign-off asked goes on the checklist of
Nick's next playtest (1064, the Act 1 playtest, if this lands before
it):

> Have an enemy attack Rue next to her while Cinder Oath is
> her equipped attack; the forecast shows she doesn't strike back.

## Scope

**In:**
- A spell flag for "can't be used to counter" (or reuse 0317's
  no-counter data, if one flag covers both cleanly: say which in the
  completion notes). Cinder Oath gets it in `assets/data` (its spell
  entry).
- Counter rules (`crates/core`, where a defender's counter attack is
  chosen): a spell with the flag never counters. *Claude's starting
  rule:* if it's the defender's equipped attack and the defender has
  another attack that could counter (a weapon or another spell in
  range), she doesn't switch to it; the defender just doesn't counter,
  as with a broken weapon. List this rule in the Completion notes (1064's checklist).
- The enemy-attack forecast shows "no counter" on Rue's side.
- The enemy AI sees this too (it scores attacks with the same forecast).
- `docs/design/magic.md`: replace the starting rule with Nick's answer.

**Out (do not do):**
- Any other spell; the spell list's description line (0453).

## Implementation steps

1. Find 0317's no-counter rule (its completion notes name the files) and
   where the counter attack is picked.
2. Add the rule and the data; update the forecast.
3. Re-run the Chapter 1 playtest bots (`cargo xtask playtest ch01`,
   docs/playtesting.md) and put the before/after win rate in the
   completion notes.

## Acceptance criteria

- [ ] An enemy attacking Rue, who has Cinder Oath equipped, gets no
      counter (a core test).
- [ ] The forecast says "no counter" for Rue in that case (a UI test).
- [ ] Rue attacking with Cinder Oath still gets no counter from the enemy
      (0317's tests pass).
- [ ] `magic.md` updated; bot numbers in the completion notes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit (core): the counter is skipped for a flagged spell; property test
  that no combat ever records a counter made with Cinder Oath.
- Integration: the enemy-phase forecast in the battle harness.

## Completion notes

- Reused 0317's `no_counter` spell flag (it covers both cleanly: a spell
  with it is never countered, and never counters). Cinder Oath already had
  it in `assets/data/spells.ron`. The rule is `BattleState::uncounterable`
  (`crates/core/src/battle.rs`), which now also checks the defender's
  equipped spell; the plan, the forecast, the combat and the AI's
  `plain_forecast` all go through it, so the forecast shows `no counter`
  on Rue's side (the existing UI line for an empty defender side).
- Tests: core `a_no_counter_spell_never_counters_when_attacked` (forecast,
  combat, no use spent) and the property test
  `a_no_counter_spell_is_never_a_counter`; 0317's tests still pass.
- `docs/design/magic.md` updated.
- Bot win rates (`cargo xtask playtest ch01`): not run here (heavy for
  Nick's machine); CI and the Act 1 playtest cover it.
- Engine friction: under 2x (about 10 production lines against 10 estimated).

**Claude's starting rule (for 1064's checklist):** if Cinder Oath is Rue's
equipped attack and an enemy attacks her, she doesn't switch to another
weapon or spell to counter; she just doesn't counter, as with a broken
weapon. Have an enemy attack Rue next to her while Cinder Oath is her
equipped attack; the forecast shows she doesn't strike back.
