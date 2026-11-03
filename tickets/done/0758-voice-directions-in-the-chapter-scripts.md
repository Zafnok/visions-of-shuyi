---
id: "0758"
title: Write voice directions into Chapter 1's script where they help
type: content
milestone: M6 Story & dialogue
model: fable-5.1 / gpt-6-astra
effort: medium
status: done
blocked_by: ["0752"]
nick_input: none
completed: 2026-11-03
---

# 0758 — Write voice directions into Chapter 1's script where they help

## Context

0752 added directions to `.dlg` lines (`assets/dialogue/README.md`,
"Directions"), and Nick heard them work on the voice tools. 0737 wrote Chapter
1's script before they existed, so it has none. 0737 said 0752 files this
ticket. They must be in before 0722 generates the chapter, or those lines are
made twice.

## Nick input

None. Directions are script craft, and the agent writes every script.

## Scope

**In:** directions in `assets/dialogue/ch01.dlg`, on the lines where the words,
the punctuation and the expression don't already make the delivery obvious, and
`@delivery` for a stretch said one way.

**Out (do not do):** rewording a line (a new word means a new line id, and that
is a script ticket's job); a direction on every line; generating clips (0722).

## Implementation steps

1. Read the story-writing skill's *Voice directions* and
   `docs/story/characters/` for each speaker's range.
2. Read each scene in order and write directions. A gruff man doesn't turn
   "excitedly" because the scene does. For sadness write "in a low voice, full
   of grief" (ADR-0109).
3. Check that `cargo xtask lines` shows every id unchanged against `main`.

## Acceptance criteria

- [x] `ch01.dlg` has directions, on a minority of its lines.
- [x] No line id changed (compare `cargo xtask lines` with `main`).
- [x] `cargo xtask check-script` passes.
- [x] All gates in the `run-gates` skill pass.

## Tests required

None new: the all-assets test and `check-script` read the file.

## Completion notes

Done 2026-11-03. 34 of Chapter 1's 90 speech and narration lines carry a
direction: 24 on the line itself and 10 through three `@delivery`
stretches (the shouting match across the fort wall in
`ch01_boss_engage_sergeant`; Tamsin's quiet stretch at the body and
Hollis's four eulogies in `ch01_victory`, each inside its `@if` part, so
the `@else` lines take none). The replies are silent and take none. The
lead's four lines take none: the lead's voice is the player's, through the
reply tones.

Where they went, by the voice cards:

- Where the words would be misread without one: Hollis's report of the
  peddler's letter ("flat and even, giving nothing away"), Harl's first
  engage line ("tired and bitter, not shouting") and Rue's line about the
  wax ("cold and exact, not shouting"), since a reader makes an enemy
  shout and an `[angry]` face hot, and Rue goes cold about the Candles.
- Where a character leaves their usual register: Tamsin slowing down at
  "Whose hand, Captain?", Hollis quiet and quick when the bell rings.
- Where a line turns in the middle: Maud reading the paper and then
  speaking, Harl muttering and then shouting at half HP.
- The death quotes (all five, and Maud's dazed retreat line), since
  nothing else tells the voice tool a line is said dying.
- Sadness is always "in a low voice, full of grief" (ADR-0109 §6).

Not directed: lines whose punctuation and expression already carry them
(Tamsin's `!` lines, Rue's questions, the `[happy]` reactions, most
retreat lines) and all the narration except what a `@delivery` covers.

`cargo xtask lines` against `main`: every id and every text identical
(directions are not hashed). No line was reworded. Whether each direction
suits its character when heard is left to the Act 1 playtest (1064), as
ADR-0109 says; 0722 generates the clips.

The gates that depend on the file passed (`check-script`, the test suite, clippy,
`check-text`, `check --target all`, `lint`, `ticket-lint`, `pr-size`,
`typos`, the skill sync check). No Rust, dependency or pin changed, so
the cross-target builds and `cargo deny` were left to CI.
