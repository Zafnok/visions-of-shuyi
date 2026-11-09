---
id: "0526"
title: The Casual bot wins the Envoy's House too often, and no bot holds its door
type: bug
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: ["0851"]
nick_input: none
completed: 2026-11-08
---

# 0526 — The Casual bot wins the Envoy's House too often, and no bot holds its door

## Context

Chapter 5's story battle (the Envoy's House, 0851) is the first hold: a
Survive of 6 turns in a narrow house with one street door onto the quay
and a roof hatch. Nick gave holds their own Casual band, 40–70% on a
Normal-tier map, and asked for the hold to be made harder to meet it
(`docs/design/playtest-bots.md`, *Holds have their own Casual band*).
0851 balanced Normal and Hardcore into their bands on seeds 1–100 and
101–200 (Normal 65 and 70, Hardcore 62 and 57) but Casual stays at 86
and 80: too easy (`docs/playtesting.md`, Chapter 5).

What 0851 saw, without tracing it:

- The Casual bot guards only the lead, who waits the hold out indoors,
  so it loses only when the lead falls. More Hounds, more turns, a
  second opening (unbarred shutters), archers on the quay: each one got
  companions killed (Normal down to 7–42) and left Casual at 78–91.
- The only thing that made the lead fall was the officer reaching the
  street door on turn 1 (starting on the near pier): at Str 12 and Spd
  11 the lead fell on turn 1 in 94–97 tries of each bot, at Str 11 and
  Spd 10 in about 11–15. **No bot stood a unit on the door tile (13, 8)
  to stop him**, though a unit there takes him alone. Hardcore, with the
  biggest thinking budget, didn't either. So the officer's strength
  moves all three bots together and can't tell a careless player from a
  careful one, which is what the Casual band is for.

A bot that held a chokepoint against a threat it can see in reach
(0523's kind of fault: the moves worth making missing from what it
tries) would let the officer be dangerous to a careless lead only.

## Nick input

