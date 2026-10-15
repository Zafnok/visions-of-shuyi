---
id: "1108"
title: "The frame (glyph buffer, sprites, backdrop, snapshots) moves to the engine, with a console size the game picks"
type: infra
milestone: "Engine: carve"
model: opus-5.5
effort: high
status: todo
blocked_by: ["1107", "1030"]
nick_input: none
completed:
---

# 1108 — The frame moves to the engine; the console size is the game's

## Context

ADR-0063 §6; `docs/engine/boundary.md`, couplings 4 and 5. The frame
(`ui/src/glyph_buffer*`, `snapshot.rs`, `console.rs`, `cinema.rs`,
`portrait.rs`, `color.rs`) is the heart of the engine, but:

- the console is a fixed 100×32 cells of 8×16 pixels
  (`console.rs:4-10`), used in 15 files in `ui`, `app` and `xtask`, plus
  `pixel_bounds` (`glyph_buffer.rs:477`) and `Backdrop::clip_px`;
- the frame imports `ImageId` and `Light` (fine after 1104: they are
  engine types now);
- `UiColor` (`color.rs:119-144`) is a closed list holding this game's
  colors (player, enemy, ally, move range, attack range, danger zone,
  HP), and `Palette::new` requires all of them.

## Nick input

None. Nothing a player sees changes.

## Scope

**In:**
- `ConsoleSpec { cols, rows, cell_w, cell_h }` passed in when the game
  starts and kept by `GlyphBuffer`; every use of the constants reads the
  spec. The game passes today's numbers.
- Colors by name: the engine's palette is named colors (1104); `UiColor`
  becomes the game's enum mapping to names, and widgets the engine owns
  ask for a small fixed set of engine names (`text`, `text_dim`,
  `panel`, `border`, `highlight`, `warning`), which the game's palette
  must define.
- Move to `shuyi-ui::frame`: `GlyphBuffer`, items, layers, backdrop,
  `snapshot`, `console` (letterboxing), `cinema` (framing math),
  `portrait` (fit an image at a whole scale), `Rgb` and `Palette`.
- Aliases in `trpg-ui`.

**Playdate:** 1030 (in `blocked_by`) already added `ConsoleSpec` in
place in `trpg-ui`, with `ConsoleSpec::PC` and `ConsoleSpec::PLAYDATE`
(50 × 15): skip step 1 and move it with the frame. If the one-bit
renderer (1032, `trpg_ui::onebit`) exists, it moves with the frame too
(it knows nothing about this game). The moved crates keep the default
`std` feature and stay in the `no-std` CI build (ADR-0070 §6), and the
Playdate front end (`crates/playdate`, 1031) passes
`ConsoleSpec::PLAYDATE` and must keep building (`cargo xtask playdate`).

**Out (do not do):** world layers, cameras and sort keys (1120); the
renderer in `app` (1110).

## Implementation steps

1. Introduce `ConsoleSpec` in place; replace the constants (the battle
   layout keeps its own numbers, derived from the spec where it can).
2. Split `UiColor` as above; the palette check lists the engine names
   plus the game's required names.
3. Move the modules with their tests.
4. A test builds a 40×20 console of 8×8 cells, draws text, a sprite and a
   backdrop, and snapshots it.

## Acceptance criteria

- [ ] No snapshot of the game changes.
- [ ] The 40×20 test passes in `shuyi-ui` with no game crate.
- [ ] `cargo xtask frame-png` still renders the same PNGs for the game
      (compare one before and after).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: moved frame tests; the new console-size test.
- Snapshot: unchanged game snapshots.

## Completion notes
