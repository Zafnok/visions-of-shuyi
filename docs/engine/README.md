# The engine

Nick, 2026-10-14: split an open-source, general 2D engine out of this
codebase; the game stays paid and is built on it.

| Document | What |
| -------- | ---- |
| [`feasibility.md`](feasibility.md) | The report: can we, what it costs the game, what I recommend (ticket 1101) |
| [`boundary.md`](boundary.md) | Every module: engine, tactics kit, game, or mixed; the couplings to cut |
| `decisions.md` | Nick's answers on the license, name and going public (written by 0059) |
| [ADR-0063](../adr/0063-an-engine-and-a-game-one-boundary-now-two-repositories-later.md) | The split: one boundary now, two repositories later |
| [ADR-0064](../adr/0064-game-styles-stages-two-kinds-of-simulation-cameras-and-projections.md) | Game styles: stages, turn and tick simulations, cameras, projections |
| [ADR-0065](../adr/0065-assets-standard-files-in-the-engines-own-data-out-and-a-deterministic-movement-kit.md) | Assets, animation, collision, optional physics |
| [ADR-0066](../adr/0066-ai-native-tooling-one-contract-three-surfaces-and-visual-scripting-as-a-view-over-text.md) | AI tooling (CLI, MCP, skills); visual scripting |
| [ADR-0067](../adr/0067-networking-in-layers-transports-sessions-sync-models-services.md) | Networking in layers |

Tickets: the `11xx` block (`tickets/README.md`), with the decisions
0059 and 0060.
