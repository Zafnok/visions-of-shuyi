# ADR-0114: A persona weighs standing between a lord and the enemy that can reach it

- **Status:** Accepted; amended by ADR-0115
- **Date:** 2026-11-08
- **Related tickets:** 0526 (this decision), 0851 (found it), 0523
- **Amends:** ADR-0095 §2 and ADR-0112 (the order)

## Context

Chapter 5's Envoy's House (0851) is the first hold: Survive 6 turns, the
lead in the street room behind the one street door (13, 8). With the
Hound officer able to reach the door on turn 1, the lead fell on turn 1
in 94–97 tries of every persona, Hardcore's too, though a unit on the
door tile keeps him off alone. So the officer's strength moved all three
bots together, and Casual (86 and 80 out of 100 with him a turn away)
couldn't be brought into the hold band without sinking Normal.

Traced (Casual, seed 1, the officer at (13, 13)): every place the
persona tried on turn 1 was an attack (Rue's spells from the quay, the
lead's strikes, Hedda's and Hollis's) or `EndPhase`. In a survive the
objective has no goals (`Goals::of` is `None`), so no unit had an
advance or a hold; the waits came only in the legal order, after the
cap. Ending the turn scored +530 against the lead's −4270 for striking
out of the door, and the officer walked onto the door tile and killed
the lead. The scoring already counts a lord in reach (`lord_at_risk`,
−5000): a unit on the door was never weighed, whatever it would score.

## Decision

In a survive, the order (`trpg_bots::order`) gains **each ready unit's
cover**, after the escapes and before the advances, while an enemy that
moves can reach a lord: the unit waiting on the tile it can reach that keeps the most of
those enemies off the lords, their reach worked out again with the unit
standing there (the engine's `threat_area` with the units moved); of two
alike, the tile nearer the lord (the doorway, not the quay in front of
it, where more can strike the unit). Only tiles one of those enemies
could reach are tried. A lord is never its own cover, and an enemy that
never moves can't be kept off. The covers come the most first:

1. each unit's best attack;
2. the enemy AI's own pick;
3. the heals;
4. each ready unit's escape, on an escape map;
5. **each ready unit's cover** (in a survive);
6. each ready unit's advance;
7. each ready unit's hold;
8. each unit's best attack on each of its other targets;
9. the other ways of striking;
10. the rest;
11. `EndPhase`, within the cap.

The scoring, the personas' settings and their weights (`personas.ron`)
are unchanged: whether a cover is worth the unit standing in it is the
persona's scoring (a careful persona's `at_risk` is ten times Casual's).

Only in a survive: where the objective has goals, the units' moves are
their advances and holds, and covers listed before them took their
places (below).

## Consequences

- With the officer on the near pier, the lead no longer falls on turn 1:
  every persona stands a unit in the door. The officer's strength now
  tells a careful player from a careless one: Casual's guard in the door
  falls to him and its army thins out, Normal's and Hardcore's don't.
  The Envoy's House is retuned on it (officer on the near pier, Str 15,
  Survive 8): Casual 58 and 63 of 100, Normal 58 and 62, Hardcore 69 and
  68, every band met (0526, `docs/playtesting.md`).
- Chapter 5 is the only story battle won by Survive, so Chapters 1 to 4
  play as before, command for command (Chapter 2 on seeds 1–100: 45, 63
  and 55 before and after).
- A cover is worked out only while a lord is in an enemy's reach, one
  `threat_area` per enemy reaching a lord for each tile a ready unit can
  wait on in that enemy's reach: a few dozen floods of the map, at each
  step of the plan. Casual's tries on the Envoy's House took 0.7 s.
- It guards the lords only. A unit other than a lord in reach is the
  scoring's (`at_risk`), and a survive still has no holds for its other
  units (no goals to measure "nearest" by).

## Alternatives considered

- **Covers on every objective** — measured on Chapters 1 to 4 (100 tries
  a range): Kell's Ford (a rout with houses) lost 13 and 18 of Normal's
  clean wins (63 → 50, 62 → 44) and 8 of Hardcore's on seeds 1–100
  (55 → 47); the covers, listed before the advances, filled the places
  the units' moves had held.

- **Holds in a survive** (each ready unit's tile out of every enemy's
  reach, with no goal to be nearest to) — the lead in the street room has
  no tile out of reach once the roofs' Hounds come down the hatch, and
  it doesn't stop the officer reaching the others; the door does.
- **Goals in a survive** (the lords' tiles, so advances walk towards
  them) — a unit walking to the lead stands beside it, not in the door.
- **Every unit at risk, not only the lords** — the floods grow with every
  unit in reach, at every step of the plan, for what the scoring's
  `at_risk` already weighs; the lord's fall is the one that loses.
- **A wider cap** — the cap is the personas' thinking budget, fixed in
  `personas.ron` (ADR-0095 §5).
