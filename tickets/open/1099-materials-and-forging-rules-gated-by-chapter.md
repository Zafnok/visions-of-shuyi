---
id: "1099"
title: "Materials and forging rules, gated by chapter"
type: feature
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["0087", "1064", "1091"]
nick_input: answer-first
completed:
---

# 1099 — Materials and forging rules, gated by chapter

## Context

Nick's storefront-hook search (2026-11-16) ended on legendary beasts
hidden at the bottom of dungeon delves. Their reward is **forging**: a
broken beast part drops a material, and the blacksmith forges it into a
weapon along a tree (Iron Spear → Steel Spear → Rime Lance). Nick on the
balance: "we shouldnt go overboard with [traits] or else we turn from
SRPG to looter", and "we need gated progression gates like you cannot
get tier 3 weps super early … we can prob gate based on act/chapter like
we're doing w steel". So **the beast part decides what kind of weapon,
the chapter decides how strong**, and forging changes what a weapon
*does* (a trait such as "a hit leaves frost on the target's tile") more
than its numbers.

This ticket builds the rules and the data; the screen is 1275, carving a
part into a material is 1098, and the grinder bot that keeps crafting
optional is 1276. **The rules come from 0087's answers** in
`docs/design/weapons-and-items.md` (*Forging*) and
`docs/design/beasts.md`; where they differ from the sketch here, the
decision wins. 0087's questions 4 (numbers), 5 (traits per weapon),
6 (farming) and 11 (the gates and the trait list) are the ones this
ticket reads.

What it builds on:

- **Items** (`crates/core/src/item.rs`): `ItemDef` (Weapon, Armour,
  Accessory, Consumable, Seal), `ItemTable`, `WeaponDef` (its `special`
  and `WeaponPassive::mods` are the model for a built-in effect),
  `WeaponInstance { def, durability_left }` and the party `Stock`.
  Loaded by `crates/content/src/item.rs` (`RawWeapon`, …) from
  `assets/data/items.ron`, the public stand-in of the private
  `content/data/items.ron` (ADR-0117).
- **The chapter gate** of the basic shop: `shop::Unlock::Chapter(n)` and
  `BasicStock::shop(chapter)` (`crates/core/src/shop.rs`,
  `crates/content/src/basic_shop.rs`), the number from
  `ChapterDef::number` (`crates/ui/src/flow.rs`, `basic_shop`). Iron from
  Chapter 2, Steel from Chapter 5, a third tier at Act 2 a placeholder.
- **Trades between battles**: `Campaign::trade` with `TownTrade`
  (`crates/core/src/worldmap.rs`) and `shop.rs`'s `buy`, `sell`,
  `repair`, `pay`, each checking before it changes anything. Repairs of
  rare weapons with materials were deferred "until materials exist"
  (`weapons-and-items.md`, *Durability*).
- **Combat**: `CombatMods` and `Unit::combat_input`, which already add a
  special weapon's passive to each of its strikes.
- **Element states on tiles** (1091: `BattleState::react`,
  `Event::TileStateChanged`), for a trait that leaves a state (step 6).

## Nick input

**Answer first:** ticket 0087. Nothing is asked after merge: it is
verified by tests; the agent's starting rules and anything left to judge
by playing go in the Completion notes for the Act 1 playtest's
checklist (1064) or the next playtest after this chain.

## Scope

**In:**
- A **material** item kind, kept in the party `Stock` as counted items.
- **Forged weapons**: ordinary `WeaponDef`s with **traits** from a trait
  table, and the traits' effects in combat (0087's list, about 10–12).
- **Recipes** (from weapon + materials + gold → forged weapon) in a new
  data table, each gated by a **tier** whose unlock is a chapter.
- `Campaign::forge`, checked before it changes anything, and the farm
  limit if 0087 sets one the rules enforce (`Campaign::gain_material`).
- Content checks: a forged weapon never outdoes its tier (0087 Q4), no
  more traits than allowed (Q5), recipes name real weapons and materials.
