---
id: "1352"
title: "Filing PRs stop colliding on 1064's one-line blocker list"
type: infra
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-11-24
---

# 1352 — Filing PRs stop colliding on 1064's one-line blocker list

## Context

From the [feedback-latency retro](../../docs/research/feedback-latency-2026-11-24.md)
(2026-11-24), bucket *Process and CI* (siblings: 1351; amended 0182,
0184). 1064 (the Act 1 playtest) has 218 blockers on one line of
frontmatter (130 on 11-08). Every PR that files tickets for Act 1 edits
that line, so two open filing PRs always conflict: #798 needed two merges
of main and a renumbering, and the merges re-run CI (about 5 h of CI
wall time in total over the range). Sessions also merge main into a
branch with no conflict, which re-runs every check.

Two rules the retro adds to `write-ticket`, with one removed in exchange:

- a sweep's clean-up ticket becomes a blocker of a feedback build only
  if the build truly uses its output (0334 has waited about 160 h on the
  clean-up 0335 since its decision);
- a design decision goes into 1064's `blocked_by` only with a one-line
  reason the playtest can't run without it (29 of 1064's blockers are
  open Nick decisions, 9 from one request).

## Nick input

None (process).

## Scope

**In:**
- 1064's blockers move out of one line: either a list file the lint
  reads (one id per line, sorted, so two PRs adding different ids merge
  cleanly), or `blocked_by` generated from tickets that say
  `blocks: ["1064"]`. The agent picks and updates `cargo xtask
  ticket-lint` and its tests; `tickets/README.md` says how.
- `write-ticket`: the two rules above; "merge main only for a real
  conflict" in `work-ticket`.
- **A rule out:** `write-ticket`'s *Check the dependency chain* bullet
  "When unsure, add it" for 1064 goes (the reason line replaces it).
- Sync the skill loaders.

**Out (do not do):** pruning 1064's current blockers beyond moving them;
the decision queue itself (0184).

## Implementation steps

1. Pick the format; move the ids; teach `ticket-lint` (with a test that
   a missing or done id is reported as today).
2. Skill and README edits; sync.

## Acceptance criteria

- [x] Two branches each adding a different 1064 blocker merge without a
      conflict (a test or a scripted check in the PR).
- [x] `cargo xtask ticket-lint` still reports a blocker that isn't a
      ticket (a test).
- [x] The skills carry the two rules and lose the one.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `crates/xtask` ticket-lint over the new format.

## Lesser targets

None (process).

## Completion notes

- **Format: `blocks: ["1064"]` on the blocker tickets**, not a list file.
  New tickets always take the highest ids, so in a sorted list file every
  filing PR would add its line at the end, and two of those still conflict.
  With `blocks`, a PR that files a blocker edits only its own ticket's
  file. 1064's 217 ids (not 218: the retro's count was one off) moved to a
  `blocks: ["1064"]` line under each ticket's `blocked_by`, done tickets
  included (no pruning). 1064 keeps `blocked_by: []` with a comment saying
  where its blockers are, and its text says how to check them.
- `ticket-lint` reads the optional `blocks` key and reports one that isn't
  a list, names an id that isn't a ticket (engine ids too), names the
  ticket itself, or repeats a pair that the other ticket's `blocked_by`
  already holds. Tests in `crates/xtask/src/tickets.rs`, among them
  `two_branches_filing_different_blockers_merge_without_a_conflict` (a
  real git repository, two branches, one merge) and
  `run_reports_a_blocks_id_that_is_not_a_ticket`.
- The readiness rule in `tickets/README.md`, `work-ticket` and
  `write-ticket` now adds "and no open ticket names it in `blocks`", with
  the grep that lists 1064's open blockers.
- `write-ticket`: "When unsure, add it" is gone; a design decision blocks
  1064 only with a one-line reason as a comment on its `blocks` line; a
  sweep's clean-up blocks a feedback build only if the build uses its
  output. `work-ticket` §6: merge `main` only for a real conflict, never
  to catch up. Loaders synced (no loader text changed).
- No gameplay rules decided.
