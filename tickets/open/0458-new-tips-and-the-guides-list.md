---
id: "0458"
title: New tips (skills, magic, terrain) and the Guides list in the battle menu
type: feature
milestone: M3 Battle UI
model: sonnet-5
effort: medium
status: todo
blocked_by: ["0455", "0456"]
nick_input: sign-off
completed:
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
  Words by Claude, plain, ≤ 3 lines of 60 characters, `{Action}`
  placeholders.
- **Guides**: a `Guides` entry in the battle's map menu
  (`crates/ui/src/screens/battle/map_menu.rs`). It opens a list of every
  tip and lesson by title (lessons first, in their order, then tips), with
  the selected one's text beside it, the keys filled in from the keymap. A
  lesson's checklist is shown as plain lines, not boxes to tick. Back
  closes it. It works at every tutorial level, `None` included, and every
  topic can be read whether seen or not (*Claude's starting rule*). The
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

- [ ] Each new tip shows once at `Full`, never at `Short` or `None`.
- [ ] The battle menu has `Guides`; it lists every tip and lesson; moving
      through the list shows each one's text with the player's keys.
- [ ] Guides works at `None`.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the new triggers.
- Snapshot / integration: the Guides view; a scripted run opening it from
  the menu and stepping through topics; a rebound key shows in the text.

## Completion notes

*(Filled in by the session that completes the ticket.)*
