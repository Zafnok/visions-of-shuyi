---
id: "1188"
title: "Move the engine pin to the engine's main (engine 2532 and 2541)"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: low
status: done
blocked_by: []
nick_input: none
completed: 2026-10-22
---

# 1188 — Move the engine pin to the engine's main

## Context

The game builds with the Shuyi Engine commit in `shuyi-engine.rev`
(`8d5055ca`, engine PR Zafnok/shuyi-engine#67, pinned by 1177). The
engine's `main` (`3f2256d4` on 2026-10-22) is six PRs past it,
#69–#74. Three of them change code:

- **#71, engine#2022**: CI only.
- **#72, engine#2532**, `shuyi playtest` on every core. It removes
  `GameTools::playtest` (the game never implemented it) and makes
  `PlaytestInput`'s fields `Option`s. The game uses neither.
- **#73, engine#2541**, `shuyi autobalance`: `GameTools::autobalance_plan`
  (with a default), `autobalance_by`, and a new `Stop::Stopped` variant.
  The game has no `match` on `Stop`.

Both PR bodies say the game's next pin move builds unchanged.

The point is 0509 (autobalance battles with the persona bots). With
engine#2541 pinned, 0509 takes the first branch of its "either order
with engine#2541": it implements the plan hooks of `cargo shuyi
autobalance` in `crates/xtask/src/game_tools.rs` instead of writing a
`cargo xtask autobalance` command with its own threads, and 1183
(moving that command onto the engine) is done by 0509's PR. This ticket
settles 0509's text to say so.

## Nick input

None. Nick asked for the pin move (2026-10-22); he schedules 0509
himself.

## Scope

**In:**
- `cargo xtask engine-pin <commit>` to the engine's `main` (the newest
  commit when the ticket is worked; at least `3f2256d4`, which closed
  engine#2541).
- Whatever the game needs to build and pass the gates on the new pin,
  game side only, keeping every behaviour the same.
- `cargo shuyi mcp --ready` after the pin moves (`engine-change` §3.6).
- 0509's text: the pin is past engine#2541; it implements the plan
  hooks and its PR moves 1183 to `done/`. The same in
  `docs/ROADMAP.md`'s lane C and its list of what the retro filed.
- The open tickets that balance a battle with 0509's command (0512,
  0848–0855, 1071, 1072) name `cargo shuyi autobalance --stage <id>`
  instead of `cargo xtask autobalance <id>`, which won't exist.

**Out (do not do):**
- 0509 itself (the knobs, the plan hooks) and 1183's move to `done/`:
  0509's PR moves it.
- Using `cargo shuyi playtest` in the game: that is 1180.
- Engine changes. If the new pin breaks something the game can't fix
  on its side, stop and follow the `engine-change` skill.

## Implementation steps

1. `cargo xtask engine --off` if a local checkout is patched in.
2. `cargo xtask engine-pin <engine main's commit>`.
3. Build and run the gates (`run-gates` skill); fix what the new engine
   needs, game side only. If a snapshot changes, say why in the PR (the
   engine ticket that changed it).
4. `cargo shuyi mcp --ready`.
5. In 0509: rewrite "Either order with engine#2541" as what holds now,
   and its Scope line and step 1 ("past engine#2127") as "the pin is
   past engine#2541 (1188)"; put engine#2541 in its `blocked_by`. In
   `docs/ROADMAP.md`, 0509 uses `shuyi autobalance` and 1183 is done by
   0509's PR. In the other open tickets, `cargo xtask autobalance <id>`
   becomes `cargo shuyi autobalance --stage <id>`
   (`grep -rn "xtask autobalance" tickets/open`; 1183's own text stays,
   since 0509's PR closes it).

## Acceptance criteria

- [x] `shuyi-engine.rev` is at or past the commit that closed
      engine#2541 (`gh api repos/Zafnok/shuyi-engine/compare/3f2256d4...<pin>`
      says `ahead` or `identical`).
- [x] `cargo xtask engine-pin --check` passes.
- [x] No behaviour changes: snapshots and play records unchanged, or
      each change explained by an engine ticket.
- [x] `cargo shuyi mcp --ready` succeeds.
- [x] 0509 no longer says "either order with engine#2541" or "past
      engine#2127"; no open ticket but 0509, 1183 and this one names
      `cargo xtask autobalance`; `cargo xtask ticket-lint` passes.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- None new; the existing suite on the new pin.

## Completion notes

Moved the pin from `8d5055ca` to engine main `3f2256d4` with
`cargo xtask engine-pin`; it passes engine PRs #69–#74. Engine tickets
2532 and 2541 are in the engine's `done/`, and the compare from the
commit that closed each to the pin says `ahead` (2532) and `identical`
(2541). No game code changed: all gates pass (3632 tests, 5 ignored),
and no snapshot or play record changed. `cargo shuyi mcp --ready` says
`ready (contract 12)`.

0509 is settled: its "either order with engine#2541" became what holds
(it implements the plan hooks, writes no `cargo xtask autobalance`, and
its PR moves 1183 to `done/`), and engine#2541 is in its `blocked_by`.
Ten other open tickets (0512, 0848–0855, 1071, 1072) named
`cargo xtask autobalance <id>`, a command that will now never exist;
they say `cargo shuyi autobalance --stage <id>`. That edit was added to
this ticket's scope while it was being worked. Nothing for Nick; no
starting rules; no follow-ups.
