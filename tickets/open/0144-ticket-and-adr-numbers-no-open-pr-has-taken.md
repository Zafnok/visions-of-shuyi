---
id: "0144"
title: Ticket and ADR numbers no open PR has taken
type: infra
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0144 — Ticket and ADR numbers no open PR has taken

## Context

`write-ticket` step 2 picks "the next unused number" from the local
`tickets/open` and `tickets/done`. Parallel sessions can't see each
other's open PRs, so they take the same number. The retro of 2026-11-08
counted four collisions in the game's last 60 PRs:

- **0759:** #532 and #533 filed the same flake under the same id, and
  CI failed on the duplicate.
- **1251, three ways:**
  - engine session's #567 held it in an open PR;
  - #568 took it from `main` and merged first;
  - pin PR #572 took 1252.
  
  #567 was redone as #578 with 1253, and the engine's references were
  rewritten.
- **ADR-0112:** #540's became 0113 in a merge commit, because ticket
  0523's PR took 0112 first.
- **Engine 3088/3089:** #536 and engine #247 renumbered them to
  3090/3091.

The engine has the same problem, at 13 renumber commits since
2026-10-22. Its twin is engine#2052.

## Nick input

None.

## Scope

**In:**
- `cargo xtask next-id ticket <block>` and `cargo xtask next-id adr`:
  the next number free on `origin/main` and in the added files of every
  open PR (`gh pr list --state open --json number,files`). With no `gh`
  access it warns that it read `main` only.
- A CI check that fails a PR adding a ticket id or ADR number another
  open PR also adds, naming the other PR. The PR opened later is the one
  that changes.
- `write-ticket` step 2 and `write-adr` use `next-id`. `work-ticket`
  re-reads the PR body for numbers after its last merge of `main`.

**Out (do not do):**
- The engine's version (engine#2052).

## Implementation steps

1. Add `crates/xtask/src/ids.rs` reading `git ls-tree origin/main` and
   the open PRs' files.
2. Add the open-PR collision check to the tickets job, after
   `ticket-lint`.
3. Update the skills, then run `pwsh -File scripts/sync-agent-skills.ps1`.

## Acceptance criteria

- [ ] `cargo xtask next-id ticket 01` skips an id an open PR adds.
- [ ] `cargo xtask next-id adr` skips an ADR number an open PR adds.
- [ ] A PR adding a ticket id that another open PR adds fails, naming
      that PR.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: next-number logic over lists with gaps and open-PR names; the
  collision check over two fixture lists.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
