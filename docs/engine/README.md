# The engine: Shuyi Engine

Nick, 2026-10-14: split an open-source, general 2D engine out of this
codebase; the game stays paid and is built on it. He chose plan B and
the name **Shuyi Engine**, and the engine has its own public repository
under MIT OR Apache-2.0: [`Zafnok/shuyi-engine`](https://github.com/Zafnok/shuyi-engine).

The engine's code was carved out in this repository's `engine/` folder
(tickets 1102–1118 and 1154) and moved to the engine's repository with
its history on 2026-10-19 (ticket 1131). This repository has no engine
code now: the game depends on the engine's crates by a git revision.

## Working with two repositories

- **The pin.** `shuyi-engine.rev` names the engine commit the game builds
  with. The same commit is the `rev` of each `shuyi-*` crate in the root
  `Cargo.toml`'s `[workspace.dependencies]`, and the source of each in
  `Cargo.lock`. `cargo xtask engine-pin <commit>` moves all three (and
  with no commit, pins the commit the local checkout is at, once it is
  pushed). CI's `cargo xtask engine-pin --check` fails when they disagree.
- **The local checkout.** Over the 30 days before the move, 24 of the 83
  game PRs merged since `engine/` existed changed it (29%; 24 of all 332
  merged PRs in those 30 days), so the engine's API is still moving and
  one working copy should edit both sides (ADR-0063 §8):
  - `cargo xtask engine` clones the engine at the pin into `shuyi-engine/`
    (git-ignored; a checkout that is there already is fetched and left
    where it is) and writes `.cargo/engine-patch.toml`, a `[patch]` that
    `.cargo/config.toml` includes when it exists. The game then builds on
    the checkout: edit the engine there, commit and push it from there.
  - While the patch is on, `Cargo.lock` names the checkout instead of the
    pin. `cargo xtask engine --off` takes the patch away and writes the lock
    for the pin again: do it before committing the game. CI never has the
    patch, and `engine-pin --check` fails on a lock that names a checkout.
  - The checkout is a workspace of its own (the game's `Cargo.toml`
    excludes it): `cargo test --workspace` inside it runs the engine's
    tests, and the engine's `run-gates` skill is its gates.
- **`cargo shuyi`** is `cargo xtask shuyi`: the engine's tools
  (`shuyi-cli`) with this game's `frame-png` and `lang-status`.
- **The boundary** is the repository wall now: the engine can't name a
  game crate it can't see. `check-engine-boundary` (1102) retired with the
  move.

## Changing the engine from the game

A game ticket that needs the engine changed follows the
[`engine-change`](../../.claude/skills/engine-change/SKILL.md) skill: first check it is
really engine work (ADR-0063 §5 and the leak test), then either an engine PR
now (merged with a merge commit) and a game PR that moves the pin, or an
engine ticket, a marked workaround and a game follow-up `blocked_by:
["engine#NNNN"]` (ticket 1152). Engine work never makes the game wait.

| Here | What |
| ---- | ---- |
| [`feasibility.md`](feasibility.md) | The report: can we, what it costs the game, what I recommend (ticket 1101) |
| [`boundary.md`](boundary.md) | Every module of the game: engine, tactics kit, game, or mixed; the couplings to cut (written before the carve; paths under `engine/` are now the engine repository's) |
| [`friction.md`](friction.md) | The engine-friction log: every time game code came out far bigger than the job, and what we did about it ([ADR-0092](../adr/0092-engine-friction-game-code-far-bigger-than-the-job-triggers-a-review.md), the `engine-friction` skill) |
| [ADR-0063](../adr/0063-an-engine-and-a-game-one-boundary-now-two-repositories-later.md) | The split: one boundary here, then the code moves to the engine's repository after the carve (done, 1131) |
| Tickets 1102–1118, 1154, 1131, 1151–1153 | The carve, the move, then living with two repositories (`tickets/`) |

| In the engine's repository | What |
| -------------------------- | ---- |
| [`README.md`](https://github.com/Zafnok/shuyi-engine#readme) | The crates, the first example (Hello, Grid), how a game depends on them |
| [`docs/carve.md`](https://github.com/Zafnok/shuyi-engine/blob/main/docs/carve.md) | What moved from here, layer by layer, and the ticket that moved it |
| [`docs/decisions.md`](https://github.com/Zafnok/shuyi-engine/blob/main/docs/decisions.md) | Nick's answers, verbatim: plan B, the name, public, the license, formats as plugins, what is parked |
| [`docs/ROADMAP.md`](https://github.com/Zafnok/shuyi-engine/blob/main/docs/ROADMAP.md) | The carve, the MVP (parity with the game), after the MVP, parked |
| [`docs/adr/`](https://github.com/Zafnok/shuyi-engine/tree/main/docs/adr) | Engine ADRs. This repository's proposals 0064–0067 are its 0003–0006. "engine ADR-NNNN" in this repository means these |
| [`tickets/`](https://github.com/Zafnok/shuyi-engine/tree/main/tickets) | 1119–1129, 1132–1135 and 1140–1150 moved there with their numbers; new ones from 2001 |
