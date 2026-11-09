---
id: "0492"
title: "The shop's sell box hides the lines behind it"
type: bug
milestone: M3 Battle UI
model: sonnet-5 / gpt-6.1-terra
effort: low
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0492 — The shop's sell box hides the lines behind it

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

- [ ] No list text shows beside the sell box (look test).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Snapshot: the sell-confirm snapshots of the three shops.

## Completion notes

*(Filled in by the session that completes the ticket.)*
