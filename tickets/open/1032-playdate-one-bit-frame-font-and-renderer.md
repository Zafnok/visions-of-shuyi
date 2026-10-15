---
id: "1032"
title: "Playdate: the one-bit frame (1-bit font, Playdate palette, a renderer from the glyph buffer to 400 × 240 pixels)"
type: feature
milestone: Post–Chapter 1
model: opus-5.5
effort: high
status: todo
blocked_by: ["1029", "1030"]
nick_input: none
completed:
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

- [ ] An ADR records how a skin asks for each one-bit effect listed in
      *Context*.
- [ ] Tests render each effect (solid, grey, white edge, dots,
      diagonals, dashes, invert, a sprite) and pin the pixels.
- [ ] The renderer builds with `--no-default-features` for
      `thumbv7em-none-eabihf` (the `no-std` CI job covers it).
- [ ] `cargo xtask frame-png --playdate title` (or any screen) writes a
      400 × 240 black-and-white PNG; one is shown in the PR.
- [ ] No PC snapshot changes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: each ink and effect, glyph drawing, item order, the 1-bit atlas
  matches the PNG.
- Snapshot: small rendered buffers.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
