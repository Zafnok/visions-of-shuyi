# ADR-0073: Mutation shards build incrementally, run the slow tests last, and share one baseline

- **Status:** Accepted
- **Date:** 2026-10-15
- **Related tickets:** 0127, 0128, 0119
- **Related ADRs:** supersedes ADR-0047; ADR-0007 (layer 7), ADR-0008, ADR-0014

## Context

ADR-0047 split the PR mutation gate into shards run by nextest. On
2026-10-03 a `ui` mutant cost 17–22 s to build and 7–14 s to test, and a
511-mutant diff took 19 min. By 2026-10-15 the same gate took 67–69 min per
shard for a 438-mutant diff on 8 shards (PR #287, run 38796893781):

| Per shard | Time |
| --- | --- |
| Setup | ~1 min |
| Unmutated baseline | ~70 s build + ~232 s test |
| Each `ui` mutant | ~45 s build + ~95 s test, two at a time |

From the shard logs and `mutants.out`:

- **Tests.** nextest runs tests in name order, stops at the first failure,
  and by default waits for the tests already running. A caught `ui` mutant
  typically fails around test 700 of 1,961, after the slow tests whose names
  sort first. About 75% of its test time is in tests over 4 s. In the
  unmutated run, 32 tests of 5 s or more take 486 of 814 thread-seconds.
- **Builds.** `Swatinem/rust-cache` sets `CARGO_INCREMENTAL=0` for the job,
  so every mutant compiled the whole `trpg-ui` crate and its two test
  programs from scratch (45–49 s) to change one function.
- **Baseline.** Every shard ran the full unmutated test run (~4 min) before
  its first mutant.

More shards would not help: each shard pays its setup and first build before
testing anything, and the account runs at most 20 jobs at once, shared with
every PR's CI.

## Decision

The PR run of `.github/workflows/mutants.yml` is four jobs:

1. `mutants (diff, plan)` counts the diff's mutants with
   `cargo mutants --in-diff git.diff --list` (nothing is built) and picks
   **1 shard up to 40 mutants, 4 up to 160, otherwise 8**. With no mutants,
   nothing else runs.
2. `mutants (diff, baseline)` runs
   `cargo nextest run --workspace --exclude trpg-app --profile mutants` once
   on the unmutated tree, at the same time as the shards. It fails when the
   tests fail without any mutant, or fail only under nextest.
3. `mutants (diff, shard N)`, a matrix, each running
   `CARGO_INCREMENTAL=1 cargo mutants --in-diff git.diff --jobs 2 --shard N/n --sharding round-robin --test-tool=nextest --baseline=skip --timeout 720`
   with `NEXTEST_PROFILE=mutants` and `timeout-minutes: 90`.
4. `mutants (diff)`, the aggregate of ADR-0014, needs all of the above and
   **is the required check**.

The `mutants` profile in `.config/nextest.toml`:

- `fail-fast = { max-fail = 1, terminate = "immediate" }`: the first
  failing test ends the run and stops the tests still running.
- Every test that takes 5 s or more gets `priority = -100`, so it runs after
  all the others. The list is written by hand; a test that is renamed or
  becomes fast only loses its place. A new test of 5 s or more is added to
  the list (`run-gates` skill).

Kept from ADR-0047: nextest is the test runner of the PR mutation gate only
(not the `test` jobs, the local gates or the weekly run); it does not run
doc tests, so a mutant only a doc test catches counts as missed; `--jobs 2`,
never more; `.cargo/mutants.toml` unchanged.

The test timeout is fixed at **720 s**: the 705 s that
`timeout_multiplier = 3.0` gave on the 232 s unmutated `ui` run, rounded up.
It must be raised if the unmutated `ui` run grows past about 240 s (the
baseline job's log shows it).

## Consequences

- No test is skipped and none checks less: a missed mutant still runs every
  test. Only the order, the stopping and the build change.
- A caught mutant no longer pays for the slow tests unless one of them is
  what catches it.
- Incremental builds make each copy's `target/` bigger in `/tmp`; the
  runner's disk has room, and those copies are never cached.
- Each shard saves the ~4 min unmutated test run; the first mutant of each
  copy does the full build the baseline used to do.
- The timeout no longer follows the size of the test suite by itself.
- The slow list goes stale as tests change. Stale entries cost nothing; a
  missing entry only makes some mutants slower.
- The baseline job is one more runner per PR with mutants, for ~5 min.

## Alternatives considered

- **More shards** (or one shard per ~25 mutants) — each shard pays ~3 min
  before its first mutant, and 16 shards would take most of the account's
  20 concurrent jobs from the other PRs' CI.
- **Fewer proptest cases or skipping the slow tests in the mutation run** —
  weakens the gate (CLAUDE.md rule 5).
- **`opt-level = 1` for the workspace crates in tests** — the slow tests
  would run faster, but every mutant's rebuild would take longer; ticket
  0128 makes the slow tests themselves cheap instead.
- **Baseline only on shard 0, skipped on the others** — keeps the check
  without a new job, but shard 0 would be ~4 min longer than the rest, and
  the wall-clock time is that of the slowest shard.
- **`--baseline=skip` with no baseline anywhere** — a tree whose tests fail
  only under nextest would make every mutant look caught.
