---
id: "1262"
title: Drop the short CARGO_HOME on Windows CI once the engine's paths are short
type: infra
milestone: M0 Foundation
model: haiku-4.5 / gpt-6-luna
effort: low
status: done
blocked_by: ["engine#2071"]
nick_input: none
completed: 2026-11-10
---

# 1262 — Drop the short CARGO_HOME on Windows CI once the engine's paths are short

## Context

The engine pin move to `ab593bb9` (PR #624, `engine-pin/ab593bb9`) broke
the Windows CI jobs. Cargo checks the engine out under
`C:\Users\runneradmin\.cargo\git\checkouts\shuyi-engine-<16 hex>\<7 hex>\`
(80 characters), and one engine ticket's file name made a path of 276
characters. libgit2 refuses anything over 260 unless git's
`core.longpaths` is set. Nick's machine has that setting, so the build
works there. PR #624 works around it with `CARGO_HOME=C:\c` on Windows
runners:

- `.github/workflows/ci.yml`, the step "A short CARGO_HOME on Windows"
  before `Swatinem/rust-cache`;
- `.github/workflows/release.yml`, the first line of "Set up the Windows
  GNU target" and its comment.

Engine ticket 2071
([engine#2071](https://github.com/Zafnok/shuyi-engine/blob/main/tickets/done/2071-no-path-too-long-for-a-windows-checkout.md))
renames every engine path over 150 characters and fails the engine's CI
on a new one (`cargo xtask check-paths`). Once the game's pin is at or
past the commit that closed it, the workaround is dead weight.

**Either order with the next engine pin move:** if the `engine-pin` PR
that passes engine#2071 is open, it removes the workaround and moves
this ticket to `done/` in the same PR; if not, this ticket moves the
pin (`engine-change` §7) and removes it.

## Nick input

None.

## Scope

**In:**
- Remove the `CARGO_HOME=C:\c` step and its comment from `ci.yml`.
- Remove the `CARGO_HOME=C:\c` line and its comment from `release.yml`.

**Out (do not do):**
- Setting `core.longpaths` on the runners instead. With the engine's
  check in place, nothing needs it. A future long path is the engine's
  bug, and its CI catches it.

## Implementation steps

1. Check the pin passes engine#2071: `git -C shuyi-engine merge-base
   --is-ancestor <2071's merge commit> $(cat shuyi-engine.rev)` (after
   `cargo xtask engine`), or move it first.
2. Delete the two workaround blocks named above.
3. Push. CI's `test (windows-latest)` must pass. The release workflow's
   Windows job runs the same checkout at the next tag.

## Acceptance criteria

- [x] `grep -rn "CARGO_HOME" .github/workflows` finds no `C:\c`.
- [x] CI's Windows jobs pass on the PR.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Integration: CI's Windows jobs, which check the engine out with the
  runner's default `CARGO_HOME`.

## Completion notes

The engine-pin PR #624 (c1ba9ec8 → ab593bb9) was still open and its pin
is one engine PR short of engine#2071 (#356, `edb6c888`), so this PR is
stacked on it and moves the pin that one step (`engine-change` §7), with
its ledger section in `docs/engine/pins.md`. The skill pack didn't
change. Then it deletes the workaround #624 added: `ci.yml` and
`release.yml` are again byte for byte what they were on `main` before
#624.

No follow-up tickets. Nothing for Nick.
