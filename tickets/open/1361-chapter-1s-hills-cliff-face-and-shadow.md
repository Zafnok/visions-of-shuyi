---
id: "1361"
title: "Chapter 1's hills: the cliff face below and the soft shadow"
type: content
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["engine#3246"]
blocks: ["1064"]
nick_input: none
completed:
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

- [ ] Chapter 1's hills and peaks hang the kit's cliff face on the tile
      below their bottom edge, rounded at its ends, never on a wall, with
      the soft shadow under rim and face.
- [ ] Every other map looks as before.
- [ ] Before/after pictures in the PR.
- [ ] The tileset import's tests pass; `assets-private.rev` moved.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- The importer: the face's ends, every mix's taller piece, the shadow's
  silhouette.
- `trpg_content::tileset`: the new fields' checks.

## Lesser targets

The Playdate map uses the glyph look (0061) unless 0067 changes that.
GBA: no target.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
