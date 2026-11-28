---
id: "1393"
title: "A player's crash report read with cargo shuyi crash --symbols and replayed with replay --crash"
type: infra
milestone: M8 Release
model: sonnet-5 / gpt-6.1-terra
effort: low
status: done
blocked_by: []
nick_input: none
completed: 2026-11-27
---

# 1393 — A player's crash report read and replayed with the engine's crash tools

## Context

The pin move of 2026-11-27 (`10cbb98b → 230080db`, `docs/engine/pins.md`)
brought two engine tools for a player's crash report:

- engine PR #625 (engine ticket 2520): `shuyi crash <report>` reads a
  `crash-<when>.ron` with no game (message, place, versions, frame, top
  screen, log, `has_record`), and `shuyi replay --crash <report>` runs the
  record the report holds through the game's `GameTools::replay` and its
  stamp check, putting the replayed panic beside the reported one
  (`same_place`). `replay` on rules that panic now answers `ok: false`
  with the step's `panic` instead of ending the tool program;
- engine PR #637 (engine ticket 3441): `shuyi crash --symbols <file>`
  names the report's frames (function, file, line, inlined frames
  included) from the debug file kept for its release.

The game already writes crash reports with its battle record (ticket
1245, `config.crash` in `crates/app/src/main.rs`, `trpg_ui::crash`) and
keeps each release's `.debug` on the private repository (ticket 0917,
`scripts/upload-debug-file.sh`). But its docs still read them by hand:

- `docs/releasing.md`, *The debug files*, "Read a player's report": an
  `objdump` / `grep` / `addr2line` recipe of about 20 lines, with the image
  base and the minus-one explained;
- `docs/session-records.md`, *Crash reports*: the battle record "copied to
  a file of its own" before `cargo shuyi replay <file>`.

`crates/tools/src/game_tools/plays.rs` (`replay`, `replay_crash`) already
reads a `BattleRecord`, so `replay --crash` should work on the game's
reports with no code change; nothing tests it yet.

## Nick input

None.

## Scope

**In:**
- `docs/releasing.md`: the by-hand recipe replaced by `cargo shuyi crash
  <report> --symbols <debug file> [--json]`, keeping the fetch with `gh
  release download` and what the first frames are; `SH3020` (a file that
  isn't the debug file) and the "N of M frames fall outside every
  function" limit (a debug file of another version) named in place of the
  `??` lines.
- `docs/session-records.md`: the record replayed with `cargo shuyi replay
  --crash <report>`, no copying; what `same_place` says.
- A test in `crates/tools/src/game_tools/tests.rs` beside
  `a_crash_record_replays_to_the_crash`: a crash report holding the game's
  battle record (written as the game writes it) replays through the
  engine's `replay` with `--crash` to its end, `ok: true`.
- `plays.rs`' doc comments, if they still say the record must be copied
  out.

**Out (do not do):**
- Any change to what a crash report holds or where it is written (1245,
  0918).
- The release workflow's debug files (0917): they are already kept.
- Sending reports anywhere (`docs/design/crash-reports.md`: nothing is
  sent).

## Implementation steps

1. `cargo shuyi docs "crash report"` and `cargo shuyi docs "replay
   --crash"`; read the `shuyi-debug-replay` skill's *From a crash report*
   for the input types (`ReplayInput`'s crash field, `CrashOutput`).
   Expected size: about 40 test lines added, about 20 doc lines removed.
2. Write the test: build a report the way `crates/app` does (the engine's
   crash report type with `trpg_ui::crash::crash_record` as its `record`;
   if the engine gives no public way to write one, a fixture `.ron` in the
   test's temp folder), then call the engine's `replay` as `cargo shuyi
   replay --crash` does and assert `ok` and the number of steps.
3. Rewrite the two docs' paragraphs. Try both commands by hand on a report
   made with the debug tools' panic (`SHUYI_PANIC_AT_FRAME`, the engine's
   `docs/crash-reports.md`) in a battle, and paste what they print in the
   PR body.

## Acceptance criteria

- [x] `docs/releasing.md` reads a report with `cargo shuyi crash --symbols`;
      no `addr2line` or `objdump` recipe is left.
- [x] `docs/session-records.md` replays a report with `cargo shuyi replay
      --crash`.
- [x] A test shows a crash report holding the game's battle record replays
      through `replay --crash` with `ok: true`.
- [x] The PR body has both commands' output on a battle's crash report.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the `replay --crash` test in `crates/tools`.

## Completion notes

- `docs/releasing.md`, *The debug files*: the `objdump` / `grep` /
  `addr2line` recipe is gone; a report is read with `cargo shuyi crash
  <report> --symbols <debug file> [--json]`, with `SH3020` and the
  "frames fall outside every function" limit named. Tried by hand, a
  debug file of another build can also name frames with the wrong
  functions and no limit line, so the doc says to fetch the one whose
  version is the report's `game`.
- `docs/session-records.md`, *Crash reports*: the record replays straight
  from the report with `cargo shuyi replay --crash <report>`; what `ok`,
  `result.crash` and `same_place` say.
- `crates/tools/src/game_tools/tests.rs`,
  `a_crash_report_replays_with_replay_crash`: the debug Quick Battle
  played to the lord's first move, its `crash_record` put in the
  engine's `CrashReport` (the engine gives `CrashReport::new`,
  `set_record` and `to_text`, so no fixture file), replayed through the
  engine's `replay` with `crash: true`: `ok`, one run of every applied
  command plus the start, no `replayed` panic. No code change was needed
  in `plays.rs`, and its doc comments never said to copy the record out.
- Deviation, the by-hand try: a debug build run with
  `SHUYI_PANIC_AT_FRAME` wrote a real report, but keys sent to its window
  from a script never reached it (Windows kept the focus elsewhere), so
  it panicked on the title with no record. The battle record in the
  report both commands were tried on was added to that real report from
  the same Quick Battle played in the Harness (`CrashReport::parse`,
  `set_record`). The debug build's `.exe` stood in for a release's
  `.debug` (it holds the same DWARF).
- The older test beside it, `a_crash_record_replays_to_the_crash`, plays
  `Down f Left f f`, which no longer reaches a command (Preparations
  needs one more Confirm since ticket 1237's drivers): its record holds
  no command and its step counts pass on 0. Left as it is (out of
  scope); the new test uses the UI test's keys and asserts a command was
  applied. Follow-up: ticket 1396.
- Starting rules: none.
