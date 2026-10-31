---
id: "0320"
title: "Specialty stores: an Alchemist and a Tailor that buy their goods at full price"
type: feature
milestone: M2 Core rules
model: implementation
effort: medium
status: done
blocked_by: ["0073", "0321"]
nick_input: none
completed: 2026-10-29
---

# 0320 — Specialty stores

## Context

Nick, PR #385 (ticket 0044, 2026-10-21): "there are specialty stores and
these will buy items at full price but the basic shop yes it buys
everything at half price. example speciality store is like alchemist -
sell/buy consumable only, tailor - sell/buy light armor only (maybe also
include leather here …)". Rules: `docs/design/weapons-and-items.md`,
*Money and shops* → *Specialty stores*.

Shops live in `crates/core/src/shop.rs` (ticket 0308): `ShopKind`
(`Armoury`, `Vendor`, `Blacksmith`) with `stocks` and `buys`, and
`sell_price` (half price for every shop). The shop screen is 0409's
`ShopScreen`.

Blocked by 0073 because the Tailor trades cloth and leather armour, and
0073 decided those weights (Cloth, Leather, Chain, Plate); 0321 puts them
in the data (`ArmourWeight::Cloth`, `ArmourWeight::Leather`).

## Nick input

None. Verified by tests. Which towns have which store is the towns' data
(1070).

## Scope

**In:**
- `ShopKind::Alchemist` (consumables only) and `ShopKind::Tailor` (cloth
  and leather armour only, by 0073's weights), each selling and buying
  **only** its own goods.
- The price a shop pays: full price at a specialty store for its goods,
  half everywhere else (`sell_price` takes the shop kind).
- The shop screen shows the sell price the shop pays and dims items the
  store won't take, with a short reason, as the Preparations screen does
  for gear a unit can't use.
- Town data can list these kinds (wherever 1007 put town shops; if 1007
  isn't done, the shop kinds and their tests are enough and 1007 / 1070
  pick them up).

**Out (do not do):**
- Which towns get which store (1070).
- Specialty stores on battle-map shop tiles or in Preparations (the
  design keeps them in towns).
- More specialty kinds than these two.

## Implementation steps

1. `core/src/shop.rs`: add the two kinds; `stocks` and a new
   `takes(item)` (what it will buy) per kind; `buys()` stays for the
   Blacksmith. `sell_price(kind, item)`: `item.price()` for a specialty
   kind's own goods, else `item.price() / 2`. Update every caller.
2. `ShopSession::sell_item` refuses an item the store won't take with a
   `ShopError` variant.
3. `trpg-content`: the shop kinds parse wherever shops are written in data;
   a specialty store listing an item outside its goods is a load error.
4. `ui`: the shop screen's sell list shows the price this shop pays and
   dims what it won't take, with the reason.

## Acceptance criteria

- [x] Unit: an Alchemist pays 300 for a Potion (price 300), refuses a sword; a Vendor pays 150 for the same Potion; a Tailor pays full for cloth and leather, refuses plate.
- [x] Content: a Tailor listing a Steel Sword fails to load, with the reason.
- [x] Snapshot of a Tailor's sell list with a refused item dimmed.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: prices and refusals per kind.
- Property: no shop ever pays more than an item's price.
- Snapshot / integration: the sell list.

## Completion notes

- `core/src/shop.rs`: `ShopKind::Alchemist` (consumables) and
  `ShopKind::Tailor` (Cloth and Leather armour), each stocking and buying
  only its goods; `ShopKind::takes` and `is_specialty`;
  `sell_price(kind, item)` pays full at a specialty store, half elsewhere;
  `sell` (so the battle's `Shop` action and `ShopSession` alike) refuses an
  item outside the store's goods with `ShopError::NotTaken`. Tests: the
  acceptance prices, every kind's goods, a Tailor session, and a property
  that no shop pays more than an item's price.
- `content/src/map.rs`: `check_shop(shop, items)`, the list check every
  place that writes shops will use (maps today, towns with 1007), saying a
  specialty store's goods ("a Tailor trades only cloth and leather armor,
  not \"steel_sword\""). A map refuses the two kinds outright ("a
  specialty store is only in towns, not on a map").
- `ui`: the shop screen has the two kinds (Buy and Sell tabs, their
  names); the Sell tab shows what this shop pays; an item a specialty
  store won't take is dimmed, with no price and the reason in place of its
  numbers ("The Tailor buys only cloth and leather armor."). Snapshot
  `tailor_sell`.

**Deviations.** Towns (1007) aren't built yet, so there is no town data to
parse the kinds from. The ticket allows this; the content acceptance test
runs `check_shop` on a Tailor listing a Steel Sword (from the real item
table) rather than loading a town file. 1007 / 1070 should call
`check_shop` for each town shop. The screen test puts a Tailor on the
Vendor's tile of the Quick Battle map, since that is the only shop screen
until towns come.

**the agent's starting rules** (for the Act 1 playtest, 1064):
- A refused item at a specialty store shows no price (rather than a price
  it would never pay).
- A Tailor and an Alchemist repair nothing (the design lists only trading).

Engine friction: estimate 90, measured 96 (1.1×), under 2×.
