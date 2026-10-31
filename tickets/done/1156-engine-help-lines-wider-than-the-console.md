---
id: "1156"
title: "The engine's help lines fit a narrow console: a line wider than the buffer is cut with … instead of losing both ends"
type: feature
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-10-18
---

# 1156 — The engine's help lines fit a narrow console

## Context

Found by 1155, which moved Hello, Grid (`engine/examples/hello-grid/`) to
a 48 × 24 console. The engine's screens print their help line with
`print_centred` (`engine/crates/ui/src/screens.rs`): it centres the text
on the buffer's width, and a text wider than the buffer starts left of
column 0, so both its ends are lost. Hello, Grid's Key bindings help
("arrows move · Enter bind · Delete clear · Backspace back", 55 cells)
shows as `ws move · Enter bind · Delete clear · Backspace` on 48 cells.
On this game's 100-cell console no help line is that wide today.

## Nick input

None.

## Scope

**In:**
- `print_centred`: a text wider than the buffer starts at column 0 and is
  cut to the buffer's width, its last cell a `…` (the Key bindings skin's
  `cut` in `engine/crates/ui/src/screens/key_bindings/glyph.rs` does
  this; move it somewhere both can use, e.g. `screens.rs`).
- A unit test in `engine/crates/ui/src/screens/tests.rs`.

**Out (do not do):**
- Wrapping help lines over two rows, or shortening any game's text.
- Changing how a text that fits is placed: every existing snapshot of the
  game's screens stays the same.

## Implementation steps

1. In `engine/crates/ui/src/screens.rs`, give `print_centred` the cut:
   when `text` is wider than `buf.width()`, print `cut(text, width)` at
   column 0.
2. Make the Key bindings skin use the shared `cut`.
3. Test: a 10-cell buffer, a 14-character text → the row reads the first
   9 characters and `…`.

## Acceptance criteria

- [x] `cargo shuyi frame-png keys.png --keys "Down Enter Down Down Down Down Down Down Enter"`
      from `engine/examples/hello-grid/` shows the help line from its
      first word, cut at the right with `…`.
- [x] No snapshot of the game (`crates/`) changes.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `print_centred` with a text wider than the buffer.

## Completion notes

- `cut` moved from the Key bindings skin to `engine/crates/ui/src/screens.rs`
  (public, beside `print_centred`); the skin imports it from there.
- `print_centred` prints a text wider than the buffer at column 0, cut to
  the buffer's width with a `…` as its last cell. A text that fits is
  placed as before, so no snapshot changed.
- Tests: the ticket's 10-cell buffer / 14-character text case, and `cut`'s
  own cases (fits, cut, 1 cell, 0 cells).
- Hello, Grid's Key bindings frame now reads
  `arrows move · Enter bind · Delete clear · Backs…`.
- No follow-up tickets. No gameplay rules decided.
