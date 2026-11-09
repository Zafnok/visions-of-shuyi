---
id: "0486"
title: "Shop lines show every number of an item"
type: bug
milestone: M3 Battle UI
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-11-09
---

# 0486 — Shop lines show every number of an item

## Context

Nick's playtest of the Act 1 build (2026-11-08):

> "all stats are not readily apparent in the buy menu for shop for
> preparations, i.e. Wt for armor"

Every shop line's detail comes from one function, `stat_line`
(`crates/ui/src/screens/battle/shop.rs` ~:670), shared by the three shops:
the battle map's shops (`shop.rs` ~:607), the town's
(`crates/ui/src/screens/town.rs` ~:357) and Preparations' Shop tab
(`crates/ui/src/screens/preparations/shop.rs`, `deal_line` ~:349, from
`deals()` ~:113). What it writes today:

| Item | Line today | What the item has that the line leaves out |
| ---- | ---------- | ------------------------------------------- |
| Weapon | `Mt 5  Hit 90  Crit 0  Rng 1  Wt 2` (`battle.info.weapon_numbers`, `ui.ron` ~:725) | its rank (`E`, `D`…: who can wield it) and, on Buy, its uses |
| Armour | `Def +2  Res +2` (`bonus_text`, `screens/battle/info.rs` ~:75) | its **weight** and **weight class** (Cloth, Leather, Chain, Plate: who can wear it), both in `ArmourDef` (`crates/core/src/item.rs` ~:173–184) |
| Accessory | `Spd +2` | nothing: `AccessoryDef` has no weight (`item.rs` ~:186–194) |
| Consumable | `Restore 10 HP` | nothing |

The snapshot `crates/ui/src/screens/battle/shop/snapshots/…armoury_buy.snap`
shows it: `Leather Vest  400  Def +2  Res +2`, no Wt. The Loadouts tab
already writes armour as `{bonus}  Wt {weight}` (`prep.armour_detail`,
`ui.ron` ~:377), and the unit info screen prefixes a weapon with its rank
(`E Iron Sword`, `screens/battle/info.rs` `weapons()` ~:281). Sell lines
already put a weapon's uses first (`12/20  Mt 5 …`, `deal_line`).

The detail column has room: it starts at cell 37 of the 100-cell box and
runs to cell 98 (`glyph/shop.rs`, `detail_x`, `row_w`), about 60 cells;
the longest line below is under 50.

**Ticket 0478 is wrong about this.** Its Scope says the Loadouts line for
armour reads "their bonuses and weight, as in the shop", built with
`stat_line`. The shop shows no weight today. This ticket makes that true.

## Nick input

None. Verified by the view tests and snapshots of the three shops, and
`cargo shuyi frame-png` pictures of the Preparations Shop tab on Buy in the
PR. *The agent's starting rules* (the order of the parts below) go on the
Act 1 playtest's checklist (1064).

## Scope

**In:**
- `stat_line`, for every shop:
  - **Armour:** its weight class, its bonuses, its weight:
    `Leather  Def +2  Res +2  Wt 1`. The class words are new keys (e.g.
    `battle.info.armour_class.leather` "Leather"), American English.
  - **Weapon:** its rank first, as the info screen does:
    `E  Mt 5  Hit 90  Crit 0  Rng 1  Wt 2` (or the rank before the name,
    if the info screen's form fits the name column better; pick one and
    use it in all three shops).
  - Accessories and consumables unchanged.
- **Buy lines of weapons show the uses a new one has** (`20/20  E  Mt 5 …`),
  as Sell lines do, in the three shops (`Deal`/offer `durability` set to
  `(max, max)` for a weapon bought).
- The tests and snapshots of the three shops updated.

**Out (do not do):**
- Saying on a Preparations line who in the army can use the item (the
  view's `unit` stays empty, `preparations/shop.rs` ~:402): the rank and
  weight class say it in the item's own terms. A line per unit is a
  design question for later, not this bug.
- The Loadouts screen's own lines (0478). Either order: if 0478 is done
  first, its armour `item_detail` gets this weight from `stat_line` when
  this lands (check its test still holds); if this is done first, 0478
  uses `stat_line` as written.
- Moving the shops onto one flow (1216). Either order: change `stat_line`
  where it lives when this is worked.
- The Playdate's shop skin: there is none today (1046 does Preparations on
  the Playdate; it paints the view's detail as it is).

## Implementation steps

1. Write the failing tests first: the Armoury's Buy line of the Leather
   Vest has `Wt 1` and `Leather`; the Iron Sword's has `E` and `20/20`.
2. Change `stat_line` (weapons: rank; armour: class and weight); add the
   class keys to `assets/lang/en/ui.ron` (and the test language pack if
   the text checks need it).
3. Set the uses of a weapon's Buy line in the three shops.
4. Refresh the snapshots (`battle/shop/snapshots`,
   `preparations/snapshots/…shop_tab_snapshot.snap`, the town's) and read
   each diff.

## Acceptance criteria

- [x] On the Preparations Shop tab, Buy, the Leather Vest's line reads
      `Leather  Def +2  Res +2  Wt 1` after its price (view test in
      `crates/ui/src/screens/preparations/tests.rs`).
- [x] The battle map's Armoury and a town's Armory show the same armour
      line (tests in `screens/battle/shop/tests.rs` and the town's tests).
- [x] A weapon's Buy line has its rank and `20/20` (or its own max) in
      all three shops (tests).
- [x] Every line fits the detail column: a test paints the longest item of
      `items.ron` (e.g. Houndmaster's Axe on Sell) and finds no clipped
      text.
- [x] `frame-png` pictures of the Preparations Shop tab on Buy are in the
      PR.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `stat_line` for a weapon, each armour class and an accessory.
- Snapshot: the three shops' snapshots updated.

## Completion notes

- `stat_line` (`crates/ui/src/screens/battle/shop.rs`) now writes a
  weapon's rank first (`E  Mt 5  Hit 90  Crit 0  Rng 1  Wt 2`) and an
  armour's weight class, bonuses and weight (`Leather  Def +2  Res +2
  Wt 1`). New keys: `battle.info.armour_class.{cloth,leather,chain,plate}`
  and `battle.info.armour_weight` (`Wt {weight}`), in English and the
  test pack. The parts of an armour line are joined so an armour with no
  bonus still reads cleanly.
- Buy lines of weapons carry `(max, max)` uses (`new_uses`) in the
  battle map's, the town's and Preparations' shops, shown as
  `20/20  E  Mt 5 …`. The battle map's shop now tells a repair line from
  a line with uses by its transaction, not by having uses.
- The rank goes in the detail column, not before the name: the info
  screen does not prefix the rank today (the ticket's Context was wrong
  about that), and the detail column has the room. Same form in all
  three shops; Sell lines get the rank too, since they share `stat_line`.
- Tests: `stat_line` for a weapon, each armour class, an accessory and a
  consumable; the Buy lines in each of the three shops; the longest
  line of `items.ron` (a weapon with its uses) painted into the shop box
  without clipping. Snapshots refreshed (the three shops, the sell
  confirmation and the special sale).
- Picture: `docs/img/0486-prep-shop-buy.png` (Quick Battle,
  Preparations, Shop, Buy).
- Follow-up: 0492. The sell box doesn't hide the list behind it, so the
  end of the longer Leather Vest line now shows as a stray `1` beside the
  box's corner in the sell confirmation.
- 0478 (Loadouts) is still open: it can use `stat_line` as written.

**For the Act 1 playtest (1064), *the agent's starting rules*:** the
order of a shop line is uses (Buy and Sell weapons), rank, numbers; for
armour, weight class, bonuses, weight.
