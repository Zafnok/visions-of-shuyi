---
id: "0874"
title: "Vix points at save export: notes on the title, a tip on Profile and Load Game"
type: feature
milestone: M7 Chapter 1 & game flow
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: ["0872"]
nick_input: none
completed:
---

# 0874 — Vix points at save export

## Context

Nick, 2026-11-11, answering 0872 (`docs/design/options.md`, *Saves
outside the game*): export lives on the Profile screen and on the Load
Game list "as long as Vix gives a tutorial for it", and:

> "both but also a quick, persistent Vix note when hovering profile or
> load game, that doesnt draw cursor focus or require any player tutorial
> input. just a note like "Did you know? You can export your [profile /
> save] from this menu!" something like that"

He checked the mockup of the note (beside the title menu, where Vix's
"set up your profile" note from 0842 sits). 0872 builds the export
itself (the Unit info key on a profile or a save).

## Nick input

None: decided in 0872. Vix's wording is the agent's, in plain words (the
dialogue pause allows short instructional tips; the writer may redo them,
`story-writing`).

## Scope

**In:**
- **The title's note:** while the title menu's cursor is on `Load Game`,
  Vix's note beside the menu says one can export a save from there; on
  `Profile`, one can export the profile. Every time, at every tutorial
  level; it never takes the cursor and asks for nothing. Built like
  0842's note (`title::glyph::note_layout`); when 0842's note is up too,
  one of them shows (the agent decides which, and says so).
- **Two tips** (`assets/data/tips.ron`, `crates/ui/src/tips.rs`), once
  per profile: the first time Load Game opens with a save in it, and the
  first time the Profile screen opens. Each names the export key in its
  `how` (never in `text`: Vix speaks it, `keyboard-input`). Kept in
  Guides like the others.
- Not on the Playdate (no export there).

**Out (do not do):**
- The export itself (0872), import (0873).
- Voice clips for the new lines: the voice tickets' pipeline picks them
  up.

## Implementation steps

1. The note's view and skin on the title, from the focused item.
2. The two tips and their triggers.
3. Tests and snapshots; `cargo shuyi frame-png` pictures in the PR.

## Acceptance criteria

- [ ] A Harness test: the cursor on `Load Game` shows the save note, on
      `Profile` the profile note, on any other item none; the cursor
      still moves through the menu as before.
- [ ] A Harness test: each tip shows once per profile, on the screens
      above, and names the export key of the player's keymap.
- [ ] Snapshots of the title with each note; frame-png pictures.
- [ ] No key named in code or text (`cargo shuyi check-keys`).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Snapshot / integration: the Harness tests above.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
