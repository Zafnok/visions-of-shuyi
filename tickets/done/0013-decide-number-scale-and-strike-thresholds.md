---
id: "0013"
title: "Decide: number scale and strike thresholds"
type: design-decision
milestone: Design decisions
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: []
nick_input: decision
completed: 2026-10-22
---

# 0013 — Decide: number scale and strike thresholds

## Context

Ticket 0001 fixed the stat list and combat rules
(`docs/design/stats-and-combat.md`), including "up to 4 strikes, 3x/4x rare".
Nick deferred **how big the numbers are** until after he has played: "maybe I
wanna be crazy like dragon ball and we're talking trillions of points endgame
who knows... figure out the scaling algorithm once I have a playtest and see if
big numbers or low numbers or even medium numbers look nicer to me."

Chapter 1 ships with FE-sized placeholder numbers (stats ~0–50, HP ≤ 80,
strike thresholds `[4, 14, 24]`). This ticket asks Nick which scale he wants, and derives the
strike-threshold rule from it. It is decided **before Act 1 is
balanced**: a new scale changes every number the chapters' autobalance
passes tune, so 0848–0854, 1070, 1071 and 1072 wait for it (re-gated
2026-10-21, 0129).

## Nick input

**Decision.** Run with the `ask-nick` skill now: Nick has played Chapter 1
three times (0804, rounds 1–3). Show real numbers from the current build
next to mock-ups at other scales.

## Questions to ask

### Q1. How big should numbers get?

Show the same unit's info panel and one attack forecast at each scale (ASCII
mock-ups at the real panel width):

**A. Small — like Into the Breach / Advance Wars:** HP 3–10, damage 1–4.
**B. FE-sized (current) — like Fire Emblem:** stats 5–40, HP 20–60.
**C. Medium — like Final Fantasy Tactics / Disgaea early game:** HP in the
hundreds, damage 30–300.
**D. Huge — like Disgaea late game / Dragon Ball power levels:** grows to
millions or more; displayed with suffixes (`1.2M`, `3.4B`, `5T`).

Recommendation: decide from the playtest; mention panel width limits for C/D.

### Q2 (follows from Q1). When do 3 and 4 strikes happen?

Offer: fixed Speed gaps (current: +4 / +14 / +24), gaps as a percentage of the
enemy's Speed (e.g. 2x at 125%, 3x at 175%, 4x at 250%; scale-free), or a
ratio table. Show how often each would occur with the Chapter 1 roster.

### Q3 (follows from Q1). The stat ceilings: how high, and fixed or rising?

Ticket 0019 removed per-class stat caps (Nick: the max is "only determined by
true caps, including the level cap, as well as the growth rates"), and
deferred the ceiling numbers here (Nick: "3C"). Ask for the hard ceiling per
stat at the chosen scale, and whether it is **fixed** all game (like Final
Fantasy Tactics' 999 HP / 99 attack) or **rises with class tier** (each tier
lifts it). Also ask for the level cap if it is still undecided, since with
no class caps the level cap and the ceilings are the only brakes on growth.

## What to record

Update `docs/design/stats-and-combat.md`: Nick's words; the scale; stat
ranges/ceilings (fixed or per tier, Q3); the exact strike rule and thresholds; how large numbers are
displayed if C/D. Update `docs/design/README.md`. Write follow-up tickets for
rebalancing data (`classes.ron`, weapons, terrain) and, if needed, widening
`StatValue` (0302 made it a single alias) and number display in the UI.

## Acceptance criteria

- [x] Nick answered Q1, Q2 and Q3 after playing.
- [x] `stats-and-combat.md` updated with scale, ranges and strike rule.
- [x] Follow-up rebalance / display tickets created.
- [x] Ticket archived.

## Completion notes

Asked Nick on 2026-10-22 with the `ask-nick` skill. The pop-up showed the
real Chapter 1 forecast (Lead vs a Red Company brigand: 9 ×2, hit 96 / 9,
hit 65) at four scales, and the strike counts of all 36 Chapter 1 match-ups
under three rules. Answers (verbatim in `stats-and-combat.md`):

- **Q1 scale:** B, FE-sized, growing towards the low hundreds late. No
  rescale of any data; no `1.2M` display; `StatValue` is already `i32`.
- **Q2 strikes:** an extra strike needs a fixed gap or a share of the
  slower side's attack speed, whichever is bigger (gaps 4 / 14 / 24,
  shares 25 / 75 / 150 %); "probably tuning numbers more", so the numbers
  are tunable. Chapter 1 plays the same; at Lv 100 4x stays rare (2 of 36
  match-ups instead of 13).
- **Q3 ceilings:** fixed all game, HP 200 and every other stat 100.
- **Q4 (level cap, asked because 0019 left it open):** 99 through Act 1,
  maybe about 300 later, never 999. Nick also revised the level pace: about
  1 level per battle on average, about 2 per battle early, 1 every 2
  battles late (`progression.md`).

Found while asking: a pure percent-of-speed rule can't work with today's
weight rules (many enemies have attack speed 0 or below, so Chapter 1
would be full of 4x); the hybrid avoids it.

Follow-up tickets: **0323** the gap-or-share strike rule and the new
ceilings; **0608** the level pace curve. Both added to `blocked_by` of
0848–0854, 1070–1072 and 1064, and to the roadmap. No rebalance ticket for
classes, weapons or terrain and no display ticket: the scale didn't change.
The docs that said their numbers "rescale with 0013" (magic, weapons and
items, combat arts, progression) now say the scale was kept. With the HP
ceiling raised from 80 to 200, 0044's note that a bigger potion is "close
to the HP limit anyway" is weaker late in the game; the heal sizes are
tunable.

*the agent's starting rules* (tunable, Nick may veto): the share is taken of
the slower side's attack speed after weapon weight, counting 0 when it is
below 0; 0608's starting curve: full EXP to Lv 39, half to Lv 69, a
quarter after (about 110 battles to Lv 99).
