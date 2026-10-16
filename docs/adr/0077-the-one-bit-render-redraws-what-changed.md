# ADR-0077: The one-bit render redraws only what changed, against a kept reference

- **Status:** Accepted
- **Date:** 2026-10-16
- **Related tickets:** 1055 (this), 1032 (the renderer, ADR-0074), 1033
  (the front end that measured it), 1040 (one-bit pictures), 1108 (the
  frame moves to the engine)

## Context

1033 measured 1032's renderer on Nick's Playdate: about 360–430 ms for
every frame whose picture changed, so the screen moved about three times
a second. The goal (1055) is a changed frame drawn and copied in 33 ms on
average, under 66 ms at most. ADR-0074 §4 wrote the renderer to be
correct and allocation-free, not fast: 64-bit coordinates and a 128-bit
shift for every byte, a pattern's byte worked out pixel by pixel for
every byte written, each cell's background filled on its own, and the
whole 400 × 240 frame redrawn when one cell changed.

The picture must not change: 1032's tests, the `frame-png --playdate`
output and what Nick has seen on the device.

## Decision

### 1. A faster full render, same pixels

`onebit::render` keeps ADR-0074's order and meaning; only how it writes
changes:

- 32-bit screen coordinates (the Cortex-M7 is a 32-bit core), 64-bit only
  for a glyph row's bits and a sprite's sampling arithmetic;
- each ink's byte for each of its four rows in a table built at compile
  time (`PATTERNS`), looked up instead of computed;
- fills write their whole bytes whole, masking only the two at the ends;
  neighbouring cell backgrounds of one ink are filled as one rectangle;
- a glyph row at a whole cell, inside the clip, is one byte written
  straight;
- only the cells that may reach into the part being drawn are visited (a
  cell's size or its glyph's, plus a pixel for the white edge), so a
  backdrop's scene is not walked where its window cuts it away;
- the ASCII glyphs' places in the one-bit atlas are kept in a table in
  `onebit::Font` (the glyph map is a tree, slow to search on the device);
- the last colour's ink is remembered while walking cells.

### 2. Redraw only what changed

`onebit::render_changed(old, new, font, images, out)` takes `out` holding
`old` as `render` drew it and makes it hold `new`. The screen is cut into
8 × 16 pixel tiles (50 × 15, a `u64` of bits per row of tiles). A tile is
marked if it touches:

- a cell that differs, grown by a pixel all round (an edged glyph's white
  edge reaches into its neighbours) and by its glyph's size if that is
  bigger than the cell;
- an item that differs at the same index in the item lists (the old one
  and the new one): its visible rectangle;
- a backdrop that differs: its window, old and new.

Each run of marked tiles along a row is redrawn from the white ground with
the whole drawing order clipped to it. Every step of the order sets a
pixel from its own place and its value before, so a region redrawn from
the ground under a clip is exactly the full render there: no case
analysis of which layer changed. A buffer of another size is drawn whole.

The Playdate front end draws its first frame whole and every later
changed frame with `render_changed` from the buffer it last drew (it
already kept that buffer to skip unchanged frames).

### 3. 1032's renderer stays, as the reference

`onebit::reference` is 1032's renderer as it was, with only phase marks
added, behind `#[cfg(any(test, feature = "onebit-reference"))]`: never
in a shipped build. Property tests draw random buffers (cells, inks,
see-through cells, rectangles, sprites with a masked picture, backdrops
zoomed and panned) with both renderers and require the same frame, and
require `render(old)` + `render_changed(old, new)` to equal `render(new)`
for random edits. An integration test does both on the game's own
Playdate frames: the title, a scene typing out frame by frame, and the
first battle with its cursor moving.

### 4. Measured on the device by a bench build

`render_marked(…, mark)` (in both renderers) calls `mark(phase)` as each
phase of the order ends. `cargo xtask playdate --bench` builds the front
end with its `bench` feature into `VisionsOfShuyi-bench.pdx`: after each
frame drawn it renders the buffer whole again with both renderers, timing
every phase with the SDK's timer, checks the three frames are equal, and
logs the numbers with the minute report. The normal build logs only
1033's minute report.

## Consequences

- A changed frame costs what changed: a cursor step or a letter typing
  out redraws a few tiles; a scroll or a new screen still draws whole.
- `render_changed` trusts `out` to hold `old`'s render. Only the front end
  calls it, with the frame it drew last.
- Sprites are still sampled a pixel at a time (the one-bit pictures of
  1040 are not here yet); a faster sprite path is the first thing to try
  if pictures make a frame slow.
- The reference is a second renderer to keep. A change to what the
  renderer draws (a new ink, a new kind of item) changes both, and the
  property tests say whether they agree; when 1108 moves the renderer
  to the engine, the reference moves with it.

## Alternatives considered

- **Only a faster full render.** Simpler, but a full frame is still the
  most work there is, every frame, for a cursor that moves one tile; and
  the device's speed was unknown when the work had to be planned.
- **Dirty cells tracked by the `GlyphBuffer` as it is written.** Exact
  and cheap to read, but every write to the frame (and the PC's) would
  pay for it, and `GlyphBuffer` is the engine's (1108). Comparing with the
  last buffer costs one pass over 750 cells.
- **Redrawing changed cells over the old frame without the full order.**
  Faster per cell, but an edge, an `Under` item or an invert rectangle
  over a changed cell would each need its own rule; redrawing the region
  from the ground under a clip needs none.
- **Golden frames of the game's screens instead of a reference renderer.**
  They would break whenever a screen's content changes; the reference
  only changes when the renderer's meaning does.
