---
id: "1158"
title: "The game flow, the title and the pick-one screens on the engine's screen results: no more hosting child screens by hand"
type: feature
milestone: "Engine: carve"
model: opus-5.5
effort: high
status: done
blocked_by: ["engine#2306"]
nick_input: none
completed: 2026-10-19
---

# 1158 — The flow and the title on screen results

## Context

The first engine-friction sweep (1153, `docs/engine/friction.md`) found
that the engine's `Transition::Pop` carries no result and nothing tells
a screen it is uncovered, so the game built both by hand:

- `crates/ui/src/flow.rs` (539 lines, estimate 390, 1.4×) holds its
  child screens itself in the 11-variant `Stage` enum, forwards to them
  in `screen()` / `screen_mut()`, and after each one pops reads its
  result in `advance()`. Eight screens keep a result field and a getter
  only for that: `familiarity.rs` `result()`, `game_over.rs`
  `result()`, `lead_select.rs` `result()`, `mode_select.rs` `result()`,
  `save.rs` `SavePromptScreen::result()` and `SlotPickerScreen::result()`,
  `preparations.rs` `outcome()`, and the battle's `*_requested()`.
- `crates/ui/src/screens/title.rs` marks itself `stale` before it
  pushes a screen and calls `refresh` on its next update, to notice it
  is uncovered (about 30 lines).

Engine ticket 2306 (`Zafnok/shuyi-engine`) adds `Transition::PopWith`,
`Screen::uncovered(ctx, result)` and `Screen::covered(ctx)`. This
ticket moves the pin past it and deletes the hand-made versions. The
battle screen's own pop-ups are 1159.

## Nick input

None.

## Scope

**In:**
- Move the pin to the engine's `main` at or past 2306's merge
  (`engine-change` skill, step 7).
- `FlowScreen` pushes its children on the stack and reacts in
  `uncovered` to the value they pop with; the `Stage` enum keeps only
  what the flow needs to know where it is (or goes, if nothing needs
  it); `screen()` / `screen_mut()` go.
- The eight screens pop with their result (`PopWith`) and lose the
  result fields and getters that only the flow read.
- Tests that read the flow's children through `battle()`,
  `preparations()`, `game_over()` read them from the stack instead
  (`ScreenStack::find`, the harness), or through a getter kept for
  tests if that is much shorter: say which in the Completion notes.
- The title: `stale` and `refresh` go; it refreshes in `uncovered`.
- Mode select, familiarity and the save prompt are the same "a menu,
  then pop with the choice" screen: if one small helper in the game
  makes all three shorter, write it.

**Out (do not do):**
- The battle screen's rewind and Guides pop-ups, and its tips'
  `set_under_scene` (1159).
- The two copies of the campaign settings in the flow (1171).
- Any change the player can see: the order of screens, their keys and
  their text stay the same.

## Implementation steps

1. Move the pin; build; fix anything 2306 changed.
2. Write the estimate before the code (`engine-friction` skill): the
   flow should shrink by about 150 lines and each child by 5 to 10.
3. The children pop with values, one at a time, each with its tests.
4. The flow on `uncovered`; then the title.
5. `cargo xtask code-lines`: the removed column should beat the added.

## Acceptance criteria

- [x] `grep -n "fn screen_mut\|fn result(" crates/ui/src/flow.rs crates/ui/src/screens/{familiarity,game_over,lead_select,mode_select,save}.rs`
      finds nothing.
- [x] `grep -n "stale" crates/ui/src/screens/title.rs` finds nothing.
- [x] Every flow test passes: New Game to the first battle, Load,
      Game Over and Retry, Suspend and resume, the end.
- [x] `cargo xtask code-lines` shows the game's production lines fell.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the existing flow tests, unchanged in what they check.
- Unit: one test per child screen that it pops with its value.

## Completion notes

**What was done.**

- **Pin:** moved to the engine's `main` at `8c07b5c` (the merge of
  engine 2306). Ticket 1160 then landed first with the pin at `ed61c0e`,
  4 commits further on, and the merge of `main` kept it.
