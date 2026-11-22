---
id: "0337"
title: "Barricades that units can break"
type: feature
milestone: M2 Core rules
model: opus-5.5 / gpt-6.1-sol
effort: high
status: blocked
blocked_by: ["0097"]
nick_input: answer-first
completed:
---

# 0337 — Barricades that units can break

## Context

Nick (2026-11-22) wants breakable terrain such as barricades for some
battles; 0097 decides the rules (HP, who can hit it, EXP, what's left).
Core today has no terrain with HP; tiles change only by the magic swaps
(0310), through `Command` → `Event` (ADR-0004). The AI (`05xx`), the bots
(`crates/bots`) and the action list (0335: one list of what a unit can do
on a tile) must all see "attack the barricade" as an action.

## Nick input

**Answer first:** 0097. Nothing to check after merge: verified by tests,
`frame-png` pictures and the bots; what's left to judge in play goes on
1064's checklist.

## Scope

**In:**
- Core: a breakable tile with HP (and defence per 0097) in the battle
  state; an attack command on it; a `TerrainBroken` event that swaps the
  tile; EXP per 0097; replays and the state hash include it.
- Data: the barricade terrain row (and its rubble) in `terrain.ron`; map
  placement with HP per tile in the map format if 0097 allows per-map HP.
- UI: the forecast against a barricade; its HP in the terrain window;
  the map scene marks it (ADR-0038), every map skin draws it and its
  break.
- AI: enemies break barricades when 0097 says they do; otherwise treat
  them as walls.
- Bots: the action is in their list (via 0335 if done, else add it where
  they list actions and note it in 0335).

**Out (do not do):** doors and one-way tiles (0338); placing barricades
in a chapter (that chapter's ticket, unless 0097 names one).

## Implementation steps

1. Core state, command, event, tests.
2. Content: terrain row, map format, validation.
3. UI: forecast, info, scene, skins (sprite from the bundle's sets,
   glyph, Playdate).
4. AI and bots.
5. A staged test battle in `assets-test/` with a barricade.

## Acceptance criteria

- [ ] Core tests: attacking a barricade lowers its HP; at 0 it becomes
      0097's leftover tile and paths through it open.
- [ ] A replay of the test battle reproduces the same hash.
- [ ] The bots break it when it's the way to the goal (a bot test).
- [ ] `frame-png` pictures of the barricade before and after in all
      skins.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit and property (core); view tests (scene); bot test; snapshot of the
  forecast.

## Lesser targets

Playdate: a one-bit tile and the same rules; no break animation beyond a
still swap if 1314 says `Reduced`. GBA: no target.

## Completion notes

