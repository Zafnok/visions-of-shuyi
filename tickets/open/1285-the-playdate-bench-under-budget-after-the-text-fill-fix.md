---
id: "1285"
title: "The Playdate bench under budget again: move the pin past the engine's faster text fill"
type: infra
milestone: M1 Engine
model: sonnet-5 / gpt-6.1-terra
effort: low
status: todo
blocked_by: ["engine#3219"]
nick_input: none
completed:
---

# 1285 — The Playdate bench under budget again

## Context

The engine pin move of 2026-11-17 (PR #732, engine PRs #460–#479) made
every frame of the `cargo shuyi bench` scenarios dearer: engine #476
(engine ticket 3218) parses every text it fills as a message, and its
`Ctx` copies its values into a new list on every call. An idle battle
frame on the Playdate went from 1,882,790 to 1,962,514 instructions,
and the Playdate's `battle-cursor-sweep` from 1,982,686 to 2,050,631,
over its 2,000,000 budget. Nick chose to merge the pin move with the
sweep in CI's `KNOWN_OVER` for now (`.github/workflows/ci.yml`, a
warning, not a failure) and fix it after (2026-11-17).

Engine ticket 3219 fills plain text without the message work. This
ticket brings it to the game.

## Nick input

None.

## Scope

**In:**
- Move the engine pin past engine#3219 (`engine-pin` skill for the PRs
  it passes).
- Take `playdate/frame-instructions/battle-cursor-sweep` back out of
  `KNOWN_OVER` in `.github/workflows/ci.yml` (and its line in
  `docs/targets.md`'s *Over budget*); if 0185 is done, out of
  `bench/known-over.txt` instead.
- CI's `bench` job passes: the Playdate's `battle-cursor-sweep` under
  2,000,000 instructions, and the other scenarios at or below their
  numbers before PR #732 (`docs/targets.md`'s table brought up to date).

**Out (do not do):**
- The one-bit drawing's cost (1248).

Estimate: none beyond the pin move's fixes.

## Implementation steps

1. Once engine#3219 is on the engine's `main`, run the `engine-pin`
   skill.
2. Read the PR's `bench` summary; update `docs/targets.md`.

## Acceptance criteria

- `KNOWN_OVER` (or 0185's `bench/known-over.txt`) no longer names
  `battle-cursor-sweep`, and CI's `bench`
  job is green on the pin move's PR.

## Tests required

None of its own: the bench is the test.

## Completion notes

*(Filled in by the session that completes the ticket.)*
