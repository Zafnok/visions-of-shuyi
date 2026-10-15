---
id: "1030"
title: "Playdate: the console size is passed in (ConsoleSpec), so a 50 × 15 frame can exist"
type: infra
milestone: Post–Chapter 1
model: opus-5.5
effort: high
status: todo
blocked_by: []
nick_input: none
completed:
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

- [ ] No snapshot changes (`cargo insta` shows none).
- [ ] `cargo xtask frame-png` renders the same PNG as before for one
      screen (compare bytes before and after; say which in the PR).
- [ ] The 50 × 15 test passes.
- [ ] `CONSOLE_W`/`CONSOLE_H`/`CELL_W_PX`/`CELL_H_PX` are gone outside
      `ConsoleSpec::PC` (grep in the PR).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the 50 × 15 buffer test.
- Snapshot: unchanged.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
