---
id: "1318"
title: "Chapter 1's ground, roads and river in Rogue Adventure"
type: content
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: ["0096"]
blocks: ["1064"]
nick_input: answer-first
completed: 2026-11-25
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

- [x] In the new look, plain, forest, road, water and bridge each have
      their own Rogue Adventure picture; the water moves through 8 frames.
- [x] Every other map looks as before.
- [x] Before/after pictures in the PR.
- [x] The tileset import's tests pass; `assets-private.rev` moved.
- [x] All gates in the `run-gates` skill pass.

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


**Done (2026-11-25).**

- **Split first** (ADR-0101): this ticket kept the ground and river;
  **1359** (hills and peaks, Lonesome Forest's kit) and **1360** (Harl's
  stone keep, Harrowby's tower, the lamps) were filed with the tile
  places 0096's mockups used. 1356 (trees) still waits on this one.
- **The importer reads plain PNG sheets**
  (`crates/tools/src/tileset_import.rs`): a sheet named `….png` is read
  from `assets-private/library/` (Tiny Tales' Tiled sheets stay under
  `tiny-tales/tilesets/`), its tiles by number; a **`Patch`** layer
  builds the 15 corner mixes from the usual 3×3 patch and its 3×3 of
  inside corners (the two diagonal mixes are two corner tiles packed as
  one picture, `Cut::Pair`); the mapping's **`moving`** makes every
  packed tile of a sheet loop along its row (`Strip`: frames, step, ms).
  The tileset file's format is unchanged: the rows are ordinary
  `Corners`, `Tiles` and `frames`.
- **A map look `rogue_adventure`** (`trpg_content::map::TILE_LOOKS`),
  named by Chapter 1's map (private content); every other map keeps
  Tiny Tales. In it: Rogue Adventure's grass (plain and every grass-based
  terrain), the darker forest floor joined at corners, roads as whole
  dirt tiles, the river joined at its banks, moving through its 8 frames
  at 160 ms (as the mockups ran), plank bridges over it. Hills, peaks,
  the fort's walls and floor and the trees keep Tiny Tales' pictures over
  this ground until 1359, 1360 and 1356.
- **Credits:** ELV Games' *Rogue Adventure World* in
  `assets/data/credits.ron` (covering the packed image, beside Tiny
  Tales) and a row in `THIRD_PARTY_ASSETS.md`. Nothing recoloured, so
  not marked as modified.
- **Tests:** the importer's (plain sheets, pairs, every patch mix, the
  strip's frames, the mapping's checks, the listing); the private
  tests: every look a map can name has every terrain (now with the new
  look), Chapter 1's map uses it with its own plain, forest, road and
  water tiles and a moving river that stands still on a lesser tier;
  the road test accepts a look whose roads are whole tiles.
- **Checked:** Chapter 1 rendered before and after (crops in the PR);
  Chapters 2–5 and the three Act 1 skirmishes are pixel-identical at
  their first frame with the new tileset.

**Deviation / finding:** re-packing also makes **Tiny Tales' coast foam
move** (the Standard coast's `.tsx` has real 3-frame loops of 150 ms; 1315
said Tiny Tales' coasts were one picture, but the tileset hadn't been
re-packed since its importer learned frames). Their first frame is the
old picture, so the maps' stills are unchanged; maps with a sea shore
now move there. That matches 0096 (shore foam should move), so it is
kept; `assets/tilesets/README.md` says so.

**No gameplay rule decided.** Left to judge in play (1064's checklist):
the river's speed (160 ms a frame) and the Tiny Tales stand-ins (trees,
hills, fort) over Rogue Adventure's ground until their tickets land.
