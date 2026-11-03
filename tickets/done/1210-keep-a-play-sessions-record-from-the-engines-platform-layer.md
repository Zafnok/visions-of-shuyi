---
id: "1210"
title: Keep a play session's record with the engine's session records, in the debug tools
type: infra
milestone: M8 Release
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-11-03
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

- [x] A session recorded in a debug build plays back to its last frame with no difference (test).
- [x] A shipped build has no recording (test or the `check` that holds it).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Integration: a kept record, played through `Harness::from_session`.

## Completion notes

Done. The recording itself came with the engine: this PR moves the pin to
engine `main` (96e97bf0), which passes engine 2519 (#239). With debug tools
on, `shuyi-app` records every run from its first frame, writes it to
`<data dir>/sessions/` when the game quits (the newest ten kept), and the
engine's debug menu gains "Save session" (a download on the web). The game's
debug menu is the engine's `DebugMenuScreen`, so the row came with no game
code. The pin's ledger section is in `docs/engine/pins.md`.

The game's side:

- **The game's start moved out of the binary** into `trpg_ui::start`
  (`start(title, startup)` and `Startup`), so the binary, the tools and the
  tests start the same game. `crates/app` calls it and no longer depends
  on `shuyi-platform`.
- **The battle's rules state is in each frame's hash**
  (`BattleScreen::state_hash`), so a playback that plays a battle
  differently is found at the frame it parts, not only when a sound or a
  screen shows it.
- **`session_from` in the game's tools**: `cargo shuyi run --session <file>`
  and `scene --session` play a tester's record (test in `xtask`).
- **A kept record**: `crates/ui/tests/it/sessions/quick_battle.session.ron`,
  a first launch into the debug Quick Battle (the layout picker, the battle,
  the debug menu's "Save session", the lord's move), played by
  `assert_session` and `Harness::from_session` to its last frame
  (`tests/it/sessions.rs`), with a test that a record missing one press is
  caught at that frame and one that recording the same script gives the
  same frames.
- **A shipped build can't record**: `crates/app/src/main.rs` refuses to
  compile a release build without the `debug-tools` feature if the engine's
  `DEBUG_TOOLS` is on (the same guard as the test files'), and the engine's
  own test holds `records(false)`.
- **Docs**: `docs/session-records.md`: where the records are, how a tester
  sends one, how to play one and how to keep and bless one as a test.

**Deviations:**

- **"On a crash where the platform allows"** is not done: the engine keeps
  no record at a panic yet, and the game can't reach the running `App`
  from its binary. Engine ticket 3060 writes it at the panic; the game gets
  it at a later pin, and the docs say to use "Save session" until then.
- **The kept record was recorded in the Harness** from the game's real
  start, not in a window, so it can be recorded again by a test
  (`write_the_quick_battle_session`, ignored) when the script changes. A
  real desktop record was checked by hand: the debug build launched and
  closed from its window wrote `<ms>.session.ron`, and `cargo shuyi run
  --session` played its 335 frames with no difference.
- **Records have no stamp** of the build (`Start::made`): stamping the
  game's builds is 1195's, for the bots' records too. Until then a record
  from another build plays and shows where it differs, and the answer says
  "the record has no stamp".

Friction: estimate 70 production lines, measured 57 added (under 2×).
No gameplay rules decided.

