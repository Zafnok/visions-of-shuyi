---
id: "0459"
title: The guide's box with the face covers the side panel on long lines
type: bug
milestone: M3 Battle UI
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: []
nick_input: sign-off
completed: 2026-10-15
---

# 0459 — The guide's box covers the side panel on long lines

## Context

Found while rendering the lessons (ticket 0457) with the bought art. The
tip box with the guide's face (0455) and the lesson box (0456) grow with
their longest line. With the face beside the text, a line of more than
about 45 characters makes the box run past the map onto the side panel,
hiding the start of its lines (the terrain name, the unit's name). The
first-turn tip ("Your move", `assets/data/tips.ron`) does it today in
Chapter 1 at `Full`. The formats allow 60-character lines
(`MAX_LINE_CHARS`). 0457 kept the lessons' lines to 45 so they fit; the
tips weren't touched.

Without the bought art (no face) the box fits.

## Nick input

**Sign-off:** after merge, look at Chapter 1's first turn at `Full`: the
tip's box stays over the map and the side panel reads in full.

## Scope

**In:**
- The box never covers the side panel: wrap or narrow the text when the
  face is shown, or keep tip lines short enough. Pick one and say which.
- A test that every shipped tip and lesson fits beside the face.

**Out (do not do):**
- Rewording the tips beyond what fitting needs (0731 writes the guide's
  voice).

## Implementation steps

1. Decide where the fix goes (the box's skin, or the text limits).
2. Fix, then render the first turn with `cargo run -p xtask --features
   private-assets -- frame-png ... --tips` and look at it.

## Acceptance criteria

- [x] At `Full` on Chapter 1's first turn the tip box doesn't cover the
      side panel.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit or snapshot: a tip with a 60-character line, with the face, stays
  within the map.

## Completion notes

- **The fix is in the box's skin** (`crates/ui/src/tips/glyph.rs`): the
  text wraps to the room the map leaves beside the face (48 cells; 66
  without the face, so then nothing changes). The formats keep their
  60-character lines, and a translation or a long key name can't push the
  box over the side panel either.
- A paragraph (lines between blank lines) whose lines all fit stays as
  written. One with a longer line is joined and wrapped again as a whole,
  so a word isn't left alone on a row (wrapping each line by itself left
  "unit" on its own row in the first-turn tip). A lesson item that is too
  long goes on under its label, its mark on the first row only.
- Tests: a 60-character line beside the face stays over the map, a long
  item wraps under its label, only a paragraph with a long line is wrapped
  again, and every tip and lesson the game has fits beside the face, its
  box over the map (top and bottom for the lessons). The old test that
  said such a box reaches over the panel is gone.
- Rendered Chapter 1's first turn at `Full` ("Not familiar") with the
  bought art: the "Your move" tip now takes four rows beside the face, the
  box ends at the map's right edge and the side panel reads in full.
- No gameplay rules decided. No follow-up tickets.
