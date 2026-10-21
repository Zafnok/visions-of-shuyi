---
id: "0741"
title: "The pre-battle talk's place reads as the green by the chapel"
type: content
milestone: M6 Story & dialogue
model: opus-5.5
effort: medium
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 0741 — The pre-battle talk reads as the chapel green

## Context

0732 draws `ch01_prebattle` over the Chapter 1 battle map, empty of units
and darkened, centred on tile (3, 12): "the green by the chapel, where the
party starts" (`docs/design/look-and-feel.md`, *Places behind
conversations*; 0029's table of which scene gets which place).

Chapter 1 playtest round 3 (0804, 2026-10-16), asked whether the empty
field felt right:

> "that's a chapel? I don't get that ambience whatsoever"

So the place behind the talk doesn't show a chapel, or doesn't read as
one. Look at what the view around (3, 12) shows today in the bought-art
look and the glyph look before proposing anything.

**Waits for the engine carve and the Playdate port** (Nick, 2026-10-16),
so `blocked_by` has 1118 and 1050.

**Re-gated 2026-10-21 (0129):** the carve is done and 1050 was folded
into the Act 1 playtest (1064), so neither 1118 nor 1050 is in
`blocked_by` any more.

## Nick input

**Decision** (`ask-nick`), on real frames of the game with the bought
art, darkened as in the game. Options, for example:

- A. Centre the view so the chapel building on the battle map is in it
  (if the map has one), and make it read as a chapel (a bell, a door, a
  yard).
- B. Add a chapel to the battle map near the start (changes the map:
  terrain under it, the playtest bots' numbers).
- C. A place of its own for the talk, like the yard (0733): the chapel
  green, drawn from the tileset.
- D. Don't call it a chapel: change the script's and beat sheet's words
  instead (needs Nick's OK, it's a story place).
- E. Describe your own.

## Scope

**In:** the question; building the answer if it's A or C (a place map,
`ch01.dlg`'s `@place` line); if B, write the map change as its own ticket
blocked by this one (it touches the battle and the bots).

**Out (do not do):** the script's text (0737); other scenes' places
(0738).

## Implementation steps

1. Render `ch01_prebattle`'s first frame (both looks) and look for the
   chapel.
2. Make the option frames; ask Nick; record the answer in
   `look-and-feel.md` (*Places behind conversations*).
3. Build A or C (the 0733 way for C), or write B's ticket.

## Acceptance criteria

- [ ] Nick picked an option; it's recorded.
- [ ] Built (A, C) with a test that the scene's place/view is the new one,
      or the follow-up ticket written (B).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the scene's place and centre, read from the scene, not
  from cells (ADR-0038).

## Completion notes

*(Filled in by the session that completes the ticket.)*
