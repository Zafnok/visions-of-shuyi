---
id: "1422"
title: "The Combat Arts menu pads its columns in cells, so Chinese names line up"
type: bug
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: low
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1422 — The Combat Arts menu pads its columns in cells

## Context

Ticket 0263 made the battle menus pad their columns in screen cells
instead of characters, so wide (Chinese) glyphs line up. The sweep 1397
(2026-11-27) found that the arts list was missed.
`crates/ui/src/screens/battle/art_list.rs` (about lines 520-528) still
builds its rows with `format!("{name:<NAME_W$}  {cost:>6}  {source}…")`.
That pads by `char` count, so a Chinese art name (two cells per glyph)
pushes the cost and source columns right.

Steps to reproduce: start a battle in Chinese, give a unit a Combat Art,
and open its arts menu. **Expected:** the cost column lines up on every
row. **Actual:** the cost column moves right by one cell per Chinese
glyph in the name.

## Nick input

None. It is checked by a snapshot test.

## Scope

**In:**
- Pad the name and the source with the game's `pad_cells`
  (`screens/mod.rs`), and right-align the cost in cells, as 0263 did
  for the other menus.
- A failing test first.

**Out (do not do):**
- Moving `pad_cells` to the engine: that is engine#2291 and game 1410.
  **Either order with 1410:** if 1410 is done, use the engine's
  function; if not, use the game's `pad_cells`.

## Implementation steps

1. Write a test that renders the arts menu with a two-cell-wide name
   (use the `test` language pack or a test art whose name has CJK
   glyphs). Check that the cost column starts at the same cell on every
   row. Watch it fail.
2. Replace the `format!` widths with cell-width padding.
3. Check that the English snapshots don't change.

## Acceptance criteria

- [ ] The new test fails before the fix and passes after it.
- [ ] The English arts-menu snapshots are unchanged.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit or snapshot (the battle's art-list tests).

## Completion notes

*(Filled in by the session that completes the ticket.)*
