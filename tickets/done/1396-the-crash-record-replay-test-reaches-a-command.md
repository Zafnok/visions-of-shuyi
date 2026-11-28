---
id: "1396"
title: The crash record replay test reaches a command again
type: infra
milestone: M8 Release
model: sonnet-5 / gpt-6.1-terra
effort: low
status: done
blocked_by: []
nick_input: none
completed: 2026-11-27
---

# 1396 — The crash record replay test reaches a command again

## Context

Found by ticket 1393. `a_crash_record_replays_to_the_crash` in
`crates/tools/src/game_tools/tests.rs` (ticket 1245) plays the debug
Quick Battle with `h.keys("Down f Left f f")` and then moves the lord.
Preparations now needs one more Confirm to reach the battle (the UI's own
test, `the_record_in_a_battle_holds_its_history` in
`crates/ui/tests/it/crash_reports.rs`, plays `Down f Left f f f`), so
the lord never moves: the record holds no command, `applied` is 0, and
the test's step counts (`applied + 1`, `applied + 2`) pass on an empty
record. Its last-hash check and the `applying` case still run, but the
test no longer shows that applied commands replay.

1393's new test beside it, `a_crash_report_replays_with_replay_crash`,
plays the UI test's keys and asserts `applied > 0`.

## Nick input

None.

## Scope

**In:**
- `a_crash_record_replays_to_the_crash`: the UI test's keys (or the
  shared battle drivers of ticket 1237, `battle::testing`, if they start
  the Quick Battle), and an assertion that the record holds at least one
  command.

**Out (do not do):**
- Any change to the crash record or to `plays.rs`.

## Implementation steps

1. Change the keys to `Down f Left f f f`, as in the UI test, and its
   comment to match (Preparations, `Fight!`, the banner).
2. After `let applied = record.history.len();`, assert `applied > 0`.
3. Run `cargo test -p trpg-tools --bins -- a_crash_record`.

## Acceptance criteria

- [x] `a_crash_record_replays_to_the_crash` fails if the record holds no
      command, and passes.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the changed test in `crates/tools`.

## Completion notes

- `a_crash_record_replays_to_the_crash` plays the UI test's keys,
  `Down f Left f f f` (Preparations, `Fight!`, the banner), with the
  comment of 1393's test beside it, and asserts `applied > 0` after
  reading the record. It passes, so the lord's move and wait now replay.
- No deviations, no follow-up tickets, no gameplay rules decided.
