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
are thin wrappers over them. The game's `crates/xtask` already depends
on `shuyi-cli`, so it can wrap them the same way today; engine ticket
3403 will later give every game `cargo shuyi trim-targets` and
`seed-target`. The game's worktrees are the biggest users of Nick's
disk (10+ GB each, `AGENTS.md`); `cargo xtask clean-merged-targets`
(ticket 0113) only frees the ones whose PR merged, and 1271 compresses
them.

## Nick input

None.

## Scope

**In:**
- `cargo xtask trim-targets [--idle-days N] [--dry-run] [--stale-hashes]`
  over `shuyi_cli::worktrees::trim`, with the game's roots (`target/`).
  If engine#3403 has landed at the pin by then, use its `cargo shuyi
  trim-targets` instead and write no wrapper.
- `cargo xtask seed-target --into <worktree>` over
  `shuyi_cli::worktrees::seed` (or `cargo shuyi seed-target`, as above).
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

Estimate: About 80 production lines (two thin wrappers and their
usage), minus `clean_targets.rs` if it goes.

## Implementation steps

1. Read `shuyi_cli::worktrees` and the engine's `crates/xtask/src/trim.rs`
   at the pin; check whether 3403 has landed.
2. Add the wrappers (or switch to `cargo shuyi`), the skill and
   `AGENTS.md` lines.
3. Run `trim-targets --dry-run`, then for real; seed a new worktree;
   measure.
4. `run-gates`.

## Acceptance criteria

- `cargo xtask trim-targets --dry-run` lists the game's idle worktrees
  and what it would free, and never names the current worktree.
- A worktree seeded with `seed-target` builds and passes `cargo test
  --workspace`.
- The PR states the time and disk numbers.

## Tests required

The wrappers' argument parsing, as `clean_targets.rs`'s is tested; the
walks themselves are the engine's and tested there.

## Completion notes

*(Filled in by the session that completes the ticket.)*
