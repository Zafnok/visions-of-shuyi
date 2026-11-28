# ADR-0100: Mutation shards of at most about 60 mutants, with a 150-minute limit

- **Status:** Accepted; notes added 2026-11-27 (0150)
- **Date:** 2026-10-30
- **Related tickets:** 0133, 0134, 0150
- **Related ADRs:** supersedes ADR-0073's shard rule (Decision §1) and the
  shards' `timeout-minutes: 90` (§3); ADR-0007 (layer 7), ADR-0014

## Context

ADR-0073 picks **1 shard up to 40 mutants, 4 up to 160, otherwise 8**,
each shard with `timeout-minutes: 90`. Past about 700 mutants that puts
more on each of the 8 shards than one can test in 90 minutes.

PR #483 (ticket 1007, about 7,000 changed lines) had about 960 mutants,
so 8 shards of ~120. From the shard logs and check-run annotations of
runs 40984462041 and 40996161089:

| Run | Shards cancelled at 90 min | Runner lost | Mutants tested per shard (logs found) |
| --- | --- | --- | --- |
| 40984462041 | 7 of 8 | 1 (shutdown signal, 73 min) | 67–93 of 120 |
| 40996161089 | 7 of 8 | 1 (lost communication, 86 min) | 67–104 of 118–119 |

A shard tests about **0.75–1.2 mutants a minute** (two at a time): a
caught `ui` mutant takes ~15–60 s, a missed one ~2 min because it runs
the whole suite, and the first build of each copy ~1.5 min. So 120
mutants need 100–160 minutes. `mutants (diff)` was red from the
cancellations alone, whatever was missed.

ADR-0073 turned down more shards because each pays ~3 minutes before its
first mutant and the account runs at most 20 jobs at once, shared with
every PR's CI. Both still hold, but neither is a reason for the gate to
fail: the repository is public, so standard runner minutes are free, and
a queued job's wait does not count toward its `timeout-minutes`, so a
shard that waits for a runner only finishes later.

The gate must not get weaker (CLAUDE.md rule 5): every mutant in the
diff is tested, with the same `--timeout 720`, `--jobs 2` and nextest
profile.

## Decision

In `.github/workflows/mutants.yml`:

1. The plan job picks **1 shard up to 40 mutants, 4 up to 160, 8 up to
   480, and above that one shard per 60 mutants (rounded up), at most
   20**. So 481 → 9, 960 → 16, 1,200 and more → 20. Up to 480 nothing
   changes.
2. Each `mutants (diff, shard N)` job has **`timeout-minutes: 150`**:
   60 mutants take 50–80 minutes at the rates above, so 150 is about
   twice that, room for a shard whose mutants are mostly missed or hit
   the 720 s test timeout.

Everything else in ADR-0073 stands: the plan, baseline, shard and
aggregate jobs; the round-robin sharding; incremental builds; the
`mutants` nextest profile and its slow list; `--timeout 720`;
`--jobs 2`; `mutants (diff)` as the required check.

## Consequences

- No mutant is skipped and none is tested less; only how many runners
  share them and how long each may take change.
- Diffs up to 1,200 mutants (about 9,000 changed lines at #483's
  density) finish in about 60–80 minutes of shard time. Above 1,200 the
  shards grow past 60 again; 20 shards of up to ~110 mutants still fit
  150 minutes at the slowest rate seen, so the gate holds up to about
  2,200 mutants. The PR-size rules (game PRs about 3,000 lines) warn
  long before that.
- A big diff can take up to 20 of the account's 20 concurrent jobs: the
  other PRs' CI queues behind its shards for up to an hour or so, and
  some of its own shards queue too. Diffs that big are rare, and most
  PRs get the same 1, 4 or 8 shards as before.
- Each extra shard repeats setup and a first build (~3 min): a 960-mutant
  diff spends about 24 more runner-minutes than on 8 shards, on free
  minutes.
- A shard that hangs now holds its runner for up to 150 minutes, not 90.
- One shard in each of #483's runs lost its runner before the time limit
  (*shutdown signal*, *lost communication*), which GitHub reports when a
  hosted runner runs out of memory or disk. Smaller shards fill less of
  `/tmp` with incremental builds, which may be enough; ticket 0134 makes
  a shard log its disk and memory so the next loss shows its cause.

## Alternatives considered

- **Only a longer limit** (8 shards, 240–360 minutes) — no extra jobs,
  but a 960-mutant diff takes 2–3 hours of wall-clock, the losses grew
  with how long a shard ran (73 and 86 minutes in), and the limit would
  need raising again for the next bigger diff.
- **Only more shards** (60 per shard, limit kept at 90) — 60 mutants at
  the slowest rate seen take 80 minutes, too close to 90 for a shard
  with a few 720 s timeouts.
- **Smaller shards, 40 mutants each** — about 24 shards for #483, more
  than the 20 jobs the account runs at once, so it would block every
  other PR's CI for its whole run for little gain over 60.
- **No cap on the shard count** — a huge diff could queue dozens of
  shards ahead of every other PR; with the cap, the limit carries a
  diff up to ~2,200 mutants and the PR-size warnings keep diffs smaller.
- **Testing fewer mutants on big diffs** (sampling, skipping crates, a
  shorter `--timeout`) — weakens the gate (CLAUDE.md rule 5).

## Notes

### 2026-11-27: the shards wait for the tests, and a later push tests only what is left (ticket 0150)

Two retros counted pushes the mutants gate caused without a finding:
the shards started beside a CI whose tests were already red, and each
push cancelled the running shards and tested every mutant of the diff
again (#648: 7 follow-up commits, one per red shard).

- **The shards wait for fmt, clippy and the unmutated tests.** A new
  job, `mutants (diff, gate)`, runs CI's `cargo fmt --check` and
  `cargo clippy` commands, and `pr` needs it and `baseline`. That is the
  cheapest way that keeps `mutants (diff)` the workflow's one required
  check (ADR-0014): `baseline` already ran the tests, and only beside the
  shards, so the gate costs one clippy build and makes the shards start
  about 10–15 minutes later on a green PR (`baseline` took 10–15 minutes
  on the PRs of 2026-11-27). Turned down: `workflow_run` on CI (its
  check runs attach to the default branch, not the PR's commit, so
  `mutants (diff)` would no longer gate the PR, and it runs with the
  base's secrets), and a job that waits for CI's checks through the API
  (it holds a runner for CI's slowest leg, macOS or Windows, about 25
  minutes, and couples the two workflows by job names). The gate tests
  only on Linux; CI's `test` still covers the other two.
- **`--iterate` over the mutants earlier pushes caught.** Each shard
  keeps its `caught.txt`, `unviable.txt` and `previously_caught.txt`
  (mutant names only, never `mutants.out`'s logs: ADR-0117, ticket
  0171), also when a newer push cancels it; `mutants (diff, save)` joins
  them with the earlier list into one Actions cache per run,
  `mutants-iterate-pr<N>-<base>-<run>`. The next run's plan restores the
  latest one, counts only the mutants left (`--list --iterate`) and
  picks the shards for them; every shard restores the same cache by its
  exact key, so the round-robin split is the plan's. `<base>` is the
  merge base of the PR's head with its base branch: a rebase or a merge
  of main starts over with every mutant.
- **What this gives up.** A mutant caught on one push is not tested
  again on the next, even if the test that caught it was since changed;
  a mutant is matched by its name, with its line and column, so code
  that moves is tested again. The weekly full run (`mutants (full,
  shard N)`) still tests every mutant of the tree.
