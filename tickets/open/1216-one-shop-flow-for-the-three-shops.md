---
id: "1216"
title: "One shop flow for the three shops: the battle's, the Preparations Shop tab and the town's"
type: infra
milestone: "Engine: carve"
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1216 — One shop flow for the three shops

## Context

The game/engine boundary audit of 2026-11-03 found the shop's controls
written three times. 0409 built the on-map shop
(`crates/ui/src/screens/battle/shop.rs`, `ShopScreen`), 0443 the
Preparations Shop tab (`screens/preparations/shop.rs`) and 1007 the town
shop (`screens/town.rs`, `TownScreen`) by copying it. All three share
`ShopScreenView` and its painter, but each has its own row type (`Offer`
at shop.rs:161, `Deal` at preparations/shop.rs:51 and town.rs:42) and
its own copy of nearly the same code: the key dispatch (shop.rs:392,
preparations/shop.rs:213, town.rs:299), choosing a line (shop.rs:426,
`make_deal`, `choose`), the sale-count stepper (shop.rs:458,
preparations/shop.rs:271, town.rs:250), the special-weapon `Ask`
(shop.rs:483, :304, :279), `special_view`, `sell_view`, the line
builder (`line`, `deal_line`) and the `ShopScreenView` with its help
key. They hold the list cursor three ways: a `ListCursor` with
`set_len`, a plain `deal: usize` with `moved` (preparations.rs:205,
:642), and a `ListCursor` rebuilt on each key (town.rs:313).

What really differs is where a trade goes. The battle queues `ShopTxn`s
into a preview `BattleState` and sends one command when it closes;
Preparations calls `Preparations::{buy, sell, repair}` at once; the town
calls `Campaign::trade` at once. A shop is this game's concept, so this
is a game-side structure (ADR-0092), with no engine change. 1215
then puts the one sale stepper on the engine's how-many question.

## Nick input

None. No change a player sees: the snapshots stay the same.

## Scope

**In:**
- A new `crates/ui/src/screens/shop.rs` with a `ShopPicker` (tab,
  `ListCursor`, `selling: Option<SellConfirm>`, `special: Option<(usize,
  Ask<bool>)>`), one `ShopRow<T>` (item, count, gold, durability, holder
  name, enabled, and the trade `T`) and a `Trader` trait the three
  screens implement. The shared items now in `battle/shop.rs` (`Tab`,
  `SellConfirm`, the `HELP*` and `SELL_SPECIAL` keys, `gold_text`,
  `kind_key`, `wont_take_key`, `stat_line`) move there.
- The picker plays its menu sounds on the `AudioQueue` it is handed, as
  `Ask::handle` does. That ends the battle's throwaway
  `AudioQueue::new(AUDIO)` (shop.rs:488) and its `ShopEffect`.
- One cursor style: the picker's `ListCursor` with `set_len`.

**Out (do not do):**
- The how-many question on the engine's widget (1215, after this).
- Moving `ShopScreenView`, `ShopLineView` or the glyph painter
  (`battle/view/shop.rs`, `battle/glyph/shop.rs`); any change of look,
  text or keys.
- The trade rules in `trpg-core` (`ShopTxn`, `Campaign::trade`,
  `Preparations`).

**Either order with 1185:** if 1185 is done, the battle shop's
throwaway queue in `step_special` is already gone: the picker takes the
queue the screen hands it; if not, remove it here, and 1185 skips that
item. **Either order with 1238:** if 1238 is done, each shop's
`ListCursor` keeps a remembered window (`cursor.window(rows)`,
`first_shown` in `ShopScreenView`): keep that on the picker's one
cursor; if not, nothing to do (1238 adds it to the picker).

## Implementation steps

1. **Read** the three files above and their tests
   (`battle/shop/tests.rs`, `preparations/tests.rs`, `town/tests.rs`).
2. **`Trader`** in `screens/shop.rs`: `type Trade: Clone`; `kind() ->
   ShopKind`; `rows(tab) -> Vec<ShopRow<Self::Trade>>` (enabled set);
   `make(&mut self, trade, count) -> bool`; `items() -> &ItemTable`;
   `gold() -> Gold`; `holder(row) -> Option<String>`. The town's
   items come from `ctx`, so it implements it on a short-lived adapter
   (`TownTrader<'a> { campaign, shop, items }`); the battle keeps its
   txn queue and preview, and its pack copy looked up again after each
   sale (`sell`, shop.rs:517), inside its `make`. If Preparations' and
   the town's `sales` build the same rows from a stock, one helper builds
   both.
3. **`ShopPicker`**: `step(&mut self, trader, action, audio) ->
   ShopStep { Stay, Leave }`, holding the dispatch, `choose`, `step_sell`
   and `step_special` once; `view(ctx, trader, header)` builds the
   `ShopScreenView` (the line, sell and special views once). The
   header is what differs: the shop's title, the unit or town name, and
   whether the help line is shown (Preparations' is empty).
4. **The three screens** keep only their trader, their `Screen` impl and
   what `Leave` does: the battle pops with its `ShopVisit`, the town with
   its `Campaign`, Preparations goes back to its tabs.
5. **Tests** that read `ShopEffect` read the sounds from
   `AudioQueue::pending` or the picker's state instead; each behavior
   tested in all three files is tested once on the picker, and each
   screen keeps the tests of its own trader.
6. **Expected size:** about 450 production lines removed and 250 added
   (a net 200 fewer); this is a refactor, so count the lines it rewrites
   (friction rows 1161 and 1162). Measure with `cargo xtask code-lines`
   and add the friction row (`docs/engine/friction.md`).

## Acceptance criteria

- [ ] `SellConfirm`'s stepping, the special-sale `Ask` and the view
      building each appear once, in `screens/shop.rs`.
- [ ] No `AudioQueue::new` outside tests in `crates/ui/src/screens`.
- [ ] Every shop snapshot (battle, Preparations, town) is unchanged.
- [ ] Production lines of the three files and the new module fall by
      about 200 (`cargo xtask code-lines`, before and after in the
      completion notes).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the picker with a fake `Trader` (dispatch, the count's ends, the
  special question, a line that is gone after a trade, the sounds).
- Snapshot / integration: the existing shop snapshots and harness tests
  of the three screens, unchanged in what they assert.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
