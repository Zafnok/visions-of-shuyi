---
id: "0097"
title: "Decide: breakable barricades and doors you pass one way"
type: design-decision
milestone: Design decisions
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 0097 — Decide: breakable barricades and doors you pass one way

## Context

Nick (2026-11-22):

> "I want to add breakable terrain / one way directional terrain to the
> roadmap for some of our battles like barricades (breakable), doors (one
> way, not from side only front)"

Nothing like it exists: `door` and `wall` (`assets/data/terrain.ron`,
`docs/design/terrain.md`) are impassable to everyone, fliers too, with no
open or unlock rule. The only tiles that change in battle are the magic
swaps (forest → burning → burnt, water → ice; 0310). Related: 0491
(Chapter 4's doorways look like doors), 1091 (element states on tiles),
1098 (a beast's broken part falls as terrain). The core rule is that state
changes only through `Command` → `Event`s (ADR-0004), so a broken
barricade is an event the bots and replays see.

## Nick input

**Decision** (`ask-nick`, with mockups of a barricade and a door on a
staged map, both looks). Questions with game examples (check before
citing):

1. **Barricades:** HP and defence like a unit (Fire Emblem's cracked
   walls and snag barriers, which any unit can attack and which show HP);
   who can attack them (anyone, or only axes/hammers/magic); do they count
   for EXP (Fire Emblem: a little); do they block arrows and spells; does
   a broken one leave rubble (a cost-2 tile) or plain ground.
2. **Doors, "one way, not from side only front":** which reading is
   meant — A. a door is passed only straight through (north–south for a
   door in an east–west wall), never entered from the side; B. a door
   lets units through only from one side (an exit, like a portcullis you
   drop through), C. Fire Emblem's doors that a thief or a door key opens
   and that then stay open, D. your own. Can enemies use it the same way.
3. **Other directional terrain:** ledges you jump down but can't climb
   (Pokémon's ledges, Advance Wars has none), one-way bridges, currents.
4. **Which battles:** which Act 1 or later maps get them, so the build
   tickets know if they are on the Act 1 critical path.

Answers go to `docs/design/terrain.md`.

## Scope

**In:** the questions, mockups, the answers; 0337 and 0338 adjusted; the
roadmap's line updated with where they fall.

**Out (do not do):** code (0337, 0338).

## Implementation steps

1. Mockups on a throwaway staged map with `frame-png`.
2. Ask Nick; record; adjust 0337, 0338 and `docs/ROADMAP.md`.

## Acceptance criteria

- [ ] `docs/design/terrain.md` has the barricade and door rules and the
      battles that use them.
- [ ] 0337 and 0338 match; the roadmap says if they're on the Act 1
      critical path.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- None (a design document).

## Completion notes

