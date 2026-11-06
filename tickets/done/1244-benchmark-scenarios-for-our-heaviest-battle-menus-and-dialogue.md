---
id: "1244"
title: Benchmark scenarios for our heaviest battle, menus and dialogue, gated by the engine's shuyi bench
type: infra
milestone: Later
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: ["engine#3008"]
nick_input: none
completed: 2026-11-05
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

- [x] The scenarios above exist and play what they say.
- [x] CI runs `shuyi bench` on every pull request and posts the table.
- [x] A number over its budget fails the job, naming the scenario, the
      frame and the target.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- CI: `cargo shuyi bench` on the game, as a gate.

## Completion notes

**What was done.** The pin moved to the engine's `main` (`e2697683`) in
its own PR, #557, for engine-pin issue #548; this PR is stacked on it.

- Six scenarios in `bench/`. Each presses the game's actions by name
  (`act("Confirm")`), so it holds for any key layout and on the
  Playdate's buttons:
  - `battle-idle`, `battle-cursor-sweep`, `battle-enemy-turn` and
    `battle-attack` (the forecast, then the combat animation);
  - `options-menu`;
  - `dialogue-portrait`.
- The game's tools open a battle at its start with `--battle <id>` (new
  `trpg_ui::game::start_on_battle`, through
  `Harness::with_layout_starting`) and a scene with `--scene <id>`.
- `GameTools::session_on` plays the Playdate: its console and buttons,
  each frame drawn in one bit as its build draws it, with its pictures.
- CI's new `export` job makes the web package without the bought art,
  after the licenses report it ships. The `bench` job runs `cargo shuyi
  bench` under Valgrind with that package, posts the table as its summary
  and uploads the reports. Both are in `ci-result`.
- `docs/targets.md` describes the targets, budgets, scenarios and
  numbers, and AGENTS.md's map lists `bench/`.
- Tests: `the_bench_scenarios_play_what_they_say` plays each file on the
  colour targets' session and the Playdate's and checks where it ends.
  The start options and `session_on` have tests of their own.

**Deviations.**
- **Biggest battle:** `side_company_ledger`, not Chapter 4. It has 22
  units (Chapter 4 has 21 with Gil) and the most enemies moving on turn 1
  (five of fifteen; Chapter 4 moves one).
- **No seed:** the scenarios start with `--battle`, `--scene` and no
  seed. The battles measured are fixed maps, and the tools had neither a
  stage nor a seed option to reuse.
- **Enemy turn shortened:** bench keeps 600 frames for a whole scenario,
  so `battle-enemy-turn` plays the turn's first 9.9 s of about 12.
- **Unit menu:** Options (18 rows) is the fullest menu. The unit menu's
  six rows open within `battle-attack`.
- **No local instruction counts:** WSL here has neither Valgrind nor
  Rust, so the instruction counts and the web package come from CI's
  `bench` job (`docs/targets.md`). The Playdate's package isn't measured:
  CI has no Playdate SDK.
- **No bought art:** the measured build has none (ADR-0040; no gate
  reads it), so the companions' portraits aren't in the numbers.

**Friction:** estimate 70 production lines, measured 90 (1.3×).

**Follow-ups:** none. A scenario over budget gets a ticket for what is
slow (step 4).

**For Nick:** nothing to check; no gameplay changed.
