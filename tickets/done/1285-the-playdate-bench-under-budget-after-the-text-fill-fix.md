---
id: "1285"
title: "The Playdate bench under budget again: move the pin past the engine's faster text fill"
type: infra
milestone: M1 Engine
model: sonnet-5 / gpt-6.1-terra
effort: low
status: done
blocked_by: []
nick_input: none
completed: 2026-11-18
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
- ~~Move the engine pin past engine#3219 (`engine-pin` skill for the PRs
  it passes).~~ (done by the pin move of 2026-11-18)
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

1. ~~Once engine#3219 is on the engine's `main`, run the `engine-pin`
   skill.~~ (done by the pin move of 2026-11-18)
2. Read the PR's `bench` summary; update `docs/targets.md`.

## Acceptance criteria

- [x] `KNOWN_OVER` (or 0185's `bench/known-over.txt`) no longer names
  `battle-cursor-sweep`, and CI's `bench`
  job is green on the pin move's PR.

## Tests required

None of its own: the bench is the test.

## Completion notes

- The pin move of 2026-11-18 (PR #761, engine #485 for engine ticket
  3219) already ran the bench: the Playdate's `battle-cursor-sweep` at
  1,918,808 (2,048,560 at #732, 1,980,877 just before it, on #731),
  with no warning for it. Its line came out of `KNOWN_OVER`
  (`.github/workflows/ci.yml`, with its comment) and the *Over budget*
  entry became a note of how it came back under.
- `docs/targets.md`'s table is #761's numbers, and its web package row
  too (9.2 MB gated, 82.0 MB whole).
- **Deviation: not every scenario is at or below its number before
  #732.** Measured against #731's `bench` job: on the Playdate,
  idle, sweep, enemy turn and attack are below it, but the options menu
  (1,891,040 → 1,907,849) and the dialogue line (2,579,587 → 2,626,898)
  are not: they fill messages with values, which still parse (engine
  #476's other half), and ticket 1299 already covers filling them from
  the forms. On the colour targets the idle battle and the sweep are
  about 0.3 million up, from game PRs after #732 (key caps #737, the
  help bar #742; the bench of each PR since #732 shows the steps), far
  inside the 20 million budget; nothing to undo here.
- **Found:** the Playdate's peak heap went from 5.7 to 9.3 MB between
  #742 and #746 (the chapter title card's lettering sheets), 0.7 MB under
  its 10 MB budget. Follow-up ticket 1300.
- No code, so no engine-friction estimate; gates: ticket-lint and
  pr-size (no Rust changed).
