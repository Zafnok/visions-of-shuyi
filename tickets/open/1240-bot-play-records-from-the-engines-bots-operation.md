---
id: "1240"
title: "Bot play records from the engine's one bots operation: the tools' bots run on playtest"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: blocked
blocked_by: []
nick_input: none
completed:
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
entrant and no checks. The pin move of 2026-11-07 passed 2544 and, to build,
replaced `Trpg::bots` with the smallest `Trpg::playtest_plan` (the
`baseline` entrant in Classic, no checks, the stage asked for or the
story's first battle) and moved the two 1193 tests onto `playtest`. That
plan still copies the `play_tries` loop to keep each try's commands: what
is left here is to give the record through the engine's hook
(`Entrant::record` on `BattleEntrant`, or a plan of its own) and play
each try with `play_try`, deleting the copy.

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
- ~~Move the pin past engine#2544~~ (done by the pin move of
  2026-11-07, which also replaced `Trpg::bots` and moved the two 1193
  tests onto `playtest`).
- `plays::plan`'s try is `play_try`, and the `PlayRecord` (battle, mode,
  seed, commands) is written through the record hook 2544 gives
  (`Entrant::record`). `PlayRecord`, `VERSION` and `replay` stay.
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

1. ~~Move the pin~~ (done by the pin move of 2026-11-07); read 2544's completion notes: the
   `Breaking:` line, the try's `commands` and `record`, and how a plan
   gives a record.
2. Implement the plan's record hook in `plays.rs`: the `PlayRecord` of a
   try, as `ron::ser::to_string` of what `bots` builds today.
3. ~~Delete `Trpg::bots`~~ (done by the pin move); delete the copied
   loop in `plays::plan`.
4. Update the docs.
5. Measure with `cargo xtask code-lines`: about 30 production lines
   removed and 10 added.

## Acceptance criteria

- [ ] `shuyi-engine.rev` is at or past the commit that closed
      engine#2544; `cargo xtask ticket-lint --check-engine` passes.
- [ ] `grep -rn "contract::bots\|fn bots" crates/xtask/src` finds nothing.
- [ ] `cargo shuyi playtest --entrant baseline --tries 3 --json` answers
      three tries, each with `commands` and a `record` that
      `cargo shuyi replay` plays to the same hashes twice.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit (`game_tools/tests.rs`): three tries with records, seeds 1-3, and
  a record replaying to the play's own end, through `playtest`.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
