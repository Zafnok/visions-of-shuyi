---
id: "1101"
title: "Research: can we split an open-source 2D engine from the game, and what would it cost the game?"
type: research
milestone: "Engine: boundary"
model: opus-5.5
effort: max
status: done
blocked_by: []
nick_input: none
completed: 2026-10-14
---

# 1101 — Research: split an open-source engine from the game

## Context

Nick, 2026-10-14: determine, high level, whether the project can be
rearchitected into two repositories and goals:

1. **an engine**, general purpose, which we add to and may ship one day
   with graphical coding (like Unreal's Blueprints), free/open source;
   it must make our game easy to make *and* make other developers'
   games easy (not only tactics games: top-down grid, top-down free
   movement, isometric grid or free, side-scrolling, and moving between
   them; no 3D), with good asset loading (raw files, tilesets, sprite
   files), animations, cutscenes, audio cues, collision, physics, "a
   serious 2D game engine", strong AI vibe-coding support and "plenty of
   shippable skills", like REA's AI skill wrappers around existing tools;
2. **the game**, paid, built on the engine as an external dependency.

Later the same day: if feasible without hurting the game's velocity,
also look at easy, abstracted **networking**: P2P (LAN or WAN), sessions,
hosted multiplayer (one player's machine, or a company's servers),
matchmaking and ELO, MMO, global chat, phased easy to hard, with both
abstractions and the lower-level pieces exposed.

The outcome asked for: new ADRs, new tickets, and a report on
feasibility, or on whether the abstraction would cost too much game
velocity.

## Nick input

None to do the research. The report ends with the decisions that are
his: 0059 (license, name, going public), 0060 (free movement in our
game), and how much engine work to schedule against game work.

## Scope

**In:** an audit of every crate (engine, tactics kit, game, mixed);
research on Rust and other 2D engines, agent tooling, asset formats,
physics, audio, scripting, licensing precedents, cost evidence and
networking; ADRs; tickets; the report.

**Out (do not do):** any code; moving any file; changing `LICENSE`.

## Implementation steps

1. Audit `core`, `content`, `ui`, `app`, `xtask` and `bots`
   module by module.
2. Research the landscape and the networking stack, with sources.
3. Write the ADRs, the boundary map, the report and the tickets.

## Acceptance criteria

- [x] `docs/engine/boundary.md`: every module classified, with the
      couplings that block a second game.
- [x] `docs/engine/feasibility.md`: the report, with sources.
- [x] ADR-0063 to ADR-0067, with index rows.
- [x] Tickets for the decisions, the boundary gate, the carve, real
      time, more styles, going public, AI tooling and networking; `cargo
      xtask ticket-lint` passes.

## Tests required

None (docs only).

## Completion notes

Done 2026-10-14 by a session with four research subagents (two crate
audits, one landscape study, one networking study).

- **Report:** [`docs/engine/feasibility.md`](../../docs/engine/feasibility.md).
- **Audit:** [`docs/engine/boundary.md`](../../docs/engine/boundary.md).
- **ADRs (all Proposed until Nick approves the plan):** 0063 (the split:
  one boundary now, two repositories later), 0064 (game styles: stages,
  turn and tick simulations, cameras, projections), 0065 (assets,
  animation, collision, optional physics), 0066 (AI tooling: one contract,
  CLI + MCP + skills; visual scripting as a view over text), 0067
  (networking in layers).
- **Tickets created:** 0059, 0060 (decisions for Nick); 1102 (the
  boundary gate, can start now); 1103–1118 (the carve, after 0804);
  1119–1125 (real time and the first free-movement example); 1126–1128
  (isometric, side view, physics); 1129–1135 (going public, AI tooling,
  visual scripting); 1140–1150 (networking).
- **Tickets changed:** 0053 (multiplayer decision: links ADR-0067), 1011
  (mods: the bundle's layers), 1017 (scripting: where the runtime lives).
- **Docs changed:** `tickets/README.md` (the `11xx` block),
  `docs/ROADMAP.md` (the engine section), `docs/adr/README.md`.
