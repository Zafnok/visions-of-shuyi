---
id: "0151"
title: "A PR's closing lines are checked, and the PR template says what work-ticket says"
type: infra
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: low
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0151 — A PR's closing lines are checked, and the PR template says what work-ticket says

## Context

`work-ticket` §6 asks every ticket PR for three closing lines: `Engine
friction:`, `PR size:` and `Nick input:`. The retro of 2026-11-08 found
all 44 ticket PRs in #513–#580 carried them, by habit, since nothing
checks them. Their contents slipped all the same:

- **Estimates written after the code:** #521, #524, #525, #533, #537 and
  #555 admitted it.
- **A size that isn't a measurement:** #563's says "measured within
  2,000". #533's says 1,925 where CI's `pr-size` counted 2,236.
- **Friction counts and rows:** #558's script and YAML were left out of
  its friction count. The 0760 row in `docs/engine/friction.md` has no
  estimate.

`.github/pull_request_template.md` is stale. It still has "Sign-off",
which Nick dropped for gameplay tickets, and lacks the three lines. None
of the PRs sampled used it.

2026-11-17 friction sweep ([report](../../docs/research/friction-2026-11-17.md), *Processes that slow us
down*): this ticket folds into 0181. Its check (the closing lines in the
PR body, and `PR size:` within 10% of `pr-size`) is now one of the checks
of 0181's `cargo xtask check-pr`, in one CI job with those of 1174, 1257
and 1239, and the `Engine friction:` line is left out when a PR changes no
production Rust. What stays here is the PR template's rewrite or removal.
Worked before or after 0181, this ticket now does only that part.

## Nick input

None.

## Scope

**In:**
- The closing-line check is 0181's `cargo xtask check-pr` (see the
  2026-11-17 line), not this ticket's.
- `.github/pull_request_template.md` rewritten from `work-ticket` §6, or
  deleted if §6 is enough. Say which and why.

**Out (do not do):**
- Judging whether an estimate came before the code (0181: the
  `**Estimated size:**` line and the pr-size hook's nudge).
- The PR-body check and its CI step (0181).
- The friction log's rows (1257).

## Implementation steps

1. Rewrite or remove the template, matching `work-ticket` §6 as it
   stands (after 0181, without `Engine friction:` on a PR with no
   production Rust).

## Acceptance criteria

- [ ] The template matches `work-ticket` §6, or is gone.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- None beyond the gates: a template change (the body checks' tests are
  0181's).

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
