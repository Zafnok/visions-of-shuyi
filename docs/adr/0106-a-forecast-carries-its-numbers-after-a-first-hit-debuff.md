# ADR-0106: A forecast carries its numbers after a first-hit debuff

- **Status:** Accepted
- **Date:** 2026-11-01
- **Related tickets:** 0330, 0325, 0312; 0328 and 0521 build on it

## Context

`combat-arts.md` starts Rending Arrow, Off Balance, Vital Point, Pinning Shot
and Pressure Point "on the first strike that hits", and asks the debuff to
change the rest of that combat. ADR-0022 fixed one validated `Forecast`
before the combat, shared by the resolution, the preview and the AI, and the
battle applied the debuff only after `resolve` returned. So the forecast and
the resolution agreed with each other and both missed the rule.

The rolls decide which strike is the first hit, so no single list of numbers
can be right. `combat.rs` must keep knowing nothing about arts (ADR-0022), and
its callers must not each work the debuff out for themselves.

## Decision

1. **`Forecast` gains `after_first_hit: Option<AfterFirstHit>`**: both sides'
   numbers from the attacker's first hit on. `attacker` and `defender` stay
   the numbers the combat begins with.
2. **The battle fills it when it plans the fight**
   (`BattleState::plan_first_hit`): the same `fighters` and `forecast` as the
   main numbers, on a copy of the defender wearing the debuff. Wearing it goes
   through `Unit::add_effect`, so a refresh replaces the old debuff rather than
   stacking, and `Bonuses::apply` keeps stats at 0 or above.
3. **Everything that plays a combat out asks `Forecast::numbers(by,
   attacker_has_hit)`**: `resolve`, `if_all_hit` and the AI's `Walk`. The strike
   that lands the hit uses the old numbers, and only the attacker's hits count.
4. **The strike list never changes in a combat.** `strike_order` reads the
   forecast's own counts and `set_after_first_hit` copies them over. A Spd
   debuff changes avoid at once and strike counts from the next combat, as the
   design's "doubled by the next attacker" says.
5. **`EffectApplied` still follows `CombatResolved`.** The numbers already use
   the debuff; the event says the target keeps it, and a target that fell gets
   none. The event order and the roll protocol are unchanged.

## Consequences

- Hit chances and a kill can change what is rolled: a corrected hit chance can
  change whether a crit roll is taken, and an earlier kill removes later strikes
  and their rolls. That is the rule working, and it can change what a recorded
  seed plays out to. Save files hold no forecast; `serde(default)` reads an
  older one as having no such numbers.
- `SideForecast::all_hit` is one set of numbers; `Forecast::attacker_all_hit`
  is the attacker's total with the change. `forecast.attacker` is what the first
  strike uses, which the forecast panel's `Hit` and the battle stage's numbers
  show. Damage per strike and totals come from the plan and are exact.
- Every `Forecast` literal names the new field.
- 0328's first-hit states can use the same path, and 0521 reads final inputs.

## Alternatives considered

- **Resolve strike by strike in `battle.rs`**, re-running `forecast` after the
  hit. It would move strike order and the roll protocol out of `combat.rs`, and
  the preview and AI would need their own copies of the loop.
- **Pass the second numbers beside the forecast** to `resolve` and `if_all_hit`.
  Each caller and event would have to carry both, and one could forget.
- **Recount strikes when Spd changes.** No design rule says how strikes would be
  added or reordered mid-combat, and the forecast would have to promise strikes
  that depend on the rolls.
