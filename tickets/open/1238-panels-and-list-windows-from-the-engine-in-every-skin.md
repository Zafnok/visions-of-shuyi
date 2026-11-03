---
id: "1238"
title: "Panels and list windows from the engine in every skin: hand-built boxes on panel, and the shop, Guides and Preparations lists keep their window"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1238 — Panels and list windows from the engine in every skin

## Context

From the game/engine boundary audit of 2026-11-03.

1. **Hand-built boxes.** 1165 moved three battle boxes onto the engine's
   `GlyphBuffer::panel` (`PanelLook`: clear, frame, a title two cells in
   with a space each side). These still write `fill_rect` + `draw_box`
   (+ a ` title ` printed at `x + 2`) by hand:
   `screens/battle/glyph/progress.rs` (three boxes), `glyph/info.rs`,
   `glyph/guides.rs`, `glyph/rewind.rs`, `glyph/shop.rs` (two), and
   outside the battle `credits/glyph.rs`, `save/glyph.rs`,
   `preparations/glyph.rs` (`paint_list`), `results/glyph.rs`,
   `camp/glyph.rs` (no fill), `world_map/glyph.rs` (a centred title),
   `lead_select/glyph.rs` and `dialogue/log/glyph.rs`.
2. **Lists that scroll on every Up.** The shop and Guides screens keep a
   `ListCursor`, but their views carry only `focus`; the skins then
   build a throwaway cursor for the window (`first_shown` in
   `glyph/shop.rs:213` and `glyph/guides.rs:96`, `window_start` in
   `preparations/glyph.rs:47`). A new cursor starts at row 0, so the
   window is always `focus + 1 - rows`: on a long list the focus sits on
   the bottom row and every Up scrolls. The engine's cursor keeps its
   window (`ListCursor::window`, read through `first_shown`), as
   `save.rs` and `class_change.rs` already use it. The three shops
   (`battle/shop.rs`, `town.rs`, `preparations/shop.rs`) all paint
   through `glyph/shop.rs`.

## Nick input

None. The one visible change (a long shop, Guides or Preparations list
moves its window only when the focus leaves it, as every other list
does) goes in the Completion notes for the Act 1 playtest's checklist
(1064). Verified by tests and snapshots.

## Scope

**In:**
- Each box above on `buf.panel(rect, &PanelLook::new(style, text,
  bg).border(..).title(..), title)`; the world map's on `panel(.., None)`
  plus `print_centred`. Camp only if nothing inside is painted before
  its box.
- The three shops, Guides and each Preparations list keep a remembered
  window: call `cursor.window(rows)` after each step and `set_len` (as
  `save.rs` does), with `rows` from the skin's constants
  (`glyph::shop::list_rows`, `glyph::guides::LIST_ROWS`, the
  Preparations boxes), and put `cursor.first_shown()` in
  `ShopScreenView`, `GuidesView` and the Preparations list view. A
  Preparations list kept as a plain index becomes a `ListCursor`.
- The three helpers and their tests go.

**Out (do not do):**
- `class_change/glyph.rs`'s question box and the battle's
  `draw_dialog_at` (1185 item 2), the cuts and centring (1185 item 4).
- `pd::window` and the one-bit scroll rule (1077).
- How a too-long text is cut on the PC (0068); `panel` cuts a title that
  overruns its border, which no title does today.
- `glyph/rewind.rs`'s window by block heights.

**Either order with 1185:** if 1185 is done, its cuts sit inside these
boxes: keep them. If not, leave the cuts and the sized question boxes to
it. **Either order with 1077:** if 1077 is done, `pd::window` is already
on `panel`; if not, leave it to 1077. **Either order with 1216** (one
shop flow): if 1216 is done, give its one shared cursor the window; if
not, do it in each of the three shops. **Either order with 1215** (the
how-many question): if 1215 is done, `paint_sell` and its box in
`glyph/shop.rs` are gone: one shop box is left; if not, move both shop
boxes, and 1215 deletes the sale's. **Either order with 1211** (the
engine's credits screen): if 1211 is done, `credits/glyph.rs` is gone:
skip it; if not, move its box, and 1211 deletes the file.

## Implementation steps

1. Read the engine's `frame/draw.rs` (`panel`, `PanelLook`) and
   `widgets/list.rs` (`window`, `first_shown`), and `save.rs`'s use.
2. Boxes, one skin at a time, its snapshots unchanged after each.
3. Windows: shop, Guides, Preparations. A test per list: from the
   bottom of a long list, Up moves the focus without scrolling.
4. Measure with `cargo xtask code-lines`.

Expected size: about 40 production lines removed and 15 added (16 boxes
at one or two lines each; the windows net about zero); about 20 test
lines removed.

## Acceptance criteria

- [ ] `grep -rn -A1 "fill_rect" crates/ui/src/screens | grep draw_box`
      finds only the question boxes 1185 owns (or none).
- [ ] `grep -rn "ListCursor::new(.*).at(.*).window" crates/ui/src/screens`
      finds no skin.
- [ ] A test per list: Up from the last row of a list longer than its
      box keeps the first row shown.
- [ ] No snapshot changes except the long-list windows, reviewed.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the list windows as above.
- Snapshot: every skin touched (reviewed, expected unchanged).

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
