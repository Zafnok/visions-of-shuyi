---
id: "1411"
title: "The sprite skin's units on the kit's piece painter: the clip mask, the row order and the glide go"
type: feature
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: blocked
blocked_by: ["engine#3250"]
nick_input: none
completed:
---

# 1411 — The sprite skin's units on the kit's piece painter

## Context

The third engine-friction sweep (1397, `docs/engine/friction.md`) found
that the sprite skin's unit painter is mostly generic.
`crates/ui/src/map_view/sprite/units.rs` has 292 production lines. Of
those, 162 stand any picture taller than its tile on the tile's foot,
draw by row, cut it so it never covers a unit on the tile above (the
clip mask, Nick's look from 0039 and 0436), glide it between two tiles
and fade or flash it. Engine ticket 3250 (`Zafnok/shuyi-engine`) moves
that into the grid kit as `grid::pieces`. The marked items carry
`// engine-friction: engine#3250, moves (game 1411 deletes it)`.

## Nick input

None. Nothing the player sees changes: the sprite look's frames are the
same.

## Scope

**In:**
- Move the pin past engine 3250, if it isn't already (`engine-change`
  skill §7, with the `engine-pin` skill's review).
- `paint_pictures` builds one `PieceLook` per unit and calls
  `pieces::paint`. The kit's functions replace:
  - `unit_dest`, `outline_dests`, `offset_px` and `toward`;
  - `Place` and `place`;
  - the three opacities.
- `paint_marks` uses `pieces::place` for its tile, row and `under`.
- What stays the game's:
  - the picture lookup (`picture_ids`, the lead by gender);
  - `unit_frame` (facing × walking frame);
  - `outline_color`;
  - the HP bar and the effect marks with their bounce.

**Out (do not do):**
- the glyph and Playdate skins' units (they draw letters, not tall
  pictures);
- any change to the look: the clip stays on, the outline stays four
  sprite items.

## Implementation steps

1. Write the estimate: about 160 lines removed, 40 added (building
   `PieceLook`s).
2. Move the pin, then swap the marked items for the kit's one at a time,
   keeping `map_view/sprite/units.rs`'s tests green.
3. Tests that call the deleted helpers directly (`place_*`,
   `offset_px_*`) move to reading the sprite items in the buffer, or
   are dropped where the engine's ported tests now cover them. Say which
   in the completion notes.

## Acceptance criteria

- [ ] `grep -n "engine#3250" crates/ui/src` finds nothing.
- [ ] The sprite skin's frame snapshots in `crates/ui/src/map_view/`
      don't change.
- [ ] The engine-friction line is in the PR body.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- The existing sprite skin tests and snapshots, unchanged in what they
  show.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
