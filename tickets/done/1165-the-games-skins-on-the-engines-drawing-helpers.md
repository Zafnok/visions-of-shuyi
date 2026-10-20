---
id: "1165"
title: "The game's glyph skins on the engine's drawing helpers: one meter, one titled box, one way to centre and cut text"
type: feature
milestone: "Engine: carve"
model: sonnet-5
effort: medium
status: done
blocked_by: ["engine#2311"]
nick_input: none
completed: 2026-10-20
---

# 1165 — The game's skins on the engine's drawing helpers

## Context

The first engine-friction sweep (1153, `docs/engine/friction.md`) found
in the game's glyph skins:

- a `█`/`░` meter loop written 7 times: `screens/battle/glyph.rs`,
  `battle/glyph/fight_box.rs`, `battle/glyph/forecast.rs`,
  `battle/glyph/info.rs`, `battle/glyph/progress.rs`,
  `battle/stage/glyph.rs`, `screens/results/glyph.rs` (and `debug.rs`);
- a label centred by `i32::try_from(text.chars().count())`, 16 times;
- a box with a title in its top border, 3 times in
  `screens/battle/glyph.rs` (`draw_dialog_at`, the banner, the panel);
- `panel_line` (`battle/glyph.rs`) and `put` (`battle/glyph/forecast.rs`)
  re-write the engine's `GlyphBuffer::print_fg` plus `screens::cut`;
- `cells_px` (`battle/stage/glyph.rs`) copies the kit's
  `grid::layout::px_rect`.

Engine ticket 2311 (`Zafnok/shuyi-engine`) adds `panel`, `meter`,
`print_centred` and `print_cut`. The last two bullets need no engine
change and are part of this ticket.

## Nick input

None.

## Scope

**In:**
- Move the pin past 2311 (`engine-change` skill, step 7).
- Every copy above on the engine's helper, **keeping each look**: if a
  meter rounds differently from the engine's, find out which is right
  from its tests and design doc; if they really differ on purpose, keep
  that one and say so in the Completion notes.
- `panel_line`, `put` and `cells_px` replaced by the engine's functions.

**Out (do not do):**
- The one-bit skins (they have their own kit, `screens/playdate.rs`).
- The forecast's kill mark (0470).
- Any change the player sees.

## Implementation steps

1. Write the estimate: about 120 lines removed, 40 added.
2. One skin at a time; snapshots unchanged after each.

## Acceptance criteria

- [x] `grep -rn "'░'" crates/ui/src` finds no meter loop outside tests
      (only the engine's call).
- [x] `grep -rn "fn panel_line\|fn cells_px" crates/ui/src` finds
      nothing.
- [x] Every snapshot is unchanged.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- The existing skin tests and snapshots, unchanged.

## Completion notes

- **Pin:** already past engine 2311 (`939f023`), so no pin move.
- **Estimate:** 40 added (the ticket's). Measured 157 added, 163 removed
  (3.9×): see the friction row. Most added lines are call sites renamed
  one for one.
- **Meters:** all seven bars paint through the engine's `meter`, by
  `meter_cells` (`screens/battle/units.rs`), which hands it cells already
  rounded; `paint_hp_bar` (fight box, side panel, info, stage plates) and
  `paint_exp_bar` (`progress.rs`; the EXP box and the results) build on
  it, and the forecast's three-shade bar is two `meter_cells` calls.
  **The rounding stays the game's, on purpose:** the HP bar rounds half
  up and shows 1 HP of 40 as empty (`hp_fill_at_the_thresholds`), the EXP
  bar rounds down so 99 of 100 is never full (`bar_cells` tests).
  `meter_fill`'s rule (never empty unless 0, never full unless full)
  would change both, a change the player sees.
- **Boxes:** the dialog (`draw_dialog_at`), the banner, the side panel,
  the forecast and the fight box paint with `panel`; the dialog's title
  goes in its border by `panel`, the others' centred titles and the
  banner's lines by `print_centred`, as does the dialogue name plate.
- **Cut lines:** `panel_line` and `put` became one `print_clipped`
  (`screens/battle/glyph.rs`), not the engine's `print_cut`: the engine
  ends a cut line with `…`, the PC glyph skins cut hard (the side panel's
  test, and the forecast's "Frost Elemental" class shows as
  `Frost Element`). Whether PC should show `…` is a look decision: 0068.
- **`cells_px`** is the kit's `px_rect` (the PC console's cells are the
  grid's, 8 × 16; its test still checks the size).
- **Deviation:** of the 16 `chars().count()` sites, only the 5 that centre
  a label moved; the others align right or lay several parts on one row,
  which no helper covers. A centred label that is wider than its box is
  now cut, where it ran over the border before (no snapshot has one).
- **Follow-up:** 0068 (decide `…` or a hard cut on PC).
- Gameplay rules decided: none.
