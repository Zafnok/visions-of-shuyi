---
id: "0860"
title: A New Game options menu
type: feature
milestone: M7 Chapter 1 & game flow
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: ["0826"]
nick_input: decision
completed:
---

# 0860 — A New Game options menu

## Context

Nick, 2026-11-01, in 0043: "yes ask at new game. probably this will need engine
support at some point, like NewGameOptions or some struct etc to build out a menu
before new game... lots of games have this new game options menu".

New Game asks several questions one screen at a time: the game mode
(`docs/design/death-and-difficulty.md`), familiarity (0835) and, after 0826,
voices. This gathers them.

## Nick input

**Decision**, asked before building with a rendered mockup of both looks: which
questions share the menu, whether anything stays its own screen (the lead's name
and gender, the mode's explanation), and the order. His answers go in
`docs/design/`.

## Scope

**In:** one screen listing the New Game choices with their current answers; a
data list of those choices, so that a new question is one entry.

**Out (do not do):** new choices; changing what any answer does; a screen for
changing them after New Game (that is Options and the rules on switching mode).

## Implementation steps

Expected size: about 150 production lines, judged by the Options screen. Follow
the `shuyi-screen` skill: logic, a view of plain data, and a skin.

1. Ask Nick (above).
2. Read the engine's ready-made screens. If the engine's options screen can carry
   this, use it. If a game-neutral part is missing, follow the `engine-change`
   skill, or the `engine-friction` skill if the code runs far over. The game
   doesn't wait on the engine.
3. Build the screen, and move the voices, familiarity and mode questions onto it
   as Nick answered.
4. All text through `ctx.text`, keys through the player's keymap, and no key
   hard-coded (`keyboard-input` skill).

## Acceptance criteria

- [ ] Nick's answers are in the design doc, verbatim.
- [ ] Harness test: each choice can be changed and starts the game with that answer.
- [ ] A new choice takes one data entry (a test adds one).
- [ ] Snapshot of the screen; `cargo shuyi check-keys` passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Snapshot and integration: the screen, through the Harness.

## Completion notes

