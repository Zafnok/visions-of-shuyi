---
id: "1210"
title: Keep a play session's record with the engine's session records, in the debug tools
type: infra
milestone: M8 Release
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1210 — Keep a play session's record with the engine's session records, in the debug tools

## Context

The pin move of 0752 (`docs/engine/pins.md`) passes engine#2518 and engine#2575:
`shuyi_platform::session` records every input, clock reading, music position,
keyboard answer and stored file that reaches a game, and `Harness::from_session`,
`play_to` and `play_all` play a record to any frame, stopping at the first that
differs. The game today can replay a battle's commands (`BattleHistory`,
ADR-0007 layer 5) and nothing outside a battle: a bug in a menu, a save or the
dialogue from a tester's run can't be reproduced.

## Nick input

None: this is debug tooling, and the shipped game records nothing (0508's rule).

## Scope

**In:** with the debug tools on, `App::record` for the whole session, a way to save
the record (a debug-menu row and on a crash where the platform allows), and a
scripted test that plays a kept record through `Harness::from_session`. Read
`docs/platform-backends.md` (*Session records*) in the engine and ADR-0072 first.

**Out (do not do):** recording in a shipped build; changing 0508's battle records,
which the bots' measures read.

## Implementation steps

1. Turn recording on where the debug tools are, and save with a debug-menu row.
2. Say in `docs/` how a tester sends a record, and how to turn one into a test.
3. Add one kept record as a test, from a short real session.

## Acceptance criteria

- [ ] A session recorded in a debug build plays back to its last frame with no difference (test).
- [ ] A shipped build has no recording (test or the `check` that holds it).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: a kept record, played through `Harness::from_session`.

## Completion notes

