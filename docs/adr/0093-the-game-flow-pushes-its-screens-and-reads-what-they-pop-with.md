# ADR-0093: The game flow pushes its screens and reads what they pop with

- **Status:** Accepted
- **Date:** 2026-10-19
- **Related tickets:** 1158, 1153, 1159; engine ticket 2306

## Context

ADR-0035 §3 made `FlowScreen` host its screens itself: an 11-variant
`Stage` enum held the current screen, `screen()` / `screen_mut()`
forwarded `update` and `draw` to it, and when it popped the flow read a
result getter (`ModeSelectScreen::result`, `BattleScreen::restart_requested`,
…). It did so because the engine's `Transition::Pop` carried nothing and
no screen heard that it was uncovered. The title screen rebuilt "I am shown
again" the same way, with a `stale` flag set before each push and checked
on its next update.

The first engine-friction sweep (1153) logged both. Engine ticket 2306
added `Transition::PopWith(ScreenResult)` (`Transition::pop_with(value)`),
`Screen::uncovered(ctx, result) -> Transition`, applied at once in the same
frame, and `Screen::covered(ctx)`.

## Decision

1. **The flow pushes its screens over itself.** `FlowScreen` keeps the
   campaign, the chapter, the battle's setup and a small `Stage` (which
   screen is over it, with only what it needs to read the answer: the mode
   picked, the level picked, whether the slot picker is saving). A step
   that goes on calls `show(stage, screen)`; the flow pushes that screen on
   its first update or, after that, from `uncovered`, so the next screen is
   on the stack in the frame the last one popped. With nothing to show the
   flow pops itself: back to the title, with no campaign in `Ctx`.
2. **Each screen pops with its answer**, as a plain type the two agree on:
   `GameMode`, `TutorialLevel`, `LeadProfile`, `bool` (the save prompt),
   `SlotOutcome`, `Preparations` (`Fight!`), `GameOverChoice`, and the
   battle's `BattleEnd` (`Won(state, unused charges)`, `Restart`,
   `Suspend(the screen)`, `Title`). Going back pops with nothing. Result
   fields and getters that only the flow read are gone. A "menu, then pop
   with the choice" screen uses `screens::pick_one`.
3. **Game Over is the battle's.** The battle pushes Game Over over itself
   at a defeat's banner (an overlay while the lost battle still shows),
   and again, without its opening animation, when the player backs out of
   the rewind screen. It reads the choice in `uncovered`: `Rewind` opens
   the rewind screen; `Retry` and `Title` pop the battle with `Restart`
   and `Title`. The battle keeps its music cue (`with_music`) and asks for
   it again itself when a rewind undoes its ending.
4. **The campaign takes in what changed when a screen over the flow pops**
   (`take_in`: a switch to Casual made on the Options screen at
   Preparations, the tutorial level, the lessons done, the playtime),
   before the flow reads the answer, so every save holds it. Preparations
   switches its own battle to Casual in its `uncovered`. Tests that look
   at the campaign mid-battle read `FlowScreen::campaign_now(ctx)`
   (test and `harness` builds).
5. **The stack shows the flow.** `FlowScreen::NAME` is `game_flow`, so
   the stack reads `["title", "game_flow", "battle"]`; on the one frame
   between being pushed and pushing its first screen, the flow is named
   and drawn as that screen. Tests find the flow's screens on the stack
   (`Harness::battle`, `battle_mut`, `screen::<T>()`).
6. **The title refreshes in `uncovered`**: its saves, menu and cinematic
   are built when it is made and each time a screen over it pops.

## Consequences

- The flow is about 70 lines shorter and every child a few lines shorter;
  a new step is a `Stage` variant, a `show` and an arm in
  `FlowScreen::advance`.
- Screens other than the flow's can now report a result by popping with
  it (ADR-0035's last consequence no longer holds).
- The flow's first screen arrives one frame after the flow is pushed. The
  frame shows it (the flow draws it), but a test that looks it up on the
  stack right after the press that pushed the flow needs one more frame
  (a Playdate button acts on its release).
- `campaign()` is the campaign as of the last screen that popped over the
  flow; `campaign_now(ctx)` is the live one.

## Alternatives considered

- **Keep hosting, read results through `uncovered` only for the title**:
  leaves the 11-variant enum and the forwarding the sweep flagged.
- **The battle pops on a defeat with a copy of itself, and the flow
  pushes Game Over over a copy it keeps for drawing**: a battle copied at
  every defeat, and the flow drawing a screen that isn't on the stack.
  Only `Suspend` carries the screen, for the rare save that fails.
- **Forward the flow's first frame to its first screen before pushing
  it**: no frame lost, but a first screen that pops or pushes on that
  frame can't be expressed as one transition.
