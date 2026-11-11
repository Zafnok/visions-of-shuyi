---
id: "0529"
title: Chapters 4 and 5 back in their bands after the guard fights
type: tuning
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: ["1071"]
nick_input: none
completed: 2026-11-10
---

# 0529 — Chapters 4 and 5 back in their bands after the guard fights

## Context

1071 put two fixed skirmishes that guard the road on the Act 1 map:
*Aldwater Toll Bridge* before Veyne Hall (Chapter 4) and *The Salt Road*
before the Envoy's House (Chapter 5). Every player wins them on the way,
so the bots' arrival army (`crates/bots/src/arrival.rs`) and the chapter
tests (`ch04_winnable`, `ch05_winnable`) now win them first, and Chapters
4 and 5 are fought with a stronger army. Their recorded playthroughs were
recorded again with it.

The bots' playtests after 1071 (100 tries, seeds 1–100; Casual / Normal /
Hardcore success):

| Battle | Before 1071 | After 1071 | Bands |
| --- | --- | --- | --- |
| ch04 | 52 / 61 / 63 | 59 / 58 / 51 | 30–55 / 50–75 / 50–75 |
| ch05 | 58 / 58 / 69 | 77 / 85 / 13 | 40–70 / 50–75 / 50–75 |

Chapter 4's Casual is 4 points over its band. Chapter 5 is out on all
three. It is no harder: every bot wins it 98% of the time. But Hardcore
loses someone in 41 tries of 100, where Normal does in 15, and uses all
three of its items (its success allows one), so it succeeds in 13. The
army: Lead lv 11, Hedda 8, Hollis 11, Tamsin 11, Aske 10, Maud 8, Rue 16,
Joss 5.

## Nick input

None. Verified by the bots' reports in the PR; what is left to judge by
playing goes on the Act 1 playtest's checklist (1064).

## Scope

**In:**
- Find why Hardcore does worse than Normal in Chapter 5 (deaths, and the
  items), as 0526 did for the door: a bot fault is fixed in the bots,
  never by changing the bands.
