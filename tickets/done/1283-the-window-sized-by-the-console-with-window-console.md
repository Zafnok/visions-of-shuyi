---
id: "1283"
title: "The window sized by the console, with Window::console"
type: infra
milestone: M1 Engine
model: sonnet-5 / gpt-6.1-terra
effort: low
status: done
blocked_by: []
nick_input: none
completed: 2026-11-17
---

# 1283 — The window sized by the console, with `Window::console`

## Context

The engine pin move of 2026-11-17 passed engine PR #469 (engine ticket
3044). The window now opens at the size and in the mode the player left
it in, per computer, and the engine adds `Window::console(title, spec,
scale, margin)`, a `const fn` that sizes a window as a console spec
at a whole scale plus a margin.

The game writes the sum by hand: `crates/app/src/main.rs`'s
`Window::new(APP.name, 1640, 1064)`, "the console at 2x (1600x1024)
plus a small margin". If the console's size changes, the window's
numbers have to be changed with it by hand; `crates/story-preview` opens
its own window the same way.

## Nick input

None: the window's first size stays 1640 × 1064.

## Scope

**In:**
- `WINDOW` in `crates/app` (and `crates/story-preview`'s if it sizes by
  the console) from `Window::console` with the game's console spec,
  scale 2 and its margin, with the icon as now.
- A test that the first size is still 1640 × 1064.

**Out (do not do):**
- Changing the remembered size or mode (the engine's, from the pin).

Estimate: about 5 production lines changed.

## Implementation steps

1. Find the console spec the game builds its frame with, and the
   margin that gives 1640 × 1064.
2. Swap the constant; add the test.
3. `run-gates`.

## Acceptance criteria

- No window size is written as a number in `crates/app` or
  `crates/story-preview`.
- The window first opens at 1640 × 1064 (the test).

## Tests required

The size test above.

## Completion notes

- `WINDOW` in `crates/app` and `crates/story-preview` is now
  `Window::console(APP.name, ConsoleSpec::PC, 2, 40)` with the icon as
  before: the game's console (100 × 32 cells of 8 × 16 = 800 × 512) at
  2x plus a 40-pixel margin each way, so 1640 × 1064 as before.
- Test: `the_window_first_opens_at_1640_by_1064` in `crates/app/src/tests.rs`
  (the story preview has no test module; it uses the same expression).
- No deviations, no follow-up tickets, no gameplay rules decided.
