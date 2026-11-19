---
id: "1274"
title: "Trim idle worktrees and seed new ones with the engine's worktrees module"
type: infra
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1274 — Trim idle worktrees and seed new ones with the engine's worktrees module

## Context

The engine pin move of 2026-11-16 passed engine PRs #453 (engine ticket
2078), #454 (2076) and #457 (2092). `shuyi_cli::worktrees` is now
public:

- `worktrees::trim::run` deletes the `incremental/` caches of every idle,
  unlocked worktree of the repository (never a worktree, never the
  current one), and with `Options::stale_hashes` the older hashes of a
  crate in `deps/` that no build read in days (on Windows only when
  last-access updates are on). On the engine's machine that is 57% of a
  `target/`.
- `worktrees::seed` starts a new worktree's `target/debug` from a warm
  worktree's: third-party output hard-linked, the workspace's own crates
  built fresh. A faster first build, and the hard links cost no disk.

The engine's own `cargo xtask trim-targets` and `seed-target --into`
are thin wrappers over them. Engine ticket 3403 (engine PR #460) gives
every game `cargo shuyi trim-targets` and `seed-target`, and is in the
pin since the pin move of 2026-11-17, so the game needs no wrapper. The game's worktrees are the biggest users of Nick's
disk (10+ GB each, `AGENTS.md`); `cargo xtask clean-merged-targets`
(ticket 0113) only frees the ones whose PR merged, and 1271 compresses
them.

## Nick input

None.

## Scope

**In:**
- `cargo shuyi trim-targets` (engine#3403, at the pin) on the game's
  worktrees, with the game's roots (`target/`) if `shuyi.ron` must name
  them; no wrapper in `crates/xtask`.
- `cargo shuyi seed-target --into <worktree>`, as above.
- The `work-ticket` skill seeds a new worktree before its first build,
  beside `cargo shuyi private-assets`; `AGENTS.md`'s environment section
  names both commands beside `clean-merged-targets`.
- Decide whether `clean-merged-targets` stays (it also deletes merged
  worktrees' whole `target/`; `trim-targets --merged` in the engine
  does the same): keep one, and delete the other's code, about 200
  lines of `crates/xtask/src/clean_targets.rs` if the engine's covers it.
- Measure: a seeded worktree's first `cargo test --workspace` time
  against an unseeded one's, and the disk `trim-targets` frees on Nick's
  machine (`--dry-run` first). Both numbers in the PR.

**Out (do not do):**
- Running the trim from the gates (the engine's gates do; decide in the
  PR whether the game's should, and file a ticket if so).
- Changing the engine's module (an engine ticket, `engine-change`).

Estimate: About 10 production lines (`shuyi.ron`, if anything), minus
`clean_targets.rs` if it goes.

## Implementation steps

1. Read `cargo shuyi trim-targets --help` and `seed-target --help` and
   the pack skill `shuyi-worktrees` at the pin.
2. Add the skill and `AGENTS.md` lines.
3. Run `trim-targets --dry-run`, then for real; seed a new worktree;
   measure.
4. `run-gates`.

## Acceptance criteria

- `cargo shuyi trim-targets --dry-run` lists the game's idle worktrees
  and what it would free, and never names the current worktree.
- A worktree seeded with `seed-target` builds and passes `cargo test
  --workspace`.
- The PR states the time and disk numbers.

## Tests required

None of the game's own if no wrapper is written: the commands are the
engine's and tested there.

## Completion notes

*(Filled in by the session that completes the ticket.)*
