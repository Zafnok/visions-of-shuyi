# ADR-0052: Terrain is painted in layers of pictures drawn between tiles, chosen from the terrain grid; a map file names its look

- **Status:** Accepted
- **Date:** 2026-10-03
- **Related tickets:** 0437, 0433, 0436, 0438, 0439, 0510, 0803
- **Extends:** ADR-0038 (§3: what a tileset file names and how a sprite skin
  paints the ground; §5: a map file's look data), ADR-0049 (§7: who writes
  the game's tileset file), ADR-0040 (the packed tiles are private files)

(Numbers 0050 and 0051 are left for pull requests open on 2026-10-03.)

## Context

- Nick decided the battle map's terrain is drawn with the bought Tiny Tales
  tilesets (`docs/design/look-and-feel.md`, *Battle map: bought tiles and
  unit sprites*). 0436 put the units on the map; the ground was still
  glyphs.
- Our maps are **one terrain id per tile** (`assets/maps/*.map`), written as
  text and later by the skirmish generator (0510). Nobody lays tiles by
  hand, so the picture must follow from the terrain grid alone.
- What was bought, looked at file by file:
  - Every tile is 16×16 pixels, our tile size. Each set is Tiled files: a
    256×256 image and a `.tsx` that tags tiles with *corner terrains*
    (`<tile id="17" terrain=",,0,0"/>`: which of the tile's four corners
    are a named terrain of that set).
  - The tiles are **overlays**: a grass tile is green on a clear
    background, to be laid over the shore's sand; trees and mountains are
    laid over grass. The artist's sample maps stack up to eight layers.
  - The corner sets' edges run **through the middle of a tile**: a tile
    tagged "the two bottom corners are grass" is grass in its lower half.
    In Tiled a terrain is painted on the points where tiles meet, not on
    tiles.
- Ticket 0437 planned one picture per tile, chosen by resolving each of the
  tile's corners to the highest-priority terrain among the four tiles
  touching it. With this art that puts every join in the middle of the
  lower terrain's tile: a river one tile wide is drawn two tiles wide, a
  plain tile beside it looks half water, and a one-tile road through a
  forest disappears. In a game where the player counts tiles, the picture
  must end where the tile ends.
- 0433's tint (a rectangle under the tile's picture, drawn at reduced
  opacity) needs a tile to be one solid picture. A stack of see-through
  pictures faded one by one lets the lower ones show through the upper.
- The game reads only RON and PNG (ADR-0005). Tiled's XML must not become a
  format the game ships a reader for.
- Bought files never enter this repository (ADR-0040), and every gate runs
  without them, so the same code has to be exercised by a public fixture.

## Decision

### 1. Pictures are drawn between tiles (the corner rule)

A **corner layer** names some terrains and has up to fifteen pictures, one
for each mix of four yes-or-no places. The skin draws one picture **centred
on every point where four tiles meet**, half a tile up and left of the
tile grid, chosen by which of those four tiles are in the layer (top-left,
top-right, bottom-left, bottom-right). No picture for a mix, or all four
out: nothing.

So each of our tiles is one *corner* of four pictures, which is exactly how
the bought sets are tagged. A picture's edge through its middle lands on
the line between two tiles: a terrain's pictures end where its tiles end,
and a single tile of a terrain is a whole blob of it. Two pictures side by
side share two of their four tiles, so they always agree on the edge they
share (a property test checks it).

`crates/ui/src/map_view/corners.rs` is the rule, pure:

- `Shown::of(scene)` is the terrain drawn on every tile of the view **and of
  the ring of tiles just outside it**, so the picture at the view's edge is
  the one the next tile out gives, and nothing changes as the camera moves.
