---
id: "0841"
title: "Setting Tutorial to None stops the tips and lessons already queued"
type: bug
milestone: M7 Chapter 1 & game flow
model: sonnet-5
effort: medium
status: todo
blocked_by: ["1118", "1050"]
nick_input: none
completed:
---

# 0841 — Tutorial None stops the queued tips and lessons

## Context

0838 added the Options row "Tutorial: Full / Short / None" while a
campaign is loaded. Chapter 1 playtest round 3 (0804, 2026-10-16): in a
battle, Nick set it to None and was asked "Do the tips and lessons
stop?"; he answered **"No"**.

Likely cause (read 2026-10-16, not yet proven by a test):

- The write path is fine: Options writes `ctx.campaign_tutorial`
  (`crates/ui/src/screens/options.rs:563-573`), `FlowScreen::sync_mode`
  copies it into the campaign every frame (`crates/ui/src/flow.rs:
  643-670`, line 706), and both readers use the live value
  (`screens/battle/tips.rs:43`, `screens/battle/lessons.rs:75`).
- But the level is checked **only when a trigger is absorbed** into a
  queue (`TipState::absorb`, `tips.rs:41-58`; `LessonState::absorb`,
  `lessons.rs:71-91`). Tips and lessons already queued, and the lesson
  that is open, are never re-checked: `due_tip` / `shown_tip`
  (`screens/battle/mod.rs:885-913`) and `shown_lesson` (`mod.rs:941-945`)
  don't look at the level. Chapter 1 queues four lessons at once on
  turn 1 (`cursor`, `keys`, `next_unit`, `moving`, all
  `FirstBattleStart`, `assets/data/lessons.ron:47-88`), so they keep
  coming after None.

Rule (`docs/design/tutorial.md`, 0838): "Tips and lessons follow it at
once."

**Waits for the engine carve and the Playdate port** (Nick, 2026-10-16),
so `blocked_by` has 1118 and 1050.

## Nick input

None.

## Scope

**In:**
- Write the failing test first: Chapter 1 at Full, turn 1 with the
  cursor lesson open and three more queued; open Options, set None, keep;
  assert no lesson is open or shown and no tip shows for the rest of the
  battle.
- Fix: when the level changes, drop queued tips and lessons the new
  level doesn't show, and close an open lesson it doesn't show (without
  ticking it; it stays not done, so it can come back if the level goes
  back up). Also check the level where a tip or lesson is shown, so a
  stale queue can't leak.
- Short → None and Full → Short likewise (Full-only lessons drop at
  Short).
- An open lesson that's dropped no longer holds End turn.

**Out (do not do):** what lessons exist; Guides (they stay readable at
every level).

## Acceptance criteria

- [ ] The repro test fails before the fix and passes after.
- [ ] Full → Short drops Full-only lessons and tips (test).
- [ ] After None, End turn is never refused by a lesson (test).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the battle harness with Options changing the level
  mid-battle, as above.

## Completion notes

*(Filled in by the session that completes the ticket.)*
