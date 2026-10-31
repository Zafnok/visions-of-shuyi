---
id: "0840"
title: "Options: the last row sits on the box's bottom border"
type: bug
milestone: M7 Chapter 1 & game flow
model: implementation
effort: low
status: done
blocked_by: []
nick_input: none
completed: 2026-10-19
---

# 0840 — Options: the last row sits on the box's bottom border

## Context

Found by 0125 while taking the README's screenshots
(`cargo xtask frame-png --keys "Down Down f"`, the Options screen opened
from the title): `Restore defaults` is printed on the panel's bottom
border line, so the border runs through the words.

The cause is in `crates/ui/src/screens/options/glyph.rs`: the panel is a
fixed `PANEL: Rect = Rect::new(20, 3, 60, 20)` (rows 3 to 22), and the
rows start at `PANEL.y + 2` with a blank line before each group. From the
title, in a build with the bought art (which shows `Map look` and
`Map zoom`), there are 16 rows and 2 group gaps, so the last row lands on row 22,
the border. With a campaign loaded the `Game mode` row is shown too
(`OptionsScreen::rows`), and the last row lands on row 23, under the box.
0838 (the tutorial level row) adds one more row, so it will get worse.

The Options screen follows ADR-0054 (view + skin): the fix is the skin's
job only; the view already says which rows there are.

## Nick input

None.

## Scope

**In:**
- The glyph skin sizes the panel from the view: tall enough for every row
  the view has, its group gaps, and one blank row under the last row
  before the border. The message row under the panel and the question box
  move with it.
- If the rows can't fit the console at all, that is a test failure, not a
  clipped screen.

**Out (do not do):**
- No change to which rows exist, their order or their text (`options.rs`,
  `view.rs`).
- No change to 0838's or 0839's work; if either landed first, its rows
  are simply counted.

## Implementation steps

1. In `options/glyph.rs`, replace the fixed height with one worked out in
   `paint` from `view.rows` (rows + `starts_group` gaps + the title row,
   the blank row under it, a blank row at the bottom, and the two border
   rows). Keep `PANEL`'s x, y and width.
2. Work out `MESSAGE_ROW` from that panel instead of the constant.
3. Look at the screen with and without a campaign loaded with
   `cargo xtask frame-png`, built with `--features private-assets` (the
   two picture rows only show with the bought art).

## Acceptance criteria

- [x] Every row label is inside the panel's border, one row above the
      bottom border at the lowest, with and without a campaign loaded:
      covered by the skin test and snapshots 0838 added (see the notes),
      not by a new test in `options/tests.rs`.
- [x] The existing Options snapshots were updated and reviewed by 0838.
- [x] All gates in the `run-gates` skill pass (1157's PR).

## Tests required

- Unit: the panel height for a view with and without the `Game mode` row.
- Snapshot / integration: the Options screen with and without a campaign
  loaded.

## Completion notes

**Fixed by 0838 (PR #291), closed by 1157 with no code change.** 0838
(the Options row for the tutorial level) found the same bug: with a
campaign the rows needed 18 lines and `Restore defaults` sat on the
border. Its skin fix grows the panel down to fit its rows (never smaller
than before) and moves the message under it. That skin is Shuyi
Engine's options skin since 1112 (1155 also fits it to the console's
height), so this ticket had no engine work left for 1157 to split off.

Checked 2026-10-19 at the engine pin of 1131:
- the engine's skin test `the_panel_grows_to_fit_its_rows` (18 rows and
  two group gaps: the last row right above the bottom border, the
  message two rows under it), and `on_a_small_console_every_row_is_inside_the_panel`;
- the game's snapshot
  `it__options__classic_switches_to_casual_at_preparations_and_never_back`
  (a campaign loaded, so `Game mode` and `Tutorial` are shown):
  `Restore defaults` is on the row above the border.

Not done: a game test that opens Options with and without a campaign and
checks every label against the border (the acceptance criterion's exact
test). The engine's test and the snapshots above already pin it.
