---
id: "1361"
title: "Chapter 1's hills: the cliff face below and the soft shadow"
type: content
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: ["engine#3246"]
blocks: ["1064"]
nick_input: none
completed: 2026-11-26
---

# 1361 — Chapter 1's hills: the cliff face below and the soft shadow

## Context

Split from 1359 (2026-11-25), the tileset overhaul 0096 decided
(`docs/design/look-and-feel.md`, *Chapter 1's map*). 1359 drew Chapter
1's hills and peaks as one raised area with Electric Lemon's *Lonesome
Forest* rim round it, recoloured to a sandy brown, every corner round
(a `Quarters` layer of `assets-src/tilesets/tiny_tales.ron`, built by
`crates/tools/src/tileset_import.rs`), and the peaks' rock inside it.
What 0096's mockups also drew, and this ticket adds:

- **The cliff face** (the kit's row 8) hangs on the tile below a hill
  (or peak) tile whose tile below isn't raised and isn't a wall: its
  left half is cell (0,8)'s left half where the hill ends to the left
  (the tile left of the hill tile isn't raised), else cell (2,8)'s; its
  right half cell (3,8)'s right half where it ends to the right, else
  cell (2,8)'s. Cells are (column, row) of
  `Lonesome_Forest_WALLS_and_CLIFF_EDGES.png`, 16 px.
- **The shadow:** the shape of the rim and the face in `(12, 18, 8)` at
  alpha 70, one pixel right and three down, under them.

The face reaches the tile below the hill and depends on the tiles beside
the hill (diagonals of the face's tile), so neither a corner picture of
one tile nor a tile layer's four sides can draw it; the shadow reaches a
pixel past its piece. Nick agreed to an engine change (2026-11-25):
**engine#3246** lets a corner layer's pictures be another size than a
tile and sit off their point (`offset`), and leave out the tiles of some
terrains (`skip`). With it, each corner picture of the face layer is a
tile wide and taller than a tile, its top the rim's quarters of the four
tiles round the point and its lower part the face of the tile below the
point's lower two tiles; the shadow is a second corner layer of the same
pieces' silhouettes at `offset: (1, 3)`, before the rim; both `skip`
walls (Chapter 1's top-right hills and peaks stand on the fort's wall).

engine#3246 (PR Zafnok/shuyi-engine#593) adds to
`shuyi_assets::tileset::CornerLayer` the fields `skip:
BTreeSet<TerrainId>` and `offset: (i32, i32)` (art pixels, times the
zoom), draws each picture at its own `ImageRect` size, and makes the
struct `#[non_exhaustive]` with `CornerLayer::new(of, tiles)`.
**2026-11-26:** the game's pin is at its merge (`96eb097c`), moved in a
PR of its own; the game builds a `CornerLayer` with `new` and sets the
fields after.

## Nick input

None: 0096 decided it. Nothing to check after merge: `frame-png`
pictures of Chapter 1's hills before and after in the PR (cropped,
ADR-0118); what's left to judge in play goes on 1064's checklist.

## Scope

**In:**
- The tileset format (`trpg_content::tileset`, `look.rs`'s `LayerFile`):
  a corner layer's picture size (in tiles down, or pixels), its `offset`
  and its `skip` terrains, checked (size inside the image, terrains
  real); `assets/tilesets/README.md`; an ADR for the format's new fields
  (`write-adr`).
- The importer: the face's pieces and their shadow silhouettes for each
  mix, from the kit's row 8 (its ends as above), packed taller than a
  tile; the mapping's rows for them in the look `rogue_adventure`.
- Re-pack; the art pushed to the private repository and
  `assets-private.rev` moved.

**Out (do not do):** the rim and the peaks (1359, done); the fort and
its walls (1360: keep its layers on top); trees (1356); map layout
changes; other chapters' maps.

## Implementation steps

1. The format's new fields in `look.rs`/`tileset.rs`, their checks and
   tests; README; ADR.
2. The importer: the face layer's taller pieces (the rim's quarters
   over the face of the tile below where the mix says) and the shadow
   pieces; test the face's ends and the shadow's shape.
3. Mapping rows, re-pack, `frame-png` before and after.

## Acceptance criteria

- [x] Chapter 1's hills and peaks hang the kit's cliff face on the tile
      below their bottom edge, rounded at its ends, never on a wall, with
      the soft shadow under rim and face.
- [x] Every other map looks as before.
- [x] Before/after pictures in the PR.
- [x] The tileset import's tests pass; `assets-private.rev` moved.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- The importer: the face's ends, every mix's taller piece, the shadow's
  silhouette.
- `trpg_content::tileset`: the new fields' checks.

## Lesser targets

The Playdate map uses the glyph look (0061) unless 0067 changes that.
GBA: no target.

## Completion notes

**Done (2026-11-26).**

- **The format** (`trpg_content::tileset::look`, ADR-0133): a `Corners`
  layer takes `size: (w, h)` (pixels from its `at` tile's top-left;
  sides 8 to 64, each picture inside the image), `offset: (x, y)` and
  `skip: [terrains]` (real ones), set on engine#3246's `CornerLayer`
  after `new`. Left out, a layer is as before; no other tileset changed.
  `assets/tilesets/README.md` documents them.
- **The importer** (`crates/tools/src/tileset_import.rs`): a `Quarters`
  layer takes `face: (left, middle, right)` and `skip`. With a face each
  picture is 16 × 24: the rim's quarters on top and, on each of the
  point's lower two tiles that is out below one that is in, the half of
  the face cell nearest the point, the whole tile down (`Face::under`:
  the end cell where the layer ends that way beside the tile above, else
  the middle). A `Shadow(of, colour, offset)` layer packs the same pieces
  as silhouettes in one RGBA colour, with the layer's size and skip and
  its own offset. A picture taller than a tile takes its cell and the one
  below in the packed image (`Packer::free`); the others fill in round
  them, so other looks' cells moved but their pictures are the same.
- **The mapping:** `lf_rim` hangs the kit's row 8 (`(0, 8)`, `(2, 8)`,
  `(3, 8)`: cells 56, 58, 59) and skips walls; `lf_shadow` is
  `(12, 18, 8)` at alpha 70, `(1, 3)`, drawn after the peaks' rock and
  before the rim in `rogue_adventure`.
- **Deviation:** the ticket's sketch read as "the face of the tile below
  the point's lower two tiles"; a corner picture knows only its four
  tiles, so the face hangs on the point's lower two tiles themselves
  (below the upper two), and the picture is 24 px tall (16 of rim, 8
  more for the face tile's lower half) rather than two tiles.
- **Art:** re-packed, pushed as Zafnok/visions-of-shuyi-assets (branch
  `t1361-hills-cliff-face-shadow`); `assets-private.rev` points at its
  commit until that pull request merges, then at `main` (`--pin`).
- **Checked:** every battle map rendered with the old and the new
  tileset: the other 13 are pixel-identical, Chapter 1 differs only
  round its hills and peaks. No face on the fort's wall below the
  top-right hills; where a wood is below a hill, its trees are drawn over
  the face.
- **Tests:** the face's ends on every mix and on five shapes, against the
  per-tile rule; the faced picture (rim, halves, rows) and the shadow's
  silhouette; a faced layer and its shadow packed with no picture over
  another's lower cell, written and read back by the game at 16 × 24,
  moved and skipping walls; the mapping's checks (a shadow of a layer
  without a face, an unknown skipped terrain); the format's new fields
  and their problems; the private test of Chapter 1's look.
- **Friction:** estimate about 250 production lines (by 1359's `Quarters`
  layer, 118 lines, plus a format change and packing), measured 310
  (1.2×). The estimate wasn't written down before coding.

**No gameplay rule decided.** Left to judge in play (1064): the face's
height and the shadow's strength on the real map; a hill on the map's
bottom row hangs no face (the engine counts the tile off the map as the
hill's own).
