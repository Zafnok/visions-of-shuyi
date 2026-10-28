---
id: "0409"
title: Shop screen and Evacuate/Open actions in battle
type: feature
milestone: M3 Battle UI
model: sonnet-5
effort: medium
status: done
blocked_by: ["0308", "0403"]
nick_input: none
completed: 2026-10-28
---

# 0409 — Shop screen, Evacuate, Open

## Context

UI for 0308's rules (`docs/design/weapons-and-items.md`, "Money and shops"):
FE on-map Armoury / Vendor / Blacksmith, villages and chests. The same shop
screen is reused between chapters through 0308's `ShopSession`.

**Villages:** since 0069 (2026-10-22) a village is a set of houses to
save (`Evacuate`), not a gift to `Visit` (`turn-structure.md`, *Save the
village*). The house is a one-tile `house` terrain (0037, `terrain.md`,
*Building tiles*), added and drawn on every map skin by 0322, so this
ticket draws no house. Build the house UI (`Evacuate` in the action menu,
the objective line's saved / lost counts) only if 0313 is done; this
ticket doesn't wait for it.

## Nick input

None. Nick no longer checks gameplay tickets after merge (Nick,
2026-10-21: "I will save all my checking for either design (i.e. 00
tickets) or for the full act 1 playtest itself"). Verification is by
tests, `cargo shuyi frame-png` pictures in the PR and the playtest bots
(0506, 0509). What the old sign-off asked goes on the checklist of
Nick's next playtest (1064, the Act 1 playtest, if this lands before
it):

> Nick buys, sells and repairs on a test map and comments.

## Scope

**In:** `ShopScreen` (Buy / Sell / Repair tabs by shop kind), `Shop`, `Evacuate`
and `Open` entries in the action menu, gold in the side panel and shop
header, gift/loot message lines. The shop screen plays the `shop` music cue chosen in
0028 (`docs/design/audio.md`), imported with `assets-src/audio/import.py` and
added to `assets/audio/audio.ron`, `THIRD_PARTY_ASSETS.md` and the credits. On
leaving, the previous track comes back (emit the battle's or scene's cue again).
If 0028 recorded "no music here", skip this.

**Out:** where between-chapter shopping sits in the game flow (0801 for linear chapters; town nodes on the world map, 1007); the Preparations screen's Shop tab with the basic shop (0443, which reuses this ticket's `ShopScreen` lists).

## Implementation steps

1. Action menu: `Shop` on a shop tile, `Evacuate` on an unsaved house's door,
   `Open` on an unopened chest (player units only).
2. `ShopScreen`: header `ARMOURY    Gold 1 250`; Buy list with price and a
   stat line; Sell list (half price; seals are never listed: no shop buys
   them, `ShopError::NotBought`, ticket 0603); a Blacksmith shows Repair with the cost
   and `12/20 → 20/20`. Unaffordable entries dimmed. Transactions collect into
   one `UnitAction::Shop`; leaving with none sends nothing (unit not done).
3. `Evacuate` / `Open`: send the action; show `House saved.` /
   `Got Potion.` (plus the house's line, if the map has one).
4. The side panel shows party gold.

## Acceptance criteria

- [x] Harness: buy a Potion on a Vendor tile → gold drops by its price, pack grows, unit done; leaving without buying → unit not done.
- [x] Harness: repair a damaged weapon at a Blacksmith; evacuate a house; open a chest.
- [x] Snapshots below.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Harness: flows above.
- Snapshot: Armoury buy tab, Blacksmith repair tab.

## Completion notes

Done (2026-10-28).

- **Action menu** (`crates/ui/src/screens/battle/mode.rs`): `Shop` on a
  shop tile, `Open` on an unopened chest, `Evacuate` on a standing house
  (player units only; legality asked of the core), after `Seize`/`Escape`.
- **Shop screen** (`screens/battle/shop.rs`, view `view/shop.rs`, skin
  `glyph/shop.rs`, ADR-0054): pushed over the battle like the Guides
  list. Header: the shop's name and `Gold 1 250`. Armory/Vendor have Buy
  and Sell tabs (Left/Right), a Blacksmith has Repair. Buy lines show the
  price and the item's numbers (weapon stats, armor/accessory bonuses,
  what a consumable does); Sell lines half price; Repair lines the cost
  and `12/20 → 20/20`. What can't be chosen is dimmed. Choices collect
  into one `UnitAction::Shop`, sent when the screen closes; closing with
  none sends nothing and the action menu is back (unit not done). The
  screen previews each choice on a copy of the battle, so the gold and
  lists shown are what the core would do.
- **Selling asks first** (Nick on the PR, 2026-10-28: "selling items
  should have confirmation screen with a up/down slider for selling
  multiple"; recorded in `weapons-and-items.md`, *Money and shops*).
  Confirm on a Sell line opens a box: `Sell Potion?`, the count between
  ▲ and ▼ (`2 / 3`), `For 300 gold`. Up and down set how many, Confirm
  sells them, Cancel sells nothing.
- **Messages**: `Got Potion.`, `Got 500 gold.`, `House saved.` on the
  message line after `Open` / `Evacuate` (`screens/battle/loot.rs`).
  Maps have no per-house lines, so none is shown.
- **Side panel** shows the party's gold on its last row, always.
- **Objective page**: the village line from `turn-structure.md`,
  `Save at least 3 of 5 houses (1 saved, 1 lost)`, in a battle with a
  village (both skins).
- Every battle snapshot gained the panel's gold row (checked: each diff
  is that one row only). The `test` language pack has the new keys.
- **Not done here: the shop music.** 0028 (which track) was still open,
  so the cue can't be imported. Follow-up **0473** plays it once 0028 is
  answered; `0028` moved from this ticket's `blocked_by` to 0473's.
- The Playdate's own shop look is 1066's (the PC skin is drawn for now,
  as for the Guides list); Chapter 1 has no shops.

Engine friction: estimate 550 production lines (the Guides screen's
logic, view and skin, plus menu entries, messages and the panel line),
measured 727 (1.32×) with the sale's confirmation, under 2×.

### Claude's starting rules (for the Act 1 playtest, 1064)

- **Lines you can't afford stay selectable to read**, just dimmed;
  Confirm on one does nothing (the "can't" sound).
- **Pack items are one Sell line per item** (`Potion ×3`).
- **The sell count starts at 1** and stops at 1 and at the number held
  (no wrapping); a weapon or worn armor gets the same box with the count
  fixed at 1. Buying and repairing don't ask.
- **Leaving the shop after buying needs no "are you sure"**: closing it
  sends what you chose and ends the unit's action, as in Fire Emblem.
- **Gold always shows in the side panel**, even at 0.
- The chest/house message stays up 1.5 s, like the other short messages.
