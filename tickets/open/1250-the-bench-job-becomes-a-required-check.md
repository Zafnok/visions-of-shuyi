---
id: "1250"
title: The bench fails on every number over budget again
type: infra
milestone: Later
model: sonnet-5 / gpt-6.1-terra
effort: low
status: todo
blocked_by: ["0185", "1248", "1285"]
nick_input: none
completed:
---

# 1250 — The bench fails on every number over budget again

## Context

Ticket 1244 added CI's `bench` job (`cargo shuyi bench` on every pull
request, `docs/targets.md`). Its first run found five numbers over their
budgets: the unit menu's frame (1247), the Playdate's one-bit drawing
(1248) and the web package (1249). The budgets stay as they are, so the
job would fail on every pull request until those land. Meanwhile it
reports those five as warnings (`KNOWN_OVER` in the job, read by
`scripts/bench-known-over.sh`) and fails only on a new one or an error,
and it is left out of `ci-result`, the one required check. 0185 makes
`bench` required and moves that list to `bench/known-over.txt`, read
by `cargo xtask bench-gate`; this ticket deletes the list once it is
empty.

## Nick input

None.

## Scope

**In:**
- Once 1248 and 1285 have emptied `bench/known-over.txt` and the
  `bench` job has no warnings on `main`: delete the file and its
  known-over handling in `crates/xtask/src/bench_gate.rs`, and the
  comments about it.
- Update `docs/targets.md`'s *Over budget* section.

**Out (do not do):**
- Raising a budget to make the job pass.

## Implementation steps

1. Check the latest `bench` run on `main` passes.
2. Delete the list and its handling; update the comments and the doc.

## Acceptance criteria

- [ ] A pull request whose scenario goes over any budget fails
      `ci-result`.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- CI: `ci-result` on this PR.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
