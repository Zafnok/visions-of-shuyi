---
id: "0072"
title: "Decide: when the battle map scrolls as the cursor nears the view's edge"
type: design-decision
milestone: Design decisions
model: reasoning
effort: medium
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 0072 — Decide: when the battle map scrolls

## Context

Ticket 0449 added arrows at the edges of the battle map's view where the
map goes on past it (`docs/design/look-and-feel.md`, *Arrows at the map's
edges*). Asked whether the scrolling as it is was what he meant ("I guess
it would just happen when you move the cursor past the viewport it
moves"), Nick, 2026-10-21:

> "we might revisit the threshold for when it moves but that should be a
> followup / 00 ticket rather than in this ticket"

Today the camera follows the cursor (`Camera::follow`, the engine's
`shuyi_ui::grid::camera`, used by `crates/ui/src/screens/battle/camera.rs`):
the view scrolls once the cursor comes within **3 tiles** of its edge
(`Camera::MARGIN`), one tile per step, and stops at the map's edge. At 2×
with the bought art the view is 17 × 15 tiles; Chapter 1's map is 24 × 16.

No design doc settles this: the 3-tile margin was a technical default.

## Nick input

**Decision**, asked with the `ask-nick` skill and pictures (Nick judges
pictures; render with `cargo shuyi frame-png`, the bought art with
`--features private-assets`, Chapter 1 at 2×; the key route is in 0449's
completion notes). Show a short sequence of frames (or a GIF) of the
cursor walking toward the right edge for each option. Offer at least:

- **A. As now: 3 tiles from the edge** (the view keeps 3 tiles of
  look-ahead), like Fire Emblem (GBA)'s map, which scrolls before the
  cursor reaches the edge.
- **B. A smaller margin: 1 or 2 tiles**: more of the view is used before
  it moves; less look-ahead.
- **C. Only when the cursor would leave the view** (margin 0), as Nick's
  words read literally; like Advance Wars at the screen's edge.
- **D. Re-centre on the cursor once it is near the edge** (a jump or a
  quick glide of half a view), like some strategy games' camera.
- **Or describe your own.**

Ask also whether the answer is the same at 1× (35 × 30 tiles at 1× with
the glyph look, 23 × 20 on the test tileset) or only at 2×.

Record the answer in `docs/design/look-and-feel.md`, in *Arrows at the
map's edges* (where it now says the view scrolls 3 tiles from the edge),
and update `docs/design/README.md`'s table.

## Scope

**In:**
- Nick's answer, recorded as above.
- If the answer changes the margin or the camera's behaviour: a ticket
  (`write-ticket` skill) that builds it. Whether that is a game-side
  setting or an engine change (`Camera::MARGIN` is the engine's) is that
  ticket's (`engine-change` skill).

**Out (do not do):**
- Any code. This is a decision ticket.
- The arrows' look (decided in 0449).
- Scrolling with the mouse, a minimap or a whole-map view.

## Implementation steps

1. Render the options as above and look at them.
2. Ask Nick (`ask-nick` skill), one round per answer he asks to see again.
3. Record his words and the rule in `look-and-feel.md`; update the README
   table.
4. If anything changes, write the building ticket; add it to 1064's
   `blocked_by` if Nick wants it before the Act 1 playtest.

## Acceptance criteria

- [ ] Nick's answer is in `look-and-feel.md` with his words.
- [ ] A building ticket exists if the answer changes the camera, or the
      completion notes say nothing changes.
- [ ] `cargo xtask ticket-lint` passes.

## Tests required

- None (a decision). The building ticket lists its own.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
