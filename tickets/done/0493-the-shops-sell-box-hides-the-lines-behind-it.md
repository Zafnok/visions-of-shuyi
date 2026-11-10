---
id: "0493"
title: "The shop's sell box hides the lines behind it"
type: bug
milestone: M3 Battle UI
model: sonnet-5 / gpt-6.1-terra
effort: low
status: done
blocked_by: []
nick_input: none
completed: 2026-11-09
---

# 0493 — The shop's sell box hides the lines behind it

## Context

Found while working 0486. The box that asks how many to sell
(`paint_sell`, `crates/ui/src/screens/battle/glyph/shop.rs` ~:181) is
painted over the list but the rows it covers keep whatever ran past its
right edge. Since 0486 made the lines longer, the Leather Vest's line
pokes out of the box's top-right corner as a stray `1` (the end of
`Wt 1`): see `battle/shop/snapshots/…sell_confirm.snap`, the row
`Leather Vest  200 ╔═══…═══╗1`. Any line long enough shows the same
bleed beside the box. The same box is painted in the Preparations and
town shops.

## Nick input

None.

## Scope

**In:**
- While the sell box is open, the list text on the rows it spans is not
  shown to the right of it (dim or blank those rows past the box, or
  draw the box as a panel over the whole list width; pick the one that
  reads best in the three shops' snapshots).

**Out (do not do):**
- Changing what a line says (0486) or where the box sits.

## Implementation steps

1. A look test in `glyph/shop.rs`'s tests: with a long detail on the row
   under the box's top border, no text of it is right of the box.
2. Fix `paint_sell` (or what calls it).
3. Refresh the three shops' sell snapshots and read each diff.

## Acceptance criteria

- [x] No list text shows beside the sell box (look test).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Snapshot: the sell-confirm snapshots of the three shops.

## Completion notes

- `paint_box` (`glyph/shop.rs`) blanks the rows the sell box spans from
  its right border to the end of the line before painting the box, so no
  text and no highlight shows beside it. Left of the box is untouched:
  the names and gold of the lines behind it still show, as before. One
  fill in the shared `paint_box`, so the battle, Preparations and town
  shops all get it.
- Look test: `nothing_of_the_lines_shows_beside_the_sale_box`.
- Snapshots: the battle shop's `sell_confirm` changed (the stray `1` and
  the Potion highlight right of the box are gone). The Preparations and
  town shops had no sell-confirm snapshot; this adds
  `shop_sell_confirm_snapshot` and `town_sell_confirm_snapshot`.
- Friction: 2.0× (estimate 5, measured 10), accepted because the fill is
  one call that rustfmt wraps plus a two-line comment.
- No gameplay rules decided.
