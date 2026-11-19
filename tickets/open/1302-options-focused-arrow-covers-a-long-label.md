---
id: "1302"
title: "The focused value's arrow no longer covers the last letter of a long Options label"
type: bug
milestone: M8 Release
model: sonnet-5 / gpt-6.1-terra
effort: low
status: in-progress
blocked_by: ["engine#3226"]
nick_input: none
completed:
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

- [ ] With "Voice on hover in the log" focused, its whole label shows.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit (engine skin): the longest label focused is painted whole.
- Integration: the focused-row walk of 1301's
  `with_pictures_the_focused_row_is_always_on_screen` matches whole
  labels, not their first 20 letters.

## Completion notes

*(In progress, 2026-11-19.)* Engine side done: engine ticket 3226,
Zafnok/shuyi-engine#511 (the options glyph skin's value column moves right
for the longest label; labels of 24 cells or fewer paint as before; all
engine gates pass). Game side on branch
`claude/options-arrow-label-coverage-20d58d`, built against the patch: the
1301 focused-row walk now matches whole labels, and the title-options
snapshot moves every value one cell right (the longest label, "Voice on
hover in the log", is 25 cells); the options tests pass.

Left: once Nick merges #511, a session of its own moves the pin to the
engine's `main` (`engine-change` §3.6, `engine-pin` review and ledger),
runs the gates, archives this ticket and opens the game PR.
