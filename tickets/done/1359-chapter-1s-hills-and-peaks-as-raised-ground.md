---
id: "1359"
title: "Chapter 1's hills and peaks as raised ground with round edges"
type: content
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: ["1318"]
blocks: ["1064"]
nick_input: none
completed: 2026-11-25
---

# 1359 — Chapter 1's hills and peaks as raised ground with round edges

## Context

Split from 1318 (2026-11-25), the tileset overhaul 0096 decided
(`docs/design/look-and-feel.md`, *Chapter 1's map*). 1318 gave Chapter 1
a map look of its own, drawn with ELV Games' *Rogue Adventure*: its
grass, forest floor, roads, river and bridges, and an importer that reads
plain PNG sheets and builds a corner layer from a 3×3 patch
(`crates/tools/src/tileset_import.rs`, `assets-src/tilesets/tiny_tales.ron`).
Its hills and peaks still show Tiny Tales' pictures over the new ground.

0096's answer for them:

- **Hills are raised ground with rounded edges:** a rim of stones round
  the hill's top, a rock face below its bottom edge, a soft shadow, and
  the hill's top is the plain's own grass. **Every corner, outside and
  inside, is drawn round** (Nick rejected square inside corners through
  four rounds: "make it rounded, smooth").
- The art is Electric Lemon's *Lonesome Forest* cliff kit
  (`assets-private/library/electric-lemon/tilesets/lonesome-forest/Summer
  Tileset/Lonesome_Forest_WALLS_and_CLIFF_EDGES.png`, 112×160), its four
  stone colours recoloured to a **sandy brown** ("two shades lighter"
  than the first brown tried). **Recolouring is a change: the credit says
  it was modified** (THIRD_PARTY_ASSETS.md; the `work-ticket` skill's
  credits rule).
- **Peaks** are Rogue Adventure's dark rock mass, inside the same raised
  ground as the hills next to them: hills and peaks are one raised area,
  one rim round both.

**How the mockups drew it** (0096's throwaway renderer; tile cells of
the kit as (column, row), quarters 8 pixels):

- The kit's colours `#000000`, `#1d2b53`, `#5f574f`, `#83769c` become
  `(64, 51, 37)`, `(106, 84, 61)`, `(150, 118, 86)`, `(184, 148, 110)`.
- Each quarter of a hill (or peak) tile by its two neighbours that way
  and the diagonal between them, RPG Maker style: neither neighbour in
  the raised area → the **outer corner** (cells (0,3), (3,3), (0,6),
  (3,6) for top-left, top-right, bottom-left, bottom-right); only the
  one across → the **top or bottom edge** ((1,3) top, (1,6) bottom, with
  its shade); only the one up or down → the **left or right edge**
  ((0,4), (3,4)); both but not the diagonal → the **inside corner**, the
  kit's lobes ((1,0) above, (1,2) below); all three → nothing (the
  grass shows). The quarter taken is the same quarter of that cell.
- **The cliff face** (the kit's row 8) hangs on the tile below a hill
  tile whose tile below isn't raised (and isn't a wall): its left half
  is cell (0,8)'s left half where the hill ends to the left, else cell
  (2,8)'s; its right half cell (3,8)'s right half where it ends to the
  right, else cell (2,8)'s.
- **The shadow:** the hill's pieces' shape in `(12, 18, 8)` at alpha 70,
  one pixel right and three down, under the pieces.
- **Peaks:** `RA_Overworld_Ground.png`, a 3×3 patch at pixel
  (336, 272), its inside corners at (336, 224), joined at corners as
  1318's ground is, drawn on the ground under the rim.

**The format may not hold this yet.** A quarter depends on the four
tiles round its corner of the grid, so the rim fits the tileset's corner
layer (each piece made of four quarters). The face and the shadow reach
the tile below and depend on the tiles beside the hill (diagonals from
the face's tile), which neither a corner layer nor a tile layer's four
sides sees. Decide the smallest addition (a tile layer that looks at all
eight neighbours, or a corner piece hung lower than one tile), with an
ADR if it changes the format's shape; check 1219 (the engine's tileset
format) first: if it has landed, the addition is the engine's
(`engine-change`).

## Nick input

None: 0096 decided it. Nothing to check after merge: `frame-png`
pictures of Chapter 1's hills before and after in the PR (cropped,
ADR-0118), with an inside corner zoomed; what's left to judge in play
goes on 1064's checklist.

## Scope

**In:**
- The kit read from Electric Lemon's library, recoloured by the importer
  (never a recoloured copy committed or hand-edited), its pieces built
  from quarters as above, packed into the tileset image.
- Chapter 1's look: hills and peaks as one raised area with the rim, the
  face below and the shadow; peaks' rock mass.
