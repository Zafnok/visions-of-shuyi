---
id: "0467"
title: "The Magic and Terrain tips never showed: a tip counts as seen only once it's shown"
type: bug
milestone: M3 Battle UI
model: sonnet-5
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-10-17
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

**Comes before the Playdate playtest.** Round 3's tickets wait for the
carve (1118) and the Playdate port (1050), Nick's priority. This one is
an exception he chose (2026-10-16): the Playdate runs the same tutorial
code, so 1050 waits for it, and it waits for nothing (the carve doesn't
move the battle screen's tutorial code). The engine carve is still the
critical path (`docs/ROADMAP.md`); this is a small fix to fit in beside it.

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

- [x] Tests (a), (b), (c) fail before the fix and pass after. **Partly:**
      (c) and the unit tests fail before and pass after; (a) and (b) pass
      both before and after (see the notes).
- [x] A tip already shown once still never shows again for that profile
      (existing tests pass).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Integration: battle harness, as in step 1.
- Unit: `TipState` marks seen on show, not on absorb.

## Completion notes

Done 2026-10-17 (Opus 5.5).

- **The fix:** a tip is queued when its trigger fires and saved as seen
  only when it is on screen as a frame ends (`TipState::show`, called
  after `BattleScreen::update`). A queued tip is **not** kept across
  Retry: it is **re-queued when its trigger fires again** (the start tip
  on the retried turn 1, Magic the next time a caster's menu opens,
  Terrain the next time the cursor rests on special ground). A trigger
  that fires while its tip is already queued doesn't queue it twice.
- **A second way a tip was lost:** Confirm or Cancel in the very frame a
  tip came due closed it before it had ever been drawn (and it was
  already saved as seen). Now a tip takes every key at once, but only
  closes on Confirm/Cancel once it has been drawn. The old test
  `cancel_dismisses_too_and_other_keys_wait` pressed Confirm on the
  first frame; it now lets the tip draw first.
- **Reproducing (a) and (b):** new `crates/ui/tests/it/ch01_tips.rs`
  plays Chapter 1 at Full from a fresh profile: Rue's action menu opened
  while the cursor lesson is open, and the cursor resting on a forest
  while it is open. Both tips **already showed** once the lesson closed,
  before the fix too: within one battle a queued tip waits for the
  lesson and then shows (also checked by hand with Rue moving during
  the "moving" lesson: the Magic tip shows right after it). So the
  queue dropping (Retry, leaving or finishing the battle with tips still
  waiting) and the undrawn-close above are the causes found; the tests
  stay as guards. (c) (`flow::a_tip_not_shown_before_a_retry_shows_after_it`)
  uses the test chapter, which can be lost by ending turns; Chapter 1
  (a rout) can't be lost that quickly.
- Unit tests in `tip_tests.rs`: `a_tip_counts_as_seen_once_it_is_shown`,
  `a_tip_not_yet_drawn_takes_the_key_but_stays`.
- No gameplay rules decided. For Nick: if a tip still never shows in the
  next playtest, say what you did just before (lost/retried, quit, or
  pressed keys fast), since the in-battle path now shows them.