- Bring Chapters 4 and 5 back inside their bands, or as near as the
  levers allow, by the battles' own numbers (`cargo shuyi autobalance
  --stage ch05`, or by hand, as `docs/playtesting.md` records for 0851
  and 0526), or by the guards' (fewer or lower Hounds give less EXP; keep
  the guards in their own bands).
- Record again whatever playthroughs the change breaks, in order:
  `ch04_winnable::record`, `ch05_winnable::record_salt_road`,
  `ch05_winnable::record`.
- The new numbers in `docs/playtesting.md`.

**Out (do not do):**
- Changing the bands or the success rules (`targets.ron`).
- New rules or new map features.

## Implementation steps

1. Read `docs/playtesting.md` (Chapters 4 and 5, 0526), 1071's Completion
   notes and `crates/bots/src/arrival.rs`.
2. Play `target/release/xtask playtest ch05 --bot hardcore` and read the
   tries that lose someone: who falls, to what, on which turn, and when
   the items go.
3. Fix a bot fault if there is one (with a test in `crates/bots`), then
   tune the numbers.
4. Record the playthroughs again as needed; run the playtests of
   aldwater, ch04, salt_road and ch05.

## Acceptance criteria

- [x] Chapter 5's three bots inside their bands, and Chapter 4's (or the
      reason a band can't be met, in the Completion notes).
- [x] The guards still inside theirs.
- [x] `ch04_winnable`, `ch05_winnable` and the bots' tests pass.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- The bots' playtest reports (before and after) in the PR.
- A test in `crates/bots` for any bot fault fixed.

## Completion notes

**A bot fault, fixed in the bots** (`crates/bots/src/persona.rs`,
ADR-0115). Traced every Hardcore try of Chapter 5 (seeds 1–100) that lost
someone, with the persona's own scoring at the end of each of its turns:
of 58 deaths, 35 were units it already knew could fall (`at_risk`),
mostly wounded on the roofs (Aske, Joss); 19 were crits on units it
didn't count at risk (Hedda at full HP on the quay, turn 1); 3 fell to
the counter of their own attack; 1 other. A survive has no goals, so the
order offered no unit a **hold** (a step out of every enemy's reach), and
0526's covers guard the lords only: the only way to clear a unit's risk
was a Potion, so Hardcore drank all three (63 tries of 100) and then
left the unit to fall. Now, in a survive, each ready unit in an enemy's
reach is offered its hold, the safe tile nearest where it stands.
Chapter 5 alone, before any tuning: Hardcore 13 → 85, its tries losing
someone 41 → 11, its median items 3 → 0; Normal 85 → 83; Casual 77 → 85.
Tests: `a_wounded_unit_steps_out_of_reach_in_a_hold` (its order check
fails without the fix) and
`a_hold_in_a_survive_is_only_for_a_unit_in_reach_and_nearest_where_it_stands`.
Only Chapter 5 and `test_hold` are survives, so every other battle plays
as before.

**The tuning** (all numbers in `docs/playtesting.md`, *After 1071 and
0529*):

- **Chapter 4** (`assets/battles/ch04.ron`): every unnamed Hound lv 7 →
  9. The autobalance on 200 tries found +4 (98 / 102 / 103 of 200, Normal
  and Hardcore on their floor); +2 is enough and gives Chapter 5's army
  less EXP.
- **Chapter 5** (`assets/battles/ch05.ron`): the Hounds by the water
  (quay, docks, boats) lv 13, the ones over the roofs lv 9, and **Survive
  10 instead of 8**. Uniform levels couldn't fit Normal and Hardcore into
  one band (Hardcore now leads by ~25 points); they lose different units
  (Normal its lightest on the roofs), so the roofs went down and the
  water up; Casual moved only with the length of the hold.
- The guards weren't changed.

| Battle | Bands | 1071 (seeds 1–100) | Now, seeds 1–100 | Now, seeds 101–200 |
| --- | --- | --- | --- | --- |
| Aldwater | 70–100 / 70–95 / 70–95 | 100 / 89 / 90 | **100 / 89 / 90** ✓ | — |
| ch04 | 30–55 / 50–75 / 50–75 | 59 ⚠ / 58 / 51 | **52 / 50 / 51** ✓ | 46 / 53 / 59 ✓ |
| Salt Road | 70–100 / 70–95 / 70–95 | 99 / 82 / 89 | **98 / 94 / 93** ✓ | — |
| ch05 | 40–70 / 50–75 / 50–75 | 77 ⚠ / 85 ⚠ / 13 ✗ | **62 / 52 / 71** ✓ | 65 / 63 / 77 ⚠ |

Chapter 5's Hardcore is 2 over on seeds 101–200; over the 200 tries the
three are 63.5 / 57.5 / 74%, in every band. One setting swings 6–13
points between the two ranges here, and Normal is near its floor on
seeds 1–100, so I stopped. Chapter 4's Normal is on its floor (50) on
seeds 1–100.

**Playthroughs recorded again**, in order: `ch04_winnable::record` (won
on turn 5, nobody lost), `ch05_winnable::record_salt_road` (turn 2),
`ch05_winnable::record` (held to turn 10). `ch05_winnable` now asserts
turn 10. Chapter 4's new playthrough leaves a different army at Chapter
5 (Aske lv 7 instead of 10, Maud 6 instead of 8): the final numbers are
with it.

**The agent's starting values** (for 1064's checklist):

- Chapter 5 is held for **10 turns** (was 8): the battle file's "until
  first light" is the only text about it.
- Chapter 5's Hounds by the water lv 13, over the roofs lv 9; the
  officer stays lv 9 (lv 12 changed nothing).
- Chapter 4's Hounds lv 9 (were 7).

**To judge by playing (1064):** whether ten turns of the Envoy's House
drag; whether the roof Hounds feel too soft next to the quay's.

**Follow-ups:** none.

**Sizes.** Engine friction: estimate 20 production lines (a hold for the
survive beside 0526's covers in `order`; the estimate was written after
the trace, not before the first line), measured 15 added, 6 removed:
under 2×. PR size: estimate 900 (two recorded playthroughs, the ADR,
the docs), measured about 920.
