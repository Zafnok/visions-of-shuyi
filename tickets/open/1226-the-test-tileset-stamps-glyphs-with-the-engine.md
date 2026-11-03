---
id: "1226"
title: "The test tileset, auto-tiles and unit sheets stamp glyphs with the engine's glyph_tiles"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: low
status: blocked
blocked_by: ["engine#2580"]
nick_input: none
completed:
---

# 1226 — The test tileset stamps glyphs with the engine

## Context

From the game/engine boundary audit of 2026-11-03. `cargo xtask
test-tileset` (0433), `test-auto` and `test-units` draw the public test
pictures in `assets-test/tilesets/` with a canvas of their own:
`Canvas` (`new`, `set`, `get`, `stamp`, `stamp_at`) and `blend` in
`crates/xtask/src/test_tileset.rs:112-196`, reused by
`test_auto.rs:153-183` and `test_units.rs:89-101`. It is the engine's
`glyph-tiles` stamp again, with the same rounded integer blend and the
same centring, but the engine's was private. One difference: on a clear
pixel ours keeps the alpha (`stamp_at`: "A pixel that is clear stays
clear"), and the engine's covers it.

[engine#2580](https://github.com/Zafnok/shuyi-engine/blob/main/tickets/open/2580-glyph-stamping-onto-a-picture-public.md)
(after engine#2391, one RGBA `Image`) makes it public:
`glyph_tiles::stamp_glyphs` with `Clear::Keep` for our rule,
`glyph_tiles::centred`, and `Image::set_pixel`.

## Nick input

None.

## Scope

**In:**
- Move the pin past engine#2580 (`engine-change` skill §7, `engine-pin`
  skill), unless an open `engine-pin` PR has already done so. It also
  passes engine#2391, whose breaks the game doesn't call (no
  `glyph_tiles` or `font_atlas::Atlas` use).
- The three tools draw on `shuyi_cli::frame_png::Image` with the
  engine's stamp in `Clear::Keep`; `Canvas` and `blend` go.
- The committed pictures stay byte for byte: the three
  `committed_files_are_what_the_tool_makes` tests must pass unchanged.
- **Either order with 1233:** if 1233 is done, `docs/engine/boundary.md`'s
  `test-tileset` row says it moves with 1226: write that its glyph stamp
  is the engine's (engine#2580) and the tool stays the game's kit
  fixtures; if not, nothing to do (1233 writes the row).

**Out (do not do):**
- Using `glyph_tiles::render` for the terrain rows, or a
  `shuyi.ron` `glyph_tiles` part: the tiles come from `TerrainDef` and
  the palette, and the discs, the class letters and the `.ron` files are
  ours.
- Changing a picture or a tile size.

## Implementation steps

1. **Pin** (as above), then read `glyph_tiles::{stamp_glyphs, centred,
   Clear}` and `Image::set_pixel`.
2. `test_tileset.rs`: `Sources::atlas` becomes an `Image` (built from
   `decode_png`'s width, height and pixels). Delete `Canvas`, `blend`
   and their two unit tests (`a_canvas_keeps_each_pixel_where_it_was_set`,
   `glyph_pixels_blend_by_their_coverage`; the engine tests both). In
   `image`, start from `Image::filled(width, height, [0; 4])`, fill with
   `set_pixel`, and stamp at `centred((x, y), TILE, 2, &src.font)` with
   `stamp_glyphs(…, Clear::Keep)`. `image` returns `Result<(u32, u32,
   Vec<u8>), String>` (a missing glyph now fails, where it was skipped);
   `run` uses `?` and the tests `.unwrap()`.
3. `test_auto.rs::image`: the same, `stamp_glyphs` at `(left, top)`.
4. `test_units.rs::sheet`: the same, at `(left, top + 3)`; it returns a
   `Result` too, and `run` and its tests follow.
5. `cargo xtask test-tileset`, `test-auto` and `test-units`: `git status
   assets-test/` shows no change.
6. **Measure** with `cargo xtask code-lines`: about 60 production lines
   removed and 15 added; about 30 test lines removed.

## Acceptance criteria

- [ ] `shuyi-engine.rev` is at or past the commit that closed
      engine#2580.
- [ ] `grep -rn "Canvas\|fn blend" crates/xtask/src` finds nothing.
- [ ] The three `committed_files_are_what_the_tool_makes` tests and
      every other test of the three tools pass, and running the tools
      changes no file in `assets-test/`.
- [ ] All gates in the `run-gates` skill pass, including
      `cargo xtask ticket-lint --check-engine`.

## Tests required

- Unit: the three tools' existing tests, the byte-identity ones first.
  The stamp's own tests are the engine's (engine#2580).

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
