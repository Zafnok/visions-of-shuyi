---
id: "0923"
title: "A livelier look for the move and attack ranges, matched to the threat arrows"
type: feature
milestone: M8 Release
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: ["0908"]
nick_input: decision
completed: 2026-11-18
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

- [x] Nick picked a look; recorded in `look-and-feel.md`.
- [x] Every map skin draws it; the Playdate's one-bit look follows.
- [x] All gates in the `run-gates` skill pass (locally, on the engine
      checkout; the pin moves once the engine PRs merge).

## Tests required

- Unit (skin): the range's cells or overlays at two fixed times.
- Snapshot: a selected unit's ranges per map skin.

## Completion notes

- **Three rounds with Nick (2026-11-18)**, on moving renders of the
  Chapter 1 battle (Ellery selected), both looks:
  1. Today's tints, see-through panels (Three Houses), edges only
     (XCOM 2), spread + shimmer (Advance Wars): "spread and shimmer looks
     best but not in the first iteration so play around more with this
     as the base".
  2. Four takes on it: he liked the stronger shimmer and the glint band,
     but not a blue and a red outline side by side ("looks amateur"),
     and asked about 3D-like effects.
  3. One outline round the whole range; shimmer, glint, raised tiles,
     raised slab: "can have each of these as options I think, and maybe
     H as the default". Asked about a still choice: "Yes, add Still".

  His words and the numbers are in `look-and-feel.md`, *Range tiles*,
  and `options.md`, *Range look*.
- **The looks are the engine's** (engine ticket 3224,
  Zafnok/shuyi-engine#505): `grid::ranges::RangeShow`, built into
  the grid skins, and `MapScene::ranges_shown`, where and when the ranges
  opened. The game side: the `RangeStyle` setting and its Options row
  (PC and Playdate), `MapSkin::with_marks` (the threat arrows and the
  ranges together), a skin rebuilt when the setting changes, `GLYPH` and
  `ONE_BIT` in the Slab look by default, and the battle screen noting
  the selected unit's tile and the time it was selected.
- **Found on the way:** with idle motion off the battle's clock stands
  still, so a spread would never reach a tile: then the ranges show at
  once (the tests run so; the game and `frame-png` have it on). The
  engine's sprite skin drew its lines at a fixed width whatever the zoom
  (now a pixel at the art's size, as Nick's renders), and its ground
  with no tiles ignored the look (fixed in 3224). The new row filled
  the PC's Options panel, so the engine lets it rise to the top row
  (Zafnok/shuyi-engine#507, under this ticket); with the bought
  pictures the list was already too long for its message (follow-up
  1301).
- Left to judge by playing (Act 1 playtest, 1064): whether the glint and
  the slab's shadow read well over every terrain, and the spread's speed.
- Snapshots: the range looks (lighter tints, outline, shadow), the new
  Options row, and the Playdate's Options list one row further down.
- Engine friction: estimate 150, measured 106 added production lines
  (under 2×). PR size: about 520.
- Follow-up: 1301 (the PC's Options list scrolls when it outgrows the
  screen).

### For the Act 1 playtest (1064)

- *Claude's starting rules:* the spread starts only when a unit is
  selected (the danger zone, an area skill's reach and the move after an
  attack show at once, in the same look); on the Playdate only the
  spread shows (one bit has no see-through tints); the row sits under
  Threat arrows on the PC and the Playdate; a save from before it loads
  as Slab; the row's names are Slab, Shimmer, Glint, Raised tiles,
  Still.
