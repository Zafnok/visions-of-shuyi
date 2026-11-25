---
id: "1355"
title: A weekly long run keeps the game's memory and sounds flat over hours of play
type: infra
milestone: Later
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1355 — A weekly long run keeps the game's memory and sounds flat over hours of play

## Context

Engine PR #564 (engine ticket 3509, engine ADR-0146) came in with the pin
move of 2026-11-24 (issue #824). It adds `cargo shuyi bench --long
[--twice]`: it plays each `*.long.ron` scenario of the game for hours of
frames in the `shuyi-bench` profile (never under Valgrind, so it runs on
Windows too), samples the live heap, each memory tag, the sounds alive and
the frame's hash, and fails when the last quarter's most is over the first
quarter's by more than the slack, naming the three tags that grew most.
With `--twice` a second process plays it again and fails at the first
sample whose hash differs. `Harness::keep_logs(false)` stops the test
driver keeping a log line per frame. The engine's own weekly run is
`.github/workflows/long-run.yml` in Shuyi Engine; its docs are
`docs/targets/README.md`, *Long runs*.

The game measures frames only for a few seconds: `bench/*.bench.ron`
(ticket 1244, `docs/targets.md`), played under Callgrind in CI's `bench`
job. Nothing measures a whole session. A slow leak (a cache per battle,
a sound never stopped, a log of every line, the dialogue log or the
voice clips kept after a scene) would show only after hours on the
Playdate, whose heap budget is 10 MB (`targets/playdate.ron`). Nick plays
Act 1 on his Playdate (ticket 1064).

## Nick input

None.

## Scope

**In:**
- One long scenario, `bench/act-loop.long.ron`, that goes round the
  game's loop from the title and back, so each part is entered many
  times: the title, a battle of the test chapter or the debug Quick
  Battle (whatever `bench/*.bench.ron`'s `start` flags reach), a few
  moves and an attack, the enemy's turn, a dialogue scene with a
  portrait, the Options menu, then back to the title. Steps press the
  game's actions by name (`act("Confirm")`), never keys, as the bench
  scenarios do (`keyboard-input` skill).
- A weekly workflow, `.github/workflows/long-run.yml`, on `schedule`
  (Mondays) and `workflow_dispatch`, running `cargo shuyi bench --long
  --twice --out dist/bench` with the private content checked out as the
  `bench` job does, posting `long-report.md` to the job summary, and on a
  failure opening an issue labelled `long-run` or commenting on the open
  one. Model it on the engine's `long-run.yml`; zizmor (the `security`
  workflow) finds nothing in it.
- `docs/targets.md`: a *Long runs* section, what the scenario covers, how
  to run it locally and what a failure names.

**Out (do not do):**
- Running it on every pull request (it takes minutes; it is weekly, as
  the engine's).
- Fixing a leak it finds: each gets its own ticket (`write-ticket`), with
  the tags the report names.
- Changing any budget in `targets/playdate.ron` or the bench scenarios.
- A long run on the Playdate itself (it is the host's heap and sounds,
  as the engine's is).

## Implementation steps

1. Read the engine's *Long runs* (`docs/targets/README.md` at the pin)
   and `examples/hello-grid/long/stage-loop.long.ron` for the fields
   (`repeat`, `idle`, `warm_up`, `sample_every`, `slack`).
2. Write `bench/act-loop.long.ron`. Start from the steps of
   `bench/battle-attack.bench.ron`, `bench/dialogue-portrait.bench.ron`
   and `bench/options-menu.bench.ron`; check by `cargo shuyi frame-png`
   (or the MCP server's `start`/`step`/`scene`) that one loop ends on
   the screen it began on. Pick `repeat` so a run is about a million
   frames; keep the engine's default slack unless a first run shows a
   flat but noisy heap, and say why in a comment if it changes.
3. Run `cargo shuyi bench --long --twice --scenario act-loop` locally and
   note its time and heap range in the file's header comment, as the
   engine's example does. If it fails, file the leak's ticket and set the
   scenario aside from the weekly run only with that ticket named.
4. Write `.github/workflows/long-run.yml` (step "In"), with pinned
   actions as the other workflows have them and `permissions:` limited
   to `contents: read` and `issues: write`.
5. Add the *Long runs* section to `docs/targets.md`.

Expected size: about 150 lines (the scenario about 40, the workflow about
80, the docs about 30).

## Acceptance criteria

- [ ] `cargo shuyi bench --long --twice --scenario act-loop` passes
      locally, and its header comment gives the frames, the time and the
      heap range measured.
- [ ] The scenario's steps name actions, not keys (`cargo shuyi
      check-keys` passes).
- [ ] `gh workflow run long-run.yml` after merge passes, and the job
      summary shows the long report's table.
- [ ] `docs/targets.md` has the *Long runs* section.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- The long run itself is the test (`bench --long --twice`). No unit test:
  the scenario's format is checked by the engine when it reads it.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
