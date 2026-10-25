---
id: "1193"
title: The MCP server's check, bots and replay on the game (GameTools for Trpg)
type: infra
milestone: "Engine: carve"
model: sonnet-5
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-10-25
---

# 1193 — The MCP server's check, bots and replay on the game

## Context

1181 registered the engine's MCP server on the game (`.mcp.json`, `cargo
shuyi mcp`). Its session tools (`start`, `step`, `scene`, `frame_png`,
`events`) work, because the game's `GameTools` (`Trpg` in
`crates/xtask/src/game_tools.rs`) gives `frame`, `session` and `lang`.
On 2026-10-25 (found while working 1189), the operations that need more of
`GameTools` answer `SH3001` ("the game's tools don't offer …"):

- `check`: `GameTools::check` (every content validator's errors, each
  with its file and line, as `ContentError`s);
- `bots`: `GameTools::bots` (`BotsInput` → `BotsOutput`, summed with
  `shuyi_cli::contract::bots::tally`);
- `replay`: `GameTools::replay` (a record re-run by the rules; for a
  turn game, `shuyi_cli::contract::replay::turns`).

The game already has each piece: content loading and validation in
`trpg-content` (today run by `cargo test`), the bots in `trpg-bots`
(`crates/bots/src/bot.rs`, `playtest.rs`), and play records (ADR-0038
mentions them; the bots write them). This ticket hands them to the
engine's contract.

`playtest` and `autobalance` also answer `SH3001`; they are 1180 and
1183, not this ticket. `modules` answers `SH1004` because the game
depends on the engine's crates one by one, not on the `shuyi` crate;
that is out of scope.

## Nick input

None.

## Scope

**In:**
- `impl GameTools for Trpg`: `check`, `bots`, `replay`.
- Integration tests in `crates/xtask/tests/it/` (a module listed in
  `main.rs`, never a new file under `tests/`) that call each through
  `shuyi_cli`'s contract as the MCP server does.

**Out (do not do):**
- `playtest_stages`, `playtest_plan`, `autobalance_plan` (1180, 1183).
- Depending on the `shuyi` crate for `modules`.
- Engine changes. A missing engine API follows the `engine-change` skill.
- New validators or bot behaviour: hand over what exists.

## Implementation steps

1. Read the trait in the engine at the pin
   (`crates/cli/src/game.rs`, `GameTools`) and the contract types each
   method returns (`contract::check`, `contract::bots`, `contract::replay`).
2. **Estimate first** (ADR-0092): about 120 production lines — three
   adapters from existing game functions to contract types, judged by
   `Trpg::lang` and `frame` in the same file.
3. `check`: find where `trpg-content` loads and validates every asset
   (the content tests in `crates/content/src/**/tests.rs` call it), call
   it on the embedded assets, and map each error to a `ContentError`
   with its file, line and code.
4. `bots`: play `input.games` games from `input.seed` with the default
   bots of `trpg-bots`, stopping at `input.turns`, and sum them with
   `tally`.
5. `replay`: read a record in the game's record format and re-run it with
   `contract::replay::turns`, a state hash per step.
6. Tests (Scope), then `cargo xtask code-lines` against the estimate.

## Acceptance criteria

- [x] The MCP server's `check` answers ok with no errors on `main`'s
      content, and an error with file and line when a test breaks a file.
- [x] `bots` with `games: 3` answers ok with three outcomes.
- [x] `replay` of a record the bots wrote gives the same hashes twice.
- [x] `cargo xtask ticket-lint` passes.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Integration: one test per operation in `crates/xtask/tests/it/`.

## Completion notes

Done 2026-10-25.

- `impl GameTools for Trpg` gives `check`, `bots` and `replay`
  (`crates/xtask/src/game_tools/plays.rs`):
  - `check` is `trpg_content::load_embedded`'s errors, which are already
    the engine's `ContentError`s (file, line, column).
  - `bots` plays the story's first battle (`new_game.first_chapter`'s,
    Chapter 1's today) with the baseline bot in Classic, the engine's
    play loop (`play_observed`) with `trpg-bots`' entrant and measures,
    summed by `tally`. The game's start options are checked, then
    ignored: they are about the look, which bots never see (ADR-0038 §5).
  - `replay` reads a `PlayRecord` and re-runs it with
    `contract::replay::turns`.
- **New: the play record format.** The game had none (0508 plans one
  and is still open), so this ticket adds the smallest: RON
  `(version: 1, battle, mode, seed, commands)`, nothing about the look.
  0508 may grow it or move it out of xtask (its human records add
  rewinds, restarts and play time); the `version` field is there for that.
- `playtest::stage` is the battle setup `cargo xtask playtest` built
  inline, pulled out so `bots` and `replay` build the same battle.

**Deviations.**
- Tests: xtask is a binary with no library and has no `tests/it/`
  (its one integration file, `tests/cli.rs`, runs the binary). An
  integration test can't break an embedded file, so the tests are unit
  tests in `crates/xtask/src/game_tools/tests.rs`, beside the session's
  (1181), calling `shuyi_cli::contract::{check, bots, replay}` with
  `&Trpg`, the functions the MCP server calls. The broken file is laid
  over the bundle with `bundle::reading`.
- The engine's `play_tries` drops a try's commands, so `bots` keeps its
  own copy of that loop (~10 lines) to write the record. Small: noted,
  not filed.

**Estimate (ADR-0092):** 120 production lines (the ticket's, judged by
`Trpg::lang` and `frame`). Measured: 115 added (0.96×), under 2×.

**Claude's starting rules:** none (dev tooling; no gameplay rule).
