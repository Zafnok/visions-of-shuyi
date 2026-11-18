---
id: "0923"
title: "A livelier look for the move and attack ranges, matched to the threat arrows"
type: feature
milestone: M8 Release
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: ["0908"]
nick_input: decision
completed:
---

# 0923 — A livelier look for the move and attack ranges

## Context

Ticket 0908 (polish of the threat arrows, 2026-11-17): shown size and
colour tunings, Nick said "ur thinking too much in the box, several
games use a more 3D opacity effect with blinking and such for the attack
range tiles and for threats". Asked whether the range tiles get their
own ticket: "Yes, own ticket".

Today's ranges (`docs/design/look-and-feel.md`: the move and attack range
tints, which Nick liked, over glyph and picture tiles) are drawn by the
engine's grid skins (`shuyi_ui::grid`), so a new look is likely an
engine change (`engine-change` skill; ask Nick before an engine ticket or
PR). It should match the threat arrows' look 0908 settles, hence
`blocked_by`.

**From 0908 (done):** the threat arrows are arcs by default (solid, still,
no glow or blinking), with "Arcs + tile" (a 1-pixel frame in the enemy
colour inside the threatened tile) and "Straight" as options
(`look-and-feel.md`, *Threat arrows*). Nick turned down see-through and
blinking looks there for readability.

## Nick input

**Decision** (`ask-nick`, before building), on moving renders of a real
battle in both looks (`cargo shuyi frame-png --gif`, a selected unit's
ranges; Chapter 1 pictures stay local, ADR-0118): 3–4 looks drawn from
how modern tactics games show ranges (see-through tints, edges only,
a slow pulse, a sweep when the unit is selected, …), today's among them,
named after the games they come from. Nick said round 2 of 0908's
animated arrows "start to sacrifice readability": keep the ranges easy
to read at a glance.

## Scope

**In:** the variants; Nick's pick; drawing it in every map skin (glyph,
picture, Playdate) from the scene and the time alone (ADR-0038,
`shuyi-animate`); `look-and-feel.md` records it.

**Out (do not do):** the danger zone's rules or when ranges show; the
threat arrows themselves (0908); the title (0922).

## Implementation steps

1. Render the variants (throwaway switches, never committed; a local
   engine copy patched by `.cargo/engine-patch.toml` works for engine-drawn
   looks) and ask Nick.
2. If the pick needs the engine (translucency, animation of range
   tints): ask Nick, then an engine ticket and PR (`engine-change`), and
   this ticket's PR moves the pin.
3. Set the look in each map skin; update the skin tests and snapshots.
4. Record Nick's words and the pick in `look-and-feel.md`.

Expected size: about 150 production lines (a range tint drawn from the
clock in each skin), judged by 0449's edge arrows.

## Acceptance criteria

- [ ] Nick picked a look; recorded in `look-and-feel.md`.
- [ ] Every map skin draws it; the Playdate's one-bit look follows.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit (skin): the range's cells or overlays at two fixed times.
- Snapshot: a selected unit's ranges per map skin.

## Completion notes

*(Filled in by the session that completes the ticket.)*
