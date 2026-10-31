---
id: "1032"
title: "Playdate: the one-bit frame (1-bit font, Playdate palette, a renderer from the glyph buffer to 400 × 240 pixels)"
type: feature
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: ["1029", "1030"]
nick_input: none
completed: 2026-10-15
---

# 1032 — The one-bit frame

## Context

The Playdate screen is 400 × 240 pixels, one bit, black or white
(`docs/design/playdate.md`). Nick chose today's 8 × 16 cells and today's
font (Terminus, `assets/fonts/atlas.png`, ADR-0012): 50 × 15 cells, the
ConsoleSpec 1030 adds. The ground is white and the glyphs black.

What the Playdate look needs from the frame, beyond black glyphs on white
(`docs/design/playdate.md`, *Telling things apart without colour*;
1037 and later skins use these):

- **solid black** cells with **white** glyphs (your units);
- a **grey** fill, a 50% checker (enemies), and glyphs that keep a **white
  edge** so they read over it ("each letter with a white edge");
- light **dots** (move range), **diagonal lines** (attack range), a
  **dashed edge** (danger zone);
- **flipped pixels**: the cursor's corner marks invert what they cover,
  so they show on white, grey and black alike;
- one-bit **pictures** (portraits, combat picture, title art) as sprite
  items (ADR-0038: a picture is a sprite item from an asset file; 1040
  makes the images).

Today the frame is cells with `fg`/`bg` `Rgb` colours from the palette
(`crates/ui/src/color.rs`, `UiColor`, `Palette`) plus items (rectangles
and sprites) and a backdrop, and `app`'s `render.rs` paints it with
macroquad. The Playdate front end needs a renderer of its own from the
same frame to a 1-bit bitmap.

## Nick input

None. The look was decided in 0061; this ticket builds the means. Nick
sees it in 1037's map.

## Scope

**In:**
- **A Playdate palette:** the palette the Playdate skins use, mapping
  every `UiColor` the game needs to a one-bit **ink**: white, black, or
  a pattern (grey checker, dots, diagonals, dashes), and a way to say
  "black glyph with a white edge". Decide how a skin asks for these with
  the frame as it is (e.g. reserved palette colours the renderer reads
  as inks, and an `Item::Rect` with an "invert" ink for the cursor), and
  record it in an **ADR** (`write-adr` skill: "The one-bit frame"). Keep
  the PC's `Cell`, palette and renderer unchanged; if the frame needs a
  new field, it must default to today's behaviour.
- **The 1-bit font:** `cargo xtask font-atlas` also writes a packed 1-bit
  version of the atlas (one bit per pixel, 8 × 16 per glyph, the same
  glyph map), committed next to `atlas.png`, so the device never decodes
  a PNG. A test checks it matches `atlas.png`.
- **The renderer:** `trpg_ui::onebit` (pure, `no_std`), turning a
  `GlyphBuffer` made with `ConsoleSpec::PLAYDATE` into a 400 × 240 1-bit
  bitmap in the Playdate's own layout (52 bytes per row, MSB first, 1 =
  white), so the front end copies it to the SDK's frame buffer as it is.
  Draws, in order: backgrounds (ink fills), items under the glyphs,
  glyphs (with the white edge where asked), items over the glyphs,
  inverts. Patterns are fixed to the screen grid so neighbouring tiles
  line up. Sprites draw from 1-bit images (1040 makes them; this ticket
  takes the image format as input and tests with a small test image).
- `cargo xtask frame-png --playdate <screen>`: renders a frame through
  the one-bit renderer to a 400 × 240 PNG (black and white), for reviews
  with Nick and for the PRs of the skin tickets.
- Speed: the renderer must leave the device time to think. Measure a full
  50 × 15 frame on the device in 1033 (this ticket can't: no frame loop
  yet); keep it simple and allocation-free per frame.

**Out (do not do):**
- Any skin (map: 1037; screens: 1038 and later).
- Image conversion (1040).
- Wide (two-cell) glyphs for Japanese and Chinese (1049).
- Changing the PC look in any way.

**Carve (1108):** either order works. If 1108 has moved the frame to
`engine/`, put the renderer next to it as an engine module (it knows
nothing about this game) and say so in the ADR; if not, it lives in
`trpg-ui` and 1108 moves it with the frame (add a line to 1108).

## Implementation steps

1. Read `docs/design/playdate.md`, ADR-0038, ADR-0054, ADR-0070 and
   `crates/ui/src/glyph_buffer.rs`, `color.rs`, `crates/app/src/render.rs`.
2. Decide the ink model; write the ADR.
3. The Playdate palette (an asset file like `assets/data/palette.ron`,
   or a palette set in code if the ADR says so).
4. The 1-bit atlas from `xtask font-atlas`, and its test.
5. `onebit::render(buf, palette, font, images, out)`; snapshot tests of
   its output as PNG (`insta` binary snapshots or a text form, `#`/`.` per
   pixel, for small test buffers).
6. `frame-png --playdate`.

## Acceptance criteria

- [x] An ADR records how a skin asks for each one-bit effect listed in
      *Context*.
