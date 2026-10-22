---
id: "0321"
title: "Four armour weights in the data: cloth, leather, chain, plate, and each class's max"
type: feature
milestone: M2 Core rules
model: sonnet-5
effort: medium
status: todo
blocked_by: ["0073"]
nick_input: none
completed:
---

# 0321 — Four armour weights in the data

## Context

Ticket 0073 replaced the three armour weights (Light / Medium / Heavy)
with four, by material: **Cloth, Leather, Chain, Plate**. Each class now
has a **max armour weight** and wears that weight and every lighter one.
The rules, the starter pieces and their numbers are in
`docs/design/weapons-and-items.md` (*Armour*); every class's max is in
`docs/design/progression.md`'s class tables (column *Max armour*).

Today:

- `crates/core/src/class.rs`: `enum ArmourWeight { Light, Medium, Heavy }`
  (already `Ord`) and `ClassDef::armour: Vec<ArmourWeight>`.
- The checks `class.armour.contains(&a.weight_class)` in
  `crates/core/src/item.rs` (two places), `crates/core/src/prep.rs` and
  `crates/core/src/battle.rs`.
- `crates/content/src/enums.rs`: `mirror!(RawArmourWeight => …)`;
  `crates/content/src/class.rs` reads `armour: [Light, Medium]`.
- `Unit::effective_stats` (`crates/core/src/item.rs`) adds armour bonuses
  but has no floor at 0, and Iron Plate now gives **Res −3**. Stats are
  never negative (`stats-and-combat.md`).
- The Preparations reason text (`crates/ui/src/screens/preparations.rs`,
  keys `prep.reason.armour.light|medium|heavy` in `assets/lang/en/ui.ron`
  and `assets-test/lang/test/ui.ron`).
- `assets/data/items.ron`: Leather Vest, Chain Mail, Iron Plate, Warded
  Robe. 0073 retires the Warded Robe and adds the Linen Robe.

## Nick input

None. Verified by tests and a `cargo xtask playtest` / `autobalance` run
on Chapter 1 (Maud and Rue now wear robes, Iron Plate is heavier and
tougher). What is left to judge by playing (the numbers' feel) goes on
the Act 1 playtest's checklist (1064).

## Scope

**In:**
- `ArmourWeight { Cloth, Leather, Chain, Plate }` in that order (lightest
  first), in `core` and the content mirror.
- `ClassDef::max_armour: Option<ArmourWeight>` replacing `armour`
  (`None` = wears no armour: the two elementals), a method
  `ClassDef::can_wear(ArmourWeight) -> bool` (`Some(max) && w <= max`),
  and every check switched to it.
- `classes.ron`: `armour: [..]` becomes `max_armour: Leather` (etc.), from
  `progression.md`'s tables; omitted for the elementals.
- `items.ron`, the four starter pieces from `weapons-and-items.md`:
  `linen_robe` (Cloth, Res +4, wt 0, 300), `leather_vest` (Leather, Def +2
  Res +2, wt 1, 400), `chain_mail` (Chain, Def +4 Res +1, wt 3, 800),
  `iron_plate` (Plate, Def +8 Res −3, wt 6, 1500). Remove `warded_robe`.
- `effective_stats`: each stat floored at 0 after the bonuses.
- Chapter 1's loadouts in `assets/data/characters.ron`: `keeper` (Maud)
  and `heretic` (Rue) wear `linen_robe`. Fix any test file in
  `assets-test/` that names `warded_robe` or a class weight that no longer
  fits.
- The reason text: keys `prep.reason.armour.cloth|leather|chain|plate`,
  English `can't wear cloth armor` / `leather armor` / `chain armor` /
  `plate armor` (American spelling), in both language packs.
- If 0443 is done: the basic shop's rows (`assets/data/basic_shop.ron`)
  sell the Linen Robe, Leather Vest, Chain Mail and Iron Plate from
  Chapter 2 (not the Warded Robe; Iron Plate no longer waits for
  Chapter 5). If it isn't, 0443 uses these pieces itself.

**Out (do not do):**
- The Steel armour (decision 0074, then its own data ticket).
- Any Tailor or specialty-store logic (0320).
- Changing the burden formula or any other number.

## Implementation steps

Expected size: about 60 production lines changed (an enum rename, one
field and one method, four call sites, a floor, a reason mapping), plus
the data and tests.

1. `crates/core/src/class.rs`: rename the variants (docs: "Cloth armour
   (robes).", …), replace `armour` with `max_armour: Option<ArmourWeight>`
   and add `can_wear`. Fix the `ArmourDef::weight_class` doc in `item.rs`.
2. Switch `item.rs` (both checks), `prep.rs` and `battle.rs` to
   `class.can_wear(a.weight_class)`.
3. `effective_stats`: `base.saturating_add(bonus).clamp(0, ceiling)`.
   Update the module docs at the top of `item.rs` ("never below 0").
4. `crates/content`: the mirror enum, `class.rs`'s raw field
   (`#[serde(default)] max_armour: Option<RawArmourWeight>`) and its
   tests.
5. Data: `classes.ron` (each class's max from `progression.md`; the
   header comment explains "wears this weight and every lighter one"),
   `items.ron`, `characters.ron`; the loadout validation in
   `crates/content/src/character.rs` keeps reporting armour a class can't
   wear (its test names Iron Plate on a class that can't wear it: pick a
   leather-max class).
6. `crates/ui/src/screens/preparations.rs`: the reason mapping and its
   test table; both `ui.ron` files.
7. Fix the remaining tests that name `ArmourWeight::Light|Medium|Heavy`
   or `warded_robe` (`crates/core/src/battle/tests.rs`,
   `battle/tests/shop.rs`, `combat/tests.rs`, `progression/class_change/tests.rs`,
   `crates/content/src/item/tests.rs`, `crates/ui/src/screens/*/tests.rs`).
   Keep each test's meaning (a class that can't wear a piece still can't).
8. Run `cargo xtask playtest` and `cargo xtask autobalance` on Chapter 1;
   note the change in the Completion notes.

## Acceptance criteria

- [ ] `ArmourWeight` has exactly `Cloth, Leather, Chain, Plate`, ordered lightest first.
- [ ] Unit test: a Plate-max class can wear all four; a Leather-max class can wear cloth and leather but not chain; a class with no max wears nothing.
- [ ] Unit test: a unit with Res 2 wearing Iron Plate has effective Res 0, not −1.
- [ ] `items.ron` has the four starter pieces with 0073's numbers and no `warded_robe`.
- [ ] Every class's `max_armour` in `classes.ron` matches `progression.md` (content test, or checked by hand and listed in the Completion notes).
- [ ] Maud and Rue start Chapter 1 in Linen Robes.
- [ ] The Preparations list shows `can't wear plate armor` on Iron Plate for a chain-max unit (snapshot or text test).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit (`core`): `can_wear`, the Res floor, the shop/prep checks with the new weights.
- Content: `classes.ron` and `items.ron` load; the loadout validation error.
- Snapshot / integration: the Preparations reason text.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
