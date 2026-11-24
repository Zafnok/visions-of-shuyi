---
id: "1353"
title: "Colors and numbers are settled live with the watch build before the PR"
type: infra
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: low
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1353 — Colors and numbers are settled live with the watch build before the PR

## Context

From the [feedback-latency retro](../../docs/research/feedback-latency-2026-11-24.md)
(2026-11-24), bucket *Tuning*. The range colors took three tickets and
PRs (0923 #766, 1319 #801, 1343 #813) to reach a strength Nick liked;
each round was a full ticket, PR and CI cycle. The engine's `cargo shuyi
run --watch` already reloads data files such as the palettes while the
game runs, so a session can change a value with Nick watching and put
only the settled value in a PR. Nothing tells a session to.

A debug tuning menu and EXP as live data were weighed and deferred: at
most one round saved, on 1319.

## Nick input

None (process). Using it needs Nick at the screen; the skill says to
offer it, not to require it.

## Scope

**In:** a short section in `work-ticket` (or the tuning part of
`write-ticket`): for a `tuning` ticket on a value in a watched data file,
offer Nick a live session with `cargo shuyi run --watch` before the PR,
and record the settled value and the pictures in the PR. Which files
`--watch` reloads, checked at the pin and listed. Sync the loaders.

**Out (do not do):** new tools; a tuning menu.

## Implementation steps

1. Check `--watch` at the pin; list the files it reloads.
2. Write the section; sync.

## Acceptance criteria

- [ ] The skill names the command and the files it reloads.
- [ ] `scripts/sync-agent-skills.ps1 -Check` passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- None (skill text).

## Lesser targets

None (process).

## Completion notes

