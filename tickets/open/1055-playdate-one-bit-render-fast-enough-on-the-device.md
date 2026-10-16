---
id: "1055"
title: "Playdate: the one-bit render fast enough on the device (one frame's time, not 360 ms)"
type: feature
milestone: Post–Chapter 1
model: opus-5.5
effort: high
status: in-progress
blocked_by: ["1033"]
nick_input: sign-off
completed:
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

- [ ] The before and after device numbers (per phase, and 1033's minute
      log on dialogue and battle) are in the completion notes and PR.
- [ ] Render and copy of a changed frame averages at most 33 ms on the
      dialogue and battle screens on Nick's Playdate, or the notes say
      what was reached and name the follow-up ticket.
- [ ] The rendered frames are unchanged: `onebit` tests pass as they
      are, and the before/after frame test passes.
- [ ] All gates in the `run-gates` skill pass, and the CI `playdate` job.

## Tests required

- Unit: any new helpers (pattern byte tables, 32-bit run masks).
- Property: if changed-cell rendering is added, it equals a full render.
- Snapshot / integration: the title, a dialogue and the battle frames
  identical before and after.
- Manual: Nick's device run.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
