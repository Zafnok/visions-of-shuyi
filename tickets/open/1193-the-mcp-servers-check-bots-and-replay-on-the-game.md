---
id: "1193"
title: The MCP server's check, bots and replay on the game (GameTools for Trpg)
type: infra
milestone: "Engine: carve"
model: sonnet-5
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
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

- [ ] The MCP server's `check` answers ok with no errors on `main`'s
      content, and an error with file and line when a test breaks a file.
- [ ] `bots` with `games: 3` answers ok with three outcomes.
- [ ] `replay` of a record the bots wrote gives the same hashes twice.
- [ ] `cargo xtask ticket-lint` passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: one test per operation in `crates/xtask/tests/it/`.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
