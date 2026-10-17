# ADR-0083: One-bit pictures, made when the Playdate build is built, at their PNG's path

- **Status:** Accepted
- **Date:** 2026-10-17
- **Related tickets:** 1040 (this), 0062 (how they look), 1032 and 1055
  (the renderer, ADR-0074, ADR-0077), 1036 (the Playdate's bundle,
  ADR-0082), 1041, 1042, 1044 (the screens that show them)

## Context

The Playdate draws pictures as one-bit sprite items (ADR-0038, ADR-0074 §4).
The game's pictures are colour PNGs: title backdrops, battle backdrops,
combat pictures, portraits. The device must not decode PNGs (ADR-0070: it
is about 700 times slower than the PC).

0062 decided the look (`docs/design/playdate.md`, *One-bit pictures*):
each picture is **enlarged to the size it is shown first** (pixels doubled
for a whole scale, a smooth resample for 1.25×) and dithered there, so
each dot is one screen pixel; style A (a brightness stretch and a 4 × 4
Bayer dither) everywhere, and for portraits three more styles (A+ lines,
F+ lines, G) for a setting that switches between them.

So a one-bit picture is usually **bigger than its PNG**, while screens,
sidecars and sprites name a picture by its PNG's path (an `ImageId`) and
name parts of it in the PNG's pixels (`Sprite::src`, battlers' sizes).

The Playdate's bundle is a table that `trpg-content`'s `build.rs` writes
(ADR-0070 §3, ADR-0082), and a test plays Chapter 1 against exactly that
table.

The renderer, `Image` and the image table had moved to Shuyi Engine
(1104, 1108), so the format and the converter are the engine's; which
pictures, at what size and in which styles is the game's.

## Decision

### 1. The format: `shuyi_assets::bitmap`, a `.1bit` file

A 16-byte header (`1BIT`, version 1, a mask flag, the number of styles,
the width and height, and the **source size**: the PNG's own size), then
each style's rows and at most one mask the styles share. Rows are the
Playdate's own bitmap layout: `ceil(width / 8)` bytes, highest bit left,
a set bit white; a set mask bit is drawn. `Bitmap::parse` reads a file
without copying and says in words what is wrong with a bad one; `encode`
writes one. `trpg_content::bitmap` re-exports it.

### 2. The converter: `shuyi_assets::bitmap::convert` (`std`)

`convert(rgba, (num, den), methods)` enlarges the picture `num / den`
times (pixels doubled if whole, otherwise a bilinear resample weighted by
opacity), then makes one style per `Method`: brightness curve, stretch
(the darkest and lightest share to black and white), rounding to N
levels, the 4 × 4 Bayer dither at screen pixels, and optionally the art's
dark lines drawn black on top (a pixel `contrast` darker than its
brightest neighbour, or at most `dark` bright; a see-through neighbour is
white, so an outline is a line). A pixel less than half opaque is
see-through. `png_pixels` (feature `png`, tools only) reads any PNG as
RGBA.

### 3. The game's table: `trpg_content::playdate_images`

Kinds by folder: portraits (2×, styles A, A+ lines, F+ lines, G in that
order), combat pictures (twice their sidecar's scale, A), battle and title
backdrops (enlarged to the screen's 400-pixel width, lightened, A). The
title's logo and Game Over words are not dithered (1044 makes them its
own way). The thresholds and the backdrops' size and lightening are
starting values the screens' tickets may tune.

### 4. Converted when the Playdate's bundle is built, at the PNG's path

`trpg-content`'s `build.rs` includes that table, converts every picture
of those kinds into `OUT_DIR/onebit/`, and the Playdate's table holds the
one-bit file **at the PNG's path** in place of the PNG. So screens and
sidecars keep naming the same image; the PNG never reaches the device.
The engine's image table reads a one-bit file's source size as the
image's size, so ids, sizes and the loaders' checks are as with the PNG.

### 5. The renderer maps the PNG's pixels onto the picture's

`onebit::Image` gains a source size (`with_source`, `from_bitmap`). A
sprite still samples at the centre of each screen pixel, now through
`Image::sample`: the point in source pixels, scaled by picture size over
source size. With the two equal it is 1032's sampling exactly; the
reference renderer does the same, and the property tests draw an
enlarged picture with both.

### 6. The front end and the tools

The Playdate front end reads each image's one-bit picture once at start
(`trpg_playdate::pictures::Pictures`, an `onebit::Images`); an image
still a PNG has none and is drawn as a grey box. `cargo xtask
playdate-images` writes the same pictures to look at (`.1bit` and a PNG
per style) into `target/playdate/images/`, and the bought art's into the
private checkout's `generated/playdate-images/`, never into this
repository (ADR-0040). `frame-png --playdate` converts the bundle's
pictures the same way and draws them.

## Consequences

- Every build of `trpg-content` converts the pictures (about 30, well
  under a second); the test that plays Chapter 1 on the Playdate's bundle
  reads the same one-bit files the device gets. `build.rs` gains
  `shuyi-assets` (with `png`) and `serde` as build dependencies.
- The device never decodes a PNG; each picture is parsed once (a header
  check) at start.
- A one-bit file at a `.png` path is not a PNG: only the Playdate's
  bundle holds them, and `Bitmap::is_bitmap` tells them apart.
- Bought art on the Playdate needs the private layer in the Playdate's
  bundle, which it doesn't have yet (ADR-0070 §3); `playdate-images`
  already converts it for 1042's work. The follow-up ticket says so.
- A new kind of picture on the Playdate is a row in the table; its
  sprites are grey boxes until then.

## Alternatives considered

- **`cargo xtask playdate-images` converts into `target/`, and the build
  embeds what is there** (the ticket's sketch). The tests and every build
  not made through `xtask playdate` would hold the PNGs while the device
  holds one-bit files, so the Chapter 1 test would test another bundle,
  and a stale folder would ship stale pictures.
- **One file per style** (`happy.a-lines.1bit`). Extra paths the image
  table doesn't list and the Chapter 1 test would find unread; a style is
  better a part of its picture.
- **Converting at the art's own size and letting the renderer enlarge.**
  0062: doubled dots look blocky, and a 1.25× fighter's face broke.
- **The source size kept by the front end instead of in the file.** The
  image table already knows the PNG's size, but only while the PNG is in
  the bundle; the file standing alone lets 1036's bundle drop the PNG.
- **The Playdate SDK's `.pdi` and `pdc`.** `pdc` converts PNGs at its own
  threshold (no Bayer, no lines, no smooth enlarging) and the result is
  loaded through the SDK's bitmaps, not drawn by our renderer into our
  frame.
