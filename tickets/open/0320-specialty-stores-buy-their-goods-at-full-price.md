---
id: "0320"
title: "Specialty stores: an Alchemist and a Tailor that buy their goods at full price"
type: feature
milestone: M2 Core rules
model: sonnet-5
effort: medium
status: todo
blocked_by: ["0073"]
nick_input: none
completed:
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
0073 decides what those weights are called in the data.

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

- [ ] Unit: an Alchemist pays 300 for a Potion (price 300), refuses a sword; a Vendor pays 150 for the same Potion; a Tailor pays full for cloth and leather, refuses plate.
- [ ] Content: a Tailor listing a Steel Sword fails to load, with the reason.
- [ ] Snapshot of a Tailor's sell list with a refused item dimmed.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: prices and refusals per kind.
- Property: no shop ever pays more than an item's price.
- Snapshot / integration: the sell list.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
