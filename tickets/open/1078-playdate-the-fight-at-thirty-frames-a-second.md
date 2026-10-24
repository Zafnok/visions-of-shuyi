---
id: "1078"
title: "Playdate: the fight at 30 frames a second: move the pin past engine#3056 and time the combat picture on the device"
type: feature
milestone: Post–Chapter 1
model: opus-5.5
effort: high
status: todo
blocked_by: ["1041", "engine#3056"]
nick_input: setup
completed:
---

# 1078 — The Playdate's fight at 30 frames a second

## Context

1041 put the one-bit combat picture on the Playdate
(`crates/ui/src/screens/battle/playdate/stage.rs`). On Nick's Playdate
(Rev B, release build with the bought art, Chapter 1's first battle,
2026-10-23) the fights stutter. Nick: "in-battle fight scene is the
choppiest. It drops a lot of frames and causes audio hitching."

| A minute on the battle, with fights | Frames a second | Render and copy min / avg / max (frames drawn) |
| ----------------------------------- | --------------- | ---------------------------------------------- |
| 1 | 24.3 | 6.9 / 136.4 / 409.3 ms (21) |
| 2 | 14.5 | 6.1 / 169.3 / 637.9 ms (57) |
| 3 | 17.7 | 6.1 / 136.9 / 600.2 ms (29) |

A frame at 30 a second has 33 ms. While a fight plays every frame
changes, and the render is most of its cost: the one-bit renderer draws
a sprite a pixel at a time, with two 64-bit multiply-divides for each
pixel (`shuyi_ui::frame::onebit::draw_sprite`, `Image::sample`), and the
stage has a backdrop 400 × 110 pixels and two fighters up to 160 × 192
each. Engine ticket **engine#3056** gives sprites a fast path: whole
bytes for a picture shown pixel for pixel, as the stage's are, and no
division per pixel otherwise. This ticket takes it into the game and
measures again. Row in `docs/engine/friction.md` (1041).

`Game::frame`'s own cost in a battle (27.9–54.3 ms on average in the
same minutes) is **1057**'s, not this ticket's.

## Nick input

**Setup** (a measurement, not a check): sideload the bench build
(Simulator: Device → Upload Game to Device), play a few fights in
Chapter 1's first battle (Tamsin reaches a brigand on turn 1; the crank
rewinds a fight to play it again), and paste the Simulator console's
minute reports.

## Scope

**In:**
- Move the engine pin past engine#3056 (`cargo xtask engine-pin`, the
  `engine-change` skill's step 7), with whatever small fixes the game
  needs to build on it.
- **Measure on the device** with the bench build (`cargo xtask playdate
  --release --bench --private-assets`, `crates/playdate/src/pd/bench.rs`):
  the render's phases during fights (its `under items` phase is the
  sprites).
- If the fight is still over 33 ms a frame, cut on the game's side
  without changing what the PC shows or what the fight does, in this
  order of preference, each measured: draw the fighters' backdrop band
  once and the fighters over it only where they moved (if the renderer's
  changed rows allow it); a narrower backdrop band (the horizon is
  *tunable*); and, last, a lower refresh rate during a fight only. What
  is cut and what it bought go in the completion notes.

**Out (do not do):**
- The renderer itself (engine#3056; a further engine change gets its
  own engine ticket).
- `Game::frame`'s cost (1057), the enemy AI's planning (1051), the
  refresh rate of the map (1059).
- Changing the look 0062 and 1041 decided (`docs/design/playdate.md`,
  *The combat picture*), except the tunables named above.

## Implementation steps

1. Read 1041's completion notes, engine#3056's, ADR-0077 and
   `crates/playdate/README.md` (*What it measures*).
2. Move the pin; run the gates.
3. Build the bench build and the normal one; ask Nick for the console
   (one Nick turn for both).
4. If needed, the cuts in the Scope's order, each measured on the PC
   first (a release-mode timing test of a fight frame through the
   Harness on `ConsoleSpec::PLAYDATE`), then on the device.

## Acceptance criteria

- [ ] The device numbers of a minute of fights, before (the table above)
      and after, are in the completion notes and the PR.
- [ ] A fight renders in 33 ms a frame or less on average on Nick's
      Playdate, or the notes say what was reached and name the
      follow-up ticket.
- [ ] No PC snapshot changes; the Playdate's stage look tests pass
      unchanged unless a tunable changed (then say which).
- [ ] All gates in the `run-gates` skill pass, and the CI `playdate` job.

## Tests required

- Unit: a game-side cut keeps the stage's look tests
  (`battle/playdate/stage/tests.rs`) passing, or changes only what the
  tunable changed.
- Manual: Nick's device run.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