- A tile a spell would change counts as what it would become.
- A tile off the map (or one the scene doesn't say) counts as the nearest
  tile that has terrain: the ground runs on to the map's edge unchanged.
- A picture is drawn only inside the tiles of the map it covers: it is cut
  into the quarters that lie in drawn tiles (`Sprite::clip`).

### 2. A look is every terrain's own tile, then layers

In a tileset file (`assets/tilesets/README.md` has the format):

- `terrain` stays 0433's table: **each terrain's own tile**, painted first,
  one per tile, on the tile. Every terrain must have one (the validator's
  `terrain "…" has no tile` is unchanged). It is the ticket's "plain
  fallback".
- `layers` (new, optional) are painted over those, in the order written:
  - `Corners(of: […], tiles: [(corners: "##..", at: (c, r)), …])`: §1.
  - `Tiles(of: […], at: (c, r), beside: […], sides: [(sides: ".#.#", at:
    (c, r)), …])`: one picture on each tile of the layer; where `sides`
    has the mix of its four neighbours (up, right, down, left) being one
    of `beside`, that picture instead. A bridge turns by it; a fort or a
    door needs no `beside`.
- `looks` (new, optional) holds other looks by name, each a `terrain`
  table and `layers` of its own. The tileset's top-level `terrain` and
  `layers` are **its own look**: what a map is painted as unless `looks`
  has the look the map names.

A mix is written as four of `#` (in) and `.` (out).
`trpg_content::tileset::mix` is the one place that orders the places.

Content validation reports, with the file, the look and the layer's
number: a terrain that doesn't exist, a mix that isn't four of `#` and `.`
or is listed twice, a corner layer's picture for no corner, a layer with no
terrain or no picture, a rectangle outside the image, a look without a tile
for some terrain, and a look name a map can't give.

### 3. A map file names its look

A `.map` header may say `look: (tiles: "indoor")`. The names a map can give
are `trpg_content::map::TILE_LOOKS` (`"outdoor"`, the default, and
`"indoor"`); any other is an error at the map file, in every build.

The look is **look data**: `MapDef::look` beside the core `BattleMap`, never
in it, so the rules, the bots and saves don't have it (ADR-0038 §5).
`Content::map_look` finds it for a battle's map by comparing the map as its
file has it; the game flow gives it to the battle screen when it starts,
restarts or continues a battle (`BattleScreen::with_look`), and the screen
puts it in `MapScene::look`. A skin without looks ignores it. Ticket 0438
adds the light to the same struct.

A tileset that lacks the look a map names paints the map with its own. So
a new look needs a name in `TILE_LOOKS` and pictures in the tilesets that
want it, and no build can fail at run time for lack of it.

### 4. Tints go over the pictures

A tinted tile (a range, a flash after its terrain changed, the cursor's
glow) gets, over everything painted on it, **its own tile's picture in one
colour** (`Paint::Solid`) at the tints' joint strength. The colour and the
strength are the ones 0433 worked out, so the result is the glyph skin's
blend; only how it is drawn changed. A tile off the map is still a plain
rectangle. A tile a spell would change is drawn as what it would become
and takes only the cursor's glow, as on the glyph skin.

### 5. One importer reads Tiled files

`cargo xtask tileset-import [--list]`:

- reads `assets-src/tilesets/tiny_tales.ron`, **the mapping**: which bought
  tile is each of our terrains' own, and each layer as a Tiled terrain of a
  bought sheet (or single tiles), per look. It is ours and committed: names
  and numbers, no art;
- reads each sheet's `.tsx` for its image, its clear colour and its corner
  table, with a small tag reader of its own (three kinds of tag; no XML
  crate, in `xtask` or anywhere);
- copies every tile used **once** into one packed image, a recoloured copy
  where the mapping asks for a tint (each pixel in a palette colour, as
  bright as it was);
- writes `assets-private/game/tilesets/tiny_tales.png` and `tiny_tales.ron`:
  the looks, then the unit pictures of `map-sprite-import`'s table.

`map-sprite-import` now only copies the unit sheets; the tileset file has
one writer. Then commit and push in `assets-private/` and run
`cargo xtask private-assets --pin` (ADR-0040).

### 6. The public fixture

`cargo xtask test-tileset` also writes `assets/tilesets/test_auto.png` and
`.ron`: 16×16 tiles, a tile for every terrain, two corner layers (a line
round the water and one round the woods), a framed fort, a bridge whose
rails turn, an indoor look, and the test unit sheets. It is written by the
same code as the bought tileset's file (`xtask`'s `tileset_ron`). The
snapshot and property tests paint with it; the debug menu's *Map skin*
goes round it.

### 7. The game's skin

Nothing changes in how the skin is chosen (ADR-0049 §6): the tileset
`tiny_tales` when the content has it. Since that tileset now has terrain,
a build with the private assets paints the whole map with the bought art,
and every other build and every gate keeps the glyph skin.

## Consequences

- A map looks right from its terrain grid alone: shores, road edges and
  woods join without anyone laying a tile, for the test map, Chapter 1
  (0803) and generated skirmishes (0510).
- A frame has more sprite items: one per tile, then one per corner point
  where a layer has a picture, then one per tinted tile. The Quick Battle
  under the fixture is about 340 snapshot lines. Each is one textured quad
  from one image.
- `MapScene` gained `rim` (the terrain one tile outside the view) and
  `look`. Whoever builds a scene fills them (`MapScene::set_terrain`); a
  scene that doesn't still paints, with the ground running on at the edge.
- 0433's tests of tints changed with how a tint is drawn; glyph-skin
  snapshots didn't.
- A corner picture that reaches past its tiles (the bought trees do, by a
  few pixels) is drawn over the neighbouring tile. That is the art's look,
  and the ground under it is still the neighbour's own.
- What one tile can't show isn't here: pictures bigger than a tile (the
  bundle's castles and houses), animated tiles (its water), hand-placed
  decoration. Each needs its own ticket.
- 0438 (lighting) recolours the ground's sprites; they are all from the
  tileset's one image, and units are separate items. 0439 (zoom) changes
  `tile_px` on screen; the cut of a picture into quarters is worked out
  from the tile size in use.
- Two open pull requests that both regenerate `tiny_tales.ron` conflict in
  `assets-private.rev` (ADR-0040 §3): the second to merge runs the
  importer again after merging `main`.

## Alternatives considered

- **One picture per tile from priority-resolved corners (the ticket's
  plan)**: with tiles whose edges run through their middle, every join
  lands half a tile off, and a tile whose four corners all resolve to a
  neighbour's terrain has nothing of its own left.
- **Tile-aligned pictures chosen by the eight neighbours (RPG Maker's
  autotiles)**: needs, for each terrain, pieces whose edge lies along the
  tile's own edge, cut into quarters and joined. The bought coast set has
  none (its edges run through the middle of a tile), so the shore would
  have to be redrawn, which the look decision rules out.
- **The importer flattens every mix into ready-made opaque tiles**: a
  picture between tiles depends on four terrains, each of sixteen, and on
  every layer at once; the combinations can't be listed ahead, and one
  missing would be a hole on a map.
- **A rectangle item with opacity for tints**: a new kind of frame item,
  drawn by `app` and by `frame-png`, for what a sprite item already does.
- **An XML crate in `xtask` for the `.tsx` files**: a dependency to audit
  for three kinds of tag in files that only one importer reads.
- **Looks as free names checked against the tilesets**: a typo in a map
  would pass or fail depending on which tilesets a build has. A short list
  in `content` fails the same everywhere.
- **The map's id in `trpg_core::BattleMap`, to look its look up by**: it
  would put a new field in every save for something the rules never read.
- **A second tileset per look**: the skin, and with it the tile size and
  the view, could change from map to map.
