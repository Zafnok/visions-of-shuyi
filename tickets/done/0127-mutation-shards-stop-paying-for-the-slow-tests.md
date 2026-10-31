---
id: "0127"
title: Mutation shards stop rebuilding from scratch and paying for the slow tests
type: infra
milestone: M0 Foundation
model: reasoning
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-10-15
---

# 0127 — Mutation shards stop rebuilding from scratch and paying for the slow tests

## Context

Ticket 0119 / ADR-0047 brought the PR mutation gate down to about 19 min for
511 mutants: a `ui` mutant cost 22 s to build and 14 s to test, two at a
time. Two weeks later the same gate takes **67–69 min per shard** for 438
mutants on 8 shards (PR #287, run 38796893781, 2026-10-15). Every shard's
log says the same:

| Per shard | Time |
| --- | --- |
| Setup (checkout, apt, tools) | ~1 min |
| Unmutated baseline | ~70 s build + ~232 s test |
| Each `ui` mutant | **~45 s build + ~95 s test**, two at a time |

Why, from the shard logs and the uploaded `mutants.out`:

1. **The tests: a few slow tests run before the one that fails.** `trpg-ui`
   has 1,961 tests. nextest runs them in name order and stops at the first
   failure, but it waits for the tests already running. A typical caught
   mutant is caught by test ~700, and by then it has run the slow tests
   whose names sort first (`dialogue::…`, `harness::…`, `map_view::…`):
   about 75% of its test time (e.g. 287 of 379 thread-seconds) goes to
   tests over 4 s. In the unmutated run, 32 tests of 5 s or more take 486
   of the 814 thread-seconds; the longest are
   `map_skin::the_skin_never_changes_the_game` (90 s),
   `screens::battle::path::tests::steered_paths_are_legal_and_follow_the_cursor`
   (38 s) and six `map_view` proptests at ~19 s each.
2. **The build: no incremental compilation.** `Swatinem/rust-cache` sets
   `CARGO_INCREMENTAL=0` for the rest of the job. So each mutant compiles
   the whole `trpg-ui` crate (library, its unit tests and the `it` test
   program) from scratch, though only one function changed: 45–49 s.
3. **Every shard runs its own unmutated baseline**: about 4 min of tests
   before the first mutant, on all 8 shards.

More shards would not fix it: each one pays ~6 min before testing any
mutant (setup and baseline), and the account runs at most 20 jobs at once,
shared with every other PR's CI (ADR-0047 Consequences).

Making the slow tests themselves faster is ticket 0128.

## Nick input

`None.`

## Scope

**In:**
- `.config/nextest.toml` (new): a `mutants` profile.
- The `plan`/`pr`/`mutants-result` jobs of `.github/workflows/mutants.yml`.
- ADR-0047 (amend it) and the `run-gates` skill if what it says changes.

**Out (do not do):**
- Weakening the gate (CLAUDE.md rule 5): no `exclude_globs`, no
  `#[mutants::skip]`, no fewer proptest cases, no tests filtered out of
  the mutation run, no lower `timeout_multiplier`. Every test still runs
  for a missed mutant; only the order changes.
- Changing the tests themselves (ticket 0128).
- The weekly full run, the `test` jobs of `ci.yml`, the local gates.
- Running `cargo mutants` on Nick's machine. Measure on CI only.
- Changing the shard thresholds (1 / 4 / 8).

## Implementation steps

1. **Slow tests last.** Create `.config/nextest.toml` with a `mutants`
   profile:
   - `fail-fast = { max-fail = 1, terminate = "immediate" }`: when a test
     fails, stop the tests still running instead of waiting for them
     (nextest 0.9.111+; CI installs the latest).
   - One `[[profile.mutants.overrides]]` with `priority = -100` whose
     filter lists, by exact name (`test(=…)`), every test that took 5 s or
     more in the unmutated run of the shard logs above. A comment says
     where the list comes from and that a renamed test only loses its
     place (it still runs).
   - The default profile stays nextest's default: nothing else uses
     nextest.
2. **Use the profile.** In the `pr` job's mutation step, set
   `NEXTEST_PROFILE: mutants`.
3. **Incremental builds.** Run the mutation step with
   `CARGO_INCREMENTAL=1` (set inline in the command, so it wins over the
   value rust-cache writes to `GITHUB_ENV`). The mutants build in their
   own copies of the tree under `/tmp`, which the cache never saves.
4. **One baseline for the run, not one per shard.** Add a job
   `mutants (diff, baseline)` that needs `plan`, runs when there are
   mutants, and runs `cargo nextest run --workspace --exclude trpg-app
   --profile mutants` on the unmutated tree (with the same setup as a
   shard). Shards run with `--baseline=skip --timeout 720` (720 s: the
   705 s that `timeout_multiplier = 3.0` gave on the 232 s baseline,
   rounded up; say so in a comment). Add the job to the `needs` of
   `mutants (diff)` (ADR-0014). A tree whose tests fail unmutated, or fail
   only under nextest, still turns the required check red.
5. **Amend ADR-0047** with a dated section: what was measured, the three
   changes, and the fixed timeout (which must be raised if the unmutated
   `ui` tests grow past about 240 s).
6. **Measure.** The PR's own run has no Rust change, so measure on a
   scratch draft PR carrying a big `ui` diff (as 0119 did, e.g. PR #287's
   diff), and write the before/after table in the completion notes:
   mutants, wall-clock time, build and test seconds per mutant. Close the
   scratch PR and delete its branch.

## Acceptance criteria

- [x] On the same diff as PR #287 (438 mutants, 8 shards), the gate
      finishes in under 35 min wall-clock with the queue empty (numbers
      and run links in the completion notes).
- [x] The same mutants are tested: the sum of "Found N mutants" over the
      shards equals the plan job's count, and the same mutants are caught
      and missed as in run 38796893781.
- [x] A scratch commit that makes one `ui` test fail on the unmutated tree
      turns `mutants (diff)` red (the scratch PR is closed afterwards).
- [x] A PR with no Rust change still passes without building.
- [x] `.cargo/mutants.toml` is unchanged.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit / property: none (CI configuration).
- Integration: the measured CI runs in the acceptance criteria, linked in
  the completion notes.

## Completion notes

**Done.** ADR-0073 supersedes ADR-0047. The PR run is now plan, baseline,
shards and the `mutants (diff)` aggregate:

- `.config/nextest.toml`: profile `mutants` stops at the first failure and
  terminates the tests still running; the 32 tests of 5 s or more run last
  (`priority = -100`).
- Shards: `NEXTEST_PROFILE=mutants`, `CARGO_INCREMENTAL=1`,
  `--baseline=skip --timeout 720`.
- `mutants (diff, baseline)` runs
  `cargo nextest run --workspace --exclude trpg-app --profile mutants` once,
  beside the shards; the aggregate needs it.

**Measured on CI, 2026-10-15**, same commit (PR #287's head `f321aecd`),
438 mutants, 8 shards, both runs at the same time (so neither had an empty
queue):

| Run | Wall-clock | Shards | Build per mutant | Tests per mutant |
| --- | --- | --- | --- | --- |
| Before: PR #287 ([run 38807102396](https://github.com/Zafnok/visions-of-shuyi/actions/runs/38807102396)) | **70 min** (21:04:57 to 22:15:18 UTC) | 41–70 min | 42.6 s | 89.2 s |
| After: scratch draft PR #288 ([run 38813314359](https://github.com/Zafnok/visions-of-shuyi/actions/runs/38813314359)) | **20 min** (21:54:07 to 22:14:07 UTC) | 11–18 min | 12.4 s | 21.1 s (median 21 s, p90 28 s, max 57 s) |

- Same mutants: the shards found 6 × 55 + 2 × 54 = 438, the plan job's
  count. Every one of the 438 has the same outcome in both runs (418
  caught, 20 unviable, none missed or timed out). The criterion names run
  38796893781, an earlier commit of PR #287 with 4 missed mutants since
  fixed; the comparison uses the run on the same commit instead.
- A failing unmutated test turns the check red: on the scratch branch the
  `ci.yml`, `codeql.yml` and `security.yml` workflows were deleted (so the
  measurement took no runners from other PRs), which made the xtask test
  `every_workflow_file_parses` fail. `mutants (diff, baseline)` went red
  in 4 min and `mutants (diff)` failed, while every shard passed. The log
  also shows the profile working: "immediately terminating 3 tests still
  running".
- No Rust change: this PR's own run found 0 mutants; no shard or baseline
  ran and `mutants (diff)` passed.

**Deviations**

- Step 5 said to amend ADR-0047; accepted ADRs are not edited, so ADR-0073
  supersedes it.
- Gates: no Rust changed, so fmt, typos, ticket-lint and the xtask tests
  (which parse every workflow file) were run locally; CI ran the rest.

**Follow-up:** ticket 0128 makes the slow `ui` tests themselves fast.
