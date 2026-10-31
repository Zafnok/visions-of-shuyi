---
id: "1155"
title: "The engine's screens fit the game's console: options and Key bindings laid out from the frame's size, not a fixed 100 × 32"
type: feature
milestone: "Engine: carve"
model: implementation
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-10-18
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

- [x] Every existing snapshot of the options and Key bindings screens is
      unchanged.
- [x] New 48 × 24 snapshots show every row's label and value inside the
      panel.
- [x] Hello, Grid runs on a 48 × 24 console, its options and Key bindings
      screens readable (`cargo shuyi frame-png` from its folder).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Snapshot: the two screens at 48 × 24, in `engine/crates/ui`.
- Integration: Hello, Grid's tests at its new size.

## Completion notes

- **Options skin** (`options/glyph.rs`): the panel is centred on the
  buffer's width, `min(60, width)` wide; labels and values are placed from
  its left edge. On a 100-cell console it is where it was (x = 20).
- **Key bindings skin** (`key_bindings/glyph.rs`): a `Layout` computed
  from the buffer's size. The panel keeps its 2-cell margin, at most 96
  wide and centred. Chosen of the ticket's two options: **every slot stays
  shown and names that don't fit are cut with `…`** (hiding slots would
  leave the cursor able to sit on one the player can't see). On a narrower
  panel the label column is as wide as the longest label (with its fixed
  keys only if each slot still gets 9 cells, enough for "Backspace"; else
  the fixed keys are cut), and the slots share the rest. The not-mapped
  note, with no room after the slots, is written over them (they are all
  empty then), clear of the focused slot's bar. The choices box is kept
  inside the panel's right edge. On 100 × 32 nothing moves (a test pins
  the layout to the old constants).
- **Deviation:** both panels also fit the console's **height** (the
  ticket only named the width): the options panel's smallest height
  leaves room for its message under it, and the Key bindings panel reaches
  down to the row over the message (`height - 3`, i.e. 29 on 32 rows as
  before). Without it the message fell off a 24-row console.
- New snapshots at 48 × 24 in `engine/crates/ui/src/screens/*/snapshots/`;
  the game's own (`crates/ui`) snapshots are unchanged.
- **Hello, Grid** at 48 × 24: `CONSOLE`, `stage::AREA` (0, 1, 48, 22),
  `skin::TALK_BOX` (bottom 7 rows), the window (784 × 784), `map::ZOOM` 1,
  and also `Talk::WIDTH` 90 → 42 (the scene's text ran past the box) with
  its long-line test now 200 characters. Snapshots re-accepted after
  looking at each (stage in glyphs and tiles, title, end); the options and
  Key bindings screens checked with `cargo shuyi frame-png`.
- **Follow-up:** 1156 — a help line wider than the console loses both
  ends (`print_centred`); Hello, Grid's Key bindings help shows it.
- Nothing for Nick to play: the game's screens don't change.
