---
id: "0456"
title: Checklist lessons from the guide; the turn waits until one is done
type: feature
milestone: M3 Battle UI
model: opus-5.5
effort: high
status: todo
blocked_by: ["0455", "0835"]
nick_input: sign-off
completed:
---

# 0456 — Checklist lessons

## Context

Ticket 0057 (`docs/design/tutorial.md`, *The lessons*). Nick: "I like the
implementation of tutorials forcing usage of buttons like use W to move up,
use A to move left, without forcing the specific order (a checklist)", and
the lessons happen inside Chapter 1's battle, each the first time it
matters (Fire Emblem Awakening's prologue, not a practice battle).

A **lesson** is a box from the guide (the tip box with the face, ticket
0455) with a checklist. Each item is something to do, ticked as the player
does it, in any order. When every item is ticked, the lesson closes. While
it is open, the game works as usual under it (the items are real actions),
but **the turn can't end** (*Claude's starting rule* for "forced"). Each
lesson has a level: `Full` lessons show only at `Full`, `Short` lessons
show at `Full` and `Short`, and none show at `None` (the campaign's
`TutorialLevel`, ticket 0835). Each lesson shows once per campaign.

This ticket builds the machinery and one test lesson. The real lessons are
ticket 0457.

## Nick input

**Sign-off:** with 0457, play the Chapter 1 opening at each level.

## Scope

**In:**
- Data: `assets/data/lessons.ron`, loaded and validated by `trpg-content`
  like `tips.ron`. Each lesson: `id`, `level` (`Full` | `Short`),
  `trigger` (when it starts: reuse the tip triggers where they fit and add
  what 0457 needs, starting with `FirstBattleStart`), `title`, `text` (≤ 2
  lines of 60 characters, `{Action}` placeholders as in `tips.ron`), and
  `items`: each a label (`{Action}` placeholders allowed) and a `done_when`
  condition.
- `done_when` conditions, at least: `Pressed(Action)` (the action was used:
  covers the cursor directions, Info, NextUnit, Rewind, DangerZone…),
  `UnitSelected`, `UnitMoved`, `ForecastOpened`, `Attacked`,
  `RewindOpened`, `TurnEnded`. Pick the condition names; each is checked
  where the battle screen already knows it happened (its mode changes and
  the events of applied commands), never from raw keys (`keyboard-input`
  skill: game code reacts to `Action`s).
- The box: the 0455 tip box with the face, the lesson's text, then one row
  per item: `[ ]` / `[x]` and its label (ticked in the "done" colour,
  label dimmed), and `<n> of <m> done`. ADR-0054: view + skin.
- While a lesson is open: `EndPhase` (End Turn, and auto-end) is refused
  with the existing "denied" sound, unless the lesson has a `TurnEnded`
  item. Everything else works. The box doesn't take input, so the map
  under it stays usable. The box sits at the top of the map, and moves to
  the bottom while the cursor is in the top half, so it never hides the
  cursor's tile. *Claude's starting rule.*
- Lessons done are kept with the campaign (`Campaign.lessons_done`, a set
  of ids, serde default empty), so a reload doesn't repeat one.
- Only one lesson at a time; a lesson waits as tips do (combat playback,
  scenes, banners, notes, the rewind screen unless one of its items needs
  it). Tips wait while a lesson is open.
- Rewind and lessons: rewinding never un-ticks an item or reopens a done
  lesson. *Claude's starting rule.*
- A test lesson in `assets-test/` used by the tests (cursor up/down/left/
  right on the test chapter's first turn).
- The bots never see lessons (they play through `core`; lessons are UI
  only). Check that the bot runs are unchanged.

**Out (do not do):**
- The real lessons and their words (0457).
- The Guides list (0458).
- Changing the familiarity level after New Game (0058).

## Implementation steps

1. Format, loader and validation in `trpg-content` (unknown action names,
   text widths, placeholders, an empty `items` list, duplicate ids).
2. `Campaign.lessons_done` in `trpg-core` (plain data) with a serde
   default.
3. A lesson tracker beside the tips tracker in the battle screen
   (`crates/ui/src/screens/battle/tips.rs` is the model): queue on trigger
   if the level allows and it isn't done; tick items from the places that
   already detect them; close and record when all are ticked.
4. Block `EndPhase` while a lesson is open (both the menu's End Turn and
   the end-turn key, and auto-end).
5. The box's view and skin, reusing 0455's face box.
6. Render a frame with `cargo xtask frame-png` and look at it.

## Acceptance criteria

- [ ] At `Full`, the test lesson opens on the first turn; pressing the four
      cursor directions in any order ticks them and closes it.
- [ ] While it is open, End Turn is refused; after it closes, End Turn
      works.
- [ ] At `None` it never opens; a `Full` lesson doesn't open at `Short`.
- [ ] A done lesson doesn't reopen after saving and loading.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the format and validation; the tracker (queue, tick, close, level
  gating).
- Property: ticking items in any order closes the lesson exactly when all
  are ticked.
- Snapshot / integration: the box's view; scripted Harness runs for each
  acceptance line.

## Completion notes

*(Filled in by the session that completes the ticket.)*
