---
id: "1353"
title: "Colors and numbers are settled live with the watch build before the PR"
type: infra
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: low
status: done
blocked_by: []
nick_input: none
completed: 2026-11-24
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

- [x] The skill names the command and the files it reloads.
- [x] `scripts/sync-agent-skills.ps1 -Check` passes.
- [x] All gates in the `run-gates` skill pass (the skill and ticket
  checks locally; no code changed, so the Rust gates run in CI).

## Tests required

- None (skill text).

## Lesser targets

None (process).

## Completion notes

- `work-ticket` has a new §4b: for a `tuning` ticket on a value Nick
  judges by eye, offer him a live session with `cargo shuyi run --watch`
  before the PR, then put only the settled value in the branch, with the
  values tried and a `frame-png` picture in the PR. `write-ticket`'s
  tuning line points at it.
- Checked at engine pin `7d60a21`: `run --watch` reads `assets/`,
  `assets-test/` and `assets-private/content/` from disk and polls them
  once a second (engine `crates/app/src/watch.rs`), but the game reloads
  only the four theme palettes and `.dlg` scenes (`crates/ui/src/hot.rs`,
  `reloads`). Everything else, including the Playdate palettes, maps,
  battles and EXP and the other data tables, is read at start: the
  section says to restart the command (no rebuild) for those. So the
  title's "numbers" are a restart away, not live.
- No gameplay rules decided. No follow-up tickets.

