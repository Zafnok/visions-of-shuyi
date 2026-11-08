---
id: "0475"
title: Vix's lines come apart from the objectives, with no key in what she says
type: content
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: ["0043"]
nick_input: none
completed: 2026-11-07
---

# 0475 — Vix's lines come apart from the objectives, with no key in what she says

## Context

Vix is voiced (0043, `docs/design/voices-languages-and-script.md`, *Voices*).
Nick, 2026-11-01: "for the button tutorials they don't need voices, but make
sure Vix has lines throughout tutorial, so a clear delineation between <Vix's
line> <tutorial objectives> should be made if it's not already entangled..."

A voice clip is made ahead of time and can't say a key the player rebound. Today
a lesson (`assets/data/lessons.ron`) has `text` and `items`, which is nearly the
split he asks for, but `text` often names keys ("Press {NextUnit} for anyone
who hasn't acted"). A tip (`assets/data/tips.ron`) has one `text` that mixes her
talk and the keys, and no list.

## Nick input

None. The script is the agent's (`story-writing` skill). What is left to judge
by playing goes on the Act 1 playtest's checklist (1064).

## Scope

**In:** the data split for lessons and tips; their new wording, in Vix's voice
card; the screens showing both parts; the check that keeps keys out of her
lines; the *Guides* list (0458) showing the same text.

**Out (do not do):** playing any voice (0476); making clips (0476); new lessons
or tips; changing when any lesson or tip appears.

## Implementation steps

1. Decide the fields by reading `crates/content/src/lesson.rs` and the tips'
   loader. A lesson's `text` becomes her line, and its `items` stay the
   objectives. A tip gains an optional `how` of one or more lines of plain
   instruction, beside its `text`. Follow `shuyi-screen`: the view gives the
   two parts apart, and the skin paints them.
2. Add a content check: a lesson's or tip's `text` may hold no `{Action}`,
   `{Cursor}` or `{Select}` placeholder. Keys go in `items` or `how`.
3. Rewrite every lesson and tip so that each has a line for her to say, which
   teaches something on its own. Keep the limits in `lessons.ron`'s header and
   her voice card (`docs/story/characters/guide.md`): short, dry, says what to
   do and stops. Every rule a line taught must still be taught by the line or
   by the objectives.
4. Update `docs/design/tutorial.md` with the rule and `lessons.ron`'s header
   comment with the new fields.

## Acceptance criteria

- [x] No lesson or tip `text` in `assets/` holds a key placeholder, and the content check fails on one that does (test).
- [x] Every lesson and tip has a non-empty `text`.
- [x] Each key named by the old wording is still named in `items` or `how` (compare in the notes).
- [x] A frame picture of a lesson and of a tip with `how` is in the PR (`cargo shuyi frame-png`), and neither overflows its box.
- [x] Existing lesson and tip tests pass, with snapshots updated on purpose.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the content check.
- Snapshot: a lesson box and a tip box with both parts.

## Completion notes

Done 2026-11-07.

- **The split.** A lesson's `text` is Vix's line and its items are the
  objectives (unchanged fields). A tip keeps `text` for her line and gains
  an optional `how`: up to 3 lines of plain instructions with the keys.
  `how` is the engine's (engine#3211, merged as
  [Zafnok/shuyi-engine#302](https://github.com/Zafnok/shuyi-engine/pull/302)):
  the tip schema is the engine's, so the field went there, generic ("a
  tip's instructions apart from its text"). The pin move that brings it
  passes 17 engine PRs, including engine#2544, which removed the `bots`
  operation, so the pin move and its break fixes are ticket 1240's PR
  (Zafnok/visions-of-shuyi#574). This PR is stacked on it.
- **The check.** `tip::spoken_problem`: a lesson's or a tip's `text`
  with any placeholder fails to load ("the text is spoken, so it names no
  key ({Confirm}): keys go in the how/items"). Tests: `a_key_in_the_spoken_text_fails`
  in both `tip.rs` and `lesson/tests.rs`, and a test that every shipped
  tip and lesson has a non-empty line with no key.
- **Screens.** `TipView` has `how`; the tip box paints it under her
  line after a blank row. The Guides list shows her line, then the items
  or the `how`, after a blank line. A language pack translates `how` as
  `tips.<id>.how` (the test pack does `battle_start`'s).
- **Wording.** Five lessons and ten tips named keys in their text; all
  are rewritten so the line says what to do in words and the keys move
  into the items or `how`. The rest are unchanged.
- **Docs.** `docs/design/tutorial.md` (*Vix is voiced*: how the split
  works), Vix's voice card (keys never in her line; the sample line), the
  `keyboard-input` skill, and the headers of `lessons.ron` and `tips.ron`.

**Every key the old wording named, and where it is now:**

| Where | Old text named | Now in |
| --- | --- | --- |
| lesson `next_unit` | {NextUnit}, {Info} | items "Jump to a unit still to act: {NextUnit}", "See a unit's stats and weapons: {Info}" |
| lesson `moving` | {Select} | items "Select a unit: {Select}", "Send it to a lit tile: {Select}" |
| lesson `attacking` | {Confirm}, {Cancel} | item "Attack: {Confirm}. Back out: {Cancel}" |
| lesson `ending_turn` | {EndTurn}, {Cancel} | item "End turn: {EndTurn}, or menu: {Cancel}" |
| tip `battle_start` | {Cursor}, {Select}, {NextUnit}, {Info} | how "Move the cursor: {Cursor}. Pick up a unit: {Select}" / "Next unit to act: {NextUnit}. A unit's details: {Info}" |
| tip `zoom` | {ZoomMap} | how "Zoom the map in or out: {ZoomMap}" |
| tip `unit_selected` | {Select}, {Cancel} | how "Send it to a blue tile: {Select}. Back out: {Cancel}" |
| tip `forecast` | {Cursor}, {Confirm}, {Cancel} | how "Pick the target: {Cursor}. Attack: {Confirm}" / "Back out: {Cancel}" |
| tip `low_hp` | {Rewind} | how "Rewind: {Rewind}" |
| tip `level_up` | {Info} | how "A unit's details: {Info}" |
| tip `enemy_phase` | {Confirm} | how "Skip the banner: {Confirm}" |
| tip `danger_zone` | {DangerZone}, {Select} | how "Danger zone on or off: {DangerZone}" / "One enemy's range: {Select} on it" |
| tip `skills` | {Info} | how "A unit's details: {Info}" |
| tip `magic` | {Info} | how "A unit's details: {Info}" |
| test lesson `test_cursor` | {Cursor} | its items already name each direction |

The rules each line taught are all still taught: "and again to hide it"
(danger zone) and "again to switch back" (zoom) are in the `how` or the
line; "backing out doesn't spend the turn" stays in the forecast's line.

Pictures (`cargo shuyi frame-png --tips --battle ch01`): the tip box
with `how` (`docs/img/0475-tip.png`) and a lesson box
(`docs/img/0475-lesson.png`); both stay inside their boxes and over the
map.

Deviations: the engine change (above) and the stack on 1240's PR.

Left for the Act 1 playtest (1064), not decided by Nick: the agent's
starting rules (*the agent's starting rule*, all look and wording):

- A tip's instructions sit under her line after a blank row, in the
  same colour, written "What: key", two to a line where they fit.
- Her lines were reworded by the agent in her voice card; whether they
  read and sound right is judged by playing and hearing them (0476
  makes the clips).

Follow-up tickets: none new (0476 already plays the voice and makes the
clips).
