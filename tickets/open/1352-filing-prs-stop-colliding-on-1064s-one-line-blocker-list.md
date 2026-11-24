---
id: "1352"
title: "Filing PRs stop colliding on 1064's one-line blocker list"
type: infra
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1352 — Filing PRs stop colliding on 1064's one-line blocker list

## Context

From the [feedback-latency retro](../../docs/research/feedback-latency-2026-11-24.md)
(2026-11-24), bucket *Process and CI* (amended 0182,
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

- [ ] Two branches each adding a different 1064 blocker merge without a
      conflict (a test or a scripted check in the PR).
- [ ] `cargo xtask ticket-lint` still reports a blocker that isn't a
      ticket (a test).
- [ ] The skills carry the two rules and lose the one.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `crates/xtask` ticket-lint over the new format.

## Lesser targets

None (process).

## Completion notes

