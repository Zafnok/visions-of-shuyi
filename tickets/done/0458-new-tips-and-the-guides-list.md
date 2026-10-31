---
id: "0458"
title: New tips (skills, magic, terrain) and the Guides list in the battle menu
type: feature
milestone: M3 Battle UI
model: implementation
effort: medium
status: done
blocked_by: ["0455", "0456"]
nick_input: sign-off
completed: 2026-10-15
---

# 0458 — New tips and the Guides list

## Context

Ticket 0057 (`docs/design/tutorial.md`, *Tips* and *Guides*). Nick: "I do
like the guides as a menu option, to refresh a player's memory via a quick
look." Fire Emblem Engage has this under System → *Guides*.

Nothing explains skills, magic or terrain today. Rewind, threat arrows,
Combat Arts and terrain magic get lessons (0457), so they get no tip.

## Nick input

**Sign-off:** after merge, open Guides from the battle menu and read a few.

## Scope

**In:**
- Three new tips in `assets/data/tips.ron`, at `Full` only (0835), in the
  guide's box (0455), with new triggers:
  - **Skills**: the first time a unit with a skill is selected (or a skill
    triggers: pick one and say which).
  - **Magic**: the first time a unit that can cast opens its action menu
    (casting and healing).
  - **Terrain**: the first time the cursor rests on a tile with a
    movement cost or a bonus other than plain ground (the side panel
    already shows `DEF +0 AVO +0`).
  Words by the agent, plain, ≤ 3 lines of 60 characters, `{Action}`
  placeholders.
- **Guides**: a `Guides` entry in the battle's map menu
  (`crates/ui/src/screens/battle/map_menu.rs`). It opens a list of every
  tip and lesson by title (lessons first, in their order, then tips), with
  the selected one's text beside it, the keys filled in from the keymap. A
  lesson's checklist is shown as plain lines, not boxes to tick. Back
  closes it. It works at every tutorial level, `None` included, and every
  topic can be read whether seen or not (*the agent's starting rule*). The
  look follows the 0057 mockup (a list on the left, the text on the
  right). ADR-0054: view + skin.
- Text keys for the menu entry and the screen in the language files
  (`test` pack too).

**Out (do not do):**
- Guides outside the battle (title, Preparations, Options): not asked.
- The guide's name and voice (0731).
- Lessons (0456, 0457).

## Implementation steps

1. The three triggers and tips, with tests that each fires once.
2. The Guides screen (an overlay over the battle, like the rewind screen)
   and its menu entry; it reads `tips.ron` and `lessons.ron` from the
   content.
3. Render it with `cargo xtask frame-png` and look at it.

## Acceptance criteria

- [x] Each new tip shows once at `Full`, never at `Short` or `None`.
- [x] The battle menu has `Guides`; it lists every tip and lesson; moving
      through the list shows each one's text with the player's keys.
- [x] Guides works at `None`.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the new triggers.
- Snapshot / integration: the Guides view; a scripted run opening it from
  the menu and stepping through topics; a rebound key shows in the text.

## Completion notes

**Done.**

- Three new tips in `assets/data/tips.ron` (Full only, in the guide's box,
  like every tip): **Skills** (`FirstSkillUnit`: a unit with a skill is
  selected), **Magic** (`FirstCaster`: the action menu opens for a unit
  that knows a spell), **Terrain** (`FirstTerrain`: the cursor rests on a
  tile that isn't plain ground). Each shows once per profile, never at
  Short or None (tests in `tip_tests.rs`).
- **Guides** is the map menu's third entry (Units, Objective, Guides,
  Options, …). It covers the map and the side panel: every lesson then
  every tip by title on the left, the highlighted one's text on the right
  with the player's keys; a lesson's checklist is plain lines under its
  text. Up/Down move (wrapping), Cancel goes back to the map menu. Works
  at every tutorial level. View `GuidesView` + glyph skin
  (`glyph/guides.rs`), ADR-0054. Tips and lessons wait while it is open.
- Text keys `battle.map_menu.guides`, `battle.guides.title`,
  `battle.guides.help` in English and the `test` pack.
- Rendered with `cargo xtask frame-png` and looked at.
- Tests that walked the map menu by a count of Downs got one more Down.

**the agent's starting rules** (Nick can veto):

- The Skills tip shows when a unit with a skill is selected. Every player
  class has an active skill, so in practice it shows on the first
  selection, right after "Moving a unit".
- The Magic tip shows when a spell-knower's action menu opens, whether or
  not a spell has a target there.
- "Not plain ground" = any DEF, AVO or healing bonus, or a move cost other
  than 1 for some movement type (impassable tiles count). Only while
  browsing the map.
- Guides sits after Objective in the map menu; Confirm does nothing in it.
- The shipped game has no lessons yet (0457), so its Guides starts with
  the tips; the test build shows the test lesson first.
- Tip words (the agent's): Skills "This unit has skills. Passive ones work on
  their own; an active one is used from Arts or Skill in the action
  menu. {Info} lists them all…"; Magic "…Pick Magic in the action menu to
  cast at an enemy or to heal an ally. Each spell has a few uses per
  battle…"; Terrain "Terrain matters. The side panel shows the DEF and
  AVO a unit gets standing on this tile…".
