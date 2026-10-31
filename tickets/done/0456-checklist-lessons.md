---
id: "0456"
title: Checklist lessons from the guide; the turn waits until one is done
type: feature
milestone: M3 Battle UI
model: reasoning
effort: high
status: done
blocked_by: ["0455", "0835"]
nick_input: sign-off
completed: 2026-10-15
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
but **the turn can't end** (*the agent's starting rule* for "forced"). Each
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
  cursor's tile. *the agent's starting rule.*
- Lessons done are kept with the campaign (`Campaign.lessons_done`, a set
  of ids, serde default empty), so a reload doesn't repeat one.
- Only one lesson at a time; a lesson waits as tips do (combat playback,
  scenes, banners, notes, the rewind screen unless one of its items needs
  it). Tips wait while a lesson is open.
- Rewind and lessons: rewinding never un-ticks an item or reopens a done
  lesson. *the agent's starting rule.*
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

- [x] At `Full`, the test lesson opens on the first turn; pressing the four
      cursor directions in any order ticks them and closes it.
- [x] While it is open, End Turn is refused; after it closes, End Turn
      works.
- [x] At `None` it never opens; a `Full` lesson doesn't open at `Short`.
- [x] A done lesson doesn't reopen after saving and loading.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the format and validation; the tracker (queue, tick, close, level
  gating).
- Property: ticking items in any order closes the lesson exactly when all
  are ticked.
- Snapshot / integration: the box's view; scripted Harness runs for each
  acceptance line.

## Completion notes

- **Format** (`assets/data/lessons.ron`, `crates/content/src/lesson.rs`):
  `id`, `level` (`Full` | `Short`), `trigger` (the tips' triggers,
  `TipTrigger`, reused as is: 0457 adds what it needs), optional
  `battles` (the battle ids it may start in; any if left out), `title`
  (≤ 30), `text` (1–2 lines of ≤ 60), `items` (1–6, each a one-line
  `label` of ≤ 56, so `[ ] ` + label is a text line), optional `face`.
  `done_when`: `Pressed("<Action>")`, `UnitSelected`, `UnitMoved`,
  `ForecastOpened`, `Attacked`, `RewindOpened`, `TurnEnded`. Checked:
  ids, lengths, placeholders, unknown action names, empty or too many
  items, two items with the same `done_when`, unknown battles, the
  guide's expressions. The story's file has no lessons yet (0457 writes
  them); its header documents the format.
- **Test lesson** (`assets-test/data/lessons.ron`, `test_cursor`): the four
  cursor directions, `battles: ["test", "quick"]`. **Deviation:** the
  `battles` field isn't in the ticket. Without it the test lesson would
  open on Chapter 1's first turn in every debug build and on Pages (both
  embed the test files). It also runs in the Quick Battle, so
  `frame-png --tips` can show it and Nick can try the machinery on Pages
  before 0457. `lessons.ron` is now the fourth table the test files add
  entries to by id (`assets-test/README.md`).
- **Translatable** like tips: `lessons.<id>.title`, `.text`, `.item_<n>`
  in a pack's `data.ron`, with the same limits (`assets/lang/README.md`).
  The test pack translates the test lesson's title and first item.
- **Campaign** (`trpg-core`): `Campaign.lessons_done`, a set of ids,
  `#[serde(default)]` and left out of a save while empty, so no
  `SAVE_VERSION` bump (as `tutorial` in 0835); the golden saves are
  unchanged. The game flow copies it into `Ctx::campaign_lessons` when a
  campaign is adopted, and back after every frame, so a suspend or a
  chapter save made right after a lesson closes keeps it.
- **Tracker** (`crates/ui/src/screens/battle/lessons.rs`): queues a
  lesson its trigger starts when the level shows it (no campaign counts as
  `Full`), its battle matches, and it isn't done or waiting; one at a time.
  A lesson is *open* once it has been on screen; only then do its items
  tick, and only then does it hold the turn. Ticks live in the battle
  screen, not the battle's state, so a rewind never un-ticks one.
- **What ticks what** (all from `Action`s, mode changes and applied
  commands' events, never raw keys): `Pressed(a)` when the map (not a box
  over it, not the rewind list) used the action in the player's phase,
  menus included, and also at the map's edge where the cursor can't move;
  `UnitSelected` when a unit becomes selected; `UnitMoved` when a selected
  unit's destination is picked (its walk, action menu or forecast opens),
  staying put too, or a move after its action is sent; `ForecastOpened`
  when the attack forecast opens; `Attacked` when a player command's
  combat had a player unit attacking; `RewindOpened` when the rewind
  screen opens; `TurnEnded` when the player's `EndPhase` is applied.
- **The turn waits:** while a lesson without a `TurnEnded` item is open,
  the End turn key and the map menu's End turn play the denied sound and
  change nothing (no prompt opens); auto-end waits silently and ends the
  phase once the lesson closes.
- **The box** (ADR-0054): `LessonView` (`crates/ui/src/tips/view.rs`)
  painted by `paint_lesson` (`crates/ui/src/tips/glyph.rs`), sharing the
  tip box's frame and face. Rendered with `cargo xtask frame-png --tips`
  on the Quick Battle, with and without the bought art, and looked at: the
  face, text, ticks and count fit; the box clears the cursor.
- **Tooling:** `Ctx::lessons` (off in tests, on in `app`),
  `Harness::with_lessons`; `frame-png --tips` now shows the lessons too.
  The battle screen's per-key code moved into `play_key` (clippy's
  function length).
- Bots: they play through `core` and never see lessons; their tests are
  unchanged and pass.
- No follow-up tickets.

**the agent's starting rules** (gameplay-affecting, for Nick to veto):

1. When a tip and a lesson start at the same moment (the first turn at
   Full), the tip shows first and the lesson opens when it is closed.
   From then on, tips wait while the lesson is on screen.
2. Auto-end doesn't play the denied sound: it waits quietly, and ends the
   turn as soon as the lesson's list is done.
3. A lesson left unfinished (battle restarted, or suspended and
   continued) opens again from an empty list; a finished one never comes
   back in that campaign, even after a rewind or a lost battle.
4. A key press counts for its item even where it does nothing on the map
   (e.g. "move it up" at the map's top edge, or up in a menu).
5. Picking where a unit goes counts as moving it even when it stays on its
   own tile.
6. The box sits against the top edge of the map, and against the bottom
   edge while the cursor is in the top half (the ticket's rule).
