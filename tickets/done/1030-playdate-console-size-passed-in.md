---
id: "1030"
title: "Playdate: the console size is passed in (ConsoleSpec), so a 50 × 15 frame can exist"
type: infra
milestone: Post–Chapter 1
model: reasoning
effort: high
status: done
blocked_by: []
nick_input: none
completed: 2026-10-15
---

# 1030 — The console size is passed in (`ConsoleSpec`)

## Context

The game draws a fixed console of 100 × 32 cells of 8 × 16 pixels
(`crates/ui/src/console.rs`: `CONSOLE_W`, `CONSOLE_H`, `CELL_W_PX`,
`CELL_H_PX`, used in ~42 files in `ui`, `app` and `xtask`). The Playdate
version keeps the 8 × 16 cells but has a 400 × 240 screen: **50 × 15
cells** (`docs/design/playdate.md`, *Size*).

[ADR-0070](../../docs/adr/0070-playdate-front-end.md) §6: the Playdate
skins must not fork the constants; they wait for engine-carve ticket
1108's `ConsoleSpec` "or do that part of 1108 first". 1108 sits behind
the engine carve chain (1103 → … → 1107), and Nick asked to rush the
port, so this ticket does that part of 1108 now, **in place** in
`trpg-ui` (nothing moves to `engine/`). 1108 is blocked by this ticket
and skips its step 1.

## Nick input

None. Nothing a player sees changes.

## Scope

**In:**
- `pub struct ConsoleSpec { cols: u16, rows: u16, cell_w: u16, cell_h: u16 }`
  in `crates/ui/src/console.rs`, with `ConsoleSpec::PC` (today's 100 ×
  32 × 8 × 16) and `ConsoleSpec::PLAYDATE` (50 × 15 × 8 × 16).
- The **frame** reads the spec instead of the constants: `GlyphBuffer`
  keeps the spec it was made with (`GlyphBuffer::with_spec`,
  `buf.spec()`), `pixel_bounds`, `Backdrop::clip_px`, `console::layout`
  (letterboxing), `cinema.rs` framing, `portrait.rs` fitting, the
  snapshot and `frame-png` helpers, and `app`'s renderer.
- `Ctx` (or whatever makes the frame each tick) holds the spec; `app`
  passes `ConsoleSpec::PC`.
- Each screen's **glyph skin** keeps its own 100 × 32 layout (the battle
  screen's `layout.rs` rects etc.): those are the PC skin's layout, not
  the console's. Where a skin reads `CONSOLE_W`/`CONSOLE_H` only to know
  the buffer's size, read `buf.width()`/`buf.height()` instead. Remove
  the four constants when nothing outside `ConsoleSpec::PC` uses them.
- A test: a `GlyphBuffer` made with `ConsoleSpec::PLAYDATE` is 50 × 15,
  prints text, places a sprite item and a backdrop, and its
  `pixel_bounds` is 400 × 240.

**Out (do not do):**
- Moving anything to `engine/` (1108).
- Colors by name, `UiColor` split (1108).
- Any Playdate skin (1037 onwards) or renderer (1032).

**Carve:** if 1108 has already landed when this is picked up, close this
ticket as done by 1108 (check `ConsoleSpec` exists and has a 50 × 15
test) and move on.

## Implementation steps

1. `grep -rn "CONSOLE_W\|CONSOLE_H\|CELL_W_PX\|CELL_H_PX" crates` and
   sort the uses into "the frame" and "a skin's layout".
2. Add `ConsoleSpec`; make the frame read it; give skins `buf.width()` /
   `buf.height()` or their own layout constants.
3. `app` and `xtask frame-png` pass `ConsoleSpec::PC`.
4. The 50 × 15 test.

## Acceptance criteria

- [x] No snapshot changes (`cargo insta` shows none).
- [x] `cargo xtask frame-png` renders the same PNG as before for one
      screen (compare bytes before and after; say which in the PR).
- [x] The 50 × 15 test passes.
- [x] `CONSOLE_W`/`CONSOLE_H`/`CELL_W_PX`/`CELL_H_PX` are gone outside
      `ConsoleSpec::PC` (grep in the PR).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the 50 × 15 buffer test.
- Snapshot: unchanged.

## Completion notes

Done 2026-10-15.

- `ConsoleSpec { cols, rows, cell_w, cell_h }` in `crates/ui/src/console.rs`
  (re-exported as `trpg_ui::ConsoleSpec`), with `PC` (100 × 32 × 8 × 16),
  `PLAYDATE` (50 × 15 × 8 × 16) and small helpers: `sized(cols, rows)`,
  `cell_px()`, `px()`.
- **The frame reads the spec.** `GlyphBuffer` keeps the spec it was made
  with (`GlyphBuffer::with_spec`, `buf.spec()`); `width()`/`height()`,
  `pixel_bounds`, cutting items under replaced cells and `blit` use it.
  `GlyphBuffer::new(w, h, fill)` stays for small buffers and makes them
  with `ConsoleSpec::PC`'s cells. `Backdrop` keeps its buffer's spec for
  `clip_px`. `console::layout(spec, w, h)` letterboxes any spec. The
  cinematic's map picture and pan framing read `ctx.console`; the portrait
  fit reads `buf.spec()`. `app`'s renderer and `xtask frame-png` read the
  buffer's spec.
- **`Ctx::console`** holds the spec (`ConsoleSpec::PC` by default); `Game`
  makes its frame from it, and `app` sets `ConsoleSpec::PC` explicitly at
  startup. `frame-png` goes through the test `Harness`, whose `Ctx` has the
  `PC` default, so it doesn't set it again.
- **The skins keep the PC layout.** Pure layout helpers in the PC glyph
  skins (title sparks and logo centring, the Game Over words, the forecast
  skull, the battle stage, the debug screens' windows) read
  `ConsoleSpec::PC`, because their rows and columns are the PC's. The map
  skins share one constant, `map_view::grid::CELL_PX`, made from
  `ConsoleSpec::PC`: their trait methods (`tile_px`, `tile_cells`) have no
  buffer to ask. Code with a buffer that only needs its size reads the
  buffer (`debug.rs`'s sample panels, the sprite test card).
- The four constants are gone: `git grep -E
  "CONSOLE_W|CONSOLE_H|CELL_W_PX|CELL_H_PX"` finds them only in this
  ticket. Tests that made a 100 × 32 buffer now say
  `GlyphBuffer::with_spec(ConsoleSpec::PC, …)`.
- Tests: `a_playdate_console_is_50_by_15_cells_of_400_by_240_pixels` (text,
  a sprite clipped at 400 × 240, a backdrop's window in cells and pixels)
  and `every_pixel_rule_uses_the_specs_cell_size` (a 4 × 6 pixel cell, so
  a mix-up with the PC's cells would fail) in
  `glyph_buffer/sprite_tests.rs`; `the_playdate_console_fits_its_own_size`
  in `console.rs`.
- **No snapshot changed**, and `cargo xtask frame-png` rendered the title
  (`--wait 2`) and the debug menu (`--keys F2 --wait 0.5`) to the same
  bytes on `main` and on this branch (SHA-256 `d4a18324…` and `543d530d…`).
- No ADR: ADR-0070 §6 already decided this; 1108 skips its step 1.
- No gameplay rules decided. Nothing a player sees changes.
