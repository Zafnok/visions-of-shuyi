---
id: "0478"
title: "Loadouts show each item's numbers, and no attack speed without a weapon"
type: bug
milestone: M3 Battle UI
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-11-09
---

# 0478 — Loadouts show each item's numbers, and no attack speed without a weapon

## Context

A friend's playtest of the Act 1 build, passed on by Nick (2026-11-07):

> "The Loadout screen of preparations doesn't tell the damage of the
> weapons only the attack speed. Additionally, when you hover armor it
> instead bugs out and displays info about weapons. Additionally, even
> though you cannot fight barehanded, it still has a tooltip saying AS
> with fist."

What the screen does today (Preparations → Loadouts, built by 0408, split
into view and skin by 0245):

- **Logic:** `crates/ui/src/screens/preparations.rs`: `stock_rows`
  (~:343), `unit_rows` (the "from X" swap rows, ~:432), `speed_slot`
  (~:531), `in_hand` (~:540), `speed_line` (~:564), `loadouts_view`
  (~:868), `gear_view` (~:884).
- **View:** `crates/ui/src/screens/preparations/view.rs`: `GearView`
  (~:105: `slots`, one `speed: Option<String>`, `benched_note`, `stock`),
  `SlotRowView` (~:119: `label`, `item`, `wear`), `ItemRowView` (~:130:
  `name`, `detail`, `usable`).
- **Skin:** `crates/ui/src/screens/preparations/glyph.rs`, `paint_gear`
  (~:147).
- **Text:** `assets/lang/en/ui.ron` ~:340–374 (`prep.no_weapon` "no
  weapon", `prep.speed` "AS {now} with {weapon}", `prep.speed_change`,
  `prep.weapon_detail`, `prep.armour_detail`).

The three reports, one by one:

