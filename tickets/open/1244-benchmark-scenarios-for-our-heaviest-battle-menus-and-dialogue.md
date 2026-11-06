---
id: "1244"
title: Benchmark scenarios for our heaviest battle, menus and dialogue, gated by the engine's shuyi bench
type: infra
milestone: Later
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: in-progress
blocked_by: ["engine#3008"]
nick_input: none
completed:
---

# 1244 — Benchmark scenarios for our heaviest battle, menus and dialogue

## Context

Engine ticket 3008 (`engine#3008`) built `shuyi bench`: it plays a game's
benchmark scenarios headless and gates each frame's instructions
(counted by Valgrind's Callgrind), the heap's peak and the package's size
against the budgets of every target the game's `shuyi.ron` lists
(engine ADR-0008 §6, engine ADR-0081). The engine ships scenarios over
its examples only. Nick asked for "expected performance benchmarks for
game exports" (the engine's `docs/decisions.md`, 2026-10-15): for this
game, that is its own scenarios over its own content, which never enter
the engine.

How a scenario is written and what `bench` reports:
`https://github.com/Zafnok/shuyi-engine/blob/main/docs/targets/README.md`.
Hello, Grid's scenarios are the worked examples
(`examples/hello-grid/bench/`).

## Nick input

None.

## Scope

**In:**
- ~~Move the engine pin to a revision with `shuyi bench` (engine#3008's
  merge, or later).~~(done by the pin move of 2026-11-05).
- `bench/*.bench.ron` scenarios over the game's content, at least:
  - the heaviest battle: the map with the most units, idle, the cursor
    sweeping it, and the enemy's turn with the most units moving;
  - an attack with its animation and forecast;
  - the fullest menu (the unit menu or the options);
  - a dialogue line with a portrait, typing out.

  Each sets its start through the game's start options (the stage, the
  seed), as `cargo shuyi run` takes them.
- Run `cargo shuyi bench` in CI on Linux on every pull request, after
  the export job, with its web package (`dist/export/web`), as the
  engine's `bench` job does; post the table in the job's summary and
  fail on a number over its budget.
- If the game ships on a one-bit target, its `GameTools::session_on`.
- Record the numbers in the game's docs where its targets are described.

**Out (do not do):**
- Measuring on devices (engine 3011, 3013).
- Raising a budget: only a device measurement changes one.

## Implementation steps

1. ~~Move the pin~~ (done by the pin move of 2026-11-05); check that `cargo shuyi bench --no-instructions` runs on
   the game (on Windows) and lists every scenario.
2. Write the scenarios; check each plays what it says with the MCP
   session tools (`start`, `step`, `scene`).
3. Add the CI job (Valgrind on Linux), and read its first table.
4. If a scenario is over a budget, file a ticket for what is slow, never
   a higher budget.

## Acceptance criteria

- [ ] The scenarios above exist and play what they say.
- [ ] CI runs `shuyi bench` on every pull request and posts the table.
- [ ] A number over its budget fails the job, naming the scenario, the
      frame and the target.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- CI: `cargo shuyi bench` on the game, as a gate.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
