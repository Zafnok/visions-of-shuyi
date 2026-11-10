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

## Nick input

None.

## Scope

**In:**
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

1. Read `.github/workflows/mutants.yml` (`plan`, `baseline`, `pr`,
   `mutants-result`, `weekly-baseline`, `weekly`), ADR-0047 and ADR-0100.
2. Make `plan` (or a `weekly-plan` job) count the whole workspace's mutants
   (`cargo mutants --list`) and emit the shard matrix for a full run, as it
   does for `--in-diff`.
3. Point `weekly` at that matrix, with `--test-tool=nextest`, the same
   `--timeout`, `timeout-minutes: 150`, and an artifact per shard.
4. Add a result job that merges the shards' `missed.txt` and fails only on
   a job error, not on missed mutants (the weekly run reports; it gates
   nothing).
5. Run the workflow by hand (`gh workflow run mutants.yml`), watch it once,
   and put its run link and the shard count in the completion notes.

## Acceptance criteria

- [ ] A `workflow_dispatch` run of the full mutation run finishes with
      every shard under 150 minutes (run linked in the completion notes).
- [ ] Its merged report lists the missed mutants in one artifact or summary.
- [ ] The PR gate's jobs are unchanged (`git diff` shows no change to `pr`
      or `mutants-result` beyond shared steps).
- [ ] zizmor passes on the changed workflow.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- None in Rust. The workflow is checked by its run (step 5) and zizmor.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