- The Electric Lemon credit (`assets/data/credits.ron`) marked as
  modified; `THIRD_PARTY_ASSETS.md` says what the game uses.
- The art change pushed to the private repository and
  `assets-private.rev` moved.

**Out (do not do):** the fort and its walls (1360: drawn over hills and
peaks, so either order works; if 1360 lands first, keep its layers on
top); trees (1356); map layout changes; new terrain rules; other
chapters' maps.

## Implementation steps

1. Read the kit and recolour it in `crates/tools/src/tileset_import.rs`
   (a colour table in the mapping file, `assets-src/tilesets/tiny_tales.ron`).
2. Build the rim's corner pieces from quarters; test every mix of four
   on a plus-shaped and an L-shaped cluster (the inside corners).
3. Add the face and the shadow (the format addition above, its checks
   in `trpg_content::tileset`, its README in `assets/tilesets/`).
4. Peaks' corner layer; re-pack; `frame-png` before and after.

## Acceptance criteria

- [x] Chapter 1's hills and peaks are drawn as one raised area with a
      round rim at every outside and inside corner. **The face below and
      the shadow are not**: they need an engine change, split to **1361**
      (see the notes).
- [x] Every other map looks as before.
- [x] Before/after pictures in the PR, with an inside corner zoomed.
- [x] The tileset import's tests pass; `assets-private.rev` moved.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- The importer: the recolour, the quarter choice for every mix, the
  face's ends.
- `trpg_content::tileset`: the format addition's checks; the sprite
  skin's test of its look (with the skin, ADR-0038).

## Lesser targets

The Playdate map uses the glyph look (0061) unless 0067 changes that.
Nothing here moves. GBA: no target.

## Completion notes

**Done in part (2026-11-25); the face and the shadow are 1361.**

- **Split, with Nick's OK** (2026-11-25): the face hangs on the tile
  below a hill and depends on the tiles beside the hill; the shadow
  reaches a pixel past its piece; and Chapter 1's top-right hills and
  peaks stand right above the fort's wall, where no face may hang. The
  engine's corner layer draws exactly one tile round its point and on
  every tile, and the tile layer sees four sides only, so neither can
  draw them; and the layers are the engine's (`shuyi_assets::tileset`,
  `shuyi_ui::grid::sprite::ground`), with 1219 not landed. Nick agreed to
  an engine ticket and PR, **engine#3246** (a corner layer's pictures
  of any size, off their point, skipping some terrains); **1361** adds
  the face and the shadow once the pin moves past it. No format change
  here, so no ADR (1361 writes it).
- **The rim** (`crates/tools/src/tileset_import.rs`): a `Quarters` layer
  in the mapping builds each corner picture from four quarters of a
  kit's cells, RPG Maker style, as 0096's mockups did per tile: an
  outside corner, an edge along the top or bottom, an edge down a side,
  an inside corner (the kit's lobes) or nothing, by the tile's two
  neighbours that way and the diagonal. Every corner, outside and
  inside, is round; mix 15 draws nothing (the plain's grass is the
  hill's top). The tileset file is unchanged in shape: ordinary
  `Corners` rows.
- **The recolour:** the mapping's `recolour` changes a sheet's colours
  as the importer reads it (Lonesome Forest's four PICO-8 stones to
  0096's sandy brown); no recoloured copy exists anywhere.
- **The peaks:** Rogue Adventure's dark rock (`RA_Overworld_Ground.png`,
  its patch and inside corners, joined at corners as 1318's ground),
  under the rim; hills and peaks are one raised area, one rim round
  both. Tiny Tales' hills and peaks stay in the other looks.
- **Credits:** Electric Lemon's *Lonesome Forest* in `credits.ron`,
  titled as modified ("stones recolored"); `THIRD_PARTY_ASSETS.md` has
  its row (modified) and the ELV row names the overworld sheet.
- **Tests:** the importer's (every mix's quarters checked against the
  mockups' per-tile rule on a plus, an L, a ring, an X and a blob, all
  16 mixes; the quarter placement; the recolour; a whole `Quarters`
  layer packed, recoloured, listed and checked); the private test that
  Chapter 1's look has the rim round hills and peaks over the peaks'
  rock and no Tiny Tales hills.
- **Checked:** Chapter 1 rendered before and after (crops and an inside
  corner zoomed in the PR); the other 13 battle maps are pixel-identical
  with the old and the new tileset.
- **Edge of the map:** a hill on the map's bottom row carries on past
  it (the engine counts a tile off the map as the nearest one on it),
  where the mockups drew its bottom edge there; Chapter 1's hill at the
  bottom shows it.

**No gameplay rule decided.** Left to judge in play (1064): the hills
without their face and shadow until 1361; the rim's sandy brown on the
real map.
