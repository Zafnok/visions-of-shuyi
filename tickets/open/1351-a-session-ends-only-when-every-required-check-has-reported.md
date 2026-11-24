---
id: "1351"
title: "A session ends only when every required check has reported"
type: infra
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: low
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1351 — A session ends only when every required check has reported

## Context

From the [feedback-latency retro](../../docs/research/feedback-latency-2026-11-24.md)
(2026-11-24), bucket *Process and CI* (siblings: 1352; amended 0146,
0181). Nick merges every PR himself and keeps doing so (asked
2026-11-24). A PR whose session ended before its slowest check (the
mutants job) reported sat red, unseen, until Nick's next session: #801
(1319) red 08:12 → fixed and merged 18:03, #802 (1314) about 9 h the same
way. A fix that takes minutes cost about 10 h each. Required checks take
18–35 minutes a push; CI time itself is not the bottleneck.

## Nick input

None (process).

## Scope

**In:**
- `work-ticket` skill, its last step: after pushing, wait for every
  required check (mutants included) to report, fix and push a red one,
  and only then end; turn on the desktop app's PR monitor (Auto-fix) for
  the PR when it opens, where the harness has one.
- The same line in `engine-change` for engine PRs.
- **A rule out:** shorten `work-ticket`'s CI section where it repeats
  what `run-gates` already says about checks (one pointer instead).
- `pwsh -File scripts/sync-agent-skills.ps1`, then `-Check`.

**Out (do not do):** auto-merge (Nick declined, 2026-11-24); changing CI
jobs (0150, 0174, 0145).

## Implementation steps

1. Edit the two skills; remove the duplicated text.
2. Sync the loaders.

## Acceptance criteria

- [ ] `work-ticket` and `engine-change` say a session waits for every
      required check and fixes a red one before it ends.
- [ ] `work-ticket` is no longer than before (a rule out for the rule in).
- [ ] `scripts/sync-agent-skills.ps1 -Check` passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- None (skill text).

## Lesser targets

None (process).

## Completion notes

