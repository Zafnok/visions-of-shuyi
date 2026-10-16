---
id: "0459"
title: The guide's box with the face covers the side panel on long lines
type: bug
milestone: M3 Battle UI
model: sonnet-5
effort: medium
status: todo
blocked_by: []
nick_input: sign-off
completed:
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

- [ ] At `Full` on Chapter 1's first turn the tip box doesn't cover the
      side panel.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit or snapshot: a tip with a 60-character line, with the face, stays
  within the map.

## Completion notes

*(Filled in by the session that completes the ticket.)*
