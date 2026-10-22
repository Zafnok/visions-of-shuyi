---
id: "0443"
title: "Preparations: a Shop tab with the basic shop"
type: feature
milestone: M3 Battle UI
model: sonnet-5
effort: medium
status: todo
blocked_by: ["0409", "0044"]
nick_input: none
completed:
---

# 0443 — Preparations: a Shop tab with the basic shop

## Context

Nick (ticket 0408, 2026-10-01; `docs/design/weapons-and-items.md`, *Battle
pack*): the Preparations screen "should also have a basic shop option --
some shops might sell unique items, this shop won't. It will only sell up
to the tier of items you have unlocked as a basic." From the first battle
with Preparations the player packs only consumables they own, so this shop
is where they get them before a story battle.

Builds on:

- the Preparations screen (0408): `crates/ui/src/screens/preparations.rs`
  (`Tab`, `Focus`, `PreparationsScreen`), which edits a `BattleSetup`
  (its `stock`, `gold`, `pack`) through `crates/core/src/prep.rs`;
- the shop rules in core (0308, `crates/core/src/shop.rs`: `Shop`,
  `ShopSession`, `ShopKind`, `sell_price`) and the shop screen (0409,
  `ShopScreen`), which this tab reuses rather than copying;
- the decision in 0044: which items are basics, the tiers and what unlocks
  them.

## Nick input

None. Nick no longer checks gameplay tickets after merge (Nick,
2026-10-21: "I will save all my checking for either design (i.e. 00
tickets) or for the full act 1 playtest itself"). Verification is by
tests, `cargo shuyi frame-png` pictures in the PR and the playtest bots
(0506, 0509). What the old sign-off asked goes on the checklist of
Nick's next playtest (1064, the Act 1 playtest, if this lands before
it):

> Nick buys Potions on the Quick Battle's Preparations, packs
> them, and comments.

## Scope

**In:**
- A `Shop` tab on the Preparations screen, between `Pack` and `Fight!`.
- The basic shop's stock as data, following 0044 (`weapons-and-items.md`,
  *The Preparations shop*): each basic item with the chapter it is sold
  from; lower tiers stay on sale.
- The consumables 0044 decided, in `assets/data/items.ron`: the Potion
  heals **20** (was 10), and a new **Hi-Potion** (`hi_potion`, heals 40,
  price 900).
- Gold shown on the screen; buying puts the item in the stock and takes the
  gold; selling as in `weapons-and-items.md` (half price).
- **Repairs** (0044: the basic shop repairs), at the Blacksmith's price
  (`shop::repair_cost`), for any weapon in the stock or in a unit's
  loadout.
- The debug Quick Battle gets some gold to try it with (a `solo_gold` field
  next to `solo_stock` in the battle file), and a chapter number so it has
  a shop (Chapter 5's, to show Iron and Steel).

**Out (do not do):**
- Unique items, town shops, the world map (1007).
- The Elixir in the basic shop (Act 2 or 3; no chapter number exists yet)
  and the support-rank special unlocks (not chosen yet): the data format
  leaves room for them, nothing more.
- Green units' own consumables (ticket 0513).
- Changing the on-map shop rules.

## Implementation steps

1. Read 0044's record in `weapons-and-items.md` (*Consumables*, *The
   Preparations shop*). In `assets/data/items.ron` set the Potion to
   `Heal(20)` and add `(id: "hi_potion", name: "Hi-Potion", effect:
   Heal(40), price: 900)`; fix the `trpg-content` tests that pin the
   Potion's heal. Chapter 1's free pack now heals 20 a Potion: run
   `cargo xtask playtest` / `autobalance` on Chapter 1 and note the change
   in the Completion notes.
2. Put the basic shop's stock in data (`assets/data/basic_shop.ron`): a
   list of `(item, unlock)` with `unlock: Chapter(n)`, as an enum so a
   later kind (a support rank, 0044) is one more variant. Rows from 0044:
   Chapter 2: Potion, the five Iron weapons, Leather Vest, Chain Mail,
   Warded Robe; Chapter 5: the five Steel weapons, Iron Plate; Chapter 8:
   Hi-Potion. If 0073 (four armour weights) is done, use its armours as
   `weapons-and-items.md` says (*The Preparations shop*); if not, use
   today's and leave a line in 0073's follow-up ticket to update the rows.
   Loaded and validated by `trpg-content` with every error
   reported (ADR-0005): unknown item, seal or accessory listed, duplicate.
3. `core`: a function that builds the basic `Shop` for a chapter (every row
   whose unlock is reached), and `BattleSetup` methods in `prep.rs` to buy
   into / sell from the stock and repair a weapon with the setup's gold,
   reusing `shop.rs` (`buy`, `sell`, `repair`, `repair_cost`). The chapter
   comes from the campaign (the chapter whose battle is being prepared).
   Unit tests.
4. `ui`: add `Tab::Shop`; draw it with 0409's shop lists (price, the
   unaffordable ones dimmed, `Gold 1 250` in the heading), with Buy, Sell
   and Repair as 0409's shop screen does them. Confirm buys one; a bought
   consumable is then on the `Pack` tab's stock list.
5. `assets-test/battles/quick.ron`: `solo_gold`, read by
   `trpg_content::battle_campaign`; document it in
   `assets/battles/README.md`.
6. Follow the `keyboard-input` skill for the help line.

## Acceptance criteria

- [ ] Harness: on the Quick Battle's Preparations, buy a Potion (gold drops by its price, the stock gains one), pack it, Fight! → it is in the battle's pack and the campaign's gold after a win reflects the purchase.
- [ ] An item the player can't afford is dimmed and can't be bought.
- [ ] The shop offers only the basics unlocked by the chapter: a unit test for Chapter 2 (no Steel, no Hi-Potion), Chapter 5 (Steel, no Hi-Potion) and Chapter 8 (Hi-Potion, Potion still on sale).
- [ ] Repairing a damaged weapon costs `repair_cost` and fills its durability.
- [ ] `items.ron`: the Potion heals 20; the Hi-Potion exists (heals 40, 900).
- [ ] Snapshot of the Shop tab.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: buying, selling and repairing through the setup (gold, stock), the stock per chapter, the data validation.
- Snapshot / integration: the Shop tab; the Harness flow above.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
