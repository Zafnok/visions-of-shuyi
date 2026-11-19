---
id: "0770"
title: "The script preview returns to its scenes when a scene ends"
type: feature
milestone: M6 Story & dialogue
model: opus-5.5 / gpt-6.1-sol
effort: low
status: done
blocked_by: ["engine#3214"]
nick_input: none
completed: 2026-11-15
---

# 0770 — The script preview returns to its scenes when a scene ends

## Context

Nick, trying the web script preview (0767) on 2026-11-15: "rather than
backing out to Lines on scene finish it should go to Scenes". The tab a
finished scene opens on is the engine's `DialoguePreview`'s, with no way
for the game to choose it, so engine#3214 changes it: a finished scene
opens the Scenes tab, the cursor on the scene just played, so Confirm
plays it again. This ticket brings that into the game by moving the pin.

The engine's `main` is far past the pin (issue #691, "22 engine PRs
behind", with public API changes), so the move may be big. If an
`engine-pin` PR for #691 lands first, this ticket only needs the pin
moved past engine#3214's merge and the test below.

## Nick input

None.

## Scope

**In:**
- ~~Move the pin past engine#3214's merge (`engine-change` §3.6, the
  `engine-pin` skill's review and `docs/engine/pins.md` section for every
  engine PR the move passes).~~ Done by the pin move of 2026-11-15
  (issue #703).
- A game Harness test: a scene of the script preview played to its end
  shows the Scenes tab, on that scene.

**Out:** any other change to the preview.

## Implementation steps

1. Check engine#3214 is in the engine's `tickets/done/` (`work-ticket` §1).
2. ~~Move the pin (`cargo xtask engine-pin`), fix the game for the engine
   PRs it passes, write the pin's section.~~ Done by the pin move of
   2026-11-15 (issue #703).
3. The test in `crates/ui/tests/it/story_preview.rs`.

## Acceptance criteria

- [x] The pin is at or past engine#3214's merge commit (the pin move of
      2026-11-15).
- [x] A Harness test plays a preview scene to its end and finds the
      Scenes tab, on that scene.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the Harness test above.

## Completion notes

engine#3214 is in the engine's `tickets/done/` and the pin (`e074d3ec`,
the pin move of 2026-11-15) is ahead of its closing commit, so no game
code changed: the engine's `DialoguePreview` already opens the Scenes tab
on the scene just played when the game's dialogue screen pops.

Added `a_scene_played_to_its_end_opens_the_scenes_tab_on_that_scene` in
`crates/ui/tests/it/story_preview.rs`: it plays the test file's second
scene (`trial_two`, so the cursor is not on row 0 by chance) to its end,
finds the tabs open on Scenes with the cursor on `trial_two`, and Confirm
plays it again from its first text box.

Deviations: none. No follow-up tickets. No gameplay rules decided.
