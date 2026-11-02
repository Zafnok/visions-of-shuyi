---
id: "0758"
title: Write voice directions into Chapter 1's script where they help
type: content
milestone: M6 Story & dialogue
model: fable-5.1 / gpt-6-astra
effort: medium
status: todo
blocked_by: ["0752"]
nick_input: none
completed:
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

- [ ] `ch01.dlg` has directions, on a minority of its lines.
- [ ] No line id changed (compare `cargo xtask lines` with `main`).
- [ ] `cargo xtask check-script` passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

None new: the all-assets test and `check-script` read the file.

## Completion notes

