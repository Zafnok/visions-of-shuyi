---
id: "1359"
title: "Chapter 1's hills and peaks as raised ground with round edges"
type: content
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["1318"]
blocks: ["1064"]
nick_input: none
completed:
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

- [ ] Chapter 1's hills and peaks are drawn as one raised area with a
      round rim at every outside and inside corner, a face below and a
      shadow.
- [ ] Every other map looks as before.
- [ ] Before/after pictures in the PR, with an inside corner zoomed.
- [ ] The tileset import's tests pass; `assets-private.rev` moved.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- The importer: the recolour, the quarter choice for every mix, the
  face's ends.
- `trpg_content::tileset`: the format addition's checks; the sprite
  skin's test of its look (with the skin, ADR-0038).

## Lesser targets

The Playdate map uses the glyph look (0061) unless 0067 changes that.
Nothing here moves. GBA: no target.

## Completion notes

