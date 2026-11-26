# Tilesets (`.ron` + `.png`)

What a **sprite map skin** paints the battle map with (ADR-0038, ADR-0049,
ADR-0052): a tile for every terrain, the layers painted over them, and a
picture for units, by the game's ids (terrain, classes, characters). The rules never read a tileset; only the
look hangs off these ids. Loaded and checked by `trpg_content::tileset`
into `Content::tilesets`, by id.

The game starts with the tileset called **`tiny_tales`** when it has it.
That one is bought art: it exists only in the private assets
(`assets-private/game/tilesets/`, ADR-0040), so a build without them
starts with the glyph skin (coloured letters). The public tilesets are
generated test fixtures, reachable from the debug menu (F2 → *Map
skin*, which goes round every tileset). They are test files: they are in
`assets-test/tilesets/`, which a build that ships doesn't have (ADR-0057).

## Format

`assets/tilesets/<id>.ron`:

```ron
(
    id: "test",                      // the file's name, without .ron
    image: "tilesets/test.png",      // a PNG in the asset bundle, by its path under assets/
    tile_px: (24, 24),               // a map tile, across × down: in the image, and on screen in console pixels
    terrain: {                       // terrain id (terrain.ron) → (column, row) in tile_px steps from the image's top-left
        "plain": (0, 0), "road": (1, 0), "forest": (2, 0),
    },
    layers: [],                      // optional: pictures painted over the tiles (below)
    looks: {},                       // optional: other looks by name (below)
    frames: [],                      // optional: cells that loop through frames (below)
    unit_px: (24, 24),               // a unit picture, across × down; may be taller than a tile
    units: (
        characters: {},              // character id (characters.ron) → a picture
        classes: { "brigand": (0, 1), "rider": (1, 1) },   // class id (classes.ron) → a picture
        fallback: (7, 4),            // the picture of a unit with neither
    ),
    units_origin_px: (0, 48),        // the pixel where the unit grid's (0, 0) starts (default (0, 0))
)
```

### Terrain: own tiles, layers, looks

`terrain` gives **each terrain's own tile**: painted first, one on each
tile. `layers` are painted over them, in the order written, and choose
their pictures from the tiles around, so a map file only ever names terrain
(ADR-0052). Every picture is a `(column, row)` of the tileset's image, like
the terrain's tiles.

```ron
layers: [
    // Pictures drawn BETWEEN tiles: one centred on every point where four
    // tiles meet, for the mix of those four being one of `of`. The four
    // places of `corners` are the tiles up-left, up-right, down-left and
    // down-right of the point: `#` in the layer, `.` not. A mix with no
    // picture draws nothing.
    Corners(of: ["water", "sea"], tiles: [
        (corners: "...#", at: (1, 1)),   // only the tile down-right is water
        (corners: "..##", at: (3, 1)),   // the two below are: a shore along the top
        (corners: "####", at: (15, 1)),  // all four (leave it out if the own tile shows the same)
    ]),
    // A picture ON each tile of `of`. With `beside`, the picture for a mix
    // of its four neighbours (up, right, down, left) being one of `beside`
    // is taken from `sides` where it has one.
    Tiles(of: ["fort"], at: (0, 4)),
    Tiles(of: ["bridge"], at: (1, 4), beside: ["water", "sea"],
          sides: [(sides: ".#.#", at: (2, 4))]),   // water right and left: the other bridge
],
```

A corner layer's picture is one tile big and its middle is where the four
tiles meet, so art whose edge runs through the middle of its tile ends
exactly where the layer's tiles end (the bought Tiled sets are drawn that
way). The tile just outside the view counts, so nothing changes as the
camera moves; a tile off the map counts as the nearest tile on it; and a
picture is drawn only inside tiles of the map.

Three fields of a corner layer change that (ticket 1361, ADR-0133), for a
cliff's face that hangs below its rim and the shadow under both:

```ron
Corners(of: ["mountain", "peak"], tiles: [ … ],
        size: (16, 24),     // each picture's size in pixels, from its `at`'s top-left
                            // (left out: a tile's); its top-left stays half a tile
                            // up and left of the point, so it reaches further down
        offset: (1, 3),     // pixels across and down every picture is moved
                            // (left out: (0, 0)); at the art's size, times the zoom
        skip: ["wall"]),    // terrains whose tiles get no part of a picture
