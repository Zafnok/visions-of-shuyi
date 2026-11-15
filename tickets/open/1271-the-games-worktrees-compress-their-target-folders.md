---
id: "1271"
title: "The game's worktrees compress their target folders, set by a git hook"
type: infra
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1271 — The game's worktrees compress their target folders, set by a git hook

## Context

The engine pin move of 2026-11-15 passed engine PRs #421 (engine ticket
2075) and #435 (engine ticket 3402). The engine's worktrees now compress
their `target/` folders with NTFS compression from a `post-checkout`
hook, its gates check it, and new games get the same: the hook in the
template's `scripts/git-hooks`, `shuyi repos` setting `core.hooksPath`
when a game has that folder, and `shuyi doctor` listing the optional
checks `git-hooks` and, on Windows, `target-compression`.

The game's worktrees are the biggest users of Nick's disk: each
`target/` folder is 10+ GB (`AGENTS.md`), and `cargo xtask
clean-merged-targets` only frees the ones whose PR merged. The game has
no `scripts/git-hooks` and compresses nothing.

## Nick input

None.

## Scope

**In:**
- `scripts/git-hooks/post-checkout`, from the engine template's (read
  it at the pin), compressing a new worktree's `target/` on Windows.
- Setting `core.hooksPath` to it: `cargo shuyi doctor`'s `git-hooks`
  check passing on the game, and how a fresh clone sets it (the README
  or `docs/`, whichever already says how to set up a checkout).
- `cargo shuyi doctor` passing `target-compression` in a new worktree.
- Measure one worktree's `target/` after `cargo test --workspace`,
  before and after, and write both numbers in the PR.

**Out (do not do):**
- 0145's pre-push hook (it can land in the same `scripts/git-hooks`
  folder later; say so in 0145 if this lands first).
- Running `shuyi repos` on the game (its repositories exist; the
  2026-11-14 pin move's row for engine#399).

Estimate: About 60 lines (a hook script and a few lines of docs).

## Implementation steps

1. Read the engine's template hook and `doctor`'s two checks at the pin.
2. Add the hook and the setup line; run `cargo shuyi doctor`.
3. Measure, then `run-gates`.

## Acceptance criteria

- `cargo shuyi doctor` reports `git-hooks` and `target-compression`
  passing in a new worktree of the game on Windows.
- The PR states the two sizes.

## Tests required

None beyond the gates; `doctor`'s output and the two sizes are in the PR.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
