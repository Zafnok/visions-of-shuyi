# ADR-0134: Trees stand by footprint on the whole map and take the ground's tint

- **Status:** Accepted
- **Date:** 2026-11-26
- **Related tickets:** 1356; extends ADR-0052 and ADR-0131

## Context

0096 decided Chapter 1's woods (`docs/design/look-and-feel.md`, *Woods:
trees by footprint*): a big swaying tree on each 3 × 3 of forest that is
free, its trunk on the bottom row's middle tile and its crown over the
rest; half-size trees (the same frames halved) on the forest left; each
tree with its own phase; motion drawn by the artist, never by the skin.

None of the tileset's pieces fits. A terrain's tile, a corner picture and
a tile picture each depend on one tile or the four round a point; a
moving cell (ADR-0131) is one cell. A tree is a picture three tiles wide
standing where the whole forest's shape puts it. Two more things:

- The map skin gets a **scene**: the visible tiles and one ring round
  them. A packing of 3 × 3s worked out on the view would change as it
  scrolls, the trees jumping.
- The kit's sprite skin (`shuyi_ui::grid::sprite`) paints the ground's
  pictures, then at once its tint wash (each tile's own picture as a
  silhouette in the range's colour) and the ranges' outlines, then the
  unit painter. A tree painted after the ground covers the tint of the
  tiles its crown reaches over, hiding the ranges there.

## Decision

### 1. The format: a look's `trees`

A look of a tileset's `looks` may have `trees` (`assets/tilesets/
README.md`): the terrains they stand on (`of`), kinds of `big` and of
`half` tree (each a frame's size, its foot, and its frames as top-left
cells and times) and a `motion` with its stills (a tree's still is its
first frame). `trpg_content::tileset::Trees` holds them, by the look's
name in `Tileset::trees`. `moving_rows` hands their motion to the stills
check with the moving cells'.

The importer reads a look's `trees` in the mapping (plain sheets of
frames, a frame's size and foot, the count and the time) and packs every
frame below the tiles, then each again halved, as the half-size kind. A
frame keeps its own size: the packed image places it on a run of cells.

### 2. Where they stand: a function of the whole map

`trpg_content::tileset::place(width, height, forest)` is pure: big trees
row by row from the top-left wherever a 3 × 3 is free; then from the
bottom row up a half-size tree on each forest tile left, taking the tile
above as its crown if that is forest left. A tree's kind and its phase
come from its place by a fixed mix of its coordinates, never a random
number.

The screens that show part of a map hand the map skin the **whole map's
terrain** beside the scene: `MapSkin::paint_on(ctx, scene, map, …)`. The
battle and rewind views and a talk's place carry it (`map`,
`place_map`). `MapSkin::paint` keeps its form and places trees on the
scene's own tiles, right for a scene that is the whole map (the
cinematics' shots) and for tests.

### 3. Draw order: over the ground, under the rest, tinted as the ground

The trees are painted by the sprite unit painter's `paint_under`, before
anything of its own: over the ground, under the path, the units, their
marks and the cursor, the top row first so a lower tree stands in front.

A tree takes the ground's tint over its own pixels: the map skin notes
where the ground's items begin in the buffer, and for each tile a tree
covers, the tree's sprite is painted again clipped to that tile in the
wash's solid colour and strength, read from what the ground painted. The
ranges' outlines on tiles under a tree are painted again after the
trees. So a range under a crown looks as on bare ground.

This reads the kit's output rather than asking the kit for a step
between the ground's pictures and its tints, which is an engine change
(the `engine-change` skill, with Nick's OK). It is marked
`engine-friction: 1356` in `crates/ui/src/map_view/sprite/trees.rs`.

## Consequences

- A burnt forest loses its trees the frame it burns, and the packing
  round it may shift: the trees follow the terrain as it is.
- Placing every frame costs a pass over the map's tiles (a few hundred);
  `cargo shuyi bench`'s biggest battle stays within budget.
- If the kit gains a step between the ground's pictures and its tint
  (or props of its own), the rewash goes and the trees paint there.
- Other chapters' looks may name trees of their own: only data.
