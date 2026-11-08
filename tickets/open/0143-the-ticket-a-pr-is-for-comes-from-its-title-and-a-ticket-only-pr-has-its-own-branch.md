---
id: "0143"
title: The ticket a PR is for comes from its title, and a ticket-only PR has its own branch
type: infra
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0143 — The ticket a PR is for comes from its title, and a ticket-only PR has its own branch

## Context

A retro of the game's PRs #513–#580 and the engine's #254–#313
(2026-11-08) found that the ticket lint's `--pr-branch` check
(`check_pr_branch`, `crates/xtask/src/tickets.rs`) reads the ticket only
from a `t<NNNN>-<slug>` branch. That has two costs:

- **Harness worktree branches skip the check silently.** 12 of the last
  60 game PRs were on `claude/<slug>-<hex>` branches, so the "ticket is
  in `done/`" check never ran: #529, #545, #550, #570, #580 and others.
  #570's archived 0843 has all 7 boxes unticked.
- **A ticket-only PR on a `t<NNNN>-` branch fails.** The lint expects
  that ticket in `done/`. #552 was closed and redone as #553 for this,
  and #567 as #578. Both were filed by engine sessions, and #567 sat red
  about 10 hours.
- **`run-gates` runs `ticket-lint` without `--pr-branch`**, so neither
  problem shows before the push.

PR titles are consistent (`[NNNN] …`), so the title is the dependable
signal. The engine's twin is engine#2051.

## Nick input

None.

## Scope

**In:**
- CI passes the PR title to `ticket-lint` (`--pr-title`, through
  `env:` as the `check-mentions` step already does).
- The title decides the kind:
  - `[NNNN] …` is a work PR, whose ticket must be in `done/`;
  - `Ticket NNNN: …` or `Tickets …: …` is ticket-only. Its diff touches
    only `tickets/`, `docs/ROADMAP.md`-like planning files and
    `docs/decisions.md`;
  - `Engine pin: …`, Dependabot and `sonar(…)` keep their treatment.
- A `t<NNNN>-` branch must match the title's ticket. Other branch shapes
  pass when the title names the ticket. Ticket-only PRs use `tickets-<slug>`.
- `run-gates` runs `cargo xtask ticket-lint --pr-branch "$(git branch
  --show-current)"`. `work-ticket` says to rename a `claude/…` branch
  with `git branch -m t<NNNN>-<slug>` **before the first push**, because a
  rename after the PR opens closes it. `write-ticket`, `tickets/README.md`
  and `engine-change` name the ticket-only form.
- The error for a `t<NNNN>-` branch on a ticket-only PR suggests the
  `tickets-<slug>` branch and the `Ticket NNNN:` title.

**Out (do not do):**
- The engine's lint (engine#2051).
- Number collisions (0144).

## Implementation steps

1. In `crates/xtask/src/tickets.rs`, add a `pr_kind(title, branch)` and
   replace `check_pr_branch` with a check over the kind and the diff's
   file list.
2. In `.github/workflows/ci.yml`'s tickets job, pass `--pr-title
   "$PR_TITLE"`, and add `edited` to the `pull_request` types.
3. Update `run-gates`, `work-ticket`, `write-ticket`, `engine-change` and
   `tickets/README.md`, then run `pwsh -File
   scripts/sync-agent-skills.ps1`.

## Acceptance criteria

- [ ] A `claude/x-1a2b3c` branch titled `[0143] …` fails while 0143 is in
      `open/`, and passes once it is in `done/`.
- [ ] A `tickets-x` branch titled `Ticket 0143: …` that only adds the
      ticket passes. A `t0143-x` branch with that title fails with the
      suggestion.
- [ ] A ticket-only title whose diff touches `crates/` fails.
- [ ] `run-gates` runs the branch check locally.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `pr_kind` over each title and branch shape; the check with
  changed-file lists.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
