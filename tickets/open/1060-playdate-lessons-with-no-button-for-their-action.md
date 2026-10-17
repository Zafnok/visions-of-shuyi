---
id: "1060"
title: "Playdate: Chapter 1's lessons can be finished when their action has no button"
type: bug
milestone: Post–Chapter 1
model: opus-5.5
effort: medium
status: todo
blocked_by: ["1035"]
nick_input: none
completed:
---

# 1060 — Playdate: lessons whose action has no button

## Context

Found in 1034's Simulator check (Chapter 1's first battle, *Not familiar*,
so lessons on). The "Finding your units" lesson
(`assets/data/lessons.ron`, id `units`) has two items: "Jump to the next
unit: {NextUnit}" (`done_when: Pressed("NextUnit")`, `B+→` on the
Playdate since 1034) and "Look at a unit's details: {Info}"
(`done_when: Pressed("Info")`). Unit info has no Playdate button
(`docs/design/playdate.md`, *Buttons and the crank*: it is in the map
menu, ticket 1035), so the item shows `! not mapped` and can't be ticked.
While a lesson is open the turn can't end (`ch01_lessons.rs`,
`the_cursor_lesson_opens_on_turn_one_and_closes_on_the_four_directions`),
so with lessons on a Playdate player is stuck on turn 1. The same holds
for any lesson item whose action has no button there (check every one:
`Pressed("…")` items and the `{Action}` placeholders in their labels and
text).

1035 adds map menu entries for the actions with no button (Danger Zone,
Auto-End, Unit Info; no Dialogue log, Nick 2026-10-17: it only opens in
a conversation, where the system menu has it) and the crank for Rewind
(`RewindBack`, which opens the rewind as `Rewind` does). It says nothing
about lessons, but a map menu shortcut chosen replays its action through
the battle's own path (ADR-0084), so the battle already notices
`Pressed(Info)` and the rest when one is chosen: check it with a test
rather than assume it. Rewind's lesson item is `RewindOpened`, which the
crank does; its label's `{Rewind}` still shows `! not mapped` there. 1058 makes the lesson box fit the screen; it
doesn't change what a lesson asks for.

## Nick input

None. The design already says where these actions are on the Playdate
(the map menu); the lessons only need to say so and count it.

## Scope

**In:**
- A lesson item whose action has no button or chord on the active device
  is ticked by the way the platform offers it: choosing its map menu entry
  (1035) counts as `Pressed(action)` (for Unit info: the entry that shows
  the unit under the cursor).
- Its `{Action}` placeholder in a lesson's label and text names that way
  instead of `! not mapped`: the language file's words for "map menu:
  Unit info" with the map menu's own button (`{Cancel}` with nothing to
  cancel opens it), through `ctx.help_keys()` / the tips placeholder code
  (`crates/ui/src/tips.rs`), never a key name in code (`keyboard-input`
  skill).
- A test walks every Chapter 1 lesson on the Playdate's bindings
  (`Ctx::use_playdate_buttons`, `Keyboard::Absent`) and finishes each one
  with the Playdate's buttons, crank (1035) and map menu only.

**Out (do not do):**
- The PC: its lessons and their text are unchanged (tests).
- The lesson box's look on the Playdate (1058).
- New lessons, or rewording a lesson beyond the placeholder.

## Implementation steps

1. Read `crates/ui/src/screens/battle/lessons.rs` (`tick`, `DoneWhen`),
   `map_menu.rs` after 1035, `crates/ui/src/tips.rs` (placeholders) and
   `assets/data/lessons.ron`.
2. List every lesson item and placeholder whose action has no button in
   `keymap.ron`'s `playdate` table.
3. Tick `Pressed(action)` when the map menu entry for that action is
   chosen (one place, where the entry acts).
4. The placeholder: when `key_name` would give `! not mapped` and the
   action has a map menu entry on this platform, name the entry instead
   (new language strings in `assets/lang/en/ui.ron` and the `test` pack).
5. The walk-through test, plus a unit test per changed rule.

## Acceptance criteria

- [ ] On the Playdate's bindings, every Chapter 1 lesson can be finished
      and the turn ends afterwards (integration test).
- [ ] No Playdate lesson label or text shows `! not mapped`.
- [ ] The PC's lessons are unchanged (existing tests pass unchanged).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the tick from a map menu entry; the placeholder's wording.
- Integration: the Playdate walk through Chapter 1's lessons.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
