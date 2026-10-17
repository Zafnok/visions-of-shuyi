---
id: "1043"
title: "Playdate: the dialogue screen's skin (text box, speaker, one-bit portraits, choices, the dialogue log)"
type: feature
milestone: Post–Chapter 1
model: opus-5.5
effort: high
status: todo
blocked_by: ["1037", "1042", "0249"]
nick_input: sign-off
completed:
---

# 1043 — The Playdate dialogue screen

## Context

Chapter 1 is told in dialogue scenes (`crates/ui/src/screens/dialogue.rs`,
the dialogue log in `dialogue/log*`). Ticket 0249 (split off from 0242)
turns the dialogue screen into a view painted by a skin (ADR-0054). **This
ticket needs 0249 done.** 1042 made the one-bit
faces; 1037 added the `SkinSet` choice.

The Playdate screen is 50 × 15 cells (400 × 240 px). Nick kept portraits
and the dialogue log (its system menu entry, 1035; no map menu entry,
Nick 2026-10-17: the log only opens in a conversation).

## Nick input

**Sign-off:** Nick reads Chapter 1's opening scene in the Simulator or on
his Playdate and comments on readability (text size is today's 8 × 16
font, decided in 0061).

## Scope

**In:**
- A Playdate paint of the dialogue view, chosen by `SkinSet`: a text box
  along the bottom (as many lines as fit with the face beside or above
  it), the speaker's name, the one-bit portrait as a sprite item, the
  reveal of text as on the PC, choices, the skip question, captions.
  Layout: *Claude's starting rule (Nick can veto)*: the PC's arrangement
  (face beside the text box) scaled to the 50 × 15 screen; if it doesn't
  fit 0062's portrait size, the face sits above the box.
- The dialogue log: a full-screen page, scrolled with the cursor.
- Scene places (0029, `look-and-feel.md`, *Places behind conversations*;
  built by 0732–0734): on the Playdate the place is drawn in the glyph
  look, darkened, as the battle map is. If 0732 isn't done, leave it out
  and add it to 0732's follow-ups.
- Text that names a button goes through the keymap (`keyboard-input`
  skill).
- Look tests with the skin; behaviour tests read the view.

**Out (do not do):**
- Voices (1048).
- Changing the PC dialogue screen or any script text.

**Carve (1105):** either order works. If 1105 moved the dialogue format to
the engine, the view and skins stay the game's.

## Implementation steps

1. Read 0242's (c) ticket's view, `dialogue.rs`, `dialogue/log.rs`,
   0062's portrait size.
2. The Playdate paint and its tests.
3. `frame-png --playdate` pictures of a scene (with the placeholder
   faces; never the bought ones) for the PR.
4. Simulator: Chapter 1's opening scene.

## Acceptance criteria

- [ ] Chapter 1's scenes play in the Simulator with faces, names, text,
      choices and the log.
- [ ] Pictures in the PR (placeholder faces only).
- [ ] No PC snapshot changes.
- [ ] Nick's sign-off comment is in the completion notes (or "no comment
      yet").
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit (look, with the skin): text box with face, a choice, the log.
- Integration (harness, Playdate skin set): a scene plays to its end.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
