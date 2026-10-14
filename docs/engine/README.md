# The engine: Shuyi Engine

Nick, 2026-10-14: split an open-source, general 2D engine out of this
codebase; the game stays paid and is built on it. He chose plan B and
the name **Shuyi Engine**, and the engine has its own public repository
under MIT OR Apache-2.0: [`Zafnok/shuyi-engine`](https://github.com/Zafnok/shuyi-engine).

| Here | What |
| ---- | ---- |
| [`feasibility.md`](feasibility.md) | The report: can we, what it costs the game, what I recommend (ticket 1101) |
| [`boundary.md`](boundary.md) | Every module of the game: engine, tactics kit, game, or mixed; the couplings to cut |
| [ADR-0063](../adr/0063-an-engine-and-a-game-one-boundary-now-two-repositories-later.md) | The split: one boundary here now, the code moves to the engine's repository after the carve |
| Tickets 1102–1118, 1131 | The carve and the move (`tickets/open/`) |

| In the engine's repository | What |
| -------------------------- | ---- |
| [`docs/decisions.md`](https://github.com/Zafnok/shuyi-engine/blob/main/docs/decisions.md) | Nick's answers, verbatim: plan B, the name, public, the license, formats as plugins, what is parked |
| [`docs/ROADMAP.md`](https://github.com/Zafnok/shuyi-engine/blob/main/docs/ROADMAP.md) | The carve, the MVP (parity with the game), after the MVP, parked |
| [`docs/adr/`](https://github.com/Zafnok/shuyi-engine/tree/main/docs/adr) | Engine ADRs. This repository's proposals 0064–0067 are its 0003–0006. "engine ADR-NNNN" in this repository means these |
| [`tickets/`](https://github.com/Zafnok/shuyi-engine/tree/main/tickets) | 1119–1129, 1132–1135 and 1140–1150 moved there with their numbers; new ones from 2001 |
