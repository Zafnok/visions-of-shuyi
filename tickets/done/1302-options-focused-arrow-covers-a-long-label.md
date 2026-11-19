---
id: "1302"
title: "The focused value's arrow no longer covers the last letter of a long Options label"
type: bug
milestone: M8 Release
model: sonnet-5 / gpt-6.1-terra
effort: low
status: done
blocked_by: []
nick_input: none
completed: 2026-11-19
---

# 1302 — The focused value's arrow no longer covers a long Options label

## Context

Found by 1301 (2026-11-19). On the PC's Options screen (the engine's glyph
skin, `shuyi_ui::screens::options::glyph`, `paint_row`) a focused row
draws `◄` just before its value. The label "Voice on hover in the log"
is as long as the label column, so when that row is focused it reads
`Voice on hover in the lo◄ On ►`: the arrow covers the last letter. Any
longer label (or a translation) is cut the same way.

## Nick input

None: no text changes, only that a label is never covered (*the agent's
starting rule*: the value column starts far enough right for the longest
label and its arrow).

## Scope

**In:** the value column leaves room for the longest label plus the
focused value's `◄`, so no label is covered; likely an engine change to
the options glyph skin (`engine-change` skill; ask Nick before an engine
ticket or PR), then the pin move here.

**Out (do not do):** rewording or shortening labels (dialogue/UI text is
Nick's and the writer's); the Playdate's skin.

## Implementation steps

1. Engine: the options glyph skin places the value column from the
   longest label's width (plus the arrow), not a fixed column; its tests.
2. Game: move the pin; update the options snapshots that move.

## Acceptance criteria

- [x] With "Voice on hover in the log" focused, its whole label shows.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit (engine skin): the longest label focused is painted whole.
- Integration: the focused-row walk of 1301's
  `with_pictures_the_focused_row_is_always_on_screen` matches whole
  labels, not their first 20 letters.

## Completion notes

Engine: ticket 3226, Zafnok/shuyi-engine#511. The options glyph skin puts
the values' column at its usual place (30 cells into the panel), or
further right when the longest label needs it, the same for every row so
the values stay lined up and don't move as the list scrolls. A focused
label then ends right before its `◄`. Labels of 24 cells or fewer paint as
before. Engine test: `the_longest_label_focused_is_painted_whole`.

Game: the pin moves `d10d1950 → 7a7d7b0c` (only #511; its ledger section
in `docs/engine/pins.md`). 1301's focused-row walk matches whole labels;
the title's Options snapshot moves every value one cell right, as "Voice
on hover in the log" is 25 cells.

Not done: the column isn't capped by the panel's width, so a label of
about 30 cells (a long translation) would push a volume slider past the
panel's border. No label is that long today; the panel would need to grow.

No gameplay rules decided: layout only.
