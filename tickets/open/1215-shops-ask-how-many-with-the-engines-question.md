---
id: "1215"
title: "The shops ask how many with the engine's how-many question"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: blocked
blocked_by: ["engine#3205", "1216"]
nick_input: none
completed:
---

# 1215 — The shops ask how many with the engine's how-many question

## Context

The game/engine boundary audit of 2026-11-03 found that the shops' "Sell
how many?" box names nothing of the game. Today it is the game's
`SellConfirm { line, count }` (`crates/ui/src/screens/battle/shop.rs:197`),
its stepping (up adds 1, down takes 1, `1..=count` with no wrap, Confirm
sells, Cancel backs out), `SellConfirmView { question, count, max, gold }`
(`battle/view/shop.rs:40`) and `paint_sell` (`battle/glyph/shop.rs:181`):
a centred double box with the question, `▲` dimmed at the most,
`n / max`, `▼` dimmed at 1, and the total under it.

Engine ticket
[engine#3205](https://github.com/Zafnok/shuyi-engine/blob/main/tickets/open/3205-a-how-many-question-widget.md)
adds `shuyi_ui::widgets::Count`, its `CountView` and a glyph painter in
this look, next to `Ask`. 1216 first gathers the three shops' copies of
the stepper into one shop flow (`screens/shop.rs`, `ShopPicker`), so
this ticket changes one call site, not three.

## Nick input

None. The look stays the same: verified by the shop snapshots.

## Scope

**In:**
- The shop flow holds the sale as the line and an engine `Count`;
  `ShopScreenView::selling` becomes `Option<CountView>`.
- `SellConfirm`, `SellConfirmView` and `paint_sell` are deleted; the
  shop's glyph skin paints the sale with the engine's `paint_in` in
  `sell_box(area)`, over the list as now.

**Out (do not do):**
- The special-weapon question (already the engine's `Ask`), the help
  keys and text (`battle.shop.help_sell` stays the game's), and the
  shop flow itself (1216).
- A step of more than one, or wrapping (not in the engine's widget).

**Either order with 1238:** if 1238 is done, `paint_sell`'s box is on
`GlyphBuffer::panel`: delete it with the rest; if not, delete it as it
is, and 1238 has one box fewer in `glyph/shop.rs`.

## Implementation steps

1. **Move the pin** past engine#3205's merge, unless an open
   `engine-pin` PR has already (`engine-change` §7, the `engine-pin`
   skill: review the engine PRs it passes).
2. Re-export `Count`, `CountEvent`, `CountView` and the `count` module
   from `crates/ui/src/widgets/mod.rs`, beside `Ask`.
3. In `screens/shop.rs` (1216's `ShopPicker`): `selling:
   Option<(usize, Count)>`, opened with `Count::new(
   "battle.shop.sell_question", row.count)`. The sale step calls
   `Count::handle(action, audio)`: `Chosen(n)` makes the trade `n`
   times as today, `Cancelled` closes it with `MenuSound::Cancel`. The
   view is `count.view_with_keys(ctx, ctx.help_keys(), &[("item",
   &item)])` `.noted(...)` with `battle.shop.sell_for` and the total.
   The accessors the tests use (`selling()`) return the line and value.
4. In `battle/glyph/shop.rs`, replace `paint_sell` with
   `count::glyph::paint_in(&ctx.palette, sell, sell_box(area), buf)`.
   The engine's roles map to the game's colors one to one
   (`crate::color::engine_color`: `Panel`, `Focus`, `Highlight`,
   `Text`, `TextDim`), so the shop snapshots must not change. If a cell
   still differs, keep `paint_sell` as the game's painter of the
   engine's `CountView` and say why in the friction row.
5. Update every other painter or test of `ShopScreenView::selling`
   (`grep -rn "selling" crates/ui/src`), and drop
   `the_sale_arrows_dim_at_the_ends` (the engine tests it); keep
   `the_sale_box_sits_over_the_list`.
6. **Expected size:** about 45 production lines removed and 10 added;
   about 30 test lines fewer. Measure with `cargo xtask code-lines` and
   add the friction row (`docs/engine/friction.md`).

## Acceptance criteria

- [ ] `SellConfirm`, `SellConfirmView` and `paint_sell` no longer exist
      (`grep -rn` finds none in `crates/`).
- [ ] The shop snapshots of the battle, Preparations and town shops are
      unchanged.
- [ ] Up and down in a sale still stop at 1 and at the line's count,
      and Confirm sells that many (the existing tests, on `Count`).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the shop flow's sale with `Count` (ends, Confirm, Cancel, the
  total in the note).
- Snapshot / integration: the existing shop snapshots and harness tests.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
