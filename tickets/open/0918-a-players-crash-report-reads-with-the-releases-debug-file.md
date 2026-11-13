---
id: "0918"
title: "A player's crash report reads with the release's debug file"
type: infra
milestone: M8 Release
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
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

## Nick input

None. The engine change needs Nick's OK before its engine ticket and PR
(the `engine-change` skill).

## Scope

**In:**
- An engine change (`engine-change` skill: engine ticket and PR in
  `Zafnok/shuyi-engine`, then this game's PR moves the pin): a crash
  report's backtrace gives each frame's address relative to the start of
  the program it is in (or the absolute address and the program's load
  address, as fields of the report), on Windows and Linux. Debug builds
  keep the names they show today.
- `docs/releasing.md` "The debug files": the exact commands that turn a
  player's report into functions and lines with the version's `.debug`
  (`addr2line -f -C -e`, with the address adjusted as the report needs),
  replacing the "Not yet usable" paragraph.

**Out (do not do):**
- Sending crash reports anywhere (they stay on the player's computer).
- macOS (no debug file is kept for it) and the web.
- Anything about where the debug files are kept (0917).

## Implementation steps

1. Read the engine's `crates/app/src/crash.rs`, `crates/platform/src/crash.rs`
   and `docs/crash-reports.md`. Decide how a frame's offset is found
   (e.g. `Backtrace::frames()` ips minus the main module's base:
   `GetModuleHandleW(null)` on Windows, `dl_iterate_phdr` or the
   `__executable_start` symbol on Linux) without a new crate if one
   isn't already in the tree; record the choice in the engine.
2. Engine ticket, PR, tests (a report's frames carry offsets; the format
   documented). Ask Nick before opening them.
3. Here: move the pin; on a by-hand release dry run with a temporary
   panic switch, or locally with `cargo shuyi export windows --release`
   and `SHUYI_PANIC_AT_FRAME` in a build that reads it, check that
   `addr2line` on the debug file names the panicking function.
4. `docs/releasing.md`.

## Acceptance criteria

- [ ] A crash report from a stripped release build gives frame offsets,
      and `addr2line` with that build's `.debug` names the panicking
      function and its file and line (output in the PR).
- [ ] `docs/releasing.md` has the commands, the "Not yet usable"
      paragraph gone.
- [ ] All gates in the `run-gates` skill pass, here and in the engine.

## Tests required

- Engine: a unit test of the report's frame format.
- Here: none beyond the gates; the check in step 3 is in the PR.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
