---
id: "0333"
title: "Armour can be got through, as 0080 decided"
type: feature
milestone: M2 Core rules
model: opus-5.5 / gpt-6.1-sol
effort: high
status: blocked
blocked_by: ["0080", "0527", "0531", "0534"]
blocks: ["1064"]
nick_input: none
completed:
---

# 0333 — Armour can be got through, as 0080 decided

## Context

Nick played the Act 1 build (Chapters 1 to 4) on 2026-11-08:

> "Aske the archer is too weak to even get XP so she snowballs inversely
> and becomes useless almost immediately. Her STR + wep are not enough to
> go thru the heavily armored units. Some things that might make this
> better are more thorough bots automated balancing (i.e. tweaking not
> just the enemy but also the player classes and units and levelups) OR
> introducing piercing mechanic with a guaranteed min hit i.e. 2 or 3 min
> hit for piercing damage"

> "Mages (i.e. Rue) are too necessary ... armored knights are literally
> impossible to kill without mages so she gets too much combat and thus
> snowballs"

**0080 decided it (2026-11-22):** Nick picked "Piercing: min hit", then
**"bow min 2 spear min 3"**. Every bow hit deals at least 2 damage, every
spear hit at least 3, for both sides, like the axe's 5. The rule set and
its edge cases are in `docs/design/weapons-and-items.md`, *Piercing*
(with the Chapters 2–4 table) and Example W5, and in
`docs/design/stats-and-combat.md`, *Attack and damage*. Nick's other
answer (the bots only *report* player-side changes) is ticket **0533**,
not this one.

**The numbers today.** Damage (`crates/core/src/combat.rs`, rules in the
module doc ~:8–60, `forecast` ~:625, `side` ~:651–735):
`power = Str (or Mag) + might × eff_mult`;
`mitigation = max(0, Def (or Res) − pierce) + terrain Def`;
`damage = max(0, power − mitigation)`, halved for a Resist affinity, then
an axe's minimum (`WeaponTrait::AxeMinDamage(5)`, the `kinds:` list of
`items.ron`, `assets/data/items.ron` and the private content's). There
is no other minimum. Each kind has exactly one trait there today (bow
`Effective(Flying, 3)`, spear `Effective(Mounted, 2)`), so a bow and a
spear need their minimum **beside** their trait. `CombatMods::pierce`
(Piercing Lance, pierce 5) is a different thing: it ignores Def. Don't
name the new field `pierce`.

The army the bots bring (`trpg_bots::arrival`, Classic), against the
guards (the only Armored enemies in Chapters 2–4, `hound_guard` /
`warden_guard`, Chain Mail +4 Def): Chapter 2's Lv 5 (23 HP, Def 15),
Chapter 3's Lv 6 (24 HP, Def 15, 18 on the gate at 15,8), Chapter 4's
Lv 9 (26 HP, Def 17, 20 on the gate at 13,12). Aske (Lv 3, Str 6, Iron
Bow, power 12) does 0 to all of them today and 2 ×2 under the rule;
Hollis's Iron Spear 1 → 3; Tamsin's Iron Spear 1 → 3 ×2.

## Nick input

None: he answered in 0080. Verified by property and unit tests,
`cargo shuyi frame-png` pictures of the forecast in the PR, and the bots;
what is left to judge by playing goes on the Act 1 playtest's checklist
(1064).

## Scope

**In:**

