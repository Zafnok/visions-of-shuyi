---
id: "1271"
title: "The game's worktrees compress their target folders, set by a git hook"
type: infra
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-11-15
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

- `scripts/git-hooks/post-checkout` is the engine template's hook at the
  pin (`cfd79d10`), with the game's comments: on `git worktree add` or a
  clone on Windows it creates `target/` and marks it NTFS-compressed.
- The setup line, `git config core.hooksPath scripts/git-hooks`, is in
  `AGENTS.md`'s environment section, beside `clean-merged-targets`: the
  README has no setup section, and `AGENTS.md` is where a checkout's
  setup (`private-assets` before the first build) already is. It was set
  on Nick's clone (`D:/tactical-rpg/.git/config`), so every worktree has it.
- In a new worktree, `cargo shuyi doctor` reports `ok git-hooks:
  core.hooksPath is scripts/git-hooks` and `ok target-compression:
  target/ is compressed`.
- Measured: this worktree's `target/` after `cargo test --workspace` holds
  4.9 GB of files and takes 2.7 GB on disk (`du --apparent-size` against
  `du`). The worktree's `target/` was already marked compressed before the
  run, so the two numbers are the same build's size uncompressed and
  compressed rather than two builds.
- 0145's pre-push hook goes in the same folder; noted there.
- No gameplay rules decided.