```

A picture taller than a tile covers rows of the image below its `at`:
nothing else may be packed there.

`looks` holds **other looks** by the name a map file gives in its header
(`look: (tiles: "indoor")`, `assets/maps/README.md`), each with a `terrain`
table and `layers` of its own:

```ron
looks: {
    "indoor": (terrain: { "plain": (0, 0), … }, layers: [ … ]),
},
```

The tileset's top-level `terrain` and `layers` are its own look: a map
outdoors, a map that names no look, and any look the tileset lacks.

### Moving tiles: `frames`

`frames` makes **cells of the image loop through frames** (ticket 1315,
ADR-0131): flowing water, a shore's foam. Every row that draws a moving
cell moves with it, in every look: a terrain's own tile, a corner
layer's mix, a tile layer's picture. So the rows stay as they are, and
a cell moves wherever it is drawn.

```ron
frames: [
    // The cell, then each frame: a cell and how long it shows, in ms.
    // The first frame is the cell itself: what stands still.
    (at: (3, 1), frames: [((3, 1), 600), ((0, 9), 200), ((1, 9), 400), ((2, 9), 200)],
     // How it shows on a lesser target (`trpg_content::motion`, ADR-0131):
     // a still for every tier below the lowest it moves at (`Full`
     // unless `lowest` says). A tile's still is its first frame.
     motion: (stills: {
         Reduced: (show: FirstFrame),                           // the Playdate
         Minimal: (show: FirstFrame, todo: "a one-bit tile"),   // a GBA one day
     })),
],
```

- The sprite skin draws a moving cell's frame for the scene's clock, so
  a picture at a given time (`cargo shuyi frame-png` after a wait) is
  always the same; the loop repeats for ever, and every tile of the cell
  shows the same frame.
- It draws the **first frame** below the lowest tier the cell moves at
  (the Playdate is `Reduced`) and while idle motion is off (a test's
  frame).
- The content test `every_moving_look_names_its_stills` checks every
  cell's stills and lists each `todo`.

### Units

A unit **picture** is written one of two ways, and a file may mix them:

- `(column, row)`: a cell of the tileset's own `image`, in `unit_px` steps
  from `units_origin_px`;
- `(image: "units/fighter_male.png", frame: (1, 0))`: a frame of an image
  file of its own, in `unit_px` steps from that image's top-left. Add
  `frame_px: (24, 24)` for a picture whose frames are another size than
  `unit_px` (MattWalkden's riders, ticket 0499); it counts its frames, its
  walking sheet's included, in those steps, and is drawn at that size.

**A walking sheet.** Add `walk: true` to a picture with an image of its
own, `(image: "units/fighter_male.png", frame: (1, 0), walk: true)`, to
say the image has the **3 × 4 layout**: 3 columns (walking frames: one
foot forward, standing, the other foot forward) × 4 rows (facing down,
left, right, up) of `unit_px` frames from its top-left. The bought map
sprites are like this: one 48×80 sheet per character or class, of 16×20
frames. Such a unit steps on the spot while it can still act, and turns
and moves its legs as it walks (ticket 0440); `frame` stays the standing,
front-facing frame, `(1, 0)`. A picture without `walk` (and every
`(column, row)` picture) is always drawn as it is: the unit only glides
from tile to tile.

**A tileset with no terrain tiles** leaves out `image`, `tile_px` and
`terrain` (keep `image` if a unit picture is a `(column, row)`). The
terrain, the ranges, the path and the cursor are then painted as glyphs,
on the glyph skin's 16×16 tiles, and only the units are pictures:

```ron
(
    id: "test_units",
    unit_px: (16, 20),
    units: (
        characters: { "lead_f": (image: "units/fighter_female.png", frame: (1, 0), walk: true) },
        classes: { "guard": (image: "tilesets/test_units/guard.png", frame: (1, 0), walk: true) },
        fallback: (image: "tilesets/test_units/fallback.png", frame: (1, 0), walk: true),
    ),
)
```

- A unit's picture is its **character's**, else its **class's**, else the
  **fallback**.
- `units` may also hold `on_foot: { "rider": (image: …, frame: …) }`: a
  mounted class's picture for a rider **off the horse** (0083), by class.
  Ticket 0334 makes the skin draw it; until then it is only checked. The lead's goes by the gender the player picked first:
  `characters` may name `lead_m` and `lead_f` as well as `lead`.
- It is drawn at its own size, centred across its tile, its feet on top
  of the 2-pixel HP bar: a picture taller than a tile reaches into the
  tile above.
- What the art doesn't show, the skin draws itself, the same for every
  tileset (`docs/design/look-and-feel.md`, *Battle map: bought tiles and
  unit sprites*): a 1-pixel outline in the unit's side's colour, grey and
  darker once it has acted, the HP bar, and an up or down arrow
  (`assets/images/effect_marks.png`) while it is under a timed effect.

## Checks

The all-assets test (ADR-0005) reports every problem in every file, with
its name:

- the file must parse, with no unknown fields, and its `id` must be its
  name;
- `image` must be a PNG in the bundle, and so must every unit picture's
  own image; every rectangle (each terrain tile, each unit picture, the
  fallback) must lie inside its image;
- each side of `tile_px` and `unit_px` must be 8 to 64 px;
- an image with `walk: true` must be big enough for 3 × 4 frames of
  `unit_px` (or the picture's `frame_px`), and each side of a `frame_px`
  must be 8 to 64 px;
- `terrain` needs `tile_px` and `image`; `tile_px` without `terrain` is an
  error; a `(column, row)` picture needs `image`;
- with `terrain`, **every terrain in `terrain.ron` must have a tile**
  (`terrain "…" has no tile`), in every look; every terrain, class and
  character named must exist;
- `layers` and `looks` need `terrain`; a layer must name a terrain and (a
  corner layer) have a picture; a mix must be four of `#` and `.`, listed
  once, and a corner layer has none for `....`; a corner layer's `size`
  has sides of 8 to 64 px, its pictures at that size lie inside the
  image, and its `skip` names real terrains; a look's name must be one
  a map can give (`trpg_content::map::TILE_LOOKS`) other than the
  tileset's own, `outdoor`;
