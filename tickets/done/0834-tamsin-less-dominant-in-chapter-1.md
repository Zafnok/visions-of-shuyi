---
id: "0834"
title: Tamsin is less dominant in Chapter 1
type: tuning
milestone: M7 Chapter 1 & game flow
model: opus-5.5
effort: medium
status: done
blocked_by: []
nick_input: decision
completed: 2026-10-15
---

# 0834 — Tamsin is less dominant in Chapter 1

## Context

Nick, Chapter 1 playtest round 2 (0804, 2026-10-14): "Tamsin in general
feels too OP for the first fight, constantly doubling on enemy and also
having highest mov".

Tamsin (`sergeant`, `assets/data/characters.ron`): Rider, **level 2**,
talent Spd, base `(21, 7, 0, 6, 7, 5, 1)` (HP, Str, Mag, Dex, Spd, Def,
Res), Iron Spear and Iron Sword, Leather Vest; Spear rank D. The Rider
class (`classes.ron`) has **Mov 7** (everyone else in the party 4–5), the
`lance_rush` active and the `charge_1` passive. These are 0803's *Claude's
starting values*, "tunable, judged in the playtest".

Follow-ups (`docs/design/stats-and-combat.md`): a unit strikes twice when its
attack speed beats the enemy's by 4 or more; attack speed is Spd plus
`as_bonus` (rank bonus minus the weight its Str doesn't carry;
`weapons-and-items.md`), and passives may add more. Red Company soldiers
have Spd 4 (Brigand, Raider), 5 (Archer, Rider), 2 (Guard) at level 1.

## Nick input

**Decision (tuning)**, with the `ask-nick` skill. First work out and show
Nick, per Chapter 1 enemy, whether Tamsin doubles it today and why (her
attack speed against theirs, with `charge_1` and rank bonuses counted).
Then offer, each with the same table after the change and the difficulty
sim's numbers:

- **A.** Spd 7 → 5 (doubles nobody at level 1 without help).
- **B.** Level 2 → 1 (one level's stats fewer).
- **C.** Mov 7 → 6 for Tamsin only (a personal penalty; riders keep 7).
  Fire Emblem's mounted units usually have the highest Mov, so say that.
- **D.** A heavier main weapon or armor that slows her (e.g. the Iron
  Spear's weight vs her Str).
- **E.** Combinations, and "describe your own".

## Scope

**In:**
- The table, the question and the answer, recorded in
  `docs/design/chapter-1.md` (*As built*) and Tamsin's sheet if her
  stats are written there.
- The data change in `characters.ron` (or `classes.ron` only if Nick
  wants every rider changed).
- The difficulty sim (`cargo xtask playtest ch01 --runs 200`; target 30–70%
  wins for the mindless bot, 0803) and the winning-replay tests; if the
  replay breaks, re-record it as `crates/ui/tests/it/ch01_winnable.rs`
  says.

**Out (do not do):**
- Other characters (say in the Completion notes if the lead or others
  double as often; Nick may want a separate ticket).
- Rule changes (the follow-up thresholds are ticket 0013's).

## Implementation steps

1. Compute the doubling table (a throwaway test or the forecast in a
   Harness) for today and each option.
2. Ask Nick; record.
3. Change the data; run the sim and the replay tests; write the numbers in
   the Completion notes.

## Acceptance criteria

- [x] `chapter-1.md` records the table, the choice and Nick's words.
- [x] `characters.ron` matches it.
- [x] Sim result inside 30–70% (or Nick accepted another number); replay
      tests pass.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Existing content tests and Chapter 1 replay tests.

## Completion notes

- **Why she doubled:** the Red Company is slow, not Tamsin fast. An Iron
  Axe weighs 6 and a Brigand's or Raider's Str carries 1 of it, so their
  attack speed is −1. Tamsin had 4 with the Iron Spear and 6 with the Iron
  Sword, and doubled 10 or 11 of the 14 enemies. Charge 1 adds might, not
  speed. The table is in `docs/design/chapter-1.md` (*Tamsin toned down*).
- **Options shown with the sim** (mindless bot, 200 tries, before 37%):
  A Spd 5 34%, B level 1 28%, C Mov 6 34% (simulated by giving every
  Rider Mov 6, the one enemy Rider too), D Chain Mail 38%, A+D 46%,
  A + no Iron Sword 30%. Recommended A+D.
- **Nick chose A + no Iron Sword:** `characters.ron` gives Tamsin Spd 5
  (was 7) and only the Iron Spear. She now doubles only the Guard. Mov 7,
  level 2 and her Sword rank E stay.
- **Sim after:** 30% won (60 of 200), the bottom edge of the 30–70% target.
  The mindless bot still loses her on turn 1–2 in most tries (it sends her
  in alone), so the sim says little about how strong she feels.
- **Replay:** the winning playthrough used her sword and broke; re-recorded
  (`cargo test -p trpg-ui --test it ch01_winnable::record -- --ignored`).
  The new flawless win takes 12 turns (was 8), nobody lost.
- **For Nick:** the lead (Iron Sword, attack speed 6) and Aske (Iron Bow,
  4) still double the same 10 enemies Tamsin used to. Not changed here
  (out of scope); say if you want a ticket for it.
- The title cinematic's footbridge combat-scene snapshot changed: the
  Brigand's hit on Tamsin is 69 (was 65), as 2 less Spd is 4 less avoid.
- No gameplay rules decided by Claude.
