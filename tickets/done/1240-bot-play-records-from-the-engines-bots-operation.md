---
id: "1240"
title: "Bot play records from the engine's one bots operation: the tools' bots run on playtest"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: ["engine#2544"]
nick_input: none
completed: 2026-11-07
---

# 1240 — Bot play records from the engine's one bots operation

## Context

The game/engine boundary audit of 2026-11-03 found that the game's
`bots` for Shuyi's tools (`bots` in `crates/xtask/src/game_tools/plays.rs`,
lines 43-84, from 1193) copies the engine's `play_tries` loop
(`entrant.start`, `entrant.bots`, `play_observed`, `measure.finish`) only
to keep each play's commands for its `PlayRecord`. The engine's `Tried`
drops them.

[engine#2544](https://github.com/Zafnok/shuyi-engine/blob/main/tickets/open/2544-one-operation-plays-a-games-bots-bots-folded-into-playtest.md)
folds the `bots` operation into `playtest`: each try carries its
`commands` and its `record` for `replay`, and `GameTools::bots` goes (a
breaking change). A game gives its bot run as a playtest plan with one
entrant and no checks. So once the pin passes 2544 the game must stop
implementing `GameTools::bots`, and the copied loop goes with it.

**How this relates to 1195** (stamping the play records): 1195 adds a
`made` field to `PlayRecord` and fills it where `bots` writes the record
today; this ticket moves where the record is written. **Either order
with 1195:** if 1195 is done, the record this ticket writes keeps
`made: Some(plays::made())`; if not, nothing to do (1195 fills `made` in
the plan's record hook when it lands).

## Nick input

None. Nothing a player sees changes; verified by the tool tests.

## Scope

**In:**
- Move the pin past engine#2544 (`engine-change` skill §7, the
  `engine-pin` skill), unless an open `engine-pin` PR has done so.
- `Trpg::bots` and `plays::bots` go; the plan's try writes the
  `PlayRecord` (battle, mode, seed, commands) through the record hook
  2544 gives a plan. `PlayRecord`, `VERSION` and `replay` stay.
- The two 1193 tests in `crates/xtask/src/game_tools/tests.rs`
  (`bots_play_three_games_with_records`,
  `a_bots_record_replays_the_same_twice`) call `playtest` with the
  `baseline` entrant instead, with the same assertions.
- `docs/engine/boundary.md`'s `bots` row and any game doc or skill that
  names the `bots` operation say `playtest`.

**Either order with 1207:** if 1207 is done, add the record to its
`Trpg::playtest_plan` and check that `--entrant baseline` with no checks
is the old bots run; if not, write the smallest plan that does what
`bots` does today (the `baseline` entrant in Classic, no checks, the
story's first battle by default, built with `plays::stage`), and 1207
grows it with the stages, personas, checks and summary.

**Out (do not do):**
- 1180's bot types and 1207's command, history and workflow, beyond the
  clause above.
- 1195's stamp (the clause above), and 0508's human play records.
- A new record format or a change to `replay`.

## Implementation steps

1. Move the pin (as above) and read 2544's completion notes: the
   `Breaking:` line, the try's `commands` and `record`, and how a plan
   gives a record.
2. Implement the plan's record hook in `plays.rs`: the `PlayRecord` of a
   try, as `ron::ser::to_string` of what `bots` builds today.
3. Delete `Trpg::bots` (`crates/xtask/src/game_tools.rs`), `plays::bots`
   and their `shuyi_cli::contract::bots` imports.
4. Move the two tests onto `playtest`; update the docs.
5. Measure with `cargo xtask code-lines`: about 50 production lines
   removed and 15 added.

## Acceptance criteria

- [ ] `shuyi-engine.rev` is at or past the commit that closed
      engine#2544; `cargo xtask ticket-lint --check-engine` passes.
- [x] `grep -rn "contract::bots\|fn bots" crates/xtask/src` finds nothing
      (it finds the engine's `Entrant::bots`, one bot per side, which
      `Recorded` passes on, and an unrelated test name: no `contract::bots`
      and no `GameTools::bots`).
- [ ] `cargo shuyi playtest --entrant baseline --tries 3 --json` answers
      three tries, each with `commands` and a `record` that
      `cargo shuyi replay` plays to the same hashes twice.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit (`game_tools/tests.rs`): three tries with records, seeds 1-3, and
  a record replaying to the play's own end, through `playtest`.

## Completion notes

Done 2026-11-07, in the pin move game ticket 0475 needed (its engine
change, engine#3211, merged after engine#2544, so the move to it had to
pass 2544).

- **The pin** moves e2697683 → c1ba9ec8, passing 17 engine PRs; the
  review is the ledger's section of 2026-11-07 (`docs/engine/pins.md`).
  Besides this ticket's change, the break fixes: the debug menu's new
  last entry "Debug view" (engine#2315) moves the debug-menu walks in the
  tests one step further and the kept Quick Battle session is written
  again; the storage test expects the `debug_view` warning; the xtask's
  icon atlas gains `wide_sets` and `font_atlas::run` takes no wide fonts
  (engine#2303); the tips' `LIMITS` sets `how_lines: 0` (engine#3211,
  used by 0475).
- **The bot run is a playtest plan:** `Trpg::bots` and `plays::bots` are
  gone; `Trpg::playtest_plan` gives `plays::plan`, the baseline in Classic
  with no checks, whose entrant (`Recorded`, wrapping `BattleEntrant`)
  writes each try's `PlayRecord` through `Entrant::record`.
  Autobalance's one try is `play_try`.
- **Tests:** `the_bot_run_plays_three_tries_with_records` and
  `a_bot_run_record_replays_the_same_twice` call `playtest` with the
  same assertions as before.

Deviations:

- `playtest` needs `--stage` when the game has several stages, and the
  game's stages are its battles (0509), so the bot run is
  `cargo shuyi playtest --stage <battle> --entrant baseline`; without
  `--stage` the engine refuses before the plan is asked. The plan itself
  falls back to the story's first battle. 1207 decides how the stages
  and defaults are offered.
- `docs/engine/boundary.md`'s `bots` row is about the crate, not the
  operation: nothing to change.
- 1195's and 1207's either-order clauses now say what holds; 0859's step
  2 now reads `Ctx::voice_playing()` (engine#2255, adopted in the ledger).

Follow-up tickets: 1251 (the battle map offers the debug view's grid
channel), from the ledger review.

Nick input: none; nothing a player sees changes.
