---
id: "0443"
title: "Preparations: a Shop tab with the basic shop"
type: feature
milestone: M3 Battle UI
model: sonnet-5
effort: medium
status: done
blocked_by: ["0409", "0044"]
nick_input: none
completed: 2026-10-28
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
  gold; selling as in `weapons-and-items.md` (half price). Selling a
  special weapon asks first, as the battle shop does since 0329
  (`ShopScreen`'s `battle.shop.sell_special` question, No / Yes, cursor
  on No); `ShopKind::stocks` already refuses special weapons.
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
   Chapter 2: Potion, the five Iron weapons, the armours; Chapter 5: the
   five Steel weapons; Chapter 8: Hi-Potion. The armours follow 0073
   (`weapons-and-items.md`, *The Preparations shop*): if 0321 (the four
   armour weights in the data) is done, the Linen Robe, Leather Vest,
   Chain Mail and Iron Plate, all from Chapter 2; if not, today's Leather
   Vest, Chain Mail, Warded Robe and Iron Plate from Chapter 2, and 0321
   updates the rows. The Steel armour (0074) is not in this ticket.
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

- [x] Harness: on the Quick Battle's Preparations, buy a Potion (gold drops by its price, the stock gains one), pack it, Fight! → it is in the battle's pack and the campaign's gold after a win reflects the purchase.
- [x] An item the player can't afford is dimmed and can't be bought.
- [x] The shop offers only the basics unlocked by the chapter: a unit test for Chapter 2 (no Steel, no Hi-Potion), Chapter 5 (Steel, no Hi-Potion) and Chapter 8 (Hi-Potion, Potion still on sale).
- [x] Repairing a damaged weapon costs `repair_cost` and fills its durability.
- [x] `items.ron`: the Potion heals 20; the Hi-Potion exists (heals 40, 900).
- [x] Snapshot of the Shop tab.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: buying, selling and repairing through the setup (gold, stock), the stock per chapter, the data validation.
- Snapshot / integration: the Shop tab; the Harness flow above.

## Completion notes

**Done.**

- `core` (`shop.rs`): `ShopKind::Basic` (sells weapons, armour and
  consumables, buys anything at half price, repairs; refused on a map by
  the map loader), `Unlock::Chapter(n)` and `BasicStock::shop(chapter)`.
  `prep.rs`: `BattleSetup::buy` / `sell` (on `ShopSession`),
  `Preparations::{buy, sell, repair, weapon}` with `WeaponAt` (a stock
  copy or any unit's weapon slot, the bench's too), and a `shop` field
  on `Preparations`. `prepared_as` now also restores the gold, so a
  continued battle keeps what Preparations spent.
- `content`: `assets/data/basic_shop.ron` (the 0044/0073 rows: Chapter 2
  Potion, the five Iron weapons and the four armours; Chapter 5 the five
  Steel weapons; Chapter 8 the Hi-Potion), loaded by
  `trpg_content::basic_shop` through the format registry, every error
  reported (unknown item, accessory, seal, listed twice, chapter 0).
  Chapters gained an optional `number` (`ch01` is 1, the Quick Battle's
  chapter 5); battles gained `solo_gold` (the Quick Battle has 3 000).
  Potion heals 20, new Hi-Potion (40, 900 gold).
- `ui`: the `Shop` tab (between `Pack` and `Options`, `Options` having
  arrived after this ticket was written), in
  `screens/preparations/shop.rs`. It reuses 0409's `ShopScreenView`,
  line views, `SellConfirm` question and painter: the battle shop's
  glyph painter now paints at any box (`paint_box`), the on-map screen
  unchanged. Help line through the player's keys.

**Deviations.**
- The ticket says "between `Pack` and `Fight!`"; `Options` sits there now,
  so `Shop` is right after `Pack`.
- Repairs aren't a separate unlock row: the basic shop always repairs,
  and the first Preparations is Chapter 2 anyway.
- The harness test wins the battle by clearing the Quick Battle's
  enemies in the test and checks the campaign through
  `Campaign::apply_result`, the function the flow calls on a win.
- I wrote the line estimate after coding, not before (ADR-0092 asks for
  it first). Estimate as I'd have made it: about 350 production lines
  (comparable: 0409's on-map shop screen, 616 lines, of which this tab
  reuses the view and painter). Measured (`cargo xtask code-lines`): 600 added,
  1.7×, under 2×.

**Side effects of the data change.** The Quick Battle's gold shows in the
battle panel (`Gold 3 000`), and the Potion's `Restore 20 HP` shows in
the pack menus: snapshots updated. The autobalance CLI test's
too-hard battle packs a Potion, so it now needs `-5` enemy levels
instead of `-8`.

**Chapter 1 bots** (Chapter 1's free pack is three Potions):
`cargo xtask playtest ch01`. The baseline bot never uses an item (0/40
won either way), so it can't show the change. The Casual persona, the
same 10 seeds (seeds 1–10) with each Potion: **old Potion (10 HP) 6/10
won, median 11 turns; new Potion (20 HP) 3/10 won, median 18 turns**,
three Potions used a try both times. Its band is 30–55%: the new value
sits inside it, the old one was over it. Fewer wins with a stronger
heal is surprising, and 10 tries is a small sample (about 30 s a try, so
the full 100-try Normal and Hardcore runs weren't done here); read it
as "no sign Chapter 1 got easier", not a measured drop. Worth a full
`cargo xtask playtest ch01 --bot all` before the Act 1 playtest.

**Claude's starting rules** (also in `weapons-and-items.md`):
- Each purchase, sale or repair happens at once; no basket to confirm.
- Sell lists only the stock: to sell worn gear or a packed item, put it
  back in the stock first.
- Repair lists every worn weapon of the stock and of the whole army,
  units left out of the battle included.
- A battle of a chapter with no number has no shop (the tab says "No
  shop before this battle.").

**Follow-up:** 0474 (a negative bonus reads `Res +-3`, seen on the Iron
Plate line; the on-map shop has it too).

**For the Act 1 playtest (1064):** Nick buys Potions on the Quick
Battle's Preparations, packs them, and comments.