- [x] Tests render each effect (solid, grey, white edge, dots,
      diagonals, dashes, invert, a sprite) and pin the pixels.
- [x] The renderer builds with `--no-default-features` for
      `thumbv7em-none-eabihf` (the `no-std` CI job covers it).
- [x] `cargo xtask frame-png --playdate title` (or any screen) writes a
      400 × 240 black-and-white PNG; one is shown in the PR.
- [x] No PC snapshot changes.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: each ink and effect, glyph drawing, item order, the 1-bit atlas
  matches the PNG.
- Snapshot: small rendered buffers.

## Completion notes

Done 2026-10-15. ADR-0074 (*The one-bit frame*) records the design.

- **Inks are reserved colours** (`trpg_content::Ink`): white, black,
  `black_edged` (a glyph keeps a white edge one pixel wide, corners too),
  grey (50% checker), dots, diagonals, dashes (a checker of 2 × 2 squares,
  so a 1-pixel line is dashed 2 on, 2 off) and invert (a rectangle item
  flips what is under it, drawn last). Each has an `Rgb` code; the
  renderer reads a cell's or item's colour back as its ink, and any other
  colour as white or black by brightness. `Cell`, `Item`, `Palette` and
  the PC renderer are unchanged. The patterns are the screen's pixels and
  match Nick's chosen mockup (`q3r2-e-grey.png`), but for the dashes (2 on
  1 off there), which must repeat within a byte.
- **The Playdate palette** is `assets/data/palettes/playdate.ron`, colour
  names to ink names, loaded by `PaletteDef::load_playdate(base)` as an
  ordinary palette of codes (unknown inks and missing names are errors; a
  content test checks the embedded file). Its `black` (the console
  background) is `white`. Not part of `Content`: 1033 loads it for the
  device.
- **The 1-bit font:** `cargo xtask font-atlas` also writes
  `assets/fonts/atlas_1bit.bin` (8,816 bytes: 551 glyphs × 16 rows, a byte
  a row, highest bit left). `the_one_bit_atlas_matches_the_png` checks it
  against `atlas.png` pixel for pixel; the up-to-date test covers it too.
  `.gitattributes` marks `*.bin` binary. `atlas.png` itself is unchanged
  (regenerating rewrote its bytes with the same pixels; the committed file
  was kept).
- **The renderer:** `trpg_ui::onebit::render(buf, font, images, out)`,
  `no_std`, allocation-free, into a `[u8; 52 × 240]` in the Playdate's
  layout (1 = white). Order: white ground, a backdrop's scene (zoomed and
  panned), cell backgrounds, `Under` items, the white edges, glyphs,
  `Over` items, invert rectangles. Fills and glyph rows go a byte at a
  time; sprites a pixel at a time. One-bit pictures are `onebit::Image`
  (the Playdate's bitmap layout, data + optional mask) found through the
  `Images` trait; a sprite without a picture is a grey box.
  `Paint::Dimmed` turns every other pixel white (the design's faded
  listener), `opacity` dithers in Bayer 4 × 4.
- **Deviation:** no palette argument to `render` (the ticket's sketch had
  one): the inks are in the colours themselves.
- **Tests** (`crates/ui/src/onebit/tests.rs`, 17): every ink's fill, the
  patterns lining up across cells, brightness fallback (including the
  exact half), glyphs both ways, the white edge (between two adjacent
  glyphs, above and below, round a block), a glyph in a pattern, the
  fallback glyph, item order, invert over white/grey/black and after an
  `Over` item, a sprite (scale, flip, clip, silhouette in black and in
  dots, dimmed, shaded, faded out, no picture), the dither, image and
  font size checks, a backdrop (zoom, window cut, pan both ways),
  cutting at the screen's edge, byte-crossing runs, the game's real font.
  All as `#`/`.` text pinned in the test.
- **`cargo xtask frame-png <out> --playdate [steps]`** runs the Harness
  with `ConsoleSpec::PLAYDATE` and the Playdate palette
  (`Harness::with_layout_and` / `at_prompt_with_layout_and`, which change
  the `Ctx` before the game starts) and writes a 400 × 240 black and white
  PNG (`--scale` defaults to 1 there). The title is all grey boxes until
  1040 (its backdrop is a picture), so the PR shows the debug menu
  (`docs/screenshots/1032-playdate-debug-menu.png`) and a hand-built frame
  of every effect (`docs/screenshots/1032-playdate-effects.png`, from a
  throwaway test, not committed).
- `trpg-ui` builds for `thumbv7em-none-eabihf` without `std`. No PC
  snapshot changed. 1108 gained a line: `Ink` and the palette loader move
  with the frame.
- **Not measured:** the renderer's speed on the device; 1033 measures a
  full frame (no frame loop yet).
- **Placeholders, not rules:** the Playdate palette's inks for things the
  design doesn't fix (dim and highlighted text, the heal range, a unit
  under a timed effect) are placeholders their skin tickets (1037, 1038+)
  decide, as the file says. Nothing a player sees changes yet, and no
  gameplay rule was decided.
