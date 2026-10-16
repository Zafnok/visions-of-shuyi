# ADR-0074: The one-bit frame: inks as reserved palette colours, a renderer to the Playdate's frame buffer

- **Status:** Accepted
- **Date:** 2026-10-15
- **Related tickets:** 1032 (this), 0061 and 0062 (Nick's Playdate look),
  1033 (the front end that shows it), 1037 and 1038+ (the Playdate skins),
  1040 (one-bit pictures), 1108 (the frame moves to the engine)

## Context

The Playdate's screen is 400 × 240 pixels, one bit (`docs/design/playdate.md`).
Nick chose today's 8 × 16 cells and font (50 × 15 cells, `ConsoleSpec::PLAYDATE`,
ticket 1030), a white ground and black glyphs, and ways to tell things apart
without colour (*Telling things apart without colour*):

- your units: **white letters in a solid black tile**;
- enemies: letters on a **grey** tile (a 50% checker), **each letter with a
  white edge** so it reads over the checker;
- the move range in light **dots**, the attack range in **diagonal lines**,
  the danger zone a **dashed edge**; terrain glyphs on a marked tile keep a
  white edge;
- the cursor's corner marks **flip the pixels** they cover, so they show on
  white, grey and black;
- one-bit **pictures** (portraits, the combat picture, title art), made
  ahead of time by the converter (1040).

The frame (`GlyphBuffer`) holds cells with `fg`/`bg` `Rgb` colours, items
(rectangles and sprites, ADR-0018, ADR-0038) and a backdrop (ADR-0048).
Skins paint views into it with colours from the `Palette` (ADR-0054).
`app`'s renderer draws it with macroquad. The PC's `Cell`, palette and
renderer must not change.

The device is slow (ADR-0070: ~700× the PC on parsing and AI) and its
C stack is about 10 KB; the frame loop (1033) must leave it time to
think.

## Decision

### 1. Inks are reserved colours: the frame doesn't change

A one-bit pixel is coloured by an **ink** (`trpg_content::Ink`):

| Ink | Name in the palette file | Code | Draws |
| --- | --- | --- | --- |
| `White` | `white` | `#ffffff` | white |
| `Black` | `black` | `#000000` | black |
| `BlackEdged` | `black_edged` | `#010101` | black; **as a glyph's fg**, the glyph keeps a white edge one pixel wide all round (corners too) |
| `Grey` | `grey` | `#808080` | a 50% checker: black where x + y is even |
| `Dots` | `dots` | `#e0e0e0` | black where y is even and x + y is a multiple of 4 (one pixel in eight) |
| `Diagonals` | `diagonals` | `#c0c0c0` | black where x − y is a multiple of 4: lines down to the right (one in four) |
| `Dashes` | `dashes` | `#404040` | a checker of 2 × 2 squares: a 1-pixel line of it is a dashed line, 2 on, 2 off |
| `Invert` | `invert` | `#ff00ff` | **as a rectangle item**, flips the pixels under it |

Each ink has a reserved **code**, an `Rgb` value. A cell or item whose
colour is a code is drawn in that ink; any other colour is drawn white if
it is at least half bright (`0.299 r + 0.587 g + 0.114 b ≥ 128`), black if
not, so a frame drawn in the PC's colours still reads. The codes are
about as dark as their inks, so a Playdate frame drawn in colour (`frame-png`
without `--playdate`, a snapshot's colours) looks roughly right too.

Patterns are functions of the **screen pixel**, not of the cell or item, so
neighbouring tiles line up, and each repeats within 8 pixels across and 4
down (the renderer paints a whole byte at a time). The patterns match the
mockup Nick chose (`docs/design/playdate/q3r2-e-grey.png`), except the dashes
(2 on, 2 off there; 2 on, 1 off in the mockup), which must repeat within a
byte.

### 2. How a skin asks for each effect

| Effect (*Context*) | The skin draws |
| --- | --- |
| Solid black, white glyphs (your units) | cells with `bg` = `Black`, `fg` = `White` |
| Grey with white-edged glyphs (enemies) | cells with `bg` = `Grey`, `fg` = `BlackEdged` |
| A white edge on terrain glyphs over a range | `fg` = `BlackEdged` |
| Move range, attack range | cells with `bg` = `Dots` / `Diagonals` (or a rectangle item in that ink) |
| The danger zone's dashed edge | 1-pixel rectangle items in `Dashes` along the edge |
| The cursor's corner marks | rectangle items in `Invert` |
| A picture | a sprite item (ADR-0038) whose image is a one-bit picture (§4) |

Every ink means the same wherever it is: a fill as a cell's `bg`, a
rectangle or a sprite's `Paint::Solid`, the glyph's pixels as a `fg`.
Two exceptions, both drawn plain black elsewhere: the white edge exists
only on glyphs, and only rectangles invert.

### 3. The Playdate's palette is a palette file of ink names

`assets/data/palettes/playdate.ron` names an ink, not a hex value, for
every colour of the default palette (`"enemy": "grey"`), and may add names
of its own for the Playdate's skins. `PaletteDef::load_playdate(base)`
loads it as an ordinary `PaletteDef` whose colours are the inks' codes,
reports unknown inks and missing names, and a content test checks the
embedded file. So `Palette`, `UiColor` and every skin work unchanged:
the front end (1033) puts this palette in `Ctx::palette`, the Playdate
skins draw with it, and today's screens drawn with it already come out
black on white. Its `black` (the console background) is `white`, as the
light theme Parchment does. It is not a colour theme the player picks.
Inks the design doesn't fix (dim and highlighted text, the heal range, a
timed effect) are placeholders their skin's ticket decides.

