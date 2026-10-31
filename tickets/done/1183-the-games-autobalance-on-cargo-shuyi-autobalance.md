---
id: "1183"
title: "The game's autobalance on cargo shuyi autobalance: the xtask wrapper goes, the game gives its plan"
type: feature
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: ["0509", "engine#2541"]
nick_input: none
completed: 2026-10-25
---

# 1183 — The game's autobalance on `cargo shuyi autobalance`

## Context

From the retro of the last 50 PRs (2026-10-22). The persona playtest
went into the game as about 1,000 lines that weren't the game's (0506),
and 1180 now moves them into the engine. 0509 (autobalance) is set to
repeat it: engine#2127 gives the search (`autobalance`, `Knob`,
`BalanceSettings`) but left out "a `shuyi autobalance` contract
operation" and "threads, time limits", so 0509 writes a game-side
`cargo xtask autobalance` (options, printing, `--apply`), and a persona
try takes 2.5–45 s of one core (ADR-0095), so a search of up to 200
runs needs the threads the game would write too.

engine#2541 adds `shuyi autobalance`: the game gives a plan (its stages,
the knobs of a stage, the entrants and checks, how to write a changed
stage back) and the command runs the search on every core with a time
limit, prints the changes and the reports, and writes with `--apply`.
This ticket moves 0509's wrapper onto it.

**If 0509 is not done when engine#2541 lands,** 0509 uses `cargo shuyi
autobalance` directly; its PR moves this ticket to `done/` with a note
("done by 0509").

## Nick input

None. The tool's output and what it writes don't change; verified by
the comparison below.

## Scope

**In:**
- Move the pin past engine#2541 (`engine-change` skill, §7).
- `Trpg` (`crates/xtask/src/game_tools.rs`) implements engine#2541's
  plan hooks with what `cargo xtask autobalance` does today: the battle
  from its id, its kind's knobs (`crates/bots/src/balance.rs`, 0509),
  0506's entrants and checks (or the playtest plan's, if 1180 is done),
  and writing the battle file for `--apply`.
- The `autobalance` arm of `crates/xtask/src/main.rs` and its module go;
  their CLI tests move to `cargo shuyi autobalance`.
- `docs/playtesting.md`: the command's new name and options.

**Out (do not do):**
- The knobs, the search, the bands, the personas (unchanged).
- Running autobalance in CI (0509 keeps it a dev tool).

**Either order with 1180:** if 1180 is done, the plan reuses the
playtest plan's entrants and checks; if not, it builds them as 0509's
wrapper does, and 1180 moves them with the rest.

## Implementation steps

1. **A baseline first**, on the current pin: `cargo xtask autobalance
   <test battle id> --runs 20 --quick 10 --budget 20 --seed 1` into a
   file outside the repo.
2. **Pin** past engine#2541; fix what it breaks.
3. The plan hooks in `game_tools.rs`; delete the xtask command.
4. **Compare**: `cargo shuyi autobalance --stage <same id>` with the same
   numbers gives the same change set and reports (the search is
   deterministic; threads don't change the result, engine#2541's rule).
5. Docs; measure with `cargo xtask code-lines` (estimate about 60 added,
   about 120 removed).

## Acceptance criteria

- [ ] `shuyi-engine.rev` is at or past the commit that closed
      engine#2541.
- [ ] `cargo xtask autobalance` is gone; `cargo shuyi autobalance` gives
      step 1's change set and reports on the same battle and numbers.
- [ ] `--apply` writes the same battle file as before (a test on a test
      battle in `assets-test/`).
- [ ] All gates in the `run-gates` skill pass, including
      `cargo xtask ticket-lint --check-engine`.

## Tests required

- Integration: `crates/xtask/tests/cli.rs` runs `cargo shuyi
  autobalance` on a test battle with a small budget.

## Completion notes

Done by 0509: 0509 implemented engine#2541's plan hooks directly, so
there was no `cargo xtask autobalance` to move. `Trpg::autobalance_plan`
(`crates/xtask/src/game_tools/balance.rs`) gives `cargo shuyi
autobalance` the battle, its kind's knobs, 0506's personas and checks and
the battle file's `write`; the pin was already past engine#2541 (1188).
