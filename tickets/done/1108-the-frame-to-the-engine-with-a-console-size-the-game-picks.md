---
id: "1108"
title: "The frame (glyph buffer, sprites, backdrop, snapshots) moves to the engine, with a console size the game picks"
type: infra
milestone: "Engine: carve"
model: reasoning
effort: high
status: done
blocked_by: ["1107", "1030"]
nick_input: none
completed: 2026-10-16
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
(it knows nothing about this game), with its `reference` renderer and the
`onebit-reference` feature the Playdate's bench build turns on (1055,
ADR-0077), and so do `trpg_content::Ink` and the Playdate palette loader
(ADR-0074). The moved crates keep the default
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

- [x] No snapshot of the game changes.
- [x] The 40×20 test passes in `shuyi-ui` with no game crate.
- [x] `cargo xtask frame-png` still renders the same PNGs for the game
      (compare one before and after).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: moved frame tests; the new console-size test.
- Snapshot: unchanged game snapshots.

## Completion notes

Stacked on 1107's PR.

- **`shuyi-ui::frame`** (`git mv` from `trpg-ui`): `glyph_buffer` (with
  the backdrop, items and layers), `snapshot`, `console`, `cinema`,
  `onebit` (with `reference`) and `color`, and `shuyi_ui::float` (the
  `libm` float maths for `no_std`). The frame imports `ImageId`, `Light`,
  `FontAtlasDef` and `Ink` from `shuyi_assets`. `portrait` is cut: fitting
  a picture to an area of cells at a whole scale (`fit_whole_scale`,
  `dim_opacity`, `draw_fitted`) is the engine's; the 32 × 16-cell frame
  and drawing a `Portrait`'s expression stay the game's.
- **Console size:** 1030 had already put `ConsoleSpec` in the frame, so
  step 1 was skipped. The engine's `ConsoleSpec` has no sizes of its own,
  only `ConsoleSpec::text(cols, rows)` (8 × 16-pixel cells, which
  `GlyphBuffer::new` uses). The game's two sizes are its own:
  `ConsoleSpec::PC` and `ConsoleSpec::PLAYDATE` stay spelled that way
  through the game's `trpg_ui::console::ConsoleSizes` trait (imported
  where they are named), so no call site changed beyond its `use` line.
- **Colors:** the engine's `Palette` is built for a game's `ColorSet`
  (`Palette::new::<UiColor>(&def)`) and also holds six `EngineColor`s for
  the engine's own widgets (`text`, `text_dim`, `panel`, `border`,
  `highlight`, `warning`). `required_names::<C>()` is the palette check:
  the set's names, then the engine names it lacks. `UiColor` stays the
  game's enum; it implements `ColorSet`, and `engine_color` says which of
  its colors each engine color is.
- **Deviation:** the ticket asked that the game's palette define the
  engine names. Renaming `panel_bg`, `panel_border` and `text_highlight`
  in the palette files would have changed every snapshot (they print
  color names), and `warning` has no color in this game. So a game's
  `ColorSet` maps each engine color onto one of its own names (by default
  the engine's own name): here `panel_bg`, `panel_border`,
  `text_highlight`, and `hp_low` for `warning`, which no widget paints yet.
  The palette check is still "the engine's names plus the game's": all
  six map into `REQUIRED_COLORS`, and a test holds
  `required_names::<UiColor>()` to it.
- **Tests:** the frame's tests moved with it and run on a small engine
  palette (`color::tests::engine_palette`) and made-up image tables.
  Tests that need this game stay in the game: the `sample_box` snapshot
  (`trpg_ui::snapshot`, so its `.snap` keeps its name), the one-bit render
  of the game's font, the PC and Playdate sizes, the portrait tests with
  `Portrait`s, and the palette tests on the game's palette. New:
  `shuyi_ui::frame::tests::a_40_by_20_console_of_8_by_8_cells` (text, a
  sprite, a clipped rectangle and a backdrop, snapshotted), and tests of
  `ColorSet`, `EngineColor` and `draw_fitted` on other cell sizes.
- **Rebased** after 1055's last commit (renderer tests for CI's mutants)
  landed under 1106: its changes to `onebit.rs` and `onebit/speed_tests.rs`
  are in the engine's copies, its proptest regressions file moved to
  `engine/crates/ui/proptest-regressions/frame/onebit/`.
- **Compared PNGs:** `frame-png` rendered six frames (title, battle,
  debug menu, battle at dusk, Playdate title and battle) from 1107's
  branch and from this one: all six are byte-identical.
- Small changes: `onebit::Font::rows` and `color::to_channel` are `pub`
  (the game uses them across the crate line); `Float`'s methods are
  `#[must_use]`. `shuyi-ui` gains `libm`, an `onebit-reference` feature
  (`trpg-ui`'s turns it on) and `insta` for its tests. `trpg-ui` drops
  `libm` and `hashbrown`. Paths to the moved files in tickets 0236 and
  1044 and in `crates/ui/README.md` point to the engine.
- **Not checked locally:** `cargo xtask playdate` compiles the Rust, but
  the SDK's `pdc.exe` crashes (0xc0000005) on this machine while packaging.
  That step reads no code this ticket changed; CI's `playdate` job runs it.

No gameplay rules were decided. Nick input: none.
