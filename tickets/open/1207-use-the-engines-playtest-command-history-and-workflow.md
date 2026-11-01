---
id: "1207"
title: "Use the engine's playtest command history and workflow"
type: feature
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: high
status: todo
blocked_by: ["1180", "engine#2532", "engine#2533", "engine#2021"]
nick_input: none
completed:
---

# 1207 — Use the engine's playtest command, history and workflow

## Context

This is the command/workflow half of 1180, split by the 2026-11-01 PR
review before implementation. 1180 moves weighted personas, targets,
threats and candidate order onto the engine, preserving the game's
semantic hooks and seeded behavior. It leaves the old xtask command
working so this ticket has a baseline to compare with.

Engine 2532 already owns playtest plans, entrant groups, threads, defaults,
timing and reports; 2533 owns summaries and history; 2021 owns the reusable
workflow. All three shipped before the game's 2026-10-25 engine pin move
to `b4591a81`, and the current pin has passed them. The game still copies
those responsibilities in `crates/xtask/src/playtest.rs` (664 lines) and
its tests (641 lines), plus a workflow loop. This ticket removes those
copies, as the original 1180 required
([ADR-0095](../../docs/adr/0095-playtest-personas-plan-their-turn-and-score-what-a-player-sees.md)).

## Nick input

None. Reports and seeded results are verified against the existing
command, not by a new gameplay sign-off.

## Scope

**In:**
- `Trpg::playtest_plan` in `crates/xtask/src/game_tools.rs`, reusing its
  existing `playtest_stages` and `playtest_defaults` (100 tries, seed 1,
  turn cap 60). Stage ids are the battles in `assets/battles/`; build a
  `BattleStage` from that battle's player slots and the same lead/army
  initialization the old command uses.
- Entrants `baseline`, `casual`, `normal`, `hardcore`, with modes as their
  labels; `all` means the three personas, default entrant is `baseline`.
  The baseline's `--mode classic|casual` remains a game option; personas
  keep their own modes. Use 1180's `BattleEntrant` and targets' checks
  for the stage's difficulty.
- Declare the old report lines as engine summary items over the same
  named measures: won, deaths in Classic or retreats in Casual per try,
  most fallen unit, items median/max and turns median. The engine
  supplies seconds per try, `was ...`, previous date/commit, history,
  report serialization, `--brief`, groups and threads.
- Delete the old `crates/xtask/src/playtest.rs` and its generic wrapper
  tests, the module/dispatch/help in `main.rs`, and the old command's
  direct CLI tests there. Retain game-specific construction and semantic
  regression coverage through the new plan and `crates/xtask/tests/cli.rs`.
  Do not relocate a runner, worker pool, history reader or formatter.
- `.github/workflows/playtest.yml` calls the engine's reusable workflow
  pinned to `shuyi-engine.rev`, with the game's lever script
  `bash .github/playtest-levers.sh`, required macroquad system packages,
  release tool build and `cargo shuyi playtest --every-stage --entrant
  all --brief`. Missing plans are errors, not `allow-no-plan` successes.
  Preserve 0514's lever filtering and 0515's concurrency: each push runs
  to completion; a newer PR commit may cancel its previous run.
- Replace the deleted wrapper's path in `.github/playtest-levers.sh`
  with the plan's new paths. Ensure `engine-pin` moves the reusable
  workflow's engine SHA with the dependency pin, using its existing
  pin-update mechanism and regression tests.
- Update `docs/playtesting.md`, ADR-0095's ownership notes,
  `docs/engine/boundary.md`, command references and this ticket's
  friction row. Keep 1180's before/after counts to verify the original
  combined reduction across `trpg-bots` and `xtask`.

**Out (do not do):**
- 1180's bot/helper migration, which is needed first. No persona tuning,
  new bands/floors, success rules or measure names.
- Training a bot (0507), human calibration (0508), campaign play (0512)
  or the modkit (1021), beyond adapting an already-shipped consumer to
  the same report. Their own feature work remains theirs.
- Reimplementing shipped engine APIs, a new engine feature, or a ticket
  only to move the engine pin.

**Either order with 0507, 0508, 0512 and 1021:** if one is done, migrate
its command/report use to `cargo shuyi playtest` here, preserving its
output (`playtest-compare` still reads personas through 1180's engine
`Persona`); if not, update its ticket to use the engine command when it
lands. 1180 independently owns the shared bot-type adaptation. 0509's
autobalance already shipped: preserve its report and plan integration;
do not resurrect its deleted private search command.

