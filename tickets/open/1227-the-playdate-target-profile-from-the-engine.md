---
id: "1227"
title: "The Playdate's facts from the engine's built-in profile: drop the game's copy of targets/playdate.ron"
type: infra
milestone: "Engine: carve"
model: haiku-4.5 / gpt-6-luna
effort: low
status: blocked
blocked_by: ["engine#3089"]
nick_input: none
completed:
---

# 1227 — The Playdate's facts from the engine's built-in profile

## Context

From the game/engine boundary audit of 2026-11-03. 1200 gave the game's
own Playdate front end its target facts from
`crates/ui/src/playdate_target.ron`, a copy of the engine's
`targets/playdate.ron` (identical bar its comments), because the game
can't `include_str!` a file of a git dependency.
`crates/ui/src/platform.rs` (`playdate_facts`, 12 production lines)
parses it, and `the_playdates_facts_are_the_engines`
(`crates/xtask/src/game_tools/tests.rs`) exists only to keep the copy
equal to the engine's at the pin.
[engine#3089](https://github.com/Zafnok/shuyi-engine/blob/main/tickets/open/3089-built-in-target-profiles-readable-from-no-std.md)
adds `shuyi_assets::target::builtin(id)` and
`shuyi_ui::PlatformFacts::builtin(id)`, readable from `no_std`.

Users of `playdate_facts`: `crates/playdate/src/start.rs:27`,
`crates/ui/tests/it/playdate.rs:41`, `crates/xtask/src/game_tools.rs:332`
and the test above.

**Either order with 1170** (the Playdate front end on the engine's
backend, which rewrites `crates/playdate/src/start.rs`): if 1170 is
done, its start may take the facts from the engine's backend already;
change whatever call of `playdate_facts` is left. If not, change
`start.rs:27`, and 1170 keeps the engine's call.

## Nick input

None.

## Scope

**In:**
- Move the pin past engine#3089 (`engine-change` skill §7, with its
  `engine-pin` ledger row), unless an open `engine-pin` PR has.
- Delete `crates/ui/src/playdate_target.ron` and `crates/ui/src/platform.rs`
  (and `pub mod platform;` in `crates/ui/src/lib.rs`); each user calls
  `PlatformFacts::builtin("playdate")`.
- Delete `the_playdates_facts_are_the_engines`; keep `engine_facts`,
  which other tests use.
- The ticket and doc lines that name `playdate_target.ron`
  (`grep -rn playdate_target docs crates`).

**Out (do not do):**
- Any other target's facts, or the PC backend's (the engine's).
- 1170's port of the front end.

## Implementation steps

1. **Pin** past engine#3089.
2. Replace the three calls with `PlatformFacts::builtin("playdate")`,
   through `trpg_ui::PlatformFacts` (re-exported in `crates/ui/src/lib.rs`)
   where a crate has no `shuyi-ui` dependency; the error type stays
   `String`, so `?` still works.
3. Delete the two files, the module line and the test.
4. **Measure** with `cargo xtask code-lines`: about 20 production lines
   and 41 lines of data removed, 3 changed.

## Acceptance criteria

- [ ] `shuyi-engine.rev` is at or past the commit that closed engine#3089.
- [ ] `grep -rn "playdate_target\|playdate_facts" crates` finds nothing.
- [ ] The Playdate tests in `crates/ui/tests/it/playdate.rs` pass
      unchanged apart from the call.
- [ ] All gates in the `run-gates` skill pass, including the CI
      `playdate` and no-std builds.

## Tests required

- Integration: the existing Playdate host tests and the xtask
  `the_title_and_options_follow_the_targets_facts`.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
