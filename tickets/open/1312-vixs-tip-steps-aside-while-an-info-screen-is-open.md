---
id: "1312"
title: "Vix's tip steps aside while unit info or another full page is open"
type: bug
milestone: M3 Battle UI
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
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

- [ ] Test: a tip showing, open unit info → the view has info and no
      tip; close it → the same tip is back.
- [ ] Test: a trigger fires during unit info → the tip shows only after
      it closes.
- [ ] Test: in unit info, the info screen's keys reach it (not the tip).
- [ ] The same for `UnitList`, `Objective` and `MapMenu` (one test each,
      or one table test).
- [ ] A `frame-png` before/after of Chapter 1's unit info with a tip due,
      in the PR.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the battle screen through the headless Harness, in
  `tip_tests.rs`; tests read the view, not cells (ADR-0038).

## Lesser targets

The Playdate skin reads the same view, so it gets the fix for free; one
test checks its view.

## Completion notes

