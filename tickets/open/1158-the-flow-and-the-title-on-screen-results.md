---
id: "1158"
title: "The game flow, the title and the pick-one screens on the engine's screen results: no more hosting child screens by hand"
type: feature
milestone: "Engine: carve"
model: opus-5.5
effort: high
status: todo
blocked_by: ["engine#2306"]
nick_input: none
completed:
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

- [ ] `grep -n "fn screen_mut\|fn result(" crates/ui/src/flow.rs crates/ui/src/screens/{familiarity,game_over,lead_select,mode_select,save}.rs`
      finds nothing.
- [ ] `grep -n "stale" crates/ui/src/screens/title.rs` finds nothing.
- [ ] Every flow test passes: New Game to the first battle, Load,
      Game Over and Retry, Suspend and resume, the end.
- [ ] `cargo xtask code-lines` shows the game's production lines fell.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the existing flow tests, unchanged in what they check.
- Unit: one test per child screen that it pops with its value.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
