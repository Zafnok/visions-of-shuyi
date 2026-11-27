---
id: "0150"
title: "CI without false reds: cancelled runs are neutral, the ARM compiler is cached, and PR runs use a fixed seed"
type: infra
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-11-27
---

# 0150 — CI without false reds: cancelled runs are neutral, the ARM compiler is cached, and PR runs use a fixed seed

## Context

The retro of game PRs #513–#580 (2026-11-08) counted red results that
were never about the change:

- **`codeql-result` red on 11 PRs with no finding.** It ran with `if:
  always()` and failed because a newer push had cancelled the `analyze`
  legs. Six of those came from the pin PRs' second push.
- **`playdate`'s "Install the ARM compiler" (`apt-get`) timed out after 8
  minutes:** in #557 and #561. It caused 5 of the last 40 failed CI runs,
  3 of them on `main`. #561 has sat red since.
- **A random proptest seed found a parser bug on macOS only:** #539,
  #544 and #557 (0759, engine#2256). A real bug found this way is good.
  Finding it on whichever PR happens to run is not: it turned `main` red
  after #556 and pushed #557 to narrow a generator.
- **Mutants are the biggest source of extra pushes:** 14 of 44 code PRs
  had a missed mutant on their first push, which took 19 follow-up
  commits. 520, 537, 550, 563 and 568 needed two rounds.

The retro of PRs #616–#735 (2026-11-17,
`docs/research/retro-2026-11-17-prs-616-735.md`, T8) found it again:
#648 took 7 follow-up commits, one per red shard. Each push cancels the
running shards (`concurrency: cancel-in-progress: true`) and re-tests
every mutant of the diff, and the shards (`pr`) need only `[changes,
plan]`, so they start beside a CI whose tests may already be red.

The engine's twin for the aggregators and mutants is engine#2055.

## Nick input

None.

## Scope

