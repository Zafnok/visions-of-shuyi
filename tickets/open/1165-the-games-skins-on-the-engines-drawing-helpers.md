---
id: "1165"
title: "The game's glyph skins on the engine's drawing helpers: one meter, one titled box, one way to centre and cut text"
type: feature
milestone: "Engine: carve"
model: sonnet-5
effort: medium
status: todo
blocked_by: ["engine#2311"]
nick_input: none
completed:
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

- [ ] `grep -rn "'░'" crates/ui/src` finds no meter loop outside tests
      (only the engine's call).
- [ ] `grep -rn "fn panel_line\|fn cells_px" crates/ui/src` finds
      nothing.
- [ ] Every snapshot is unchanged.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- The existing skin tests and snapshots, unchanged.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
