---
id: "1245"
title: Crash reports with the game's version and the battle's record
type: feature
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 1245 — Crash reports with the game's version and the battle's record

## Context

Engine PR #275 (3046) came in with the pin move of 2026-11-05 (issue
#548). `shuyi_app`'s panic hook now writes a crash report through the
game's storage (`crash-<when>`, five kept on the desktop, one on the
web): the panic's message and place, the frame, the top screen, the
log's last lines, the engine's and the game's versions, and a record of
the game's play when the game gives one. At the next start,
`Setup::last_crash` hands the game the newest report not yet offered,
once. Engine ADR-0079 and the engine's `docs/crash-reports.md` say how.

The game starts through `shuyi_app::run` with its own `AppConfig`
(`crates/app/src/main.rs`), not `run_game`, so its reports carry no
game version and no record, and nothing at the next start reads them.
The battle already keeps a session record that `shuyi run`, `scene`
and the MCP tools play (ticket 1210, `trpg_ui::start`, `session_from` in
the tools): that record is what a crash report should carry, so a crash
in a battle replays.

The Playdate's crash reports are 1170's (engine 3076), not this one's.

## Nick input

**Decision**, asked before building, with mockups: what the player
sees at the start after a crash. Options to put to him (games do each):
nothing in game (the report is only a file for bug reports); a one-time
notice on the title ("The game closed unexpectedly last time. A report
was saved."); a notice that also says where the file is or opens the
folder. Wording and look follow his answer.

## Scope

**In:**
- `crates/app/src/main.rs`: the config names the game's version
  (`shuyi.ron`'s `app`, `APP.version`), as `run_game` does.
- The game's record in each report: `Start::crash_record` with the
  battle's session record (1210's), so a report from a battle replays
  with `cargo shuyi replay` / `run`.
- The next start reads `Setup::last_crash` and does what Nick chose.
- A line in `docs/` where bug reports are described (or the README's
  section on reporting a problem) naming where the reports are kept.

**Out (do not do):**
- The Playdate (1170).
- Sending reports anywhere: they stay on the player's machine.

## Implementation steps

1. Read engine ADR-0079 and `docs/crash-reports.md` at the pin.
2. Set the version in `config()`; give `Start::crash_record` a hook
   that returns the current battle's record text (none outside a battle).
3. Wire `Setup::last_crash` to the screen Nick chose (a view and a skin,
   ADR-0054; its text by key, ADR-0045).
4. Check by hand: `SHUYI_PANIC_AT_FRAME=<n>` on a debug build in a
   battle writes a report with the version and a record that replays.

## Acceptance criteria

- [ ] A report written in a battle names the game's version and holds a
      record that `cargo shuyi replay` plays to the frame it crashed on
      (test through the headless backend, as the engine's hello-grid
      test does).
- [ ] The start after a crash does what Nick chose, once (Harness test
      with a storage holding a report).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the record hook's text in and out of a battle.
- Integration: the next start's behaviour, through the Harness.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
