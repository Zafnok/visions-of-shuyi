---
id: "1166"
title: "Frame time and hurried clocks from the engine: the 13 dt cleanups and three hold-to-hurry clocks go"
type: feature
milestone: "Engine: carve"
model: sonnet-5
effort: medium
status: todo
blocked_by: ["engine#2312"]
nick_input: none
completed:
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
  became private, `input.dt` becomes `input.dt()`.
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

- [ ] `grep -rn "dt.is_finite()" crates/ui/src` finds nothing outside
      tests.
- [ ] The progress, playback, AI phase and conversation tests pass
      unchanged.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- The existing tests, unchanged.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
