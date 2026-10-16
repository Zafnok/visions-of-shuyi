---
id: "0467"
title: "The Magic and Terrain tips never showed: a tip counts as seen only once it's shown"
type: bug
milestone: M3 Battle UI
model: sonnet-5
effort: medium
status: todo
blocked_by: ["1118", "1050"]
nick_input: none
completed:
---

# 0467 — The Magic and Terrain tips never showed

## Context

0458 added three tips at Full: Skills, Magic and Terrain
(`assets/data/tips.ron`: `magic`, trigger `FirstCaster`, lines 88–94;
`terrain`, trigger `FirstTerrain`, lines 95–101). Chapter 1 playtest
round 3 (0804, 2026-10-16), on a fresh profile at Not familiar:

> "I didn't get the magic trigger for some reason, or are you saying
> these will be added later? also didn't get terrain"

They are in the build, so this is a bug. Read 2026-10-16 (a likely
cause, not yet proven):

- **A tip is marked seen when it's queued, not when it's shown.**
  `TipState::absorb` calls `seen.mark(...)` (`crates/ui/src/screens/
  battle/tips.rs:51-56`). A queued tip that never reaches the screen is
  lost for that profile for good. A queue is dropped on Retry (a new
  `BattleScreen`, `flow.rs:445`, `TipState::default()` at
  `screens/battle/mod.rs:489`), on suspend/quit, or when the battle ends.
- **Tips wait** while a lesson is open (`shown_tip`, `mod.rs:888-891`)
  and for many other things (`due_tip`, lines 898–913). At Full, turn 1
  has a long chain of lessons.
- **Frame order lets lessons go first:** tips are absorbed at the start
  of `update` (`mod.rs:2517-2518`), lessons again at its end
  (2598–2601), so a lesson with the same frame's trigger opens before
  the tip is queued.
- Triggers: `FirstTerrain` only in `Mode::Idle` with the cursor on
  special terrain (`mod.rs:806-813`); `FirstCaster` only in
  `Mode::ActionMenu` with a Magic entry (815–818). Seen tips live in the
  profile (`crates/ui/src/tips.rs:59`).

**Waits for the engine carve and the Playdate port** (Nick, 2026-10-16),
so `blocked_by` has 1118 and 1050.

## Nick input

None.

## Scope

**In:**
1. Reproduce first, as failing tests on Chapter 1 at Full from a fresh
   profile: (a) open Rue's or Maud's action menu while a lesson is open,
   close the lesson, assert the Magic tip shows; (b) rest the cursor on a
   forest tile with nothing selected while the turn-1 lessons run, then
   finish them, assert the Terrain tip shows; (c) queue a tip, Retry the
   battle, assert the tip shows in the new try.
2. Fix: mark a tip seen **when it's shown**; a queued tip survives until
   shown (kept across Retry within the session, or re-queued when its
   trigger fires again, *say which*).
3. If (a) or (b) still fails after (2), find the other cause (frame
   order, a trigger that only fires once) and fix it.

**Out (do not do):** new tips; the zoom tip (0466).

## Acceptance criteria

- [ ] Tests (a), (b), (c) fail before the fix and pass after.
- [ ] A tip already shown once still never shows again for that profile
      (existing tests pass).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: battle harness, as in step 1.
- Unit: `TipState` marks seen on show, not on absorb.

## Completion notes

*(Filled in by the session that completes the ticket.)*