- **The minimum for bows (2) and spears (3)**, in the axe minimum's place
  in `side`: after mitigation (terrain included), before the sword
  follow-up and crit. The numbers are data, not code (ADR-0004): a field
  on the kind's entry in `items.ron` (e.g. `(kind: Bow, trait:
  Effective(Flying, 3), min_damage: Some(2))`, the session's choice of
  shape), read by `trpg_content::item`, carried into the weapon of
  `CombatantInput` (`WeaponStats`). The axe keeps `AxeMinDamage(5)`, or
  moves to the same field if that is simpler: the session decides,
  without changing what an axe does. When an art raises an axe's minimum
  (Crushing Swing, 8; Felling Blow), the largest minimum applies.
- **The rule's edges** as the design doc says: both sides (enemy archers,
  guards and riders pierce too); every bow and spear, the special ones
  too (Tor's Bow, the Fordwarden's Spear); arts and actives made with
  one (Piercing Lance ignores 5 Def first, then the minimum holds); a
  broken bow or spear keeps its minimum; a miss deals 0; magic, swords
  and gauntlets have none.
- **EXP needs no change**: a combat that deals the minimum is a damaging
  combat already (`progression.rs`, `exp_for_combat`). One test shows
  Aske's 22.
- **On screen** (the design doc's *agent's starting rules*): the forecast
  shows the number only, no mark, as for the axe's minimum. The unit
  info screen's trait text (`trait_text`,
  `crates/ui/src/screens/battle/art_list.rs` ~:122, and where 0489's
  `Spear  D  ×2 vs Mounted` line is built) names both traits:
  `Bow  D  ×3 vs Flying, min damage 2`, reusing `battle.art.desc.axe_min`
  ("min damage {n}") or a string of its own, in every language pack the
  string checks cover; both skins (glyph and Playdate) if the line is on
  both.
- The AI and the persona bots read forecasts, so they follow the rule
  with no change; check one AI test that scores an attack on a guard.
- **The doc comments**: `combat.rs`'s module doc (the formula block and
  the Axe line) and the header comment of both `items.ron` files say the
  new rule. The design docs already say it (0080).
- **Re-balance Chapters 1 to 4.** Every bow and spear damage changes, so:
  the winnable tests' recorded playthroughs (`crates/ui/tests/it/ch01_winnable.rs`
  to `ch04_winnable.rs`; each header says how to record on purpose),
  then the bots on each chapter, both seed ranges, inside their bands
  (`cargo xtask playtest chNN --bot all`; `cargo shuyi autobalance
  --stage chNN` where a chapter leaves its band), as 0524 (Chapters 1
  and 2) and 0855 (Chapter 1 after the armour weights) did. The tables
  go in `docs/playtesting.md`, with 0527's per-unit lines before and
  after (Aske's *can hurt* share and EXP share should rise).
- **0527's test of the old rule** (`crates/bots/src/units/tests.rs`,
  `at_chapter_2_aske_cant_hurt_the_hound_guards_and_rue_can`) fails once
  a bow's minimum is in: change it to say Aske can now hurt both
  `hound_guard`s, and check a unit that still can't (Maud: her Iron
  Gauntlets' power 4 against Def 15, and no attack spell).

**Out (do not do):**
- The options Nick didn't pick (0080): ignoring Def, a minimum for every
  hit, softer guards, an Armorslayer, an attack vow for Maud, Rending
  Arrow at rank D.
- The bots trying player-side changes: **0533** (blocked by 0527).
- The EXP cut for a level gap (0610), Joss's stats (0870) and who wields
  the Houndmaster's Axe (0332). **Either order with each:** whichever
  lands second re-records the playthroughs and re-runs the bands with the
  first in.
- A forecast redesign (0480, asks Nick). No mark is asked for, so nothing
  waits on it.
- New Combat Arts or skills.

## Implementation steps

1. Read the design doc's *Piercing* and Example W5, `combat.rs` (`side`,
   `CombatRules`, `CombatMods`, `WeaponStats`), its tests
   (`crates/core/src/combat/`), `trpg_content::item` (the kinds' list),
   and the done tickets 0524 and 0855 for how a re-balance was run.
2. Add the kind's minimum to the data and to the weapon's stats; apply it
   in `side`; keep `core` pure.
3. Update the doc comments and the unit info screen's trait text, with
   its snapshot tests.
4. Re-record the four playthroughs in order (1 → 4), reading each new
   fixture's last lines (who fell, how many turns).
5. Run the bots on Chapters 1 to 4; autobalance a chapter that leaves its
   band; record the tables.

**Expected size** (ADR-0101): about 60 production lines (a data field,
its loading, one line in `side`, the trait text), 200 of tests, the
re-recorded fixtures (generated) and the playtesting tables.

## Acceptance criteria

- [ ] The rule is the one in `weapons-and-items.md`, *Piercing*, with its
      numbers in the data, not in code.
- [ ] Unit tests of Example W5: Aske (Iron Bow) against a Lv 5
      `hound_guard` on a plain gets `dmg 2 ×2`, no counter; on a gate
      still 2; Hollis (Iron Spear) gets 3 and the guard's counter 3.
- [ ] A unit test: a crit of a minimum bow hit deals 6; a broken Iron Bow
      still deals 2; Piercing Lance against a guard gives the larger of
      its damage and 3.
- [ ] Property tests (below) pass.
- [ ] The unit info screen shows `×3 vs Flying, min damage 2` for bows
      and `×2 vs Mounted, min damage 3` for spears (snapshot).
- [ ] `frame-png` pictures of the forecast of Aske's attack on a guard,
      glyph and Playdate skins, are in the PR.
- [ ] The four winnable tests pass with re-recorded playthroughs, and the
      bots are in their bands on Chapters 1 to 4 (tables in
      `docs/playtesting.md` and the PR).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the worked examples (W5, W5b, W5c) and the edges above, in
  `crates/core/src/combat/` tests; the content loader reading the field.
- Property (`proptest`, as the combat tests do): a minimum never lowers
  damage (new damage ≥ old damage for every input); a bow hit that lands
  deals at least 2 and a spear hit at least 3; damage still never rises
  with the defender's Def; nothing changes for a sword, gauntlet, axe or
  spell.
- Snapshot: the unit info screen's trait line.
- Integration: the winnable tests of Chapters 1 to 4; the bots' runs.

## Completion notes

*(Filled in by the session that completes the ticket.)*
