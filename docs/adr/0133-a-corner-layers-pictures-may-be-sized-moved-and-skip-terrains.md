# ADR-0133: A corner layer's pictures may be sized, moved, and skip terrains

- **Status:** Accepted
- **Date:** 2026-11-26
- **Related tickets:** 1361 (engine#3246); extends ADR-0052

## Context

0096 decided that Chapter 1's hills and peaks are one raised area with a
round stone rim (1359), a cliff face below their bottom edge and a soft
shadow under both. The rim is a corner layer (ADR-0052): one picture a
tile big, centred on each point where four tiles meet, chosen by which of
the four are in the layer. The face and the shadow don't fit that:

- the face hangs on the tile **below** a hill tile, and its ends depend on
  the tiles beside the hill tile, not beside the face's own tile;
- the shadow is the rim's and the face's shape moved a pixel right and
  three down, so it reaches past the piece it belongs to;
- Chapter 1's top-right hills stand right above the fort's wall, where no
  face may hang.

engine#3246 (Zafnok/shuyi-engine#593, at the game's pin since 96eb097c)
gave the engine's `CornerLayer` three fields: `skip` (terrains whose tiles
get no part of its pictures), `offset` (art pixels, times the zoom) and
pictures drawn at their own `ImageRect` size, the top-left still half a
tile up and left of the point. The game's tileset format had no way to
write them.

## Decision

### 1. The format: `size`, `offset` and `skip` on `Corners`

A `Corners` layer of a tileset file (`assets/tilesets/README.md`) takes
three optional fields, read by `trpg_content::tileset::look`:

- `size: (w, h)` — each picture's size in pixels, from the top-left of
  its `at` tile; left out, `tile_px`. Each side 8 to 64 px, like a tile's.
  Every picture at that size must lie inside the image.
- `offset: (x, y)` — pixels across and down every picture is moved, at
  the art's size; left out, `(0, 0)`.
- `skip: [terrains]` — terrains whose tiles get no part of a picture; each
  must be a real terrain.

They go one to one into the engine's `CornerLayer` (built with `new`, the
fields set after, as it is `#[non_exhaustive]`). A `Tiles` layer has none
of them. Left out, a layer reads and draws as before, so no tileset file
changes.

### 2. The importer: a faced `Quarters` layer and its `Shadow`

In the mapping (`assets-src/tilesets/tiny_tales.ron`):

- a `Quarters` layer takes `face: (left, middle, right)` (kit cells) and
  `skip`. With a face each picture is 16 × 24: its top the rim's quarters
  as before, and on each of the point's lower two tiles that is out of
  the layer below one that is in, the half of a face cell nearest the
  point, the whole tile down. A half is the `left` (or `right`) cell's
  where the layer ends that way beside the tile above, else `middle`'s.
  The mix of the four tiles round the point decides it, so a corner
  picture still carries it; half a tile taller is what reaches the face
  tile's bottom.
- a `Shadow(of: "<layer>", colour: (r, g, b, a), offset)` layer is the
  pictures of a faced `Quarters` layer as silhouettes in that colour,
  written with the same size and skip and its own offset. A look lists it
  where it is drawn (before the rim).

### 3. The packed image holds pictures taller than a tile

The importer packs a picture taller than a tile in a cell **and the cell
below it**, both taken: the next picture goes in the first cell (row by
row) that is free, and a tall one where the cell below is free too. A
picture's `at` is still its top-left tile. Shorter pictures fill in round
the tall ones, so the image stays as small as before plus their space.

## Consequences

- Chapter 1's hills hang the kit's face and cast the shadow, never on the
  fort's wall; every other look is unchanged (no layer of theirs uses the
  fields).
- Any later art that overhangs its point (a tree's crown, a roof's eave, a
  shadow) can be a corner layer with a size and an offset rather than a
  new layer kind.
- The face's pictures cover six tiles each instead of four, so the skin
  cuts each into more pieces; the shadow doubles that layer's sprites.
  Chapter 1's map is small; the bench (`cargo shuyi bench`) still gates
  the frame.
- The packed image's cells are no longer in packing order: the importer
  keeps each picture's place (`slots`) and every cell taken.
- A face is decided by the four tiles round its point only, so a hill on
  the map's bottom row hangs no face (the engine counts a tile off the map
  as the nearest on it).

## Alternatives considered

- **A tile layer for the face** — a tile layer sees its tile's four sides,
  not the diagonals the face's ends depend on, and draws only on its own
  tile, so it can't reach the shadow's extra pixels.
- **The size in tiles down** — simpler packing, but a 16 × 32 picture
  would carry eight empty rows and cut into more pieces for nothing;
  pixels say exactly what the art is.
- **Tall pictures in a band of their own after the grid** — every
  picture's place would be known only once all were packed (moving
  frames are packed last), so every `at` would need rewriting; taking the
  cell below as it packs needs no second pass.
- **A shadow drawn by the skin** (a darkened copy under each rim picture)
  — the look is the tileset's (ADR-0038): a silhouette picture keeps it
  data, and its colour and offset in the mapping.