- **The flow** (`crates/ui/src/flow.rs`) pushes its screens over itself
  instead of hosting them. `show(stage, screen)` queues the next one; the
  flow pushes it on its first update, or from `uncovered` in the frame
  the last one popped. `Stage` is now a small `Copy` enum (which screen
  is up, with the mode and level picked, and whether the slot picker
  saves). `screen()`, `screen_mut()`, `stage()`, `battle()`,
  `battle_mut()`, `preparations()` and `game_over()` are gone; with
  nothing left to show the flow pops itself and clears the campaign from
  `Ctx`. Its name on the stack is `game_flow`, so the stack reads
  `["title", "game_flow", "battle"]`.
- **The children pop with their values:** mode select (`GameMode`),
  familiarity (`TutorialLevel`), lead select (`LeadProfile`), the save
  prompt (`bool`), the slot picker (`SlotOutcome`), Preparations
  (`Preparations` on `Fight!`; `PrepOutcome` and `outcome()` are gone),
  Game Over (`GameOverChoice`) and the battle (`BattleEnd`). Every result
  field and getter went. `screens::pick_one` is the "a menu, then pop
  with the choice" helper; mode select, familiarity, the save prompt and
  Game Over use it.
- **Game Over is the battle's own.** A defeat's banner pushes it over the
  battle; it is an overlay while the lost battle still shows. Backing out
  of the rewind screen pushes it again, skipped. The battle reads the
  choice in `uncovered`: Rewind opens the rewind screen; Retry and Title
  pop with `BattleEnd::Restart` and `BattleEnd::Title`.
  - The battle keeps its own music cue (`with_music`) and asks for it
    again when a rewind undoes the ending. That replaces
    `take_music_back`.
  - `restart_requested()` and `suspend_requested()` went: the battle pops
    with `Restart` or `Suspend(itself)`. The flow puts the battle back if
    the suspend save fails.
- **The campaign takes in, when a screen over the flow pops,** what used
  to be synced every frame (`take_in`): the switch to Casual, the
  tutorial level, the lessons done and the playtime. Preparations
  switches its own battle to Casual in its `uncovered` (back from
  Options).
- **The title:** `stale` went. It refreshes when it is made and in
  `uncovered`, so `refreshed()` went too.
- ADR-0093 records this and supersedes ADR-0035 §3.

**Tests.**

- Unit: a pop-with test for each child screen (mode, familiarity, lead,
  save prompt, slot picker, Preparations, Game Over, and the battle's
  `Won` and `Suspend`).
- The flow's unit tests drive a real `ScreenStack`, plus new tests:
  - the flow pushes its first screen;
  - New Game reads each answer and goes back a step on Cancel;
  - `Title` on Game Over ends the flow.
- **How the tests find the flow's screens:** from the stack
  (`Harness::battle`, the new `Harness::battle_mut`,
  `screen::<PreparationsScreen>()` and the like). `Harness::flow_mut` went.
- **Tests that read the campaign mid-battle** now read
  `FlowScreen::campaign_now(ctx)` (test and `harness` builds). It is the
  campaign with what the flow takes in when the battle pops.
- Every screen-name list that went through the flow gained `"game_flow"`.

**Deviations.**

- **The flow's first screen arrives one frame after the flow is pushed.**
  A pushed screen hears nothing until its first update. On that frame the
  flow is named and drawn as its first screen, so nothing shows. Two
  tests needed one more frame before looking it up: a Playdate button
  acts on its release.
- **Game Over moved under the battle** rather than staying a flow step.
  The flow would otherwise have had to keep a copy of the lost battle to
  draw it and to rewind it. The flow is 81 lines shorter, not about 150,
  because about 55 of them moved into the battle.

**Engine friction:** estimate 200 (ticket: the flow −150, each child −5
to −10), measured 313 added, 409 removed (1.6×: under 2×). Production
lines fell by 96. One "accept" row is logged in `docs/engine/friction.md`
for the one-frame start.

**Follow-ups:** none.

**For Nick:** nothing to decide, and nothing a player should see change.
The screens, their order, keys and text are as before.
