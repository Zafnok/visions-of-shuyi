# ADR-0112: A persona weighs every unit's move before any unit's attack on a second target

- **Status:** Accepted
- **Date:** 2026-11-03
- **Related tickets:** 0523 (this decision), 0520 (found the stall), 0506
- **Amends:** ADR-0095 §2 (the order)

## Context

On Chapter 2 (Kell's Ford) Hardcore's clean wins had a tail to turns 12
and 13 that Normal's didn't, so "Hardcore is faster than Normal" failed
on seeds 101–200. Seed 116, traced: from turn 4 to turn 10 the persona
ended its turn with nobody moving, the army just out of reach of Silas
Hendry and his guards at the ford's north end.

0520 read it as scoring: every step towards the ford is `at_risk`
(−3000) while ending the turn costs nothing. The trace showed otherwise.
At the stall, every candidate the persona weighed scored below ending the
turn (−150 to −5170), but **Hollis stepping into the ford scored +122**
(not at risk, his counter worth more than his danger) and Rue's advance
+12. Neither was weighed: they were places 12 and 16 of the order, and
Hardcore tries 12 commands, the 12th being `EndPhase`. Places 1 to 9 were
the attacks of the three units who could reach the ford's mouth, each on
each of its three targets (Hendry and two guards), all blows they
couldn't survive; then Maud's potion and the lord's advance. ADR-0095's
order lists "each unit's best attack on each target" first, so a unit
with three targets in reach takes three places before any unit moves.

The ticket named three changes to the scoring; they were compared with a
fourth, the order, on the same tries (Hardcore on seeds 101–200, with the
army Chapter 1 leaves; the median turns of the wins with nobody dying):

| Change | Hardcore succeeds | Its wins by turn (3 / 4 / 5 / 6 / 7 / 8+) | Median | Normal's median | Faster | Guarded-ford test |
| ------ | ----------------- | ----------------------------------------- | ------ | --------------- | ------ | ----------------- |
| None (`main`) | 57 | 13 / 10 / 10 / 13 / 6 / 11 | 5 | 4 | ✗ | fails |
| 1. `turns` counted when a plan ends its turn | 61 | 13 / 15 / 13 / 11 / 6 / 6 | 5 | 4 | ✗ | fails |
| 2. `turns` for a turn passed with nothing done | 62 | 13 / 10 / 15 / 15 / 6 / 5 | 5 | 4 | ✗ | fails |
| 3. `approach` by the safest tile on the way | not run | | | | | |
| 4. Each unit's best target first, its other targets after every unit's move | 60 | 11 / 9 / 21 / 10 / 7 / 7 | 5 | 6 | ✓ | passes |

Changes 1 and 2 cost Hardcore 200 points (its `turns`) for ending a turn;
Normal and Casual weigh `turns` at 0, so they don't change. They shorten
the tail by a side effect: a unit waiting where it stands now beats
ending the turn, and once it has waited its attacks leave the list and
Hollis's advance gets in. That is a fault fixed by accident, for one
persona. Change 3 was not run: at the stall every unit already stood on
the safest tile nearest the goal (each unit's hold was waiting where it
stood, worth 0 in the trace), so it changes no score there; and the move
that wins was cut by the cap, whatever its score.

## Decision

The order (`trpg_bots::order`, ADR-0095 §2) lists **each unit's best
attack** (its best target's best way of striking, from its best tile),
the best first, where it listed each unit's best attack on each target.
Each unit's best attack on **each of its other targets** comes after
every unit's advance and hold, before the other ways of striking. The
rest of the order is unchanged:

1. each unit's best attack;
2. the enemy AI's own pick;
3. the heals;
4. each ready unit's escape, on an escape map;
5. each ready unit's advance;
6. each ready unit's hold;
7. each unit's best attack on each of its other targets;
8. the other ways of striking;
9. the rest;
10. `EndPhase`, within the cap.

A plan still holds a unit's attack on its second target: from the next
step on, the order is the state's own, and a unit's best target changes
as blows land.

The scoring is unchanged, and so are the personas' settings and weights
(`personas.ron`).

## Consequences

- On Chapter 2, Hardcore is faster than Normal on both seed ranges (5
  against 6, and 5 against 6.5), and no try passes a turn with nobody
  moving: its longest win is now turn 8, every turn of it fought.
- It changes every persona, so the bands move. Casual plays Chapter 2
  much better (47 and 50 out of 100 before, 75 and 64 after, over its
  30–55% band), as often as Normal wins it with nobody dying. Two runs of
  the autobalance (100 and 60 evaluations) found no enemy setting that
  puts every persona in its band, so Chapter 2 stays as 0520 tuned it and
  ticket 0524 re-tunes it (with Chapter 1, whose army it inherits).
- Chapter 1 doesn't move: Hardcore succeeds in 145 of 200 tries before and
  146 after (seeds 1–200), and is faster than Normal both times. It sits at
  the top of its 50–75% band either way (77 on seeds 101–200 before, 76 on
  1–100 after), which is Chapter 1's tuning, not this change.
- A unit's attack on a second target is weighed later than before. With
  Hardcore's 12 places and six units, the second targets of the first
  units still fit when few units can move; with many units they may not,
  as the advances did before. A move the cap cuts is never weighed,
  whatever it would score: the order is where the persona's judgment
  starts.

## Alternatives considered

- **Count `turns` when a plan ends its turn** (the ticket's first
  option) — Hardcore alone, and only by a side effect (above); still
  slower than Normal on seeds 101–200, and the guarded-ford test still
  stalls.
- **A cost for a turn passed with nothing done** (the second) — the same.
- **Score `approach` by the safest tile on the way** (the third) — at the
  stall every unit already stood on it; it changes nothing the trace
  shows.
- **A wider cap** — the cap is the personas' thinking budget, fixed in
  `personas.ron` (ADR-0095 §5); a bigger one would only move the edge.
- **Each unit's advance right after its own attacks** — the three units
  with three targets would still hold nine of the first places before the
  sturdier unit's advance.
