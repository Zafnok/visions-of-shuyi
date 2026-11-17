---
id: "0484"
title: "A unit's stats are easy to see in battle, as decided in 0078"
type: feature
milestone: M3 Battle UI
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: ["0078"]
nick_input: none
completed:
---

# 0484 — A unit's stats are easy to see in battle

## Context

A friend's playtest of the Act 1 build, passed on by Nick (2026-11-07):
"Make stats easier to see, rather than needing to go to unit info or
leveling up". 0078 asked Nick how; this ticket builds his answer,
recorded in `docs/design/look-and-feel.md`, *Side panel* (read it
first: it has the layout, the formulas and the starting rules).

In short: under today's unit block the side panel adds the stats (Str,
Mag, Dex, Spd, Def, Res, Mov, two to a row), the equipped weapon or
attack spell with its uses, and the combat numbers (Atk, Hit, Crit, Avo,
AS, two to a row), for every unit, enemies included. The stats are the
numbers a fight uses: **with the equipped armor and accessory** and the
timed effects (Nick: "prob with gear"). The Playdate doesn't change.

Where things are:

- The side panel: `crates/ui/src/screens/battle/panel.rs` (`view`,
  `unit_view`), its view `PanelUnitView` in `view.rs` (~:175), painted
  by `paint_panel_unit` in `glyph.rs` (~:327; the picture look paints
  the same panel). Room: the panel is 30 × 30 cells; the unit block ends
  at `UNIT_ROW + 3`, gold sits at `GOLD_ROW`, about 18 rows free.
- Unit info: `battle/info.rs` `view` lists the stats as
  `effect_bonuses(&unit.effects).apply(unit.stats)`: **without gear**,
  so Ellery's Def reads 4 there while her fights use 6 (Leather Vest
  +2). 0078 decided the battle stats show gear; unit info must show the
  same numbers.
- Gear-adjusted stats: `Unit::effective_stats` (`crates/core/src/item.rs`
  ~:717). The fighter as the maths sees it: `Unit::combat_input`
  (`item.rs` ~:785), `BattleState`'s `combatant` and the skill bonuses
  in `battle/skills.rs` (`bonuses`, ~:266: passives, timed effects,
  auras). Formulas: `crates/core/src/combat.rs` (`attack_speed` ~:390,
  `avoid` ~:412, power/hit/crit ~:605–660).
- The weapon line: `art_list::header` already gives
  `Iron Sword 20/20` or a spell's uses.

0078's mockups computed every number this way and matched unit info
and the weapon's own numbers (Ellery on Chapter 1's start: Str 8, Mag
1, Dex 7, Spd 7, Def 6, Res 5, Mov 5; Iron Sword 20/20; Atk 13, Hit
104, Crit 3, Avo 14, AS 5. The Brigand six tiles right of her and two up: Atk 14, Hit
79, Crit 1, Avo 8, AS −1).

## Nick input

None: answered by 0078. Verified by view tests and `frame-png` pictures;
the starting rules in *Side panel* go on the Act 1 playtest's checklist
(1064).

## Scope

**In:**
- The panel's new fields and their glyph painting (both looks).
- One-sided combat numbers computed in `core` (a pure function next to
  the combat maths, so the panel never recomputes formulas in `ui`):
  Atk, Hit, Crit, Avo, AS of a unit standing where it stands, with its
  equipped weapon or attack spell, counting only the bonuses that hold
  in any fight (*Side panel*'s starting rule).
- Unit info's stat numbers switched to the same gear-adjusted numbers
  (its layout unchanged; only timed effects mark a stat as changed).

**Out (do not do):** the forecast (0480); any change to the Playdate's
corner window (Nick: "No flip button"); a key that flips the panel;
unit info's layout; panels on the engine's widgets (1238; if it is done,
build inside its panel).

## Implementation steps

Expected size: about 200 production lines (a core function of ~60 like
`attack_speed`, ~60 in `panel.rs`/`view.rs`, ~60 painting, a few in
`info.rs`), plus tests.

1. **Core:** add a function (e.g. `BattleState::stat_card(unit) ->
   Option<CardNumbers>` or a `CombatantInput::one_sided(&rules)`) that
   returns the gear-and-effect-adjusted `Stats` and `atk`, `hit`,
   `crit`, `avo`, `attack_speed` (`Option` for the three that need a
   weapon). Build it from `combatant` and the always-on bonuses, and
   reuse `attack_speed` and `avoid` (make `avoid` `pub(crate)` if
   needed) rather than copying formulas. Property test: for any unit
   and target, the forecast's hit equals `hit − target's avo` and its
   crit `crit − target's Dex / 4`, before clamping, when neither side
   has a fight-only bonus.
2. **Unit info:** `info::view` lists the adjusted stats from step 1;
   `changed` still compares against the stats without timed effects
   only.
3. **Panel view:** add to `PanelUnitView` the stat cells (name and
   value, the timed-effect mark), the weapon line (`Option<String>`
   from `art_list::header`), and the combat cells (`--` without a
   weapon for Atk, Hit, Crit). Stat names through `stat_name`; the new
   labels (`Atk`, `Hit`, `Crit`, `Avo`, `AS`) as text keys in the
   language files (`check-text`).
4. **Glyph skin:** paint them in `paint_panel_unit`, two cells to a row
   at `TEXT_X` and `TEXT_X + 12`, labels `TextDim`, values `Text`, one
   blank row between the unit block, the stats and the weapon (the
   layout in *Side panel*). Nothing past `TEXT_W`.
5. Snapshots of the panel (an ally, an enemy, a unit with no weapon);
   `frame-png` pictures of Chapter 1 hovering Ellery and a Brigand, both
   looks, for the PR.

## Acceptance criteria

- [ ] Hovering a unit gives the panel view the stats, weapon line and
      combat numbers of *Side panel*, and the stats equal unit info's
      for that unit (test on an ally with armor and an enemy).
- [ ] Unit info shows gear-adjusted stats (Ellery's Def 6 at Chapter 1's
      start).
- [ ] The combat numbers agree with the forecast (the property test of
      step 1).
- [ ] The glyph skin paints them in both looks (snapshots; `frame-png`
      pictures in the PR); the Playdate's corners are unchanged.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit + property (core): the one-sided numbers against the forecast.
- Unit (ui): the panel view's numbers against unit info's.
- Snapshot: the side panel, glyph.

## Completion notes

*(Filled in by the session that completes the ticket.)*
