---
id: "1155"
title: "The engine's screens fit the game's console: options and Key bindings laid out from the frame's size, not a fixed 100 × 32"
type: feature
milestone: "Engine: carve"
model: sonnet-5
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1155 — The engine's screens fit the game's console

## Context

Found by 1118, the engine's first example (`engine/examples/hello-grid/`).
The glyph skins of the engine's options screen and Key bindings screen
place their panels at fixed cells laid out for this game's console,
100 × 32 (`ConsoleSpec::PC`):

- `engine/crates/ui/src/screens/options/glyph.rs`: `PANEL =
  Rect::new(20, 3, 60, 20)`, values at `PANEL.x + 30`.
- `engine/crates/ui/src/screens/key_bindings/glyph.rs`: `PANEL =
  Rect::new(2, 1, 96, 29)`, three slot columns 18 cells apart.

On a smaller console the panels run off the right edge and the values
can't be seen (1118 first tried 48 × 24: the options' values were cut
off). So 1118 gave the example the game's 100 × 32 console. That is
something of the game in the engine's way (ADR-0063 §8.2): a game made
with the engine should pick its own console size. The layout picker's
skin already centres its panels on the buffer's width
(`layout_picker/glyph.rs`, `(buf.width() - PANEL_W) / 2`).

## Nick input

None.

## Scope

**In:**
- The options screen's glyph skin: the panel centred on the buffer's
  width, as wide as `PANEL.w` or the buffer, whichever is narrower; the
  values' column from the panel's left edge. On a 100-cell console it
  stays exactly where it is now (centred: `(100 - 60) / 2 = 20`).
- The Key bindings screen's glyph skin: the panel as wide as the buffer
  allows (keeping its 2-cell margin), the slot columns' pitch and width
  from the panel's width, and the screen showing fewer slot columns, or
  shorter key names cut with `…`, when they don't fit. On a 100-cell
  console nothing moves.
- Hello, Grid's console back to a size of its own (48 × 24, its map at
  1× in the sprite skin), with its snapshots re-accepted after looking at
  them.

**Out (do not do):**
- Changing the game's screens or its console.
- The debug screens (`engine/crates/ui/src/debug/glyph.rs`): developers'
  tools, sized for the game; a ticket of their own if needed.
- A new layout for the screens: they keep their look, only placed by the
  buffer's size.

## Implementation steps

1. In `options/glyph.rs`, replace the fixed `PANEL.x` with a function of
   `buf.width()`; pass the panel down to `paint_row` and the question and
   message painters instead of the constants.
2. In `key_bindings/glyph.rs`, compute the panel and the slot columns
   from `buf.width()`; when three slots don't fit, show as many as fit
   and say which are hidden in the column headings, or cut names (pick
   one and write it in the module's docs).
3. Add a glyph snapshot of each screen on a 48 × 24 console
   (`ConsoleSpec::text(48, 24)`) beside the existing 100 × 32 ones; the
   existing snapshots must not change.
4. Set `hello_grid::CONSOLE` to 48 × 24, its `stage::AREA`, `skin::TALK_BOX`
   and the window to match, and `map::ZOOM` to 1; update its tests' view
   sizes and re-accept its snapshots after looking at them.

## Acceptance criteria

- [ ] Every existing snapshot of the options and Key bindings screens is
      unchanged.
- [ ] New 48 × 24 snapshots show every row's label and value inside the
      panel.
- [ ] Hello, Grid runs on a 48 × 24 console, its options and Key bindings
      screens readable (`cargo shuyi frame-png` from its folder).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Snapshot: the two screens at 48 × 24, in `engine/crates/ui`.
- Integration: Hello, Grid's tests at its new size.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
