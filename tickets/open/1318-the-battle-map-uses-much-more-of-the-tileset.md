---
id: "1318"
title: "The battle map's tileset overhaul"
type: content
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["0096"]
blocks: ["1064"]
nick_input: answer-first
completed:
---

# 1318 — The battle map's tileset overhaul

## Context

Nick (2026-11-22): "I want a tileset overhaul in general. I think Tiny
Tales is ok for character sprites but the tilesets are lacking or we're
not making full use of it." 0096 decides the direction.

Today `assets-src/tilesets/tiny_tales.ron` (private art, packed by
`cargo xtask tileset-import`, 0437, ADR-0052) uses a few of the bundle's
sets, and many terrains sit on one shared grass tile (forest, thicket,
mountain, peak, fort, burning, burnt). The bundle's index is
`assets-private/library/tiny-tales/INDEX.md` (`cargo shuyi
private-assets --library`). Related open tickets: 0471 (tiles for poor
houses, a throne, a gate), 0491 (Chapter 4's doorways), 1219 (tileset
looks on the engine's format, after engine#2243; either order: if 1219
lands first, write the new rows in its format), 0067 (bought tiles on the
Playdate's map).

## 0096's answer (2026-11-24)

The direction is **a new tileset, not more of Tiny Tales'**:
`docs/design/look-and-feel.md`, *Which tileset* and *Chapter 1's map*.
Chapter 1 is drawn with **ELV Games' Rogue Adventure** (Tiny Tales stays
for the characters); which set the other chapters use is not decided yet.
What Chapter 1 needs, as the mockups drew it:

- Ground, forest floor and water joined at their corners (a dual grid:
  each grid corner takes a piece by the four tiles round it), from
  `RA_Ground_Tiles.png` and `RA_Animated_Water.png`; roads as whole dirt
  tiles that never taper; plank bridges.
- **Hills:** Electric Lemon's *Lonesome Forest* cliff kit
  (`Lonesome_Forest_WALLS_and_CLIFF_EDGES.png`), its four stone colours
  recoloured to a sandy brown, built from quarter tiles (outer corner,
  edge, inside corner, empty) so every corner is round, with the kit's
  cliff face (row 8) hung below the bottom edge and a soft shadow; peaks
  in Rogue Adventure's dark rock mass inside the same raised ground.
  Recolouring is a change: credit it as modified.
- **Harl's fort:** the stone keep from `RA_Ruins.png` (wall tops, round
  pillar caps, the tall face with arrow slits under every wall seen from
  the south); drawn last, over hills and peaks.
- Harrowby: the ruins' round pillar as a small tower. Few rocks.
- The moving parts (water, trees) are 1315's; this ticket can draw their
  frame 0 first.

This is more than one PR: split it with `write-ticket` before coding
(a likely cut: ground and water; hills; fort and details).

## Nick input

**Answer first:** 0096. Nothing to check after merge: `frame-png`
pictures of every Act 1 map in the PR; what's left to judge in play goes
on 1064's checklist.

## Scope

**In:** per 0096 — new or distinct ground tiles per terrain, more
detail tiles (edges, transitions, decorations), any new pack's credits
row in `THIRD_PARTY_ASSETS.md`; the art change pushed to the private
repository and `assets-private.rev` moved (`cargo shuyi private-assets
--pin`).

**Out (do not do):** moving tiles (1315); effects (1316); map layout
changes; new terrain rules.

## Implementation steps

1. Pick tiles per 0096 from the library; edit `tiny_tales.ron`; re-pack.
2. `frame-png` every Act 1 map before and after (cropped, per the memory
   on staging fights where a real map can't be committed).
3. If a split is needed to stay under ~2,000 lines, split by set (ground,
   then details) before coding.

## Acceptance criteria

- [ ] Every terrain 0096 named has its own ground tile.
- [ ] Before/after pictures in the PR.
- [ ] The tileset import's tests pass; `assets-private.rev` moved.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- The importer's existing tests; a test that every battle terrain has a
  tile row.

## Lesser targets

The Playdate map uses the glyph look (0061) unless 0067 changes that;
no Playdate work here. GBA: no target.

## Completion notes

