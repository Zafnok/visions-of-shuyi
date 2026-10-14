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
- [x] ADR-0063 to engine ADR-0006, with index rows.
- [x] Tickets for the decisions, the boundary gate, the carve, real
      time, more styles, going public, AI tooling and networking; `cargo
      xtask ticket-lint` passes.

## Tests required

None (docs only).

## Completion notes

Done 2026-10-14. The session used four research subagents: two crate
audits, one landscape study and one networking study.

- **Report:** [`docs/engine/feasibility.md`](../../docs/engine/feasibility.md).
- **Audit:** [`docs/engine/boundary.md`](../../docs/engine/boundary.md).
- **Nick's answers:** plan B, with the extra styles and networking
  parked until the engine reaches parity with the game. He named the
  engine **Shuyi Engine** and wants it public, under **MIT OR
  Apache-2.0**, with formats as plugins. His words are in the engine
  repository's
  [`docs/decisions.md`](https://github.com/Zafnok/shuyi-engine/blob/main/docs/decisions.md).
- **The engine's repository:**
  [`Zafnok/shuyi-engine`](https://github.com/Zafnok/shuyi-engine) was
  created and set up in the same session (its ticket 2001).
- **ADRs:**
  - ADR-0063 (the split) is Accepted here.
  - The proposals 0064–0067 moved to the engine repository as its
    ADR-0003 to ADR-0006.
  - It also gained ADR-0001, ADR-0002 (the rules it inherits from this
    repository) and ADR-0007 (formats are plugins).
- **Tickets here:**
  - 0059: the tactics kit and contributions; narrowed after Nick's
    answers.
  - 0060: free movement in the game.
  - 1102: the boundary gate; it can start now.
  - 1103–1118: the carve, after 0804, with the crates named `shuyi-*`
    from the start.
  - 1131: the code moves to the engine repository.
  - The proposed 1130 (license and name) was dropped: both are settled.
- **Tickets moved to the engine repository**, with their numbers:
  - 1119–1129 and 1132–1135: real time, styles, physics, the tool
    contract, docs, MCP, skills, visual scripting. The styles and visual
    scripting are parked.
  - 1140–1150: networking, parked.
- **New engine tickets:** 2002 (its gates); 2201–2204 (formats as
  plugins, Yarn and Ink dialogue, more image and audio formats, an
  optimized pack); 2901 (research: better images with machine learning).
- **Tickets changed here:**
  - 0053 now links engine ADR-0006.
  - 1011 notes where the bundle's layers will live.
  - 1017 notes where the script runtime will live.
- **Docs changed:**
  - `CLAUDE.md` (the `engine/` row);
  - `tickets/README.md` (the `11xx` block);
  - `docs/ROADMAP.md` (the engine section and Nick's queue);
  - `docs/adr/README.md`;
  - `docs/engine/README.md`.