**In:**
- `ci-result` and `codeql-result` (and any other aggregator) report
  neutral when the run itself was cancelled (`if: ${{ !cancelled() }}`,
  or by reading the legs' conclusions).
- The ARM toolchain comes from a cache (`actions/cache` on the
  downloaded `.deb`s or a pinned tarball), with one retry on the
  download.
- PR CI runs proptest with a fixed seed set per run in the workflow, so
  a red is reproducible. A scheduled job explores random seeds on
  `main` and files an issue with the failing case.
- The mutants aggregator writes each MISSED mutant (file, line,
  replacement) to the job summary. A `run-gates` step lists the diff's
  mutants before the first push with `cargo mutants --list --in-diff`,
  which lists them and runs nothing, and names the assertion that kills
  each one in new code.
- A follow-up push re-tests only what changed: the shards save
  `mutants.out` to a cache keyed by the PR, restore it on the next push
  and run `cargo mutants --iterate`, which skips mutants an earlier run
  already caught. The cache holds only the outcome lists `--iterate`
  reads, never `mutants.out`'s logs: these shards check out the private
  content, and the test output in the logs can quote it (ADR-0117,
  ticket 0171).
- The mutants shards start only after fmt, clippy and the tests pass:
  a gate job in `mutants.yml` that `pr` needs, or `workflow_run` on CI.
  A PR whose tests are red runs no shard.

**Out (do not do):**
- Running mutants locally (never on Nick's machine).
- Fixing 0759 (engine#2256 and 0759).

## Implementation steps

1. Edit the aggregator jobs in `.github/workflows/ci.yml` and
   `codeql.yml`.
2. Cache the ARM toolchain install in the `playdate` job.
3. Set `PROPTEST_RNG_SEED` (or the proptest config in use) in PR runs.
   Add the scheduled random-seed job, with `issues: write` only there.
4. Add the MISSED summary to `mutants.yml`, and the `run-gates` step.
   Keep zizmor clean.
5. In `mutants.yml`, restore and save each shard's `mutants.out` by PR
   and add `--iterate` to the shards' `cargo mutants`; check that the
   round-robin shards still split the mutants left, and that a cache
   from an older base (a rebase) is dropped.
6. Gate `pr` on fmt, clippy and the tests: the cheapest way that keeps
   `mutants (diff)` the one required check (ADR-0014). Record the
   choice in ADR-0100's notes.

## Acceptance criteria

- [x] A push that cancels a running CI leaves the older run's
      `codeql-result` neutral (run link in the notes).
- [x] The `playdate` job's install step reads the cache on a second run.
- [x] PR runs print the proptest seed they used.
- [x] A missed mutant shows in the mutants job summary.
- [x] A second push to a PR re-tests only the mutants the first run
      didn't catch (`--iterate`; both runs linked in the notes).
- [x] A push whose tests fail starts no mutants shard.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Workflow runs on the branch, linked in the notes.

## Completion notes

**What was done.**

- **Aggregators** (`ci-result`, `codeql-result`, `mutants (diff)`) judge
  their legs with `scripts/ci-result.sh`. A failed leg fails them. A
  cancelled leg fails them only while the run's commit is still the head
  of the PR (or branch). When a newer commit replaced it, the job passes
  with a notice naming both commits. ADR-0014 has a dated note.
- **ARM compiler:** ticket 0192 had already put the `.deb`s in the
  Actions cache (main's run [46051246884](https://github.com/Zafnok/visions-of-shuyi/actions/runs/46051246884) restores
  `arm-none-eabi-debs-resolute-v1`). This ticket adds one retry of the
  whole download, each try capped at 3 minutes.
- **Proptest seed:** `PROPTEST_RNG_SEED: "150"` in `ci.yml` and
  `mutants.yml` (workflow `env`, so the tests, coverage, the mutants
  baseline and shards all draw the same cases). The `test` legs and the
  mutants baseline print it. `proptest-explore.yml` (daily and by hand)
  runs the proptest crates with a random 45-bit seed on all three OSes.
  A `report` job, the only one with `issues: write`, opens or comments
  on one `proptest-seed` issue per failing test. Each issue carries the
  seed, the reproduce line, proptest's minimal failing input and the
  `cc` regression lines.
- **Mutants:**
  - The `mutants (diff)` summary lists every MISSED mutant (and every
    timeout) from small per-shard `mutants-outcomes-*` artifacts. The
    `run-gates` skill now lists the diff's mutants before the first
    push (`cargo mutants --in-diff … --list`, which builds nothing) and
    names the killing assertion for each.
  - Shards run with `--iterate`. Each shard keeps its `caught`,
    `unviable` and `previously_caught` lists, even when cancelled, and
    never `mutants.out`'s logs. `mutants (diff, save)` joins them into
    one cache per run, `mutants-iterate-pr<N>-<merge base>-<run>`. The
    plan restores the latest one and counts only the mutants left. Each
    shard restores the same key, so the round-robin split stays the
    plan's.
  - A new `mutants (diff, gate)` job (fmt, clippy) and the existing
    `baseline` (the unmutated tests) must pass before any shard starts.
    ADR-0100's new notes give the choice and the alternatives turned
    down.

**Runs on the branch (PR #874; the probe commits are reverted).**

- Cancelled run: push `705b26ad` cancelled CI run
  [46058848901](https://github.com/Zafnok/visions-of-shuyi/actions/runs/46058848901)'s Windows test leg, and its `ci-result`
  passed with the notice. Push C cancelled CodeQL run
  [46063190820](https://github.com/Zafnok/visions-of-shuyi/actions/runs/46063190820)'s `analyze (rust)`, and its
  `codeql-result` passed with the notice.
- Cache and seed: run [46058848901](https://github.com/Zafnok/visions-of-shuyi/actions/runs/46058848901)'s `playdate` job
  restored the ARM cache, and its `test (ubuntu-latest)` printed
  `proptest seed 150`.
- Missed mutant in the summary: Mutants run
  [46061349823](https://github.com/Zafnok/visions-of-shuyi/actions/runs/46061349823). The probe's `replace
  zz_probe_0150_missed -> bool with false` is listed, and 4 caught
  mutants were saved.
- Iterate: push B's mutants run (cancelled by C before any shard ran)
  still saved the 4. Push C's run [46064362351](https://github.com/Zafnok/visions-of-shuyi/actions/runs/46064362351)
  restored them ("Iteration excludes 4"), tested only the missed one,
  and listed it again.
- A failing test starts no shard: push D's run
  [46067637152](https://github.com/Zafnok/visions-of-shuyi/actions/runs/46067637152) had red `baseline` (the failing test)
  and `gate` (D's long line also failed fmt), and every shard skipped.
  The first run's empty summary
  ([46058848725](https://github.com/Zafnok/visions-of-shuyi/actions/runs/46058848725)) found a bug, fixed on the branch:
  download-artifact unpacks a single matched artifact into the folder
  itself, so the lists are read with `find`.

**Deviations.**

- "Neutral" is a pass with a notice, not a skipped job. With `if:
  !cancelled()` the aggregate is skipped, and branch protection counts
  skipped as passing, so a run cancelled by hand on the head commit
  would let it merge with nothing tested. Reading the legs' results
  plus the PR's head keeps that case red.
- The ARM cache was already done (0192): only the retry is new.
- `proptest-explore.yml` has not run yet. A schedule or "Run workflow"
  button only exists once the file is on `main`, so its first run is
  after merge. Its issue quotes proptest's minimal failing input. A
  case drawn from the private content could show in a public issue,
  as the logs already can (ADR-0117 §4 accepted logs).
- The gate makes the shards start about 10–15 minutes later on a green
  PR (`baseline`'s time). In exchange, a red PR runs no shard.
- `--iterate` doesn't test a caught mutant again on a later push, even
  if the test that caught it changed. A rebase or a merge of main starts
  over, and the weekly full run tests every mutant.

**Follow-up tickets:** none.

**Notes for Nick:** none: CI only, nothing in the game changes.
