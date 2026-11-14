---
id: "0918"
title: "A player's crash report reads with the release's debug file"
type: infra
milestone: M8 Release
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-11-14
---

# 0918 — A player's crash report reads with the release's debug file

## Context

0917 keeps each release's `<package>.debug` (Windows, Linux) on a release
of the private assets repository (`docs/releasing.md`, "The debug files").
It holds the symbols and line tables of the shipped, stripped program at
the same addresses, so it can name the function, file and line of any code
address in it.

But a crash report holds no address to read. The engine's
`crates/app/src/crash.rs` fills `backtrace` with
`std::backtrace::Backtrace::force_capture().to_string()`: the standard
library's short form, which in a stripped build is `<unknown>` frames
with no addresses (the engine's `docs/crash-reports.md` says so). And the
program is loaded at a different place each run (ASLR), so even the full
form's absolute addresses need the place the program was loaded to read.

Found while working 0917.

**From engine#2598 (done, engine PR #416, at the pin since the move of
2026-11-14):** a report's `frames` (`shuyi_platform::crash::CodeFrame`,
`offset: Option<u64>`) holds the panicking thread's return addresses as
offsets into the executable on Windows and Linux (`None` in another
module; none on macOS and the web), with no new crate. The engine's
`docs/crash-reports.md`, *Reading the frames*, has the `addr2line`
commands; its PR shows them naming the game's panicking frame from a
stripped Windows release. What is left here is the game's half: the
check on the game's own release package and `docs/releasing.md`.

## Nick input

None. (The engine change is done: engine#2598.)

## Scope

**In:**
- ~~An engine change (`engine-change` skill: engine ticket and PR in
  `Zafnok/shuyi-engine`, then this game's PR moves the pin): a crash
  report's backtrace gives each frame's address relative to the start of
  the program it is in (or the absolute address and the program's load
  address, as fields of the report), on Windows and Linux. Debug builds
  keep the names they show today.~~ Done by engine#2598 (engine PR #416).
- `docs/releasing.md` "The debug files": the exact commands that turn a
  player's report into functions and lines with the version's `.debug`
  (`addr2line -f -C -e`, with the address adjusted as the report needs),
  replacing the "Not yet usable" paragraph.

**Out (do not do):**
- Sending crash reports anywhere (they stay on the player's computer).
- macOS (no debug file is kept for it) and the web.
- Anything about where the debug files are kept (0917).

## Implementation steps

1. ~~Read the engine's `crates/app/src/crash.rs`, `crates/platform/src/crash.rs`
   and `docs/crash-reports.md`. Decide how a frame's offset is found
   (e.g. `Backtrace::frames()` ips minus the main module's base:
   `GetModuleHandleW(null)` on Windows, `dl_iterate_phdr` or the
   `__executable_start` symbol on Linux) without a new crate if one
   isn't already in the tree; record the choice in the engine.
2. Engine ticket, PR, tests (a report's frames carry offsets; the format
   documented). Ask Nick before opening them.~~ Done by engine#2598.
3. Here: ~~move the pin~~ (done by the pin move of 2026-11-14); on a by-hand release dry run with a temporary
   panic switch, or locally with `cargo shuyi export windows --release`
   and `SHUYI_PANIC_AT_FRAME` in a build that reads it, check that
   `addr2line` on the debug file names the panicking function.
4. `docs/releasing.md`.

## Acceptance criteria

- [x] A crash report from a stripped release build gives frame offsets,
      and `addr2line` with that build's `.debug` names the panicking
      function and its file and line (output in the PR).
- [x] `docs/releasing.md` has the commands, the "Not yet usable"
      paragraph gone.
- [x] All gates in the `run-gates` skill pass, here and in the engine.

## Tests required

- ~~Engine: a unit test of the report's frame format.~~ Done in engine#2598.
- Here: none beyond the gates; the check in step 3 is in the PR.

## Completion notes

- **Engine:** engine ticket 2598 (Zafnok/shuyi-engine#416). A
  crash report gains `frames`, the panicking thread's stack as offsets
  into the program (`(offset: Some(n))`, or `None` in a system library),
  on Windows and Linux. The stack is walked by the system's own unwinder
  (`RtlCaptureStackBackTrace` and `GetModuleHandleExW` on Windows,
  glibc's `backtrace()` and `/proc/self/maps` on Linux), so no crate was
  added. The `backtrace` text stays as it was, so a debug build shows
  the names it shows today. Engine ticket 2520 (`shuyi crash
  --symbols`) had planned these frames with the `backtrace` crate; it
  now builds on 2598's.
- **Here:** Nick moved the pin to the engine PR's merge himself
  (Zafnok/visions-of-shuyi#683, `4ce11a3b`), so this PR has no pin
  move; `docs/releasing.md`
  "The debug files" has the commands that read a player's report with
  the version's `.debug` (the image base from `objdump -p`, each offset
  minus 1, `addr2line -f -C -i -p`), in place of "Not yet usable".
- **Checked by hand** (step 3): `cargo shuyi export windows --release`
  against the engine change, then the stripped `visions-of-shuyi.exe`
  with `SHUYI_PANIC_AT_FRAME=3`. Its report held 18 frames in the program
  and 2 outside, and the documented commands named the panicking
  function at the report's `location` (`shuyi_app`'s `frame_or_crash`,
  `lib.rs:583`), and the game's `main.rs:89` and `:94` under it. The
  output is in the PR. The test report was removed from the data folder
  afterwards.
- Deviation: the ticket's step 1 suggested `Backtrace::frames()`; that
  is unstable Rust, so the system's unwinder took its place.
- No gameplay rules decided. Nothing for the Act 1 playtest.
