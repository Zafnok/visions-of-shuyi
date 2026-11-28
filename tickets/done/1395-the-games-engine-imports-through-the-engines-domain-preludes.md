---
id: "1395"
title: "The game's engine imports through the engine's domain preludes where they cover them"
type: infra
milestone: M1 Engine
model: haiku-4.5 / gpt-6-luna
effort: low
status: done
blocked_by: []
nick_input: none
completed: 2026-11-27
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

- [x] No `use` in `crates/` names a deep engine path for an item a prelude
      its crate links re-exports (a `grep` per prelude in the completion
      notes, with the counts before and after).
- [x] No snapshot or test changes besides its `use` lines.
- [x] The Playdate `no_std` builds of `trpg-core`, `trpg-content` and
      `trpg-ui` still build (`run-gates`).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- None new: the existing suites, unchanged, are the check.

## Completion notes

**Done.** Every engine `use` whose item a prelude re-exports, in a crate
(and a context) that links that prelude, now names the prelude: 61 files (core 4, bots 6, ui 28, tools 22, story-preview 1),
`use` lines only (+94 / −102 after `cargo fmt`). A `use` that mixed covered
and uncovered items was split: the covered ones go to the prelude, the rest
keep their path.

How it was found: a script read the three preludes' `pub use` trees at pin
`230080db` (with each crate's root re-exports, so `shuyi_core::grid::Grid`
and `shuyi_core::Grid` are both the prelude's `Grid`), parsed every `use`
tree in `crates/`, and moved each leaf the statement's context links. The
context per statement: `shuyi_core::prelude` in `trpg-core`, `trpg-bots`,
`trpg-ui`, `trpg-tools` and `trpg-content`'s tests (a dev-dependency);
`shuyi_platform::prelude::*` in `trpg-tools`, and in `trpg-ui` only in its
tests and in code behind `cfg(feature = "std")` or `cfg(test)` (a gated
module, a gated `mod … {}` or an attribute on the `use`), so the `no_std`
build is unchanged; `shuyi_cli::prelude` in `trpg-tools`.

Leaves moved, before → after (the same check run again after the rewrite
finds none left, by any path spelling):

| Prelude | Deep paths before | After |
| --- | --- | --- |
| `shuyi_core::prelude` | 21 | 0 |
| `shuyi_cli::prelude` | 72 | 0 |
| `shuyi_platform::prelude` (top) | 24 | 0 |
| `shuyi_platform::prelude::text` | 16 | 0 |
| `shuyi_platform::prelude::test` | 21 | 0 |
| `shuyi_platform::prelude::grid` | 2 | 0 |
| `shuyi_platform::prelude::audio` | 1 | 0 |
| `shuyi::prelude` (facade, `trpg-story-preview`) | 1 | 0 |

What stays deep, as the ticket's *Out* says: `shuyi_assets::*` and
`shuyi_ui::*` items no prelude holds; items a prelude holds but the context
doesn't link (`trpg-ui`'s `no_std` code, such as `shuyi_assets::font::CellWidths`
in the skins; `trpg-content`'s and `trpg-bots`' playtest and asset types,
whose prelude is `shuyi-platform`'s); module globs that re-export a whole
engine module (`pub use shuyi_ui::storage::*`); modules imported as
modules (`shuyi_core::path`, `shuyi_cli::contract::check`); and
`crates/app/src/app_info.rs`'s `shuyi::platform::AppInfo`, which `cargo
shuyi app-info` writes and checks (the engine's own `imports.ron` allows
it for the same reason).

Deviations: `trpg-story-preview`'s one `use` (`shuyi::assets::bundle::Layer`
→ `shuyi::prelude::Layer`) was included, though the ticket's *In* doesn't
name it, because the acceptance criterion covers all of `crates/`
(`trpg-content`'s tests were checked too and name no covered item).

Gates: every `run-gates` command passed locally, the three Playdate `no_std` builds among them; `cargo test --workspace` unchanged and green. PR size about 270 lines. `pr-size` flagged `crates/tools/src/playtest.rs` (engine#2532's marker): a dated line in 1207 says the `use` change needs nothing there.

Engine friction: none: `use` lines only. No follow-up tickets. Nothing
for Nick: no behavior change.