**Either order with 0520:** if it is done, reuse its shared campaign-army
provider for the plan/stage adapter and preserve its carried levels,
resources, pack setup and Chapter 2 reports. If not, retain the existing
starting-army behavior here and leave 0520 the active plan/provider paths
and engine command syntax. Do not restore a fresh-army setup over 0520's
correction or make that balance correction wait for this cleanup.

## Implementation steps

1. Before changes, save the old command's Chapter 1 persona and baseline
   reports (20 tries, seed 1, turn cap 60) outside the repository. Include
   both baseline modes and representative test battles. Keep per-try
   measures, checks/verdicts and displayed summary numbers; timing is
   recorded but cannot be compared for exact equality.
2. Read the engine's current `GameTools::playtest_plan`, `PlaytestPlan`,
   `Summary` and reusable workflow contract. Implement the small plan
   adapter from the game's existing `BattleStage`, entrants and targets.
3. Run matching `cargo shuyi playtest --stage <id> --entrant all --tries
   20 --seed 1 --turns 60 --report <file>` and baseline cases. Compare
   normalized per-try measures and verdicts, not JSON formatting. Find
   any behavior difference; do not retune the personas to conceal it.
4. Remove the old wrapper and replace direct CLI coverage with tests of
   the engine command via the game's tool program; retain only tests of
   game-owned semantics and adapters in the game.
5. Replace the workflow loop, preserve lever/concurrency behavior and pin
   updates, then update docs and consumer references. Exercise the
   reusable workflow on this PR and record its run.
6. Measure production lines and PR size, write the friction row and
   completion evidence, and run the gates including
   `cargo xtask ticket-lint --check-engine`.

**Expected size:** about 80–100 new production lines for the plan and
about 1,305 deleted wrapper/test lines, plus a small replacement workflow,
focused CLI tests, dispatch removal, docs and the ticket move. Plan about
1,950–2,000 total changed lines, including deletions and tests, rather
than counting added production lines alone. Re-estimate before coding;
if the full diff no longer fits, split a standalone piece without
dropping any acceptance requirement (ADR-0101).

## Acceptance criteria

- [ ] The pin is past the three named engine tickets and the plan runs;
      `cargo xtask ticket-lint --check-engine` passes.
- [ ] `crates/xtask/src/playtest.rs` and its wrapper tests are gone;
      `cargo xtask playtest` reports unknown command or xtask usage.
- [ ] New-command persona and baseline runs on the same seeds give the
      old command's per-try measures and check verdicts, including both
      baseline modes; no persona settings or decoded target values change.
- [ ] Every original summary metric listed in Scope has the same number
      and mode meaning. Seconds per try is still reported; the second
      matching run shows `was ...` and previous date/commit from history.
- [ ] `--threads 1` and the default give the same ordered tries,
      measures, verdicts and summary values, ignoring only measured
      wall-clock timing and history timestamp/commit metadata.
- [ ] `all` selects exactly the three personas, the default is baseline,
      and an unknown stage/entrant or invalid mode gives a clear error.
- [ ] The PR's Playtest workflow uses the engine reusable workflow and
      passes, with every battle's table in its job summary; a missed
      band is reported and only a floor/tool failure fails the run.
- [ ] The lever regression tests still skip art/dialogue-only changes
      and play for the new plan's changes; push runs cannot cancel one
      another, and the next pin update moves the workflow SHA too.
- [ ] Against 1180's recorded pre-migration baseline, combined
      `trpg-bots` and `xtask` production lines are down by at least 700,
      measured with `cargo xtask code-lines`, with no relocated copies.
- [ ] The friction log has this step's measured row, and completion
      notes name the engine PRs and link the workflow and comparisons.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the game tool program exposes the plan; the CLI tests
  invoke `cargo shuyi playtest` on test battles, including modes/groups,
  bad options, threads and history's second-run comparison.
- Behavior: preserve `crates/bots/tests/baseline.rs`; keep a small seeded
  per-try measure/verdict fixture if it runs within a minute.
- Workflow: lever-script and pin-update regressions, plus the real PR run
  through the reusable workflow and its complete stage summary.

## Completion notes

*(Filled in by the session that completes the ticket: comparisons,
engine PRs, workflow run, line counts, deviations and friction evidence.)*
