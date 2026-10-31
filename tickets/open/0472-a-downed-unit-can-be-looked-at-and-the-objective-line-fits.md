---
id: "0472"
title: "A downed unit shows its panel under the cursor, and every battle's objective line fits on the PC and the Playdate"
type: bug
milestone: M3 Battle UI
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: ["1039"]
nick_input: none
completed:
---

# 0472 — A downed unit can be looked at, and the objective line fits

## Context

From the retro of the last 50 PRs (2026-10-22). Two small gaps where
0319 (#393), 0324 (#404) and the Playdate battle skin 1038 (#392) met:

1. **A downed unit can't be looked at.** 0319 added units that go down
   instead of falling (a boss who can't die, `Defeat::GoesDown`): they
   stay on the map (`BattleState::down()`), and all three map skins draw
   them. But the unit under the cursor comes from
   `BattleScreen::hovered` (`crates/ui/src/screens/battle/mod.rs`),
   which searches only `state.units()`. So the PC side panel and the
   Playdate's corner pop-ups (which show while browsing) never show a
   downed boss, though during playback they do (`panel_view`).
2. **Nothing checks that the objective line fits.** 0324 puts the exit's
   name on the objective line (`objective_text` in
   `crates/ui/src/screens/battle/map_menu.rs`, with an escape's
   "(n/N out)" counter after it). Content checks only that `exit_name`
   is one line (`crates/content/src/battle.rs`). On the PC the
   objective box runs past the 70-cell map view once the name passes
   about 37 characters; the Playdate window cuts each line to fit
   (`window_of` in `crates/ui/src/screens/battle/playdate.rs`), so a
   long name, or its translation, cuts the counter off. Chapter 6's "the
   Brennish lines" fits (44 cells on the Playdate); the next one may not.
   The battle notes have this check already
   (`every_battles_notes_fit_the_map_view` in `notes.rs`).

## Nick input

None. Verified by tests and `cargo shuyi frame-png` pictures in the PR
(a downed unit under the cursor, PC and Playdate). That a downed unit
shows its panel when the cursor is on it is a *the agent's starting rule*
(the panel already shows it during playback); it goes in the Completion
notes for the Act 1 playtest's checklist (1064).

## Scope

**In:**
- What's under the cursor for **looking**: the side panel (PC) and the
  corner pop-ups (Playdate) show a downed unit under the cursor, as
  playback's `panel_view` does.
- **Commands don't change**: picking a unit to move, targets, the
  danger zone toggle and every other use of the unit under the cursor
  still see only units on the field. Split the lookup if `hovered`
  serves both (`hovered` for commands, `looked_at` for the panel).
- A test like `every_battles_notes_fit_the_map_view`, for the objective
  line: every battle in `assets/battles/` and `assets-test/battles/`,
  in every language pack the build embeds, at its longest counter
  ("(N/N out)" with N its player slots), fits the PC objective box and
  the Playdate window.
- If a shipped battle fails it, shorten its `exit_name` in the battle
  file (say which, and why, in the PR).

Blocked by 1039 only because it rewrites the Playdate battle skin's
pop-ups (`screens/battle/playdate.rs`) that this ticket feeds.

**Out (do not do):**
- Wrapping the objective line onto two lines, or cutting it differently.
- A content check in `trpg-content` (it can't know the skins' widths).

## Implementation steps

1. **Read** `hovered` and every caller (`grep -n "hovered()" crates/ui`),
   `panel_view`, the Playdate skin's corner pop-ups
   (`crates/ui/src/screens/battle/playdate.rs`), `objective_text` and
   `notes.rs`'s fit test.
2. **A failing test first**: on `test_hold` (`assets-test/battles/`),
   down the boss, put the cursor on it; the side panel's view names it.
3. Add `looked_at()` (units, then `state.down()`), use it where the
   panel and pop-ups are built; leave command paths on `hovered()`.
4. **The fit test** in `map_menu.rs`'s tests (or `notes.rs`): build the
   longest objective line per battle and language, measure it in cells
   (`crate::screens::cells` or the helper the notes test uses) against
   the PC box's inner width (from `layout::MAP_VIEW` and the box's
   frame) and the Playdate window's inner width (the 46 cells
   `window_of` leaves, from its layout constants).
5. `cargo shuyi frame-png` pictures for the PR: a downed unit under the
   cursor on the PC and on the Playdate.

Expected size: about 30 production lines (the lookup and its two
callers), plus tests.

## Acceptance criteria

- [ ] With the cursor on a downed unit, the PC side panel and the
      Playdate pop-up show it (tests on the view; pictures in the PR).
- [ ] A downed unit can't be picked, targeted or moved (a test: Confirm
      on it does what Confirm on an empty tile does).
- [ ] The objective fit test passes for every battle and language pack,
      and fails on a battle whose `exit_name` is 60 characters (a test
      with a built battle).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the looked-at lookup; the objective fit test.
- Snapshot / integration: the side panel view on `test_hold` with the
  boss down.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
