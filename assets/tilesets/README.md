# Tilesets (`.ron` + `.png`)

What a **sprite map skin** paints the battle map with (ADR-0038, ADR-0049,
ADR-0052): a tile for every terrain, the layers painted over them, and a
picture for units, by the game's ids (terrain, classes, characters). The rules never read a tileset; only the
look hangs off these ids. Loaded and checked by `trpg_content::tileset`
into `Content::tilesets`, by id.

The game starts with the tileset called **`tiny_tales`** when it has it.
That one is bought art: it exists only in the private assets
(`assets-private/game/tilesets/`, ADR-0040), so a build without them
starts with the glyph skin (coloured letters). The tilesets in this folder
are generated test fixtures, reachable from the debug menu (F2 → *Map
skin*, which goes round every tileset).

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

### Units

A unit **picture** is written one of two ways, and a file may mix them:

- `(column, row)`: a cell of the tileset's own `image`, in `unit_px` steps
  from `units_origin_px`;
- `(image: "units/fighter_male.png", frame: (1, 0))`: a frame of an image
  file of its own, in `unit_px` steps from that image's top-left. The
  bought map sprites are like this: one 48×80 sheet per character or
  class, 3 columns (walking frames) × 4 rows (facing down, left, right,
  up) of 16×20 frames; the standing, front-facing frame is `(1, 0)`.

**A tileset with no terrain tiles** leaves out `image`, `tile_px` and
`terrain` (keep `image` if a unit picture is a `(column, row)`). The
terrain, the ranges, the path and the cursor are then painted as glyphs,
on the glyph skin's 16×16 tiles, and only the units are pictures:

```ron
(
    id: "test_units",
    unit_px: (16, 20),
    units: (
        characters: { "lead_f": (image: "units/fighter_female.png", frame: (1, 0)) },
        classes: { "guard": (image: "tilesets/test_units/guard.png", frame: (1, 0)) },
        fallback: (image: "tilesets/test_units/fallback.png", frame: (1, 0)),
    ),
)
```

- A unit's picture is its **character's**, else its **class's**, else the
  **fallback**. The lead's goes by the gender the player picked first:
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
- `terrain` needs `tile_px` and `image`; `tile_px` without `terrain` is an
  error; a `(column, row)` picture needs `image`;
- with `terrain`, **every terrain in `terrain.ron` must have a tile**
  (`terrain "…" has no tile`), in every look; every terrain, class and
  character named must exist;
- `layers` and `looks` need `terrain`; a layer must name a terrain and (a
  corner layer) have a picture; a mix must be four of `#` and `.`, listed
  once, and a corner layer has none for `....`; a look's name must be one
  a map can give (`trpg_content::map::TILE_LOOKS`) other than the
  tileset's own, `outdoor`.

## Files

| File | What |
| ---- | ---- |
| `test.ron`, `test.png` | The **test tileset**: 24 × 24 tiles, so nothing can quietly assume the glyph skin's 16 × 16. Each terrain's tile is its `bg` colour with its two glyphs in its `fg` colour; each class's picture is a grey disc with the first two letters of its name in white, by class id; then a `??` fallback. Not art: generated from our own data and the font atlas |
| `test_auto.ron`, `test_auto.png` | The **test tileset with layers and looks**, shaped like the bought one: 16 × 16 tiles (each terrain's `bg` colour and glyphs); a 2-pixel line round the water (with what lies on it) and round the woods, as corner layers, in the glyph colour of the layer's first terrain; a frame on the fort; rails on the bridge, which turn where water is to its left and right; an `indoor` look with a line round the floor instead; and the test unit sheets' pictures. Not art: generated the same way |
| `test_units.ron`, `test_units/*.png` | The **test unit sheets**: no terrain tiles, and one 48×80 sheet per class of the Quick Battle (and a `??` fallback), shaped like the bought map sprites: 3 × 4 frames of 16 × 20, each a grey figure with the class's first two letters, a band across its head by facing and its feet by walking frame. Not art: generated the same way |

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
