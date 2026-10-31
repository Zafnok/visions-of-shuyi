---
id: "0457"
title: The lessons - basic controls and what's different here
type: content
milestone: M3 Battle UI
model: reasoning
effort: medium
status: done
blocked_by: ["0456"]
nick_input: sign-off
completed: 2026-10-15
---

# 0457 — The lessons

## Context

Ticket 0057 (`docs/design/tutorial.md`, *The lessons*) decided which
lessons exist; ticket 0456 builds the checklist machinery
(`assets/data/lessons.ron`). This ticket writes the lessons, adds the
triggers and `done_when` conditions they need, and checks that each can
happen in Chapter 1 (except terrain magic, which waits for a battle where
it can happen).

**Basic controls** (level `Full`):

1. Moving the cursor: up, down, left, right (turn 1).
2. Choosing a unit, moving it, and its action menu.
3. Attacking: the forecast, then confirming (the first time an enemy is
   in reach).
4. The next unit and a unit's details.
5. Ending the turn.

**Different here** (level `Short`, so it shows at `Full` and `Short`):

6. The keyboard controls: the cursor on keys, and every key can be changed
   in Options (turn 1).
7. Rewind: undoing turns, with a limited number of charges per map. The
   lesson opens the rewind screen and closes it; it never spends a charge
   (0057).
8. Threat arrows: which enemies can reach the tile you're moving to (the
   first time they show).
9. Combat Arts: their own menu, and some cost uses per battle (the first
   time a unit with an art opens its action menu).
