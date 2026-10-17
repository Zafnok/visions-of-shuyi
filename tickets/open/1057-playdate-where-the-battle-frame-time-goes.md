---
id: "1057"
title: "Playdate: find and cut where the battle's Game::frame time goes (18 ms a frame at least on the device)"
type: feature
milestone: Post–Chapter 1
model: opus-5.5
effort: high
status: todo
blocked_by: []
nick_input: sign-off
completed:
---

# 1057 — Where the battle's `Game::frame` time goes on the Playdate

## Context

1055 made the one-bit render fast (ADR-0077). Its bench run on Nick's
Playdate (Rev B, release build, 2026-10-16, PR for 1055) also logged the
game's own frame:

| A minute on | Frames a second | `Game::frame` min / avg / max | Render and copy avg / max (frames drawn) |
| ----------- | --------------- | ----------------------------- | ---------------------------------------- |
| Dialogue | 19.0 | 2.3 / 14.3 / 53.5 ms | 16.9 / 40.6 ms (53) |
| Battle | 27.3 | **18.2** / 25.5 / **432.3** ms | 17.6 / 25.7 ms (9) |

In the battle the game itself costs at least 18 ms every frame, even on
frames whose picture doesn't change (only 9 of about 1,640 frames were
drawn), so it alone uses most of a 30 fps frame (33 ms); and one frame
took 432 ms. 1033's minute in a battle had a minimum of 4.1 ms, so what
the battle does each frame depends on its state (whose phase, the
cursor, idle motion, the guide). Nobody knows yet where the time goes.
The normal build the same day: battle `Game::frame` 18.1 / 27.5 / 484.6
ms at 29.2 frames a second; a dialogue scene 2.4 / 18.0 / 57.4 ms (1033:
8.8 ms on average), so a scene's frames are worth timing too.

The enemy AI's per-command replanning is **1051**'s (a pause before each
enemy acts). This ticket is the rest: the steady cost of a battle frame
in the player's phase, and spikes that aren't the AI.

## Nick input

**Sign-off:** sideload the release build as in 1033/1055 and paste the
Simulator console after a minute in Chapter 1's first battle, moving the
cursor around and opening a menu. The battle should run near 30 frames a
second.

## Scope

**In:**
- **Measure first, on the device.** Extend the Playdate bench build
  (`cargo xtask playdate --release --bench`, `crates/playdate/src/pd/bench.rs`,
  ADR-0077 §4) to time the parts of `Game::frame` in a battle: input,
  the screen's `update`, building the map scene, the skins painting the
  buffer, anything else the numbers point at. Time them from the front
  end around calls `trpg-ui` exposes for it (as `onebit::render_marked`
  does), never with a clock inside `core` or `ui` (ADR-0004). Record the
  numbers in the completion notes.
- Cut the biggest costs where the numbers point, without changing what
  any screen shows or what the game decides (snapshots and play records
  unchanged).
- Find what the 432 ms spike was (log the slowest frame's parts) and fix
  it, or name the ticket that owns it.
- **Goal:** `Game::frame` in a battle's player phase at most 10 ms on
  average on the device, the minute's frame rate at least 28. If it
  can't be reached, record the best reached and what is left in a
  follow-up ticket.

**Out (do not do):**
- The enemy AI's planning (1051).
- The render (1055, done) and the loading time (1036).
- Changing the refresh rate (1060 takes the battle map to 50 after
  this), any skin or what a screen shows.

## Implementation steps

1. Read ADR-0004, ADR-0070, ADR-0077 and 1055's completion notes;
   `crates/playdate/src/pd/entry.rs` (`Running::frame`) and
   `src/pd/bench.rs`; `trpg_ui::Game::frame` and the battle screen
   (`crates/ui/src/screens/battle*`).
2. Time a battle frame's parts on the PC first (a release-mode timing
   test like 1055's probe, through the Harness on `ConsoleSpec::PLAYDATE`
   in the first battle) to know where to put the device's marks.
3. Add the marks to the bench build, build it, ask Nick to sideload and
   paste the console (one Nick turn: also hand him the normal build).
4. Cut the biggest parts; keep every test and snapshot as it is.
5. Measure on the device again; put before / after in the completion
   notes and the PR.

## Acceptance criteria

- [ ] The device numbers per part of a battle frame, before and after,
      are in the completion notes and the PR.
- [ ] `Game::frame` in the battle averages at most 10 ms on Nick's
      Playdate, or the notes say what was reached and name the
      follow-up ticket.
- [ ] No snapshot, play record or bot result changes.
- [ ] All gates in the `run-gates` skill pass, and the CI `playdate` job.

## Tests required

- Unit: any new timing report lines (`crates/playdate/src/report.rs`).
- Property / integration: whatever replaces work done each frame (a
  cache, work done once) gives the same result as before, over the
  Harness's battle frames.
- Manual: Nick's device run.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
