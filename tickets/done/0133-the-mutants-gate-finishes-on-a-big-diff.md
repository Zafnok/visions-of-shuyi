---
id: "0133"
title: The mutants gate finishes on a big diff
type: infra
milestone: M0 Foundation
model: reasoning
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-10-30
---

# 0133 — The mutants gate finishes on a big diff

## Context

The required `mutants (diff)` check (`.github/workflows/mutants.yml`,
ADR-0073) splits the PR's mutants over shards: **1 shard up to 40
mutants, 4 up to 160, otherwise 8**, each shard with
`timeout-minutes: 90`. A big enough diff puts more mutants on each of
the 8 shards than one can test in 90 minutes, and the gate goes red
from cancelled shards whether or not any mutant is missed.

PR #483 (ticket 1007, about 7,000 changed lines) had about 960 mutants:
8 shards of ~120. In runs 40984462041 and 40996161089, read from the
shard logs (`gh run view <run> --repo Zafnok/visions-of-shuyi --job <id>
--log`) and the check runs' annotations:

| Run | Shards cancelled at 90 min | Runner lost | Mutants tested per shard (logs found) |
| --- | --- | --- | --- |
| 40984462041 | 7 of 8 | 1 (shutdown signal at 73 min) | 67–93 of 120 |
| 40996161089 | 7 of 8 | 1 (lost communication at 86 min) | 67–104 of 118–119 |

A shard tests about 0.75–1.2 mutants a minute (two at a time; a caught
`ui` mutant takes ~15–60 s, a missed one ~2 min because it runs the
whole suite). So ~120 mutants need 100–160 minutes, and 90 is too few.

The repository is public, so standard runner minutes are free; the
account runs at most 20 jobs at once (ADR-0073), shared with every PR's
CI. A queued job's wait does not count toward its `timeout-minutes`.

CLAUDE.md rule 5: the gate must not get weaker. Every mutant in the
diff is still tested, with `--timeout 720` and the `mutants` nextest
profile, as now. Only the number of shards and their time limit change.

A separate ticket of the same session family adds warnings-only PR-size
rules (game PRs about 3,000 lines); that is not this ticket's job.

## Nick input

None. A CI change, verified by the workflow's own run on this PR and by
PR #483's re-run after this one merges.

## Scope

**In:**
- The plan job's shard rule in `.github/workflows/mutants.yml`: more
  shards for big diffs, so each shard has at most about 60 mutants.
- The `pr` job's `timeout-minutes`, raised for headroom.
- ADR-0100, superseding ADR-0073's shard rule and shard time limit
  (ADR-0073's status line and the ADR index updated).
- The `run-gates` skill's sentence on how many shards there are.

**Out (do not do):**
- Anything that tests fewer mutants or tests them less: `--timeout`,
  the nextest profile, `--jobs 2`, `.cargo/mutants.toml`, skipping
  crates or files.
- The weekly full run (`weekly`, `weekly-baseline`).
- PR #483's branch: it re-runs its mutants after this merges, since a
  `pull_request` run uses the merge ref and so `main`'s workflow.
- The runner losses (one shard a run): ticket 0134.
- PR-size rules.

## Implementation steps

1. In `.github/workflows/mutants.yml`, job `plan`, step *Count mutants
   and pick shards*: keep 1 shard up to 40 and 4 up to 160 and 8 up to
   480; above 480, `ceil(mutants / 60)` shards, at most 20. Update the
   comment above the step.
2. Job `pr`: `timeout-minutes: 90` → `150`.
3. Write ADR-0100 (`write-adr` skill) with the evidence above; set
   ADR-0073's status to *Accepted; its shard rule and shard time limit
   superseded by ADR-0100*; update both rows of `docs/adr/README.md`.
4. `.claude/skills/run-gates/SKILL.md`, *Mutation testing (CI only)*:
   "up to 8" → "up to 20".

## Acceptance criteria

- [x] The plan step gives 1 shard for 40 mutants, 4 for 160, 8 for 480,
      9 for 481, 16 for 960, 20 for 1,200 and 20 for 5,000 (checked by
      running the step's shell on those counts).
- [x] `timeout-minutes: 150` on `mutants (diff, shard N)`; `--timeout
      720`, `--jobs 2` and `NEXTEST_PROFILE=mutants` unchanged.
- [x] ADR-0100 exists and is in the index; ADR-0073's status names it.
- [ ] This PR's own `mutants (diff)` check passes (or is skipped as a
      PR without Rust changes) and `actionlint`/CI is green.
- [x] The PR body says that PR #483 re-runs its mutants after this
      merges.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- None in Rust: a workflow change. The shard rule is checked by running
  its shell on the counts above; the workflow is checked by CI.

## Completion notes

- `mutants.yml`: the plan step picks 1 shard up to 40 mutants, 4 up to
  160, 8 up to 480, then `ceil(mutants / 60)`, at most 20; the shards'
  `timeout-minutes` is 150. The step's shell gave 1, 1, 4, 4, 8, 8, 9,
  10, 16, 20, 20 for 0, 40, 41, 160, 161, 480, 481, 541, 960, 1,200 and
  5,000 mutants. `--timeout 720`, `--jobs 2`, the nextest profile and the
  weekly run are unchanged.
- ADR-0100 records it and supersedes ADR-0073's shard rule and time
  limit; ADR-0073's status line and the index name it. `run-gates` says
  up to 20 shards.
- Found while reading the logs: one shard in each of #483's runs lost
  its runner before the limit (out of memory or disk, by GitHub's
  message). Follow-up ticket 0134 makes a shard log its disk and memory.
- The ticket's own mutants check: this PR changes no Rust, so the plan
  job finds no mutants and the shards don't run. The new rule first
  runs on PR #483's re-run after this merges (a `pull_request` run uses
  the merge ref, so `main`'s workflow).
- Gates: no Rust changed, so the build, clippy and test gates are
  unaffected; ran `cargo xtask ticket-lint`, `cargo shuyi check-skills
  .claude/skills`, `typos` and `cargo test -p xtask` (394 passed). CI
  runs the rest.
- Nothing for Nick to play.
