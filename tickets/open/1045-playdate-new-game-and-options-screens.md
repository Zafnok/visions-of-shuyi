---
id: "1045"
title: "Playdate: New Game and Options screens in one bit (mode, familiarity, lead select, options, button bindings)"
type: feature
milestone: Post–Chapter 1
model: sonnet-5
effort: high
status: todo
blocked_by: ["1037", "1035", "0247"]
nick_input: sign-off
completed:
---

# 1045 — The Playdate's New Game and Options screens

## Context

Each screen builds a view a skin paints (ADR-0054). The Playdate gets a
one-bit paint of each Chapter 1 screen, chosen by 1037's `SkinSet`, at
50 × 15 cells. This ticket does the screens from New Game to the first
scene, and Options:

| Screen | View today |
| ------ | ---------- |
| Mode select (`screens/mode_select*`) | `ModeSelectView` |
| Familiarity (`screens/familiarity.rs`) | the mode screen's view |
| Lead select (`screens/lead_select.rs`), with the letter grid `NameEntry` | after 0247 |
| Options (`screens/options*`) | `OptionsView` |
| Key bindings (`screens/key_bindings*`), its controller side | `KeyBindingsView` |

The Playdate has no keyboard (1033), so the lead's name is typed on the
letter grid, and Key bindings shows only the Playdate's buttons, chords
(1034) and crank (1035): players rebind everything there
(`keyboard-input` skill).

## Nick input

**Sign-off:** Nick goes through New Game to the first scene and opens
Options in the Simulator or on his Playdate, and comments.

## Scope

**In:**
- A Playdate paint for each screen above, one-bit, 50 × 15, in the
  style 1038 set for windows (black frame, white inside).
- Options: only rows that mean something on the Playdate (e.g. no
  window/fullscreen, no keyboard layout, no colour theme; language only
  if the build has more than one: one build per language, 0061). Decide
  which rows from what each does; the hiding is logic in the options
  screen (view) under the "no keyboard"/Playdate setting, so the PC
  keeps every row. List the hidden rows in the PR.
- Key bindings on the Playdate: the controller side only, naming the
  Playdate's buttons, chords and crank, and capturing a chord when
  rebinding (1034's API).
- Profiles: if 0837 (the Profile screen) has landed, its Playdate paint
  too; if not, 0837 adds a line (see its note).
- Look tests with each skin; behaviour tests read the views.

**Out (do not do):**
- Title screen (1044); preparations, results, saves, game over, credits
  (1046).
- Any PC change.

**Carve (1112):** either order works. If generic screens and widgets
moved to the engine, their Playdate paints are the game's skins over
them.

## Implementation steps

1. Read ADR-0054 and each screen's view; `docs/design/playdate.md`.
2. One paint at a time with its look test.
3. Options rows: decide and test (Playdate hides, PC keeps).
4. `frame-png --playdate` pictures of each screen for the PR.

## Acceptance criteria

- [ ] In the Simulator: New Game → mode → familiarity → lead (name typed
      on the grid) → the first scene; Options opens from the title and
      the system menu; a binding can be changed and restored.
- [ ] Pictures of each screen in the PR.
- [ ] No PC snapshot changes; the PC's Options keeps every row.
- [ ] Nick's sign-off comment is in the completion notes (or "no comment
      yet").
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit (look, with each skin).
- Integration (harness, Playdate skin set): New Game to the first scene;
  rebinding on Key bindings.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
