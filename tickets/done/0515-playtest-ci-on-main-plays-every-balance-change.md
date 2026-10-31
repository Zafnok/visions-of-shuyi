---
id: "0515"
title: "Playtest CI on main plays every balance change: a push's run is never cancelled by the next push"
type: infra
milestone: M4 Enemy AI
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-10-26
---

# 0515 — Playtest CI on main plays every balance change

## Context

From the retro of the last 50 PRs (2026-10-22). Ticket 0514 made
`.github/workflows/playtest.yml` play only when a balance lever changed
(`.github/playtest-levers.sh`): on a pull request it diffs `HEAD^1
HEAD`, on a push to `main` it diffs `github.event.before HEAD`. The
workflow's `concurrency` is one group per ref with
`cancel-in-progress: true`.

On `main` those two rules lose changes. On 2026-10-22 #401 (0513, green
units' items) and #402 (0321, armour weights), both balance levers,
merged 20 seconds apart. Both push runs were cancelled when #400 (1056,
a Playdate build fix, no lever) was pushed three minutes later. #400's
run diffed only its own push, found no lever, and skipped. So the two
balance changes together were not played on `main` until #403 (0608, a
lever) 1.5 hours later. It matched 0855's numbers that time; nothing
guaranteed it.

The playtest only fails on a floor (a battle blocked), so a missed run
on `main` hides exactly the regression it exists to catch.

## Nick input

None. CI only; verified by the workflow's own runs.

## Scope

**In:**
- `.github/workflows/playtest.yml`: a push to `main` gets a concurrency
  group of its own commit, so the next push never cancels it and every
  push's own range is judged. Pull requests keep one group per ref with
  `cancel-in-progress: true` (a newer commit of the same PR supersedes
  the old one, and its range `HEAD^1 HEAD` covers it).
- The comment block at the top of the workflow says why (this ticket).
- `docs/playtesting.md`, where it describes when CI plays: one line.

**Out (do not do):**
- Changing `.github/playtest-levers.sh`'s list of levers.
- Changing when the job fails (a floor only).
- The engine's reusable workflow (engine#2021; this repository's file
  is what 1180 moves onto it, and engine#2021's ticket carries this
  rule since 2026-10-22).

## Implementation steps

1. Change the concurrency block to:
   ```yaml
   concurrency:
     # A push to main is never cancelled: the next push diffs only its
     # own commits, so a cancelled run's balance change would go unplayed
     # (ticket 0515). A PR's newer commit supersedes its older one.
     group: playtest-${{ github.event_name == 'push' && github.sha || github.ref }}
     cancel-in-progress: ${{ github.event_name != 'push' }}
   ```
2. Check that `workflow_dispatch` still plays (it groups by ref, as
   before) and that `zizmor` and `actionlint` (whichever CI runs on
   workflows) pass.
3. Update the header comment and `docs/playtesting.md`.

Expected size: no Rust; about 6 YAML lines and the comments.

## Acceptance criteria

- [x] The workflow's concurrency is as in step 1, with the comment.
- [x] A PR with two quick pushes still cancels the first run (push an
      empty commit to the PR right after the first; the first run shows
      cancelled).
- [x] The Completion notes say how to see it on `main`: the next two
      pushes that land close together each keep their Playtest run
      (`gh run list --workflow Playtest --branch main`).
- [x] All gates in the `run-gates` skill pass, and the workflow linters
      CI runs.

## Tests required

- CI: the workflow's own runs on the PR and on `main` (above).

## Completion notes

- `.github/workflows/playtest.yml`: the concurrency group is the commit
  (`github.sha`) on a push and the ref otherwise; `cancel-in-progress` is
  off on a push. So a push to `main` is never cancelled by the next one,
  and every push's own range `before..HEAD` is judged. Pull requests and
  "Run workflow" keep one group per ref with cancelling, as before. The
  header comment says why.
- `docs/playtesting.md`: one sentence where it says when CI plays.
- zizmor (the workflow linter CI runs, in `security.yml`; CI runs no
  actionlint) reports nothing on the workflow. No Rust changed.
- The PR's cancel check: an empty commit pushed right after the first
  one; the first run shows cancelled (see the PR).
- **To see it on `main`:** the next time two pushes land close together,
  `gh run list --workflow Playtest --branch main` shows a run for each
  push, neither `cancelled`.
- No deviations, no follow-up tickets, no starting rules.
