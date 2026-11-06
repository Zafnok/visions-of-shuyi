---
id: "1250"
title: The bench job becomes a required check
type: infra
milestone: Later
model: sonnet-5 / gpt-6.1-terra
effort: low
status: todo
blocked_by: ["1247", "1248", "1249"]
nick_input: none
completed:
---

# 1250 — The bench job becomes a required check

## Context

Ticket 1244 added CI's `bench` job (`cargo shuyi bench` on every pull
request, `docs/targets.md`). Its first run found five numbers over their
budgets: the unit menu's frame (1247), the Playdate's one-bit drawing
(1248) and the web package (1249). The budgets stay as they are, so the
job fails on every pull request until those land. It runs, posts its
table and shows red, but it is left out of `ci-result`, the one required
check, so that it doesn't block every other pull request meanwhile.

## Nick input

None.

## Scope

**In:**
- Once 1247, 1248 and 1249 are done and the `bench` job passes on
  `main`, add `bench` to `ci-result`'s `needs` in
  `.github/workflows/ci.yml` and remove the comment that says why it
  isn't.
- Update `docs/targets.md`'s *Over budget* section.

**Out (do not do):**
- Raising a budget to make the job pass.

## Implementation steps

1. Check the latest `bench` run on `main` passes.
2. Add `bench` to `ci-result`'s `needs`; update the comment and the doc.

## Acceptance criteria

- [ ] `ci-result` needs `bench`, and a pull request whose scenario goes
      over a budget fails `ci-result`.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- CI: `ci-result` on this PR.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