1. **No damage.** Stock rows of weapons show `Mt Hit Wt Rng` and wear, but
   the unit's own slots show only the name and wear, the swap rows ("from
   Test Scout") only wear and the holder, and nothing ever shows Crit. The
   unit info screen (`screens/battle/info.rs`, `weapons()`) and the shop
   (`screens/battle/shop.rs`, `stat_line`) already share one format,
   `battle.info.weapon_numbers` ("Mt {might}  Hit {hit}  Crit {crit}  Rng
   {range}  Wt {weight}"); `stat_line` also covers armour and accessory
   bonuses (bonuses only, no weight: 0486 adds it).
2. **Armour shows weapon info.** Not a fall-through: 0408 specified it.
   On an armour or accessory slot the speed line uses the weapon in hand
   (`AS 5 → 6 with Iron Sword`), because armour weight changes attack
   speed (`docs/design/weapons-and-items.md`, *Attack speed*). The
   tests in `preparations/tests.rs` (~:258–284) assert it. With no line
   about the armour itself, the only thing on screen names a sword, so
   it reads as the wrong item's info.
3. **"AS with fist".** `in_hand` puts `prep.no_weapon` in the speed
   line when the highlighted weapon slot is empty or nothing is
   equipped. `core::prep::speed` (`crates/core/src/prep.rs` ~:312)
   always returns a number (weight 0 with no weapon,
   `weapons-and-items.md` ~:228). A unit can't attack without a
   weapon or spell, so the number means nothing. The tests at
   `preparations/tests.rs` ~:165, :184 and :759 assert `AS 7 with no
   weapon`.

`docs/design/weapons-and-items.md` (*Loadout (per unit)*, *On the
Preparations screen*, ~:327–374) says what the screen does but not which
numbers it shows. This ticket adds no new kind of number: it shows the
shop's and the info screen's line for the highlighted item.

## Nick input

None. Verified by the view tests and `cargo shuyi frame-png` pictures of
the Loadouts screen in the PR. Whether the line reads well goes on the
Act 1 playtest's checklist (1064).

## Scope

**In:**
- A **highlighted-item line** in `GearView` (one `Option<String>`, e.g.
  `item_detail`): the numbers of the item under the cursor, wherever the
  cursor is (a slot, a stock row, a swap row), built with the shop's
  `stat_line` so weapons read `Mt 5  Hit 90  Crit 0  Rng 1  Wt 2`;
  armour reads its bonuses and weight, and accessories (which have no
  weight) their bonuses. `stat_line` shows no armour weight today.
  **Either order with 0486:** if 0486 is done, `stat_line` already has
  armour's weight; if not, armour's line is `prep.armour_detail`
  (`{bonus}  Wt {weight}`) and 0486 moves it onto `stat_line`.
  An empty slot has no line.
- **The speed line names no item when the change is armour or an
  accessory:** `AS 5 → 6` alone (a new key, e.g. `prep.speed_change_gear`),
  painted under the item line, so the only item named is the highlighted
  one. On a weapon it keeps `AS 5 → 6 with Iron Sword`.
- **No speed line when no weapon or spell would be in hand** after the
  change (the highlighted weapon slot is empty, or the unit holds none):
  `speed` is `None`. `prep.no_weapon` goes if nothing else uses it.
- The glyph skin paints the new line; the Playdate skin of Preparations,
  if it exists when this is worked (1046), paints it too, cut to fit by
  its own rule; if 1046 isn't done, add a line to 1046 saying the view
  has `item_detail`.
- `weapons-and-items.md`, *On the Preparations screen*: one sentence for
  each of the three rules above.

**Out (do not do):**
- A face on the swap rows (0479).
- Changing how attack speed is worked out, or `core::prep::speed`.
- The shop tab (0443) and the shared shop flow (1216).
- Moving the lists onto the engine's panels (1238); if 1238 is done
  first, add the line inside its panel.

## Implementation steps

1. Read 0408's done ticket and `preparations.rs` around the functions
   above; read `shop::stat_line` and make it `pub(crate)` if it isn't.
2. Add `item_detail` to `GearView`; fill it in `gear_view` from whatever
   the cursor is on (the slot's item, the stock row's item, the swap
   row's item).
3. Change `speed_line`: return `None` when no weapon or spell would be
   in hand; drop "with {weapon}" when the highlighted item is armour or
   an accessory.
4. Paint `item_detail` in `paint_gear` (one row, under the slots, above
   the speed line).
5. Update the tests that asserted the old text (~:165, :184, :258–284,
   :759) to the new rules, and add the new ones below.
6. Update the doc lines and refresh the snapshots.

## Acceptance criteria

- [x] With the cursor on a weapon slot, a stock weapon or a swap row
      weapon, the view's `item_detail` is that weapon's
      `Mt … Hit … Crit … Rng … Wt …` (a test for each place).
- [x] With the cursor on armour (slot, stock or swap row), `item_detail`
      is its bonus and weight, and the speed line is `AS a → b` with no
      weapon name (test).
- [x] With the highlighted weapon slot empty, or a unit holding no
      weapon or spell, `speed` is `None` (test); no frame says "no
      weapon" next to AS.
- [x] `frame-png` pictures of the Loadouts screen on a weapon, on armour
      and on an empty slot are in the PR.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `gear_view` / `speed_line` for each case above
  (`crates/ui/src/screens/preparations/tests.rs`).
- Snapshot: the Loadouts screen snapshots updated.

## Completion notes

- `GearView` has `item_detail`: the numbers of the item under the cursor
  (a slot's, a stock row's, another unit's), from the shop's
  `stat_line` (0486 was done, so armour already reads its class, bonuses
  and weight: `Cloth  Res +4  Wt 0`). `None` on the units list, an empty
  slot and the put-back row. A weapon's line is 36 characters and the
  slots box has 32, so the glyph skin cuts it at a two-space gap
  (`E  Mt 5  Hit 90  Crit 0  Rng 1` / `Wt 2`) rather than mid-number.
- `speed_line` returns `None` when no weapon or spell would be in hand;
  `prep.no_weapon` is gone. On an armour or accessory slot (and its
  stock) the line is `AS 5 → 6` / `AS 5` (new keys `prep.speed_gear`,
  `prep.speed_change_gear`).
- **Deviation:** the ticket names only the *change* on armour without a
  weapon; the armour slot itself (no change highlighted) also drops the
  weapon name (`AS 5`), for the same reason: the only item named is the
  highlighted one.
- **Deviation:** a weapon going into an empty slot used to read
  `AS 7 → 5 with Iron Sword`, where 7 was the speed with nothing in hand.
  With no weapon before the change there is no speed to change from, so
  it reads `AS 5 with Iron Sword`.
- The "Not in this battle" note no longer hangs off the speed line, so a
  benched unit with no weapon still shows it.
- 1046 (Playdate Preparations) isn't done: a line in its Scope says the
  view has `item_detail`. 1238 isn't done either; nothing to do there.
- `weapons-and-items.md`, *On the Preparations screen*: one sentence per
  rule.
- Pictures: `docs/img/0478-loadout-{weapon,armour,empty-slot}.png`.
- For the Act 1 playtest (1064): whether the item line reads well,
  including the weapon line wrapping onto a second row.