### 4. The renderer: `trpg_ui::onebit`

`onebit::render(buf, font, images, out)` (pure, `no_std`, allocation-free)
draws `buf` into `out: &mut Frame`, a `[u8; 52 × 240]` in the Playdate's
own frame buffer layout: rows of 52 bytes (50 of pixels, 2 unused), a
byte's highest bit its left pixel, **a set bit white**. The front end
copies it as it is (or renders straight into the SDK's buffer). Pixels
past 400 × 240 are cut off. The order:

1. the white ground (every byte set);
2. a backdrop's scene in its window, at its pixel offset and zoom, by
   steps 3–8 (ADR-0048);
3. cell backgrounds (never a see-through cell's);
4. `Under` items, in order;
5. the white edges of every `BlackEdged` glyph, then
6. every glyph, so one glyph's edge never covers another glyph;
7. `Over` items, in order;
8. the `Invert` rectangles of both layers, in order: last of all.

There is no palette argument (the ticket's sketch had one): the inks are
in the colours themselves.

- **The font** (`onebit::Font`) is the glyph map `atlas.ron` plus the
  **one-bit atlas** `assets/fonts/atlas_1bit.bin`, written by `cargo xtask
  font-atlas` next to `atlas.png`: each glyph in cell index order,
  `cell_h` bytes, a byte a row, the highest bit the left pixel, a set bit
  the glyph (so cells are at most 8 pixels wide). No header: the glyph map
  has the sizes. A test checks it matches `atlas.png` pixel for pixel. A
  glyph the font lacks is drawn as the fallback `?`, as on the PC.
- **Pictures** (`onebit::Image`) are the Playdate's own bitmap layout: rows
  of `ceil(width / 8)` bytes, highest bit left, a set bit white; and an
  optional mask of the same shape, a set bit drawn. The renderer finds them
  through the `onebit::Images` trait (`()` has none). 1040 makes the
  pictures and their store. A sprite samples its image at the nearest pixel
  (the pixel centre, as `app` does); `Paint::Solid` draws its silhouette in
  that ink; `Paint::Dimmed` turns every other pixel white (the faded
  listener, `docs/design/playdate.md`); `Paint::Shaded` draws it as it is
  (one-bit has no light: the converter makes dusk pictures if they are
  wanted). An `opacity` below 255 dithers the picture away in a Bayer 4 × 4
  order. A sprite without a picture is a grey rectangle.
- A frame that was **faded** (`GlyphBuffer::fade`) has its colours scaled
  off the codes, so it draws white or black by brightness: a fade on the
  Playdate is a cut to black until a cinematic ticket (1044) needs better.
- **Speed:** fills and glyph rows are written a byte at a time (a pattern's
  row is one byte; a glyph row at a whole cell is one byte), sprites a
  pixel at a time. 1033 measures a full 50 × 15 frame on the device; a
  faster sprite path is the first thing to try if it is too slow.

### 5. Where it lives

1108 has not moved the frame yet, so the renderer is a module of
`trpg-ui` beside `glyph_buffer`, and `Ink` sits in `trpg-content` beside
`PaletteDef`. Neither knows anything of this game; 1108 moves them with
the frame and the palette (its ticket says so).

### 6. Seeing it

`cargo xtask frame-png <out.png> --playdate [steps]` runs the Harness with
`ConsoleSpec::PLAYDATE` and the Playdate's palette (through
`Harness::with_layout_and`, which changes the `Ctx` before the game starts)
and writes the frame through `onebit::render` as a 400 × 240 black and
white PNG (`--scale` enlarges it), for reviews with Nick and the skin
tickets' PRs. Until 1040 there are no one-bit pictures, so sprites are grey
boxes there.

## Consequences

- The PC is untouched: no new field on `Cell`, `Item` or `Palette`, and no
  PC snapshot changes. A skin asks for a one-bit effect by colour name, as
  it asks for any colour.
- An ink is only as safe as its code is unique: a PC colour that happened
  to equal a code would draw as that ink, but PC frames are never drawn
  one-bit except to look at them, and the codes are odd values
  (`#010101`, `#ff00ff`).
- Anything that computes colours (`lerp`, `scale`, `dimmed`, `fade`) leaves
  the codes; the Playdate skins pick inks by name and don't blend.
- New inks are a row in the table: a variant, a name, a code and a
  pattern that repeats within 8 × 4 pixels.
- The one-bit atlas is one more generated file to keep in step; the
  `xtask` tests catch a stale one.

## Alternatives considered

- **A new field on `Cell` and `Item` for the ink** (or an `Item::Pattern`):
  says what it means, but every PC cell and item grows, every constructor
  and snapshot changes, and the PC renderer must ignore it. The ticket asks
  to keep the PC's frame as it is.
- **Dithering any colour by its brightness** (no reserved codes): no
  control over which pattern a range gets, and the edge and the invert
  can't be said at all.
- **A Playdate palette in code** instead of a file: the file reads like the
  other palettes, is checked by content validation, and lets a skin ticket
  change an ink without code.
- **Hex codes in the Playdate palette file**: no new loader, but
  `"enemy": "#808080"` hides that it means a checker; ink names say it.
- **Decoding `atlas.png` on the device**: a PNG decoder on the device and
  ~300 KB of RGBA at start-up, for a font that is one bit to begin with.
- **The halo as a cell of white** (as some one-bit games box their letters):
  covers the checker between the two letters of a tile, which Nick's mockup
  keeps.
