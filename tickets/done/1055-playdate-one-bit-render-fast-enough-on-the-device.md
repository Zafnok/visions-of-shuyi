---
id: "1055"
title: "Playdate: the one-bit render fast enough on the device (one frame's time, not 360 ms)"
type: feature
milestone: Post–Chapter 1
model: opus-5.5
effort: high
status: done
blocked_by: ["1033"]
nick_input: sign-off
completed: 2026-10-16
---

# 1055 — The one-bit render fast enough on the device

## Context

Ticket 1033 ran the game on Nick's Playdate (Rev B) and logged, over a
minute each (PR #304):

| Screen | Frames a second | `Game::frame` min / avg / max | Render and copy min / avg / max (frames drawn) |
| ------ | --------------- | ----------------------------- | ---------------------------------------------- |
| Dialogue | 14.8 | 2.7 / 8.8 / 58.7 ms | 336.6 / 358.6 / 387.8 ms (90) |
| Dialogue | 27.2 | 2.0 / 5.4 / 217.2 ms | 345.8 / 360.4 / 384.4 ms (15) |
| Battle | 22.8 | 4.1 / 30.3 / 542.3 ms | 360.0 / 426.1 / 448.7 ms (26) |

The game's own work is fine; the drawing is not. Every frame whose
picture changed costs **about 360–430 ms** in `trpg_ui::onebit::render`
plus the row copy (`crates/playdate/src/pd/entry.rs`, `Running::frame`;
the copy is 12 KB of compares, so nearly all of it is the render). So
while anything moves (a cursor, a dialogue line typing out, an enemy
walking) the screen updates about 3 times a second. A frame at 30 fps is
33 ms, and the game needs some of that too.

The renderer is 1032's (`crates/ui/src/onebit.rs`, ADR-0074). It was
written to be correct and allocation-free, not measured on the device
(1032's notes: "Not measured: the renderer's speed on the device").
Likely costs on a 32-bit Cortex-M7 at 168 MHz (to be measured, not
assumed):

- `i64` coordinates everywhere and a `u128` shift per byte in
  `Target::run`: multi-word arithmetic on a 32-bit core.
- `ink_byte` recomputed for every byte written (8 calls of
  `ink_is_white`, each with `rem_euclid` / `div_euclid`), when a pattern
  byte only depends on the ink and `y % 4`.
- Every cell's background filled one 8 × 16 rectangle at a time, then
  every cell walked again for edges and glyphs (`draw_layers`), and the
  whole 400 × 240 frame redrawn when only one cell changed.
- A backdrop scene (`buf.backdrop()`, the title and scenes) walked cell
  by cell even where its window clips most of it away.

## Nick input

**Sign-off:** sideload the release build as in 1033 and paste the
Simulator console after a minute on a dialogue scene and a minute in the
first battle. Moving the cursor and a line of dialogue typing out should
look smooth, not a few steps a second.

## Scope

**In:**
- **Measure first, on the device.** A way to see where a render's time
  goes on the Playdate: e.g. a debug-only front-end mode that renders a
  few fixed frames (a dialogue screen, the battle) many times and logs
  each phase's time (backgrounds, items, edges, glyphs, invert), with the
  phases timed by the front end around calls the renderer exposes for it,
  or by rendering buffers that hold only one kind of thing. Record the
  before numbers in the completion notes.
- **Make the render fast** where the numbers point: 32-bit arithmetic in
  the hot paths, pattern bytes looked up rather than computed per byte,
  whole-byte fills for whole bytes, and only the cells (and the items
  over them) that changed since the last frame redrawn, if a full render
  stays too slow. The picture must stay exactly the same (1032's tests in
  `crates/ui/src/onebit/tests.rs` and the `frame-png --playdate` output).
- **Goal:** render and copy of a changed frame at most **33 ms** on
  average on the dialogue and battle screens (1033's minute log), max
  under 66 ms. If the goal can't be reached, record the best reached and
  what is left in the completion notes and a follow-up ticket.

**Out (do not do):**
- Any skin or change to what a screen shows (1037 on). Speed only.
- `Game::frame`'s own spikes (217 ms in a dialogue, 542 ms in the battle):
  the enemy AI's are 1051's; others go in a new ticket if the
  measurements here show where they come from.
- The loading time (4.6 s measured in 1033): 1036.
- Changing the refresh rate (1033's README: 30 until measured).

## Implementation steps

1. Read ADR-0074, `crates/ui/src/onebit.rs` and its tests, 1032's
   completion notes, and `crates/playdate/src/pd/entry.rs` (the frame
   loop and the once-a-minute log, `src/report.rs`).
2. Add the device measurement (Scope, *Measure first*); build with
   `cargo xtask playdate --release`, ask Nick to sideload and paste the
   console; note the numbers.
3. Speed up the phases that cost most, one at a time, keeping every
   test in `onebit/tests.rs` and the `frame-png --playdate` PNGs of the
   title, a dialogue and the battle byte-identical (add a test that
   renders those screens through the Harness and compares with the frame
   from before the change, if none does yet).
4. If a full render is still over the goal: render only what changed.
   `GlyphBuffer` comparison per cell (and per item) in the front end or a
   `render_changed(old, new, …)` in `onebit` that redraws the changed
   cells' rectangles (plus any item touching them) into the existing
   frame. Tests: rendering changed cells into the old frame gives exactly
   the full render of the new buffer, for random buffers (property test).
5. Measure on the device again (Nick sideloads); put before / after in
   the completion notes and the PR.

## Acceptance criteria

- [x] The before and after device numbers (per phase, and 1033's minute
      log on dialogue and battle) are in the completion notes and PR.
- [x] Render and copy of a changed frame averages at most 33 ms on the
      dialogue and battle screens on Nick's Playdate, or the notes say
      what was reached and name the follow-up ticket.
- [x] The rendered frames are unchanged: `onebit` tests pass as they
      are, and the before/after frame test passes.
- [x] All gates in the `run-gates` skill pass, and the CI `playdate` job.

## Tests required

- Unit: any new helpers (pattern byte tables, 32-bit run masks).
- Property: if changed-cell rendering is added, it equals a full render.
- Snapshot / integration: the title, a dialogue and the battle frames
  identical before and after.
- Manual: Nick's device run.

## Completion notes

Done 2026-10-16. ADR-0077 (*The one-bit render redraws only what
changed, against a kept reference*) records the design.

**What was done**

- **A faster full render** (`trpg_ui::onebit::render`), same order and
  pixels: 32-bit screen coordinates; each ink's byte for its 4 rows in a
  compile-time table (`PATTERNS`); fills write their middle bytes whole;
  neighbouring cell backgrounds of one ink filled as one rectangle; a
  glyph row at a whole cell inside the clip written as one byte; only the
  cells that may reach into the clip visited (`Reach::span`, which also
  stops a backdrop's scene being walked where its window cuts it away);
  the ASCII glyphs' atlas offsets in a table in `onebit::Font`; the last
  colour's ink remembered while walking cells.
- **Redraw only what changed** (`onebit::render_changed(old, new, ...)`):
  the screen's 8 × 16 tiles touched by a changed cell (grown a pixel for
  edges), a changed item (old and new) or a changed backdrop window are
  redrawn, each run of them from the white ground under a clip, so the
  result is the full render by construction. The front end
  (`crates/playdate/src/pd/entry.rs`) draws the first frame whole and
  every later changed frame this way.
- **1032's renderer kept as the reference** (`onebit::reference`, with
  phase marks added; `cfg(test)` or the `onebit-reference` feature, never
  in a shipped build).
- **Measuring**: `render_marked(..., mark)` in both renderers;
  `cargo xtask playdate --release --bench` builds
  `VisionsOfShuyi-bench.pdx` (feature `bench`, `src/pd/bench.rs`), which
  renders each drawn frame again with both renderers, times every phase,
  checks the frames match and logs it with the minute report
  (`report::Phases`). CI clippies the bench build too.
- **Tests**: `onebit/tests.rs` untouched and passing. New
  `onebit/speed_tests.rs`: property tests (random cells, inks,
  see-through cells, rectangles, masked sprites, zoomed and panned
  backdrops) that the fast render equals the reference, and that
  `render(old)` + `render_changed(old, new)` equals `render(new)` over
  random edits (cells, items, fills that cut items, backdrops); the
  pattern table against `ink_is_white` on every pixel; the ASCII table
  against the glyph map (with and without a fallback); the one-byte run
  path; tile runs; phase marks. `tests/it/playdate.rs` gained
  `the_one_bit_frames_are_the_references_whole_or_changed`: the title,
  the opening scene typing out frame by frame and the first battle with
  its cursor moving, on the Playdate's console and palette, each frame
  whole and changed-only, all equal to the reference (this is the
  before/after frame test: the reference *is* the before). `report.rs`,
  `xtask playdate` (`--bench`) tests.

**Measured** (PC, release, a whole frame, reference → fast): title 251 →
5.7 µs, dialogue 203 → 3.8 µs, a scene over the battle 235 → 8.7 µs,
battle 266 → 12.5 µs.

**Measured on Nick's Playdate** (Rev B, bench build, 2026-10-16; the
per-phase lines are each drawn frame rendered whole again with each
renderer):

| A minute on | Frames a second | `Game::frame` min / avg / max | Render and copy min / avg / max (frames drawn) |
| ----------- | --------------- | ----------------------------- | ---------------------------------------------- |
| Dialogue (1033) | 14.8 | 2.7 / 8.8 / 58.7 ms | 336.6 / 358.6 / 387.8 ms (90) |
| Battle (1033) | 22.8 | 4.1 / 30.3 / 542.3 ms | 360.0 / 426.1 / 448.7 ms (26) |
| **Dialogue (now)** | 19.0 | 2.3 / 14.3 / 53.5 ms | **5.4 / 16.9 / 40.6 ms** (53) |
| **Battle (now)** | 27.3 | 18.2 / 25.5 / 432.3 ms | **9.8 / 17.6 / 25.7 ms** (9) |

A whole frame, min / avg / max:

| Screen | 1032's render | Fast render |
| ------ | ------------- | ----------- |
| Dialogue | 335.5 / 400.7 / 574.9 ms | 8.7 / 13.4 / 23.7 ms |
| Battle | 392.9 / 441.5 / 488.3 ms | 14.2 / 16.6 / 20.1 ms |

Each phase, avg / max ms (dialogue; battle):

| Phase | 1032's | Fast |
| ----- | ------ | ---- |
| ground | 0.3 / 0.3; 0.3 / 0.3 | 0.3 / 1.3; 0.3 / 0.3 |
| backdrop | 0.0; 0.0 | 0.0; 0.0 |
| backgrounds | 316.9 / 336.2; 321.8 / 336.7 | 2.1 / 3.8; 2.9 / 3.9 |
| under items | 19.9 / 28.4; 5.2 / 16.6 | 1.1 / 2.4; 0.3 / 0.7 |
| edges | 5.8 / 7.0; 5.6 / 6.8 | 3.9 / 5.2; 4.0 / 4.4 |
| glyphs | 48.7 / 252.7; 108.4 / 129.7 | 5.1 / 17.2; 9.0 / 10.8 |
| over items | 9.2 / 14.6; 0.1 / 0.3 | 0.8 / 2.6; 0.1 / 0.1 |
| invert | 0.0; 0.0 | 0.0 / 0.8; 0.0 |

No frame on the device differed from 1032's render, and no changed-only
redraw differed from a whole render. **Goal met**: render and copy of a
changed frame averages 16.9 ms (dialogue) and 17.6 ms (battle), at most
40.6 ms. The "Render and copy" line in the bench build times only what
the normal build does (the bench's renders come after it).

The normal build (`VisionsOfShuyi.pdx`, same day), a minute each:

| A minute on | Frames a second | `Game::frame` min / avg / max | Render and copy min / avg / max (frames drawn) |
| ----------- | --------------- | ----------------------------- | ---------------------------------------------- |
| Dialogue | **29.7** (1033: 14.8) | 2.4 / 18.0 / 57.4 ms | **3.2 / 10.0 / 40.8 ms** (182) |
| Battle | **29.2** (1033: 22.8) | 18.1 / 27.5 / 484.6 ms | **12.4 / 17.1 / 24.1 ms** (13) |

Both screens now run at the 30 frames a second the front end asks for.

**Deviations**

- *Measure first*: instead of a device run before any change and another
  after, one bench build carries both renderers and times them on the
  same frames, so Nick sideloaded once. The PC's phase timings guided the
  work (backgrounds were 80% of the time there too).
- The before/after frame test compares with 1032's renderer kept in the
  code rather than with frames saved from before: saved frames would
  break whenever a screen's content changes.

**Follow-up**: **1057**, where the battle's `Game::frame` time goes. The
game's own frame in that battle cost 18 ms at the least and 432 ms once,
not the render; it is not the AI's per-command planning (1051). Not in
1050's `blocked_by`: the battle runs at 27 frames a second.

No gameplay rules were decided here.