10. Terrain magic: a spell can change a tile (the first time a spell that
    can change a tile is in a unit's Magic menu).

## Nick input

**Sign-off:** after merge, play the Chapter 1 opening at `Full` and at
`Short`. The items and timings below that the design doc doesn't fix are
*the agent's starting rules*; list them in the PR for Nick to veto.

## Scope

**In:**
- The ten lessons in `assets/data/lessons.ron`: items, triggers, levels.
  Words are the agent's, in plain American English, ≤ 60 characters a line;
  every key through `{Action}` placeholders (`keyboard-input` skill). The
  guide's voice comes later (0731); write them plainly now.
- At `Full`, lessons 1 and 6 both start on turn 1: merge what 6 teaches
  into 1 when the level is `Full`, or run 6 after 1. Pick one and say which.
- New triggers and conditions the lessons need (in 0456's format), with
  tests.
- Lesson 7's timing: suggested first player phase of turn 2 (something has
  happened that can be undone). Lesson 5's: when no unit is left to act on
  the first turn (before auto-end would fire; auto-end waits for it).
- Check against Chapter 1's data (`assets/data/` and the battle file): if a
  lesson's moment can't happen there (no unit with an art, say), report it
  in the completion notes and the PR. Don't change the map or the units.

**Out (do not do):**
- New tips and the Guides list (0458).
- The guide's name and voice (0731).
- Changing Chapter 1's map, units or script.

## Implementation steps

1. Write the lessons; add triggers and conditions as needed.
2. A scripted Harness test per lesson on the shipped Chapter 1 (or the test
   chapter where Chapter 1 can't reach it), at the level it belongs to,
   doing its items in a shuffled order.
3. A test that at `Short` none of lessons 1–5 opens, and at `None` none
   opens.
4. Render the turn-1 lesson box(es) with `cargo xtask frame-png
   --features private-assets` and look at them.

## Acceptance criteria

- [x] Each lesson opens at its moment at the right levels and closes when
      its items are done.
- [x] Rewind's lesson leaves the rewind charges unchanged.
- [x] Every key in the lessons' text comes from the keymap
      (`cargo xtask check-keys` or the content check passes; no letter
      written in).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Snapshot / integration: one scripted run per lesson; the level gating;
  the rewind charge count.

## Completion notes

- **The ten lessons** are in `assets/data/lessons.ron`, in this order:
  `cursor`, `keys`, `next_unit`, `moving` (all start on turn 1),
  `attacking`, `ending_turn`, `rewind`, `threat_arrows`, `combat_arts`,
  `terrain_magic`. None is limited to a battle, so each one waits for its
  moment in whichever battle that comes, Chapter 1 or later.
- **Lessons 1 and 6 at `Full`: 6 runs after 1** (they aren't merged).
  Lesson 1 is the four cursor directions. Lesson 6 is "move the cursor"
  plus "open Options from the map menu". At `Short`, lesson 6 is the
  first lesson.
- **New triggers** (lessons only, `TipTrigger`): `NoUnitLeft`,
  `SecondTurn`, `ThreatArrowsShown`, `ArtsAvailable` (an action menu
  whose Arts can be used), `TileSpellAvailable` (a spell list with a
  spell that can be cast on a tile from there). **New conditions**
  (`DoneWhen`): `CursorMoved`, `ActionMenuOpened`, `ArtsOpened`,
  `TileAimed`, `ThreatPointed`, `SafePointed`, `RewindClosed` (closed
  without rewinding), `OptionsOpened`.
- **Machinery changes** (0456's tracker):
  - A lesson that still needs a unit to act, when none is left that
    turn, is **put away** unfinished. It comes back, from an empty
    list, the next time its trigger fires in a later turn. Without this
    the turn could never end (for example, "attack" open when nobody can
    reach an enemy).
  - **Auto-end waits while any lesson is in play**, open or about to
    open, not only one that holds the turn. Lesson 5 is the player's to
    do. The triggers are checked again just before auto-end, so lesson 5
    starts before auto-end could end the turn.
- **Checked on Chapter 1** (`crates/ui/tests/it/ch01_lessons.rs`, from
  New Game on the shipped story): every lesson happens there. The attack
  and Arts lessons use the Sergeant (a Rider, the only unit that reaches
  the road's Brigand on turn 1). The threat arrows show on the Sergeant
  where he stands. Terrain magic happens on turn 1: Rue (Mage) on (3, 10)
  next to the western forest, with Cinder Oath. So nothing had to fall
  back to the test chapter.
- **Box width:** the format allows 60-character lines, but with the
  guide's face beside the text, a line over about 45 runs the box onto
  the side panel. The lessons keep to 45 (labels to 41), and a content
  test holds them to it. The first-turn tip does run over the panel
  today: follow-up **0459**.
- `{Menu}` has no key by default (Cancel opens the map menu), so the
  lessons name `{Cancel}` for the map menu.
- 0456's tests now use only the test lesson (the story's lessons start in
  the test chapter and the Quick Battle too).
- Rendered turn 1 at `Full` (the tip, then the cursor lesson, the keys
  lesson, the next-unit lesson) and at `Short` (the keys lesson), with the
  bought art, and looked at them. **Deviation:** the ticket's command
  `cargo xtask frame-png --features private-assets` doesn't exist; the
  working form is `cargo run -p xtask --features private-assets --
  frame-png <out.png> --tips ...`.
- Follow-up: **0459** (the guide's box covers the side panel on long
  lines).

**the agent's starting rules** (gameplay-affecting, for Nick to veto):

1. **The order on turn 1 at Full:** the cursor (move it up, down, left,
   right), then the keys (move the cursor; open Options from the map
   menu), then finding your units (next unit; a unit's details), then
   moving a unit (select one; pick where it goes; see its menu). Each
   one opens when the one before is done.
2. **Attacking** opens the first time a selected unit could reach an
   enemy: open the forecast on an enemy, then attack.
3. **Ending the turn** opens when every unit has acted (any turn, so
   usually turn 1), even with auto-end on: auto-end waits until the
   player ends the turn.
4. **A lesson that can't be finished this turn is put away.** Example:
   "Attacking" is open but every unit has acted. The lesson goes away
   without a tick, the turn can end, and it comes back from scratch the
   next time a unit could reach an enemy. The same goes for "Moving a
   unit", "Threat arrows", "Combat Arts" and "Terrain magic". While units
   are left, the turn still waits, even if none of them can do it (you
   have to make them wait first).
5. **Rewind** opens on turn 2's player phase: open the rewind screen,
   then close it without rewinding. Rewinding instead doesn't tick the
   second item.
6. **Threat arrows** opens the first time the arrows show (in Chapter 1,
   the cursor on the Sergeant): point a selected unit's path at a tile
   with arrows, and at one with none. Pointing at its own tile counts.
7. **Combat Arts** opens the first time a unit's menu has Arts it can
   use right there: open Arts.
8. **Terrain magic** opens the first time a unit's spell list has a
   spell it could cast on a tile from where it stands: aim a spell at a
   tile.
