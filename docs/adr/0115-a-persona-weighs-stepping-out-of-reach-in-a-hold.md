# ADR-0115: A persona weighs stepping out of reach in a hold

- **Status:** Accepted
- **Date:** 2026-11-10
- **Related tickets:** 0529 (this decision), 1071 (found it), 0526
- **Amends:** ADR-0095 §2, ADR-0112 and ADR-0114 (the order)

## Context

After 1071 the army reaches Chapter 5's Envoy's House (Survive 8) two or
three levels stronger, having won the guards on the road. Hardcore,
which should play better than Normal, then succeeded in 13 tries of 100
to Normal's 85: someone fell in 41 of its tries (Normal 15), and it drank
all three Potions in 63 (its success allows one).

Traced (every Hardcore try of seeds 1–100 that lost someone; a death
counted as the persona's own scoring saw the unit when it ended its
turn): of 58 deaths, 35 were units the scoring already knew could fall
(`at_risk`: every enemy that can reach them striking and hitting fells
them), most of them wounded on the roofs (Aske, Joss) or in the street
(Maud); 19 were crits on units it didn't count at risk (Hedda at full HP
on the quay, turn 1, crit by the officer: a crit is luck to the forecast
a player reads too); 3 fell to the counter of their own attack; 1 other. A wounded unit in reach had no way out to
try: a survive has no goals (`Goals::of` is `None`), so the order offered
no unit an advance or a **hold** (ADR-0112: the tile out of every enemy's
reach nearest the goals), and ADR-0114 added covers for the lords only.
The waits came in the legal order, after the cap. What was left to clear
`at_risk` (−3000) was a Potion (−300 to Hardcore): it drank them, and
when they ran out the unit stayed and fell. Normal, which stays in the
house, needed the way out less.

## Decision

In a survive (an objective with no goals), the order (`trpg_bots::order`)
offers **each ready unit standing in an enemy's reach its hold**: the
unit waiting on the tile it can reach that no enemy can strike next phase
(the danger zone, as for the holds), the nearest where it stands (as the
crow flies; the first of two alike). It takes the holds' place in the
order, after the covers:

1. each unit's best attack;
2. the enemy AI's own pick;
3. the heals;
4. each ready unit's escape, on an escape map;
5. each ready unit's cover (in a survive);
6. each ready unit's advance;
7. each ready unit's hold — **in a survive, only for a unit in reach,
   nearest where it stands**;
8. each unit's best attack on each of its other targets;
9. the other ways of striking;
10. the rest;
11. `EndPhase`, within the cap.

A unit out of every reach has no hold to offer in a survive (it would
only wait where it is). Where the objective has goals, the holds are as
ADR-0112 has them. The scoring, the personas' settings and weights
(`personas.ron`) are unchanged: whether stepping back is worth more than
a blow is the persona's scoring.

## Consequences

- On the Envoy's House as 1071 left it (seeds 1–100): Hardcore 13 → 85
  of 100, its tries losing someone 41 → 11, its median items 3 → 0;
  Normal 85 → 83; Casual 77 → 85. Hardcore now plays the hold better
  than Normal, as the personas want, and the battle is retuned on it
  (0529, `docs/playtesting.md`, Chapter 5).
- Chapter 5 is the only story battle won by Survive (and `test_hold` the
  only test battle), so every other battle plays as before, command for
  command.
- One danger-zone check per wait the order already weighs: no new floods
  of the map. Hardcore's tries took 59 s instead of 81 (fewer Potions to
  weigh, fewer turns of wounded units).
- The crits stay: the scoring counts what a forecast shows if every
  strike hits, not a crit, like a player reading it. A careful persona
  can still lose a unit at full HP to one.

## Alternatives considered

- **Holds on the lords' distance** (the out-of-reach tile nearest a
  lord) — rejected by ADR-0114 for keeping the lord safe, and for a
  wounded unit on a roof the nearest tile out of reach is not the one
  nearest the lead, which may be across the enemy.
- **A hold for every ready unit** (in reach or not) — a unit out of reach
  would offer its own tile, a wait that changes nothing, in a place the
  cap counts.
- **Counting crits in `at_risk`** — a crit triples a blow, so nearly every
  unit in reach would be at risk and no persona would stand anywhere; a
  player reading the forecast weighs the hit, not the crit.
- **Lower Hardcore's `items` cost** so it drinks and keeps its units —
  the personas' weights are fixed (ADR-0095 §5), and the Potions were
  spent because nothing else was offered.