None (a bots fault; the band is Nick's).

## Scope

**In:**
- Trace the three bots on the Envoy's House with the officer in turn-1
  reach (`assets/battles/ch05.ron`, the officer at (13, 13); `cargo
  xtask playtest ch05`, a few seeds command by command, as 0523 did) and
  find why none blocks the door.
- Fix it in the bots (`crates/bots`), with a test that fails without the
  fix, if the cause is a bot's; or say why it can't be.
- Then retune the Envoy's House so Casual lands in 40–70% with Normal
  and Hardcore still in their bands, both seed ranges; record a new
  winning playthrough if it changes (`ch05_winnable`).
- Chapters 1 to 4 still in their bands after it.

**Out (do not do):**
- Changing the bands (Nick's).

## Acceptance criteria

- [ ] `cargo xtask playtest ch05 --bot all` meets every band on seeds
      1–100 and 101–200; Chapters 1 to 4 likewise. *Chapter 5 ✓ on
      both; Chapters 1 and 2 ✓; Chapters 3 and 4 miss on `main` already
      (after 0846) and play the same here: noted in 0870, which retunes them.*
- [x] A test of the fix fails without it.
- [x] All gates in the `run-gates` skill pass.

## Completion notes

**Done, but Chapters 3 and 4 were already out of band on `main`** (0870 retunes them).
No bot stood in the Envoy's House's door because a survive gives the
order nothing to move with; the order now offers each ready unit its
cover (ADR-0114, amending ADR-0095 §2 and ADR-0112), and the house is
retuned on it.

- **The trace** (a throwaway test calling the persona on the arrival
  army, Casual seed 1, the officer at (13, 13)): every place Casual tried
  on turn 1 was an attack (Rue's Fire and Frost from the quay, the lead's
  and Hedda's and Hollis's strikes) or `EndPhase`. A survive has no goals
  (`Goals::of` is `None`), so no unit had an advance or a hold, and the
  waits came only in the legal order, after the cap of 10. It ended its
  turn (+530 against −4270 for the lead striking out of the door); the
  officer walked onto the door tile and killed the lead. The scoring
  already weighs a lord in reach (−5000); the door was never weighed.
- **The fix** (`crates/bots/src/persona.rs`, `covers`): in a survive,
  while an enemy that moves can reach a lord, each ready unit's wait on
  the tile that keeps the most of those enemies off the lords (their
  `threat_area` worked out again with the unit standing there), the
  nearer the lord on a tie (the doorway, not the quay in front of it),
  listed after the escapes. With it Casual's seed 1 stands Hollis in the
  way and wins.
- **The test** `a_persona_stands_in_the_door_between_the_lord_and_the_enemy`
  (`persona/tests.rs`): a room with one door, a 1-HP lord, an unarmed
  guard who can't be hurt, and a swordsman who fells anything and can
  reach every tile the lord can. Every persona holds the turn; the
  guard's wait just inside the door is the first wait in the order (it
  failed without the fix: the lord's own wait came first); in a rout,
  no cover.
- **Covers only in a survive.** Offered on every objective first, they
  cost Normal 13 and 18 clean wins of 100 on Kell's Ford (63 → 50,
  62 → 44) and Hardcore 8 on seeds 1–100 (55 → 47): listed before the
  advances, they took the units' moves' places. Chapter 5 is the only
  story battle won by Survive, so Chapters 1 to 4 play as on `main`,
  command for command (Chapter 2 seeds 1–100: 45 / 63 / 55 both).
- **The retune** (`assets/battles/ch05.ron`, `assets/data/characters.ron`):
  the officer on the near pier (13, 13) instead of its far end, at the
  door on turn 1; his Str 15 instead of 13; Survive 8 instead of 6. The
  bots now separate the way a careless player does: Casual's guard in
  the door (Hedda, mostly) falls to the officer on turns 2–3, its army
  thins out (three or more retreats in 87 tries of 100), and the lead
  falls on turns 4–8.

| Persona | Band | `main` (1–100, 101–200) | Final, 1–100 | Final, 101–200 |
| ------- | ---- | ----------------------- | ------------ | -------------- |
| Casual | 40–70% (hold) | 86 ⚠, 80 ⚠ | **58 ✓** | **63 ✓** |
| Normal | 50–75% | 65, 70 | **58 ✓** | **62 ✓** |
| Hardcore | 50–75% | 62, 57 | **69 ✓** | **68 ✓** |

  The steps and what was tried and not kept are in `docs/playtesting.md`
  (Chapter 5): officer stats alone took Casual to 71–80, Survive 7 to
  63–72, Survive 8 to 58–63.
- **New winning playthrough** `crates/ui/tests/it/fixtures/ch05_winning.ron`
  (recorded with `ch05_winnable::record`): held to the end of turn 8,
  nobody lost; `ch05_winnable` now asserts turn 8.
- **Chapters 1 to 4** (100 tries a range; the same as `main`):

| Battle, seeds | Casual | Normal | Hardcore | Faster |
| ------------- | ------ | ------ | -------- | ------ |
| ch01, 1–100 | 53 ✓ | 60 ✓ | 68 ✓ | ✓ |
| ch01, 101–200 | 45 ✓ | 66 ✓ | 71 ✓ | ✓ |
| ch02, 1–100 | 45 ✓ | 63 ✓ | 55 ✓ | ✗ (5 vs 5) |
| ch02, 101–200 | 45 ✓ | 62 ✓ | 64 ✓ | ✓ |
| ch03, 1–100 | 60 ⚠ | 81 ⚠ | 55 ✓ | ✓ |
| ch03, 101–200 | 64 ⚠ | 74 ✓ | 61 ✓ | ✓ |
| ch04, 1–100 | 39 ✓ | 44 ✗ | 60 ✓ | ✗ (0525) |
| ch04, 101–200 | 48 ✓ | 46 ✗ | 64 ✓ | ✗ (0525) |

  0846 (the lead Str 7 → 8) checked only Chapter 1, and every later
  army comes from it; Chapters 3 and 4 were tuned before it. 0870 (Joss
  a little weaker, from Nick's playtest) already has to put Chapters 3
  and 4 in their bands; this table is now in its Context.

**Deviations:** the acceptance box for Chapters 1 to 4 is not met, for
the reason above (not this ticket's change; out of scope to retune
here). The ticket's officer position (13, 13) was kept as the retune's
start; the two other levers (Str, the turn count) are the agent's.

**Follow-up tickets:** none new: Chapters 3 and 4's misses went into
0870's Context, which already retunes them.

**The agent's starting values** (tunable, for the Act 1 playtest, 1064):

1. **The hold is 8 turns**, not 6 ("until first light"; no line names
   the count).
2. **The Hound officer starts at the near end of the wide pier (13, 13)**
   and reaches the street door on turn 1; **Str 15** (40 HP, Spd 11, Def
   8 as before). A unit in the door keeps him off the lead.

**Left to judge by playing (1064):** whether 8 turns feels long for the
first hold; whether the officer at the door on turn 1 reads as "the boss
comes with the docks" (0851 had him there on turn 2); whether a player
sees that a unit in the door stops him.

Engine friction: estimate 45, measured 81 (1.8×), under 2×.
