---
id: "0447"
title: "Player units' blue outline: easier to see on the bought tiles"
type: tuning
milestone: M3 Battle UI
model: sonnet-5
effort: medium
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 0447 — The player's blue outline, easier to see

## Context

Nick, playing Chapter 1 (ticket 0804, 2026-10-12):

> "the blue outline is hard to see for friendlies, the red is very clear."

A sprite unit's side is a 1-pixel outline in its side's colour
(`docs/design/look-and-feel.md`, *A sprite unit's side*, decided
2026-10-02: player blue, enemy red, ally green, neutral yellow). Red stands
out on grass and water; the palette's blue is dark and close to both.

Code: `crates/ui/src/map_view/sprite/units.rs` (the outline is painted in
`paint_units` with `ctx.palette.get(faction_color(unit.faction))`, dimmed
when the unit has acted; `outline_dests`). The colour names are in the
palette (ADR-0018); `faction_color` picks one per side.

## Nick input

**Decision, asked first** (`ask-nick`; frames from `cargo xtask frame-png`
built with `--features private-assets`, on grass, water, road and a dark
tile, with fresh and already-acted units of both sides; sent to him, never
committed). Offer at least:

- **A. A lighter blue** for the player's outline (a brighter palette blue
  or cyan), same 1 pixel.
- **B. A 2-pixel outline** for every side, same colours.
- **C. The colour with a dark edge round it** (1 pixel of colour, 1 pixel
  of near-black outside), for every side.
- **D. A and B together.**
- **E.** "Describe your own."

Say honestly what each does to the acted (dimmed) look and to units
standing next to each other. Expect more than one round.

## Scope

**In:**
- The outline of sprite units on the battle map, as Nick picks.
- `look-and-feel.md` records his answer with the date.

**Out (do not do):**
- The glyph look's unit colours.
- The cursor (0444), range tints (decided, 75%), zoom (0439).
- The HP bar and the marks over a unit.

## Implementation steps

1. Ask Nick (above) and record the answer.
2. Change the outline in `sprite/units.rs` (and the palette or
   `faction_color` if a colour changes). It follows ADR-0038: only the
   sprite skin's painting changes; the map scene is untouched.
3. If the outline gets wider, check the clip mask described at
   `units.rs` ("The clip mask") still leaves no edge standing alone.
4. Update `a_unit_is_its_outline_its_picture_and_its_hp_bar` and review
   the changed sprite-skin snapshots.
5. Render Chapter 1's first turn, look at it, send it to Nick.

## Acceptance criteria

- [ ] `look-and-feel.md` records Nick's pick, with the date.
- [ ] The outline is what he picked, with a test naming its rectangles
      and colour.
- [ ] Every glyph-skin snapshot is unchanged.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit (with the skin, `sprite/units.rs`): the outline's rectangles and
  colour for a fresh and an acted unit.
- Snapshot: the sprite skin's.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
