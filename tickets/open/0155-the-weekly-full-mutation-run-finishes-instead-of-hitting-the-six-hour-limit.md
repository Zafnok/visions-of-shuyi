---
id: "0155"
title: The weekly full mutation run finishes instead of hitting the six-hour limit
type: infra
milestone: M0 Foundation
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0155 — The weekly full mutation run finishes instead of hitting the six-hour limit

## Context

Found by 0153's measurement (`docs/research/0153-keeping-the-story-private.md`,
§1.4). From 2026-10-11 to 11-10, every weekly full mutation run
(`mutants.yml`, jobs `weekly-baseline` and `weekly`, Monday 06:00 UTC and by
hand) hit GitHub's 6-hour job limit on all 4 shards: 4 runs cancelled, 1
failed. That is about 1,445 runner minutes a week with no result, and no
full mutation report since the code grew.

The PR gate fixed the same problem with small shards. ADR-0047 introduced
the shards with nextest; ADR-0100 limits a shard to at most 60 mutants and
150 minutes, planned by the `plan` job. The weekly job still runs
`cargo mutants --shard N/4 --baseline=skip` over the whole workspace, with
no nextest, no timeout and no plan.

The repository is public, so the minutes are free today. The wasted hours
still hold 4 of the 40 concurrent jobs every Monday morning.

The retro of PRs #616–#735 (2026-11-17,
`docs/research/retro-2026-11-17-prs-616-735.md`, T9) found the weekly run
hasn't finished in 8 weeks: the scheduled runs of 11-16, 11-09, 11-02,
10-26, 10-12 and 10-05 were cancelled at the limit, and those of 10-19
and 09-28 failed. It asks for this ticket next in the CI work, and for
the schedule off until it lands.

## Nick input

None.

## Scope

**In:**
- First, the schedule off: the `cron` line of `mutants.yml` commented
  out (with a line naming this ticket), `workflow_dispatch` kept, so no
  more 6-hour runs burn while the rest is built. It comes back on only
  once a hand-run of the new plan finishes (step 7).
- The weekly run planned like the PR gate: as many shards as it takes at
  60 mutants or fewer each (ADR-0100), with nextest and the PR gate's
  per-mutant timeout. GitHub allows up to 256 jobs in a matrix. If the
  whole workspace needs more, use the largest shards that still finish in
  150 minutes, and record the count in the ADR.
- The report merged into one artifact (or a job summary) listing the
  missed mutants, as the PR gate's result job does.
- An ADR amending ADR-0100 if the weekly run's rule differs from the PR
  gate's (`write-adr` skill), else a line in ADR-0100's notes.

**Out (do not do):**
- Changing the PR gate (`mutants (diff)`), its shard size or its timeout.
- Fixing the missed mutants the report finds: file them as tickets
  (`write-ticket`).
- Running mutation testing on Nick's PC (never).

## Implementation steps

1. Comment out the `schedule:` cron in `.github/workflows/mutants.yml`
   (keep `workflow_dispatch`) as the branch's first commit. If the rest
   of this ticket can't land within the week, split that one-line change
   off as its own ticket and PR first (`write-ticket`).
2. Read `.github/workflows/mutants.yml` (`plan`, `baseline`, `pr`,
   `mutants-result`, `weekly-baseline`, `weekly`), ADR-0047 and ADR-0100.
3. Make `plan` (or a `weekly-plan` job) count the whole workspace's mutants
   (`cargo mutants --list`) and emit the shard matrix for a full run, as it
   does for `--in-diff`.
4. Point `weekly` at that matrix, with `--test-tool=nextest`, the same
   `--timeout`, `timeout-minutes: 150`, and an artifact per shard.
5. Add a result job that merges the shards' `missed.txt` and fails only on
   a job error, not on missed mutants (the weekly run reports; it gates
   nothing).
6. Run the workflow by hand (`gh workflow run mutants.yml`), watch it once,
   and put its run link and the shard count in the completion notes.
7. Only if that run finished within the limit, turn the cron back on.
   Otherwise leave it off and say so in the completion notes.

## Acceptance criteria

- [ ] A `workflow_dispatch` run of the full mutation run finishes with
      every shard under 150 minutes (run linked in the completion notes).
- [ ] Its merged report lists the missed mutants in one artifact or summary.
- [ ] The Monday cron is on only if that run finished; while it is off,
      `mutants.yml` names this ticket beside it.
- [ ] The PR gate's jobs are unchanged (`git diff` shows no change to `pr`
      or `mutants-result` beyond shared steps).
- [ ] zizmor passes on the changed workflow.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- None in Rust. The workflow is checked by its run (step 6) and zizmor.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
