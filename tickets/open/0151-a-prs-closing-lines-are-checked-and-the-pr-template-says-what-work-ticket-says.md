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

## Nick input

None.

## Scope

**In:**
- A CI step that reads the PR body (passed through `env:`) on a ticket
  PR and fails when:
  - a line is missing;
  - `PR size:` has no measured number;
  - the measured number differs from `cargo xtask pr-size`'s count by
    more than 10%;
  - `Engine friction:` has no `estimate N`.
- `.github/pull_request_template.md` rewritten from `work-ticket` §6, or
  deleted if §6 is enough. Say which and why.

**Out (do not do):**
- Judging whether an estimate came before the code (0148's skill text
  covers that).
- The friction log's rows (1257).

## Implementation steps

1. Add a `cargo xtask check-pr-body` reading `$PR_BODY`, used only on
   ticket PRs (the kind 0143 reads from the title, or `[NNNN]` until
   0143 lands).
2. Wire it in the tickets job beside `check-mentions`, with `edited` in
   the `pull_request` types.
3. Rewrite or remove the template.

## Acceptance criteria

- [ ] A ticket PR body without `PR size:` fails the step.
- [ ] A `PR size:` 30% off `pr-size`'s count fails.
- [ ] The template matches `work-ticket` §6, or is gone.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the body parser over fixture bodies.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