- Public stand-in data, test data, and the real data in the private
  repository (`content-change` skill).

**Out (do not do):**
- The forge screen (1275). Carving a broken part (1098). Beasts and their
  parts (1095–1097). The grinder bot (1276). Delve loot (1277) and hunts
  (1279): they hand materials over through `Campaign::gain_material`.
- Forging armour or accessories, forging at an on-map Blacksmith or in
  Preparations, unless 0087 asks for it (then a follow-up ticket).
- Materials as rewards for stars or visions (0048's decision).
- The frost wyrm's own materials and recipes (1280).

## Implementation steps

1. Read 0087's *Forging* (`weapons-and-items.md`), `beasts.md` and
   *The grinder bot* (`playtest-bots.md`). Measure the trait list: if more
   than about six traits need new combat code (not `CombatMods` fields
   that exist), the ticket won't fit about 2,000 lines; split it before
   coding with `write-ticket` (this number keeps materials, recipes,
   gates and the traits that reuse `CombatMods`; the rest is a new
   ticket blocked by it, which 1276 and 1280 then name too).
2. **Materials** (`item.rs`): `ItemDef::Material(MaterialDef { name,
   rarity, price })` (`rarity` only if 0087 has common and rare parts),
   `ItemTable::material`, and the `name`/`price` arms. In `shop.rs`, no
   shop stocks a material; whether one is sold, and where, is 0087's (if
   it is silent: never, as a seal, *the agent's starting rule*). Every
   other `match` on `ItemDef` in `core`, `content`, `ui` and `bots` gets
   its arm (the Preparations pack takes consumables only; a material is
   never packed).
3. **Traits** (`item.rs`): `TraitId`, `ForgeTrait { name, effect:
   TraitEffect }` in a new `ItemTable::forge_traits` (battle data, so it
   stays in `GameTables::items`), and `WeaponDef::traits: Vec<TraitId>`.
   A weapon with traits is **forged**: `WeaponDef::is_forged`; no shop
   stocks one (`ShopKind::stocks`, as for `is_special`). `TraitEffect`
   has one variant per kind of effect 0087 lists; a trait made of combat
   numbers returns `CombatMods`, added in `Unit::combat_input` next to the
   passive's mods.
4. **Recipes and tiers** (new `crates/core/src/forge.rs`): `Tier { id,
   unlock: Unlock, caps: BTreeMap<WeaponKind, ItemId> }` (the shop weapon
   a forged weapon of that kind and tier may not outdo), `Recipe { id,
   from: ItemId, to: ItemId, tier, materials: Vec<(ItemId, u32)>, gold:
   Gold }`, `ForgeTable { tiers, recipes, farm }`, and
   `ForgeTable::recipes_for(&self, weapon, chapter) -> impl Iterator`.
   Reuse `Unlock` (so a special unlock, 0044, works here too).
5. **Forging** (`forge.rs` and `campaign.rs`): `forge(recipe, copy,
   stock, gold, chapter, items) -> Result<Forged, ForgeError>` checks the
   tier is reached, the copy is the recipe's `from`, the materials and the
   gold are there, then takes them and turns the copy into `to`
   (durability as 0087 says; if silent, full, *the agent's starting
   rule*). `Campaign::forge(&mut self, forge, items, chapter, order:
   &ForgeOrder) -> Result<Forged, ForgeError>` with `ForgeOrder { recipe,
   at: ForgeAt::{Stock(usize), Held { unit, slot }} }`, as `TownTrade`'s
   `RepairStock` and `RepairHeld`. On `Err` nothing changes. The chapter
   number is the caller's, as `BasicStock::shop` takes it. If 0087 makes
   a forged weapon's repair cost materials, `shop::repair` takes them
   too (and the *Durability* rule's deferral ends).
6. **On-hit traits.** A trait that does something on a hit (leaves a
   tile state, 0087's example) emits its event where the strike resolves
   (`crates/core/src/battle.rs` / `combat.rs`), so replays and rewind see
   it. **Either order with 1091** (which leaves forged element traits to
   this ticket): if 1091 is in `done/`, call its `react` (the tile's
   state, `Event::TileStateChanged`) here; if not, the content check
   refuses such a trait with "needs element states (1091)" and 1098,
   which needs both 1091 and this ticket, adds its effect.
7. **The farm limit** (0087 Q6): if the rules cap repeatable materials,
   `Campaign::gain_material(&mut self, forge, item, count, source) ->
   u32` (what was kept) with a `Campaign` field counting them, saved with
   `#[serde(default, skip_serializing_if = …)]` so older saves load
   (ADR-0039, as `battles_won`). One-time sources (a battle's
   `clear_items`, chests' `Loot::Item`) add to the stock as any item.
8. **Content** (`trpg-content`): `RawMaterial` and `RawTrait` lists and
   `RawWeapon::traits` (serde default) in `item.rs`; a new
   `crates/content/src/forge.rs` modelled on `basic_shop.rs`
   (`FORGE_PATH = "data/forge.ron"`, `TableRon::<ForgeTable>` registered
   in `formats.rs`, `Content::forge`). Every error reported (ADR-0005):
   unknown item, a `from` or `to` that isn't a weapon, a `to` that isn't
   forged or not of `from`'s kind (if 0087 keeps the kind), a material
   that isn't one, an unknown tier or trait, more traits than 0087
   allows, and **a forged weapon over its tier's cap** by more than 0087's
   allowance (might, hit, crit). Add `assets/data/forge.ron` to
   `.github/playtest-levers.sh`.
9. **Data**: the public stand-in `assets/data/forge.ron` and the
   materials and traits in `assets/data/items.ron` (plain, no story
   names), each format described in the file's comment as `basic_shop.ron`
   does; test materials, a test forged weapon and test recipes (ids
   `test_…`) in the test files' part (`bundle::test_part`, as
   `assets-test/data/characters.ron` is merged; give the item and forge
   loaders a test part if they lack one); the game's real tiers and
   traits, and any material or recipe 0087 names that isn't a beast's
   (those come with their beast, 1280), in the private `content/data/`
   through the `content-change` skill (a pull request there, the pin
   moved here).
10. Expected size: about 800 production lines (`forge.rs` about 250, like
    `shop.rs`'s 426 for less; the content loader about 200, like
    `basic_shop.rs`'s 199; items, campaign and traits about 350), plus
    tests and data: about 1,900 changed lines in all. Measure with `cargo
    xtask code-lines`; at 3× run `engine-friction` (ADR-0092).

## Acceptance criteria

- [ ] `forge::tests`: a recipe forges in its tier's chapter and is
      refused before it (`ForgeError::TierNotReached`), without the
      materials or the gold, or on another weapon; on every refusal the
      stock, gold and weapon are unchanged.
- [ ] `Campaign::forge` forges a stock copy and a held weapon; the
      forged copy keeps its slot and its equip.
- [ ] Each trait in 0087's list has a test that shows its effect in a
      combat (`combat` or `battle` tests), and a forged weapon's
      forecast matches the strike it forecasts.
- [ ] No shop stocks a material or a forged weapon (`shop::tests`).
- [ ] The content check reports each bad recipe, trait and tier above
      (one test each), and passes on the real content with
      `cargo shuyi check`.
- [ ] If the farm limit is built: `gain_material` keeps no more than the
      cap, and a save made before the field loads.
- [ ] The PR's playtest job (`.github/workflows/playtest.yml`) reports
      every Act 1 battle's numbers unchanged from `main`'s (the bots'
      army never forges).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: materials in the stock, recipes per chapter, forging and every
  refusal, each trait's effect, the farm cap, the content checks.
- Property: forging never changes gold, stock or weapon on `Err`; a
  forged weapon is never over its tier's cap plus 0087's allowance
  (generated recipes against the checker).
- Snapshot / integration: a battle replay with a forged weapon gives the
  same hash twice (determinism); the bots' Act 1 numbers unchanged.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
