---
id: "1166"
title: "Frame time and hurried clocks from the engine: the 13 dt cleanups and three hold-to-hurry clocks go"
type: feature
milestone: "Engine: carve"
model: sonnet-5
effort: medium
status: done
blocked_by: ["engine#2312"]
nick_input: none
completed: 2026-10-20
---

# 1166 — Frame time and hurried clocks from the engine

## Context

The first engine-friction sweep (1153, `docs/engine/friction.md`) found:

- `if dt.is_finite() { dt.max(0.0) } else { 0.0 }` (or the same with
  `input.dt`) 13 times in `crates/ui/src`
  (`grep -rn "dt.is_finite()" crates/ui/src`);
- "a held key makes time pass faster; a press finishes or goes on"
  written three times: `screens/battle/progress.rs` (the clock of the
  progress pages), `screens/battle/playback.rs`,
  `screens/battle/ai_phase.rs`;
- the conversation screen (`screens/dialogue.rs`) writing "Confirm
  reveals the page, else turns it, else the box is done" by hand.

Engine ticket 2312 (`Zafnok/shuyi-engine`) promises a clean `dt`, adds
a hurry clock and `TextBox::confirm`.

## Nick input

None.

## Scope

**In:**
- Move the pin past 2312 (`engine-change` skill, step 7); if `dt`
  became private, `input.dt` becomes `input.dt()`. (Done by 1167, whose
  pin move to engine 2313 included 2312: the pin is past it and every
  `input.dt` is `input.dt()`.)
- The 13 cleanups go.
- The three clocks use the engine's hurry clock, each with its own
  speed-up and delay as today.
- The conversation screen uses `TextBox::confirm`.

**Out (do not do):**
- Any change in speeds or timing the player can feel: a clock that ran
  at 4× after 0.3 s still does.

## Implementation steps

1. Write the estimate: about 90 lines removed, 20 added.
2. Move the pin; the cleanups; then each clock with its tests.

## Acceptance criteria

- [x] `grep -rn "dt.is_finite()" crates/ui/src` finds nothing outside
      tests.
- [x] The progress, playback, AI phase and conversation tests pass
      unchanged.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- The existing tests, unchanged.

## Completion notes

- The pin was already past engine 2312 (moved by 1167).
- The frame-time cleanups: 8 were left in `crates/ui/src`, not 13 (the
  others went with earlier carve tickets). The cue queue, the walk in
  `mode.rs` (twice) and the title song's clock use the engine's
  `frame_seconds`, since their own tests hand them a NaN frame time; the
  credits roll only ever gets `FrameInput::dt()`, so its check just goes.
- The three clocks: the progress pages and the fight playback each pass
  time through a `HurryClock` with no delay and their `fast` factor
  (they hurried from the first held frame, as they still do). The AI
  action uses two: the pan and highlight at `fast`, the walk at its held
  speed, split where the walk starts, as before. Their own "a press
  finishes" logic stays (the progress page's `confirm`, the playback's
  Cancel): it already sits in one place each.
- The conversation screen's Confirm is `TextBox::confirm`: reveal, else
  next page, else the next box.
- No timing or speed changed; every existing test passes unchanged.
- Friction: estimate 20 added, measured 34 (1.7×), under 2×. Fewer lines
  went than the estimate's 90 because only 8 cleanups were left.
- No gameplay rules decided.
