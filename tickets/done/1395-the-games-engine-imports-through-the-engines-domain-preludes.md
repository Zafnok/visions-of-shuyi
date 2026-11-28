---
id: "1395"
title: "The game's engine imports through the engine's domain preludes where they cover them"
type: infra
milestone: M1 Engine
model: haiku-4.5 / gpt-6-luna
effort: low
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1395 — The game's engine imports through the engine's domain preludes

## Context

The pin move of 2026-11-27 (`10cbb98b → 230080db`, `docs/engine/pins.md`)
brought engine PR #634 (engine ticket 3438, engine ADR-0161): domain
preludes, re-exports only, so a game imports one path per domain instead of
the module an item happens to live in:

- `shuyi_core::prelude`: the RNG, `TurnRules`, `TickSim` and the step's
  types, timers, `Pos`, `Dir`, ease and tween, a state's hash, and the bots
  that play a turn game (`Bot`, `RandomBot`, `Rollout`, `play`);
- `shuyi_platform::prelude::{grid, text, audio, test}` (reached as
  `shuyi::prelude::…` through the facade), `test` holding the Harness, the
  session files, the playtest runner and autobalance;
- `shuyi_cli::prelude`: `GameTools`, `BasicTools`, `main_with`, the project,
  and the contract operations a game's tests call.

The engine moved its own templates, examples and skills onto them and
checks it with `cargo xtask check-imports` (an engine-only gate). The book's
*Tools for agents* has *Which prelude to import* (`cargo shuyi docs
prelude`).

This game imports the engine by deep paths in about 250 files (`use
shuyi_ui::text…` 57 times, `shuyi_ui::grid…` 47, `shuyi_assets::font…` 37,
`shuyi_core::bots::playtest…` 9, `shuyi_cli::codec…` 15, and so on). An item
the engine moves behind a prelude breaks those paths at a pin move; through
the prelude it doesn't, and an agent writes the one path the book names.

## Nick input

None.

## Scope

**In:**
- Every `use` of an engine item that one of the preludes re-exports, in the
  crates that link that prelude's crate: `trpg-core` and `trpg-bots`
  (`shuyi_core::prelude`), `trpg-ui` and `trpg-tools` (`shuyi_core::prelude`
  and `shuyi_platform::prelude::*`), `trpg-tools` (`shuyi_cli::prelude`).
  Tests too.
- Named imports from the prelude (`use shuyi_core::prelude::{Pos, Dir};`),
  or the glob where a file already imports most of a domain and no name
  clashes with the game's own.

**Out (do not do):**
- Paths no prelude covers (`shuyi_assets::*`, which has no prelude, and most
  of `shuyi_ui::*`): they stay as they are.
- Adding a dependency on `shuyi-platform` or `shuyi` to a crate that doesn't
  link it (`trpg-core`, `trpg-content`, `trpg-bots`). `trpg-ui` links
  `shuyi-platform` only with `std` (optional, and always in its tests): a
  `shuyi_platform::prelude` path in its code goes behind the same
  `cfg(feature = "std")` as the deep path it replaces, so its `no_std`
  Playdate build (ADR-0070) still builds.
- Any behaviour change; any rename the engine's 3442 (US names beside the
  deprecated UK ones) brings: that is its own pin move's fix.
- A game-side import check (the engine's `check-imports` reads the engine's
  folders only).

## Implementation steps

1. `cargo shuyi docs "which prelude to import"` and read each prelude's
   module in the pinned engine (`cargo doc -p shuyi-core --no-deps --open`,
   or the source under `~/.cargo/git/checkouts/shuyi-engine-*/230080d*/`)
   for the exact list of what it re-exports.
2. Per crate, replace each covered deep path with the prelude's
   (`grep -rn "use shuyi_core::" crates/core`, then `crates/bots`,
   `crates/ui`, `crates/tools`). One commit per crate.
   Expected size: about 300 changed lines, all `use` lines.
3. `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`
   (unused imports), the whole `run-gates`.

## Acceptance criteria

- [ ] No `use` in `crates/` names a deep engine path for an item a prelude
      its crate links re-exports (a `grep` per prelude in the completion
      notes, with the counts before and after).
- [ ] No snapshot or test changes besides its `use` lines.
- [ ] The Playdate `no_std` builds of `trpg-core`, `trpg-content` and
      `trpg-ui` still build (`run-gates`).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- None new: the existing suites, unchanged, are the check.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
