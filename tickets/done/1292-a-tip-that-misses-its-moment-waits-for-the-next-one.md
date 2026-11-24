---
id: "1292"
title: "A tip that misses its moment waits for the next one, not for a key press"
type: tuning
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: []
blocks: ["1064"]
nick_input: none
completed: 2026-11-18
---

# 1292 — A tip that misses its moment waits for the next one, not for a key press

## Context

Nick, 2026-11-17, after playing: "why are Vix's tip triggers so random,
they don't flow very well, like they pop up after a delay and stuff".

Two rules from 0495 (`docs/design/tutorial.md`, *How a tip box is put
away*) make tips come up late:

- **Missed moment → wait in a queue.** A tip whose trigger fires while a
  lesson, another tip or a screen is up stays queued, and comes up once
  the player is back on the map with nothing open. That can be a turn
  after whatever set it off.
- **One at a time → wait for a press.** After a tip is closed, a lesson
  is done, or a screen opened from the battle goes, the next tip waits
  for a press on the map.

Told this, Nick decided (2026-11-17): "how about just try again showing
on its moment rather than randomly delaying in a queue waiting for a key
press". So:

- A tip shows only **in its moment**. If its moment passes while it
  can't show, it isn't kept waiting: it is dropped, stays unseen, and
  shows the next time its trigger fires and nothing is in the way.
- No tip waits for a press on the map. When one closes, another whose
  moment is still on (fired by the same selection, say) comes up at
  once.

What 0495 fixed stays fixed: a tip takes the keys until Confirm or
Cancel closes it, and only a press made while it is on screen closes it
(`TipState::on_screen`), so a key meant for a menu never closes a tip.

The code: `crates/ui/src/screens/battle/tips.rs` (`TipState`: `queue`,
`fresh`, `waiting`, `go_stale`, `wait_for_play`, `played`) and
`crates/ui/src/screens/battle/mod.rs` (`due_tip`, `take_in_tips`,
`tick_lessons`, the `wait_for_play` call when a screen is pushed, the
`played` call on a press, and `lessons_end_shown`).

## Nick input

None: decided above (2026-11-17). Whether it now feels right goes on the
Act 1 playtest's checklist (1064).

## Scope

