---
id: "0831"
title: "Game Over's words as a picture: a language can bring its own"
type: feature
milestone: M7 Chapter 1 & game flow
model: implementation
effort: low
status: todo
blocked_by: ["0830"]
nick_input: none
completed:
---

# 0831 — Game Over's words per language

## Context

Ticket 0830 made the words `GAME OVER` on the Game Over screen a picture
in the logo's block lettering (`assets/title/game_over.png`, generated
from `assets-src/title/game_over.txt`; `docs/design/look-and-feel.md`,
*Game Over screen*). A picture reads the same in every language: the
language files' `game_over.heading` is only shown when `title.ron` names
no picture.

English is the only shipped language today, so nothing is wrong yet. The
first translated language (tickets 0042, 0045; ADR-0045) needs a way to
show its own words.

## Nick input

None. (Which words a language shows is that language's translation.)

## Scope

**In:**
- A language pack can name its own Game Over picture, or none: with none,
  that language shows its `game_over.heading` as text in the picture's
  place (the fallback 0830 built).
- The `test` language pack (`assets-test/lang/test/`) uses it, so a test
  covers the switch.

**Out (do not do):**
- Drawing the words for any real language.
- The logo: the game's name is not translated here (0045 asks).

## Implementation steps

1. Decide where a language names its picture (ADR-0045's files) and load
   it with the language.
2. `game_over/glyph.rs` takes the picture of the language in use.
3. Document it in `assets/title/README.md` and the language pack's README.

## Acceptance criteria

- [ ] In English the screen is as 0830 left it.
- [ ] Test: a language with its own picture shows it; one with none shows
      its heading as text.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the loader's new field and its errors.
- Snapshot / integration: Game Over in the `test` language.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
