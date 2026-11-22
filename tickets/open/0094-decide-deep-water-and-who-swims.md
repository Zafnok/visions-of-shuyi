---
id: "0094"
title: "Decide: deep water in rivers, and who can swim"
type: design-decision
milestone: Design decisions
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 0094 — Decide: deep water in rivers, and who can swim

## Context

Nick, after watching a new player play Chapter 1 (2026-11-22):

> "We should probably have both shallow and deep water. Shallow crossable
> by what can cross it now w same movement penalties. And deep only by
> flying or swimming units."

What exists (`docs/design/terrain.md`, the movement table near line 98;
`assets/data/terrain.ron`; `crates/core/src/terrain.rs`):

- `water` is already "river, shallow": foot 5, mounted and armored can't
  enter, flying 1, +10 avoid.
- `sea` is flying only. So deep water already exists as a rule, but only
  as open sea, never inside a river.
- Movement types are foot, mounted, armored and flying. There is **no
  swimming** movement type, and no Act 1 class flies (the only flier is
  the tier 3 `flier`, `assets/data/classes.ron`).
- Ice (0310) turns water or sea into ice for the rest of the battle.

So Nick's first half is decided as it stands (shallow keeps today's
costs). What is open is what deep river water is, who swims, and where it
goes. Not on the Act 1 critical path unless Nick says so: it changes
maps, and Chapter 1's river (`ch01.map`, Harrowby Ford, two river rows
crossed by bridges) was balanced without it.

## Nick input

**Decision** (`ask-nick`, with map mockups of a river showing a shallow
ford and deep stretches, in both the picture look and the glyph look).
Questions, each with game examples (check each before citing):

1. **Deep river water: a new tile, or `sea` used in rivers?** A. a new
   `deep water` tile with its own look (darker, no ripples showing the
   bottom), same rule as sea. B. reuse `sea` inside rivers. C. describe
   your own.
2. **Who swims?** A. no one in Act 1; deep water is a wall for the
   player's army until a flier joins (Fire Emblem: only fliers cross
   deep water, pirates and berserkers cross sea in the GBA games).
   B. a class or skill that swims (Fire Emblem's pirates walking on sea;
   Advance Wars' naval units). Which one, and its cost in deep and
   shallow water. C. enemies only (a pirate or beast band), D. your own.
3. **Does ice still freeze deep water?** (0310 freezes water and sea
   today: walking across a frozen deep river may be a feature.)
4. **Where it goes:** which Act 1 maps get deep stretches (Chapter 1's
   river, the ford skirmish `act1_skirmish_ford`, Chapters 2–5), or only
   new maps after Act 1.

Answers go to `docs/design/terrain.md` (the movement table and the tile
list).

## Scope

**In:** the questions above, the mockups, the answers in
`docs/design/terrain.md`, updating 0336's steps to match.

**Out (do not do):** any code or map change (0336).

## Implementation steps

1. Stage a small river map (throwaway `zz_` battle file, see the memory
   on staging fights) and render it with `cargo shuyi frame-png` in both
   looks.
2. Ask Nick with `ask-nick`.
3. Write the answers into `docs/design/terrain.md`; adjust 0336.

## Acceptance criteria

- [ ] `docs/design/terrain.md` says what deep water is, who crosses it at
      what cost, what ice does to it, and which maps use it.
- [ ] 0336's steps match the answer.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- None (a design document).

## Completion notes

