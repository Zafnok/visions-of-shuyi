---
id: "1417"
title: "The enemy and Other phases' move playback on the grid kit's"
type: feature
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: blocked
blocked_by: ["1217", "1228", "engine#3256"]
nick_input: none
completed:
---

# 1417 — Enemy-phase playback from the grid kit

## Context

The third engine-friction sweep (1397) measured
`crates/ui/src/screens/battle/ai_phase.rs` at 182 code lines against
about 40. When an AI unit moves, its `AiAction` pans the camera, marks
the unit, walks it along its path with step sounds, and hurries while
Confirm is held. This is a sequence every grid-tactics game plays.
Engine ticket 3256 adds it to the grid kit as `MovePlayback`, with the
pacing as data. The game keeps its `PACING` numbers, its unit list and
what follows the move (the combat's playback).

## Nick input

None.

## Scope

**In:**
- Move the pin past engine 3256 (`engine-change` skill, §7) if it isn't
  there yet.
- `AiAction` holds a `MovePlayback` plus `unit`, `before` and `then`.
  Its clock, times, steps, gait and camera methods forward to the
  engine's or go.
- `Pacing` becomes the engine's `MovePacing`, with the game's numbers
  (0.25 s pan, 0.2 s highlight, the map skin's walk speeds, ×4 held).

1217 and 1228 come first: they put `ai_phase.rs`'s walk helpers (1217)
and its camera lerp (1228) on the engine. This ticket then replaces what
is left in one pass, instead of fighting them over the same functions.

**Out (do not do):**
- Any change to the timings or to what the player sees.
- Choosing the AI's moves.

## Implementation steps

1. Write the engine-friction estimate first: about 140 lines removed and
   30 added.
2. Move the pin and rebuild `AiAction` on `MovePlayback`. The battle
   screen's calls (`tick`, `tiles_entered`, `gait`, `camera`,
   `shows_cursor`, `units`, `repan`) keep their names.
3. Delete the marked code (`// engine-friction: engine#3256`).
4. Keep `ai_phase.rs`'s tests and the battle's AI-phase snapshots
   passing unchanged.

## Acceptance criteria

- [ ] `grep -rn "engine#3256" crates` finds nothing.
- [ ] The `ai_phase` unit tests and the battle's AI-phase tests pass
      unchanged.
- [ ] A seeded replay of a test battle's enemy phase gives the same
      state hashes and the same step-sound count as before.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit/integration: the existing ones.

## Completion notes

*(Filled in by the session that completes the ticket.)*
