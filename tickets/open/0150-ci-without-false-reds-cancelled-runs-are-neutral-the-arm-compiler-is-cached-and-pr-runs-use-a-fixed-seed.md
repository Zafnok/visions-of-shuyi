---
id: "0150"
title: "CI without false reds: cancelled runs are neutral, the ARM compiler is cached, and PR runs use a fixed seed"
type: infra
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
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

- [ ] A push that cancels a running CI leaves the older run's
      `codeql-result` neutral (run link in the notes).
- [ ] The `playdate` job's install step reads the cache on a second run.
- [ ] PR runs print the proptest seed they used.
- [ ] A missed mutant shows in the mutants job summary.
- [ ] A second push to a PR re-tests only the mutants the first run
      didn't catch (`--iterate`; both runs linked in the notes).
- [ ] A push whose tests fail starts no mutants shard.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Workflow runs on the branch, linked in the notes.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
