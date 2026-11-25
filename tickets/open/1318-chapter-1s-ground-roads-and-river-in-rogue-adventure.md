---
id: "1318"
title: "Chapter 1's ground, roads and river in Rogue Adventure"
type: content
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: high
status: in-progress
blocked_by: ["0096"]
blocks: ["1064"]
nick_input: answer-first
completed:
---

# 1318 — Chapter 1's ground, roads and river in Rogue Adventure

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

**Split on 2026-11-25** (the overhaul is more than one PR, ADR-0101):
this ticket keeps its number for the first piece, the ground, roads,
river and bridges, and the pieces after it are **1359** (hills and peaks)
and **1360** (Harl's fort, Harrowby's tower, the lamps). The trees are
1356's.

## 0096's answer (2026-11-24)

The direction is **a new tileset, not more of Tiny Tales'**:
`docs/design/look-and-feel.md`, *Which tileset* and *Chapter 1's map*.
Chapter 1 is drawn with **ELV Games' Rogue Adventure** (Tiny Tales stays
for the characters); which set the other chapters use is not decided yet.
What this piece draws, as the mockups drew it:

- Ground, forest floor and water joined at their corners (a dual grid:
  each grid corner takes a piece by the four tiles round it), from
  `RA_Ground_Tiles.png` and `RA_Animated_Water.png`; roads as whole dirt
  tiles that never taper; plank bridges.
- **2026-11-24, from 1315:** a tileset's cells can loop through frames
  (`frames`, `assets/tilesets/README.md`), drawn by the sprite skin at
  the scene's clock and still on the Playdate. The river's 8 frames of
  `RA_Animated_Water.png` go in with this ticket's water: give each water
  cell its `frames` and `motion` (`Minimal` with a `todo`, as `cargo
  xtask tileset-import` writes them).

**Where the mockups' tiles are.** 0096's mockup renderer (a throwaway
Python script, never committed) drew them from these cells of the bought
sheets, in pixels from the top-left (16-pixel tiles):

| What | Sheet | Where |
| ---- | ----- | ----- |
| Grass, every tile | `RA_Ground_Tiles.png` | (32, 80) |
| Forest floor, a darker grass joined at corners | `RA_Ground_Tiles.png` | a 3×3 patch at (144, 64), its inside corners in the 3×3 at (144, 16) |
| Road, whole tiles | `RA_Ground_Tiles.png` | (32, 208) |
| River, joined at corners, 8 frames 64 pixels apart, 160 ms each | `RA_Animated_Water.png` | a 3×3 patch at (64·f, 192), inside corners at (64·f, 144) |
| Plank bridge | `RA_Jungle.png` | tile (30, 39) |

A 3×3 patch is the usual layout: its eight outer tiles are the edge and
the outside corners, its middle the full tile; the inside corners are
the four corner tiles of the second 3×3; a point with two diagonal tiles
in the layer draws two outside-corner pieces.

## Nick input

**Answer first:** 0096 (answered). Nothing to check after merge:
`frame-png` pictures of Chapter 1 before and after in the PR (cropped:
ADR-0118, no whole real map); what's left to judge in play goes on 1064's
checklist.

## Scope

**In:**
- The importer reads a **plain PNG sheet** of another bought library
  (Rogue Adventure has no Tiled files): a tile by its column and row,
  and a corner layer from a 3×3 patch and its inside corners (above),
  each frame of a moving one a step further along the sheet.
- A **map look for Rogue Adventure** that a map file names, so
  Chapter 1 is drawn with it and every other map keeps Tiny Tales; its
  grass, forest floor, roads, river (moving) and plank bridges.
- Until 1359, 1360 and 1356 land, the terrains this piece doesn't draw
  keep Tiny Tales' pictures in the new look (hills, peaks, the fort's
  walls, the trees), over Rogue Adventure's ground.
- Chapter 1's map names the look (the private repository, the
  `content-change` skill); the art change pushed to the private
  repository and `assets-private.rev` moved (`cargo shuyi private-assets
  --pin`).
- The ELV Games credit (`assets/data/credits.ron`), covering the packed
  image as Tiny Tales' does; `THIRD_PARTY_ASSETS.md` says what the game
  now uses.

**Out (do not do):** hills and peaks (1359); the fort, Harrowby's
tower, lamps and rocks (1360); trees (1356); effects (1316); map layout
changes; new terrain rules; other chapters' maps (their set isn't
decided).

## Implementation steps

1. Extend `crates/tools/src/tileset_import.rs`: a tile may come from a
   PNG sheet of a library other than Tiny Tales' (by its path under
   `assets-private/library/`), and a corner layer may be built from a
   patch layout with frames, writing the same tileset rows (`Corners`,
   `frames`) the game already reads.
2. Add the look's name to `trpg_content::map::TILE_LOOKS` and its rows to
   `assets-src/tilesets/tiny_tales.ron`; re-pack.
3. Give `ch01.map` the look (private content PR first).
4. `frame-png` Chapter 1 before and after.

## Acceptance criteria

- [ ] In the new look, plain, forest, road, water and bridge each have
      their own Rogue Adventure picture; the water moves through 8 frames.
- [ ] Every other map looks as before.
- [ ] Before/after pictures in the PR.
- [ ] The tileset import's tests pass; `assets-private.rev` moved.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- The importer's existing tests, and tests of the plain-sheet tile and
  the patch layer (every mix's picture, each frame's step).
- A test that every battle terrain has a tile row in every look (the
  tileset checks already require it; a test that names the new look).

## Lesser targets

The Playdate map uses the glyph look (0061) unless 0067 changes that;
the river's frames have their stills (`FirstFrame`) for `Reduced` and
`Minimal`. GBA: no target.

## Completion notes