- `frames` need `terrain`; a moving cell is listed once, is drawn by a
  terrain or a layer, has two frames or more, the first the cell itself,
  each lasting 1 ms or more and lying inside the image; its stills show
  `FirstFrame` (`Nothing` would leave a hole in the map).

## Files

The test tilesets, in `assets-test/tilesets/`:

| File | What |
| ---- | ---- |
| `test.ron`, `test.png` | The **test tileset**: 24 × 24 tiles, so nothing can quietly assume the glyph skin's 16 × 16. Each terrain's tile is its `bg` colour with its two glyphs in its `fg` colour; each class's picture is a grey disc with the first two letters of its name in white, by class id; then a `??` fallback. Not art: generated from our own data and the font atlas |
| `test_auto.ron`, `test_auto.png` | The **test tileset with layers and looks**, shaped like the bought one: 16 × 16 tiles (each terrain's `bg` colour and glyphs); a 2-pixel line round the water (with what lies on it) and round the woods, as corner layers, in the glyph colour of the layer's first terrain; a frame on the fort; rails on the bridge, which turn where water is to its left and right; an `indoor` look with a line round the floor instead; the water's tile **moving** through four frames of 600, 200, 400 and 200 ms, a ripple line lower down in each after the first (`frames`); and the test unit sheets' pictures. Not art: generated the same way |
| `test_units.ron`, `test_units/*.png` | The **test unit sheets**: no terrain tiles, and one 48×80 sheet per class of the Quick Battle (and a `??` fallback), shaped like the bought map sprites: 3 × 4 frames of 16 × 20, each a grey figure with the class's first two letters, a band across its head by facing and its feet by walking frame. Every entry has `walk: true`. Not art: generated the same way |

These files are **generated**; never edit them by hand. After changing the
terrain, the classes, the palette or the font, regenerate them:

```bash
cargo xtask test-tileset
```

A test in `xtask` fails if the committed files differ from what the tool
makes.

## Adding a tileset

1. Put the image and its `.ron` here (bought art goes in the private assets
   repository's `game/` instead, ADR-0040).
2. Give every terrain a tile, or none; give classes and characters pictures
   where the art has them.
3. Run the content tests: they name anything missing or outside the image.

The bought art isn't added by hand (each command's `--list` prints its
table):

- `cargo xtask map-sprite-import` copies the map sprites the game uses
  into `assets-private/game/units/`, from the table in
  `crates/xtask/src/map_sprite_import.rs`;
- `cargo xtask tileset-import` packs the bought terrain tiles the game
  uses into `assets-private/game/tilesets/tiny_tales.png` and writes
  `tiny_tales.ron`: the looks, from the mapping
  `assets-src/tilesets/tiny_tales.ron` (which bought tile or Tiled terrain
  stands for each of ours) and the bought sets' own `.tsx` corner tables,
  then the unit pictures of the table above. Run it after either changes.
  A tile it packs that the `.tsx` animates (Tiled's `<animation>`) gets
  its frames in `frames`, each still on a lesser tier (`Minimal` with a
  `todo`); a loop whose frames are all one picture is left out (Tiny
  Tales' hills and woods are such loops, 0096; its Standard coast's foam
  truly moves, three frames of 150 ms, first packed by 1318).
  A pack with no Tiled files (ELV Games' *Rogue Adventure*, the look
  `rogue_adventure` of Chapter 1's map, 1318) is read as plain sheets
  (`….png` under `assets-private/library/`): a tile by its number, a
  `Patch` layer joined at corners from the usual three-by-three layout,
  and the sheet's moving tiles by the mapping's `moving` (each frame a
  step along the row: the river's 8 frames). A `Quarters` layer joins
  each corner picture from quarters of a kit's cells, RPG Maker style
  (an outside corner, an edge, an inside corner or nothing, by a tile's
  two neighbours that way and the diagonal), so inside corners are round
  too: Chapter 1's hills and peaks, one raised area with Electric
  Lemon's *Lonesome Forest* rim round it (1359), its stones recoloured
  by the mapping's `recolour` as the importer reads the sheet. With a
  `face` (1361) its pictures are 16 × 24, the kit's cliff face (row 8)
  hung on the tile below the rim's bottom edge, rounded where the hill
  ends beside the tile above, and a `Shadow` layer is those pictures'
  silhouettes in one see-through colour, moved a pixel right and three
  down (the tileset's `size`, `offset` and `skip`); each such picture
  takes its cell and the one below it in the packed image.
  Harl's keep in Chapter 1 (1360), from *Rogue Adventure*'s ruins, needs
  no new field: a `Walls` layer is a tile layer of the wall's top by its
  walls beside it (across, up and down, else a pillar's cap); a `Faces`
  layer is a corner layer that hangs the wall's face, tile-sized, on the
  tile below each wall tile (`offset: (8, 24)` from the wall's top-left
  point; a pillar's face where the wall goes on up) and skips walls and
  gates; a `Standing` layer is a corner layer of 16 × 32 pictures
  standing on each of its tiles (`offset: (8, -8)`: Harrowby's tower,
  drawn over by the keep's stone where a fort is by a floor). A `Tiles`
  layer may leave its `tile` out (a see-through cell) and give `near`,
  its picture beside any of `beside`.