**In:**
- A queued tip whose moment passes (the mode changed, a screen covered
  the battle, or it couldn't show for a lesson or another tip) leaves
  the queue unseen, ready to be queued again when its trigger fires.
- Remove the wait for a press on the map before a tip (closing a tip,
  finishing a lesson, coming back from a screen).
- Check every tip's trigger fires again when its moment comes back.
  Triggers read from the state each frame (`triggers_now`,
  `detect_wade_tip`, `detect_zoom_tip`) do. Those fired by an event or
  once per battle (`FirstLevelUp`, `FirstEnemyPhase`,
  `FirstBattleStart`) fire again at the next level-up, enemy phase or
  battle. Make sure a dropped tip isn't lost for the rest of the
  campaign: it isn't marked seen until it was on screen (0467).
- Rewrite *How a tip box is put away (0495)* in
  `docs/design/tutorial.md`: keep A and the key rule, replace the
  "missed its moment" and "one at a time" bullets with Nick's 2026-11-17
  rule and a plain example (selecting a unit for the first time while a
  lesson is up: no tip then; the next time a unit is selected with no
  lesson up, *Moving a unit* comes up).

**Out (do not do):**
- The lessons' order and the box at the end of the lessons (0465): a
  lesson done still goes straight to the next lesson. The end box may
  keep waiting for a press after the last lesson; if removing the
  tips' wait leaves it with no reason to, say so in the completion
  notes.
- What a tip says, its triggers' conditions, the Guides list.
- The tip box over the Load Game and Profile screens (`tips/screen.rs`).
- Moving the guide out of the battle screen (1291). Either order works:
  if 1291 is done, the tip logic named here is in
  `screens/battle/guide.rs` and `triggers.rs`; if this lands first, 1291
  moves the new, smaller logic.

## Implementation steps

1. In `TipState`, replace `go_stale`'s "clear `fresh`" with "drop every
   queued tip that isn't on screen" (`queue` and `fresh`), and remove
   `waiting`, `wait_for_play`, `played` and `waiting()`. `fresh` may then
   be redundant (everything queued is fresh): remove it if so.
2. In `due_tip`, a tip is due when it is on screen or queued and in its
   moment (the phase and banner check stays); drop the `browsing` /
   `is_fresh` / `waiting` part. A tip blocked by a lesson or another tip
   this frame stays queued only while the mode is unchanged; then
   step 1 drops it.
3. Remove the callers: `tick_lessons`, the push of a screen over the
   battle, the press on the map, `lessons_end_shown` (keep its own wait
   if 0465 needs it, as its own small flag).
4. Keep the voice preload right: `coming_guide_lines` still lists the
   queued tips and the unseen ones (0494, 0496).
5. Update the doc comments of `TipState` and `due_tip`, and the design
   doc (Scope).
6. Update the tests that assert the old waits (`tip_tests.rs`,
   `lesson_tests.rs`, the 0495 tests): each now asserts the new rule.

Expected size: a net removal in production code (about −40 lines), plus
the tests.

## Acceptance criteria

- [x] A tip that fires while a lesson is up doesn't show after the
      lesson; it shows the next time its trigger fires with nothing in
      the way (test).
- [x] Two tips fired by one moment: closing the first brings up the
      second at once, with no key press between (test).
- [x] Back from Options (or any screen over the battle), no tip comes up
      that fired before or under it; a key pressed to close the map menu
      closes the menu, never a tip (the 0495 test, kept).
- [x] A dropped tip is still unseen in `TipsSeen` (test).
- [x] `docs/design/tutorial.md` states the new rule with Nick's words and
      date.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `TipState` dropping stale tips and keeping them unseen.
- Integration: the battle screen with the test chapter: the cases in the
  acceptance criteria, driven by actions through the harness.

## Completion notes

Done as planned, with three small deviations:

- `TipState` lost `fresh`, `waiting`, `wait_for_play`, `played`,
  `waiting()` and `is_fresh`; `go_stale` now drops every queued tip but
  the one on screen. `due_tip` is the old gates and the phase and banner
  check alone.
- **A playback ends no tip's moment** (deviation, needed): the level-up
  tip fires as its combat starts, and the mode then goes Combat → Idle,
  so dropping on every mode change would have dropped it at every
  level-up and it would never show. A change *out of* a playback mode
  (`Combat`, `AiAction`) drops nothing. Test:
  `the_level_up_tip_comes_up_after_its_combat` (and checked it fails
  without the exception).
- **A trigger fired under a screen is not noted** (`TipState::fire`
  ignores it while covered), so it can't be taken in on the frame the
  screen goes.
- **A lesson done still goes straight to the next lesson** (0465's order,
  kept): the old press-wait was what kept a tip from coming up between
  two lessons. Its replacement is a small flag, `AfterLesson::next_first`,
  set when a lesson closes with another in line and cleared when that one
  opens.
- The end-of-lessons box still has a reason to wait for a press after the
  last lesson (a key meant for the lesson must not answer it), so it keeps
  that wait as its own flag, `AfterLesson::end_waits`.

What the player sees changes in one place worth noting: backing out of
the action menu to a unit still picked up is that unit's selection, so a
*Moving a unit* tip that missed its moment under a lesson comes up there.

*Claude's starting rules* (for the Act 1 playtest, 1064):
- A combat's playback doesn't end a tip's moment: the level-up tip shows
  once the map is back after the fight.
- A lesson done goes straight to the next lesson, never to a tip between
  them (as before).
- The box at the end of the lessons waits for a press on the map after
  the last lesson (as before).

Whether the new timing feels right goes on the Act 1 playtest's
checklist (1064).
