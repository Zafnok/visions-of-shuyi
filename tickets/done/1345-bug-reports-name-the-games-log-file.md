---
id: "1345"
title: "Bug reports name the game's log file, kept beside its crash reports"
type: infra
milestone: M1 Engine
model: opus-5.5 / gpt-6.1-sol
effort: low
status: done
blocked_by: []
nick_input: none
completed: 2026-11-23
---

# 1345 — Bug reports name the game's log file

## Context

The engine pin move of 2026-11-23 passed engine PR #522 (engine ticket
3041, log sinks in shipped builds). Every desktop build of the game now
writes `logs/latest.log` in its storage folder (since ADR-0116,
`Documents/My Games/…` on Windows) and keeps the three runs before it
(`previous-1.log` to `previous-3.log`). The first line names the engine,
the system, the build and the game's version; a panic is logged with its
message, `file:line` and thread, whether or not a crash report is written.
On the web the same lines go to the browser's console.

The game gets the files at the pin, but nothing tells a player or a
tester they exist. `docs/session-records.md` (*Crash reports*) says where
the crash reports are kept (`docs/design/crash-reports.md`, rule 2: the
path is in the docs, never on screen), and `docs/releasing.md` says what
a release ships; neither names the log.

## Nick input

None. Rule 2 of `docs/design/crash-reports.md` (no path on screen) is
kept: this ticket changes docs only.

## Scope

**In:**
- `docs/session-records.md`: a *The log file* section beside *Crash
  reports*: where `logs/` is on each desktop (the folder the crash
  reports are in), the four files it keeps, what a line reads
  (`{seconds} f{frame} {LEVEL} {target}: {text}`), the web's console,
  and `SHUYI_LOG` (engine PR #524) for a tester who wants more lines.
- Wherever the game asks a tester or player for a bug report (search
  `docs/`, `README.md`, the itch.io and playtest page texts under
  `docs/` and `.github/`): attach `latest.log` with the crash report.
- A test, if the game keeps a list of its storage folder's files
  anywhere (search `crates/` for `crash` and the storage folder's
  names), that `logs/` is in it.

**Out (do not do):**
- Anything on screen (rule 2 of the crash-report design).
- Changing what the engine logs or its filter's default.

Estimate: about 40 lines of docs, no production code.

## Implementation steps

1. Read the engine's *Logs* in `docs/platform-backends.md` and *The log
   file* in `docs/targets/desktop.md` at the pin.
2. Write the section in `docs/session-records.md`; link it from the
   bug-report texts the search finds.
3. Run a debug build once and check the folder and the first line match
   what the docs say.

## Acceptance criteria

- [x] `docs/session-records.md` names `logs/latest.log`, its three kept
      runs and where they are on Windows, macOS and Linux.
- [x] Every text that asks for a bug report asks for the log too.
- [x] All gates in the `run-gates` skill pass.

## Tests required

None beyond the gates (docs only), unless the search finds a list of the
storage folder's files.

## Completion notes

- `docs/session-records.md` has *The log file*, beside *Crash reports*:
  where `logs/` is on Windows, macOS and Linux (the crash reports'
  folder), the four files, the first line, what a line reads, the
  2 MiB cap, what a shipped build writes, `SHUYI_LOG` with an example,
  the web's console, and that a log is sent only to the developers.
- The texts that ask for a report now ask for the log too: the crash
  report paragraph and *Sending one* in `docs/session-records.md`, and
  *Read a player's report* in `docs/releasing.md` (the developer's side).
  The search found no bug-report ask in `README.md`, `.github/` (the
  playtest workflow has no page text) or elsewhere in `docs/`.
- No list of the storage folder's files in `crates/`, so no test.
- Step 3: a debug build was run once; `Documents\My Games\Visions of
  Shuyi\logs\latest.log` appeared, its first line `0.000 f0 INFO
  platform: Shuyi Engine 0.1.0, windows x86_64 debug, storage
  tactical-rpg, game version 0.3.0`, the format as written.
- Gates: the docs and ticket gates ran locally (skill sync, ticket-lint,
  check-mentions, check-leaks, typos, pr-size, the xtask tests); no Rust
  changed, so the build, clippy and test gates are left to CI.
- No starting rules: nothing on screen changed (rule 2 of
  `docs/design/crash-reports.md` kept).
