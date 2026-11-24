---
id: "1312"
title: "Vix's tip steps aside while unit info or another full page is open"
type: bug
milestone: M3 Battle UI
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: []
blocks: ["1064"]
nick_input: none
completed: 2026-11-23
---

# 1312 — Vix's tip steps aside while unit info or another full page is open

## Context

Nick, after watching a new player play Chapter 1 (2026-11-22):

> "The unit info screen should not be blocked by a Vix tip modal -- since
> she just told you press E for unit info and you can't see shit. Same w
> all screens like this, shouldnt be blocked by her tip. Should
> temporarily rm the tip and bring back"

Unit info is not a pushed screen: it is `Mode::Info { unit, back }`
(`crates/ui/src/screens/battle/mode.rs`), drawn in the battle's own view.
The glyph skin (`screens/battle/glyph.rs`, `paint`) draws info in
`paint_dialogs`, then the lesson, the end-of-lessons box and the tip box
on top; the Playdate skin (`screens/battle/playdate.rs`) does the same.
`Guide::due_tip` (`screens/battle/guide.rs`) never looks at the mode, so a
tip fires and shows inside `Mode::Info`, and `Guide::take_key` takes the
keys (via `overlay_key` in `mod.rs`).

Pushed screens (Options, Guides, Rewind, Shop, scenes, Game Over) already
hide tips through `Screen::covered`/`uncovered` →
`Guide::covered`/`uncovered`. The same in-battle problem exists for
`MapMenu`, `UnitList`, `Objective` and the end-turn, restart and suspend
prompts.

## Nick input

None: Nick said how it should work (hide the tip, bring it back). Verified
by the tests below.

## Scope

**In:**
- While the battle is in a full-page mode (`Info`, `UnitList`,
  `Objective`, `MapMenu`, and any other mode that fills the screen or
  asks a question), the tip and the lesson box are not drawn and don't
  take keys.
- A tip already showing when such a mode opens is kept (not marked
  stale, not lost) and shows again, as it was, when the mode closes.
- A tip that fires while such a mode is open waits for it to close.
- A lesson whose step **is** opening that page (for example "press
  {Info} to see a unit") completes as it does today; its next text shows
  after the page closes, not over it.

**Out (do not do):** moving the tip to another place on screen; new tip
or lesson text (dialogue writing is paused).

## Implementation steps

1. Add `Mode::covers_guide(&self) -> bool` in `mode.rs` listing the
   full-page modes.
2. Pass it into the guide's `Moment` (`BattleScreen::moment`, `mod.rs`)
   and make `Guide::due_tip`/`shown_tip` and the lesson box return
   nothing while it's true, **without** calling `go_stale` (a "paused"
   state like `covered`, not a mode change).
3. In `BattleScreen::view`, leave `view.tip` and the lesson out while
   covered; skins need no change.
4. Tests (below), including the Playdate skin's view.

## Acceptance criteria

- [x] Test: a tip showing, open unit info → the view has info and no
      tip; close it → the same tip is back.
- [x] Test: a trigger fires during unit info → the tip shows only after
      it closes.
- [x] Test: in unit info, the info screen's keys reach it (not the tip).
- [x] The same for `UnitList`, `Objective` and `MapMenu` (one test each,
      or one table test).
- [x] A `frame-png` before/after of Chapter 1's unit info with a tip due,
      in the PR.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the battle screen through the headless Harness, in
  `tip_tests.rs`; tests read the view, not cells (ADR-0038).

## Lesser targets

The Playdate skin reads the same view, so it gets the fix for free; one
test checks its view.

## Completion notes

- `Mode::covers_guide` (`mode.rs`) lists the pages and questions: `Info`,
  `MapMenu`, `UnitList`, `Objective`, `EndTurnPrompt`, `RestartPrompt`,
  `SuspendPrompt`. The guide's `Moment` carries it as `paged`; while it is
  true, `due_tip` and `lesson_moment` give nothing, so the tip, the lesson
  and the box at the end of the lessons are left out of the view and take
  no keys. The skins needed no change.
- `Guide::take_in_mode` skips a page's mode: the queued tips keep the
  moment of the mode under it and are judged by the mode it closes to (a
  page closing to the mode it opened from keeps them; closing elsewhere
  makes them stale, as any mode change does). Nothing is marked stale or
  seen while the page is open. Her voice stops when the page opens and
  does not replay the same line when it closes, as with pushed screens.
- Tests: `tip_tests.rs` (a tip on screen steps aside for each of info,
  the map menu, the unit list and the objective page and comes back; a
  tip fired on each waits; unit info takes its keys with a tip waiting,
  and the Playdate's view has no tip), `lesson_tests.rs` (the open lesson
  steps aside for unit info). The cursor lesson's property test now checks
  the lesson box only while no page is open: Cancel on the map opens the
  map menu, where the box now steps aside.
- Pictures: `docs/screenshots/1312-info-before.png` (Chapter 1, the cursor
  lesson over unit info, the fix turned off) and `1312-info-after.png`.
- No gameplay rules decided.
