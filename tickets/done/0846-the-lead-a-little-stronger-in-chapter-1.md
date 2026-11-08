---
id: "0846"
title: "The lead a little stronger in Chapter 1"
type: tuning
milestone: M7 Chapter 1 & game flow
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: ["0845"]
nick_input: decision
completed: 2026-11-08
---

# 0846 — The lead a little stronger in Chapter 1

## Context

Chapter 1 playtest round 3 (0804, 2026-10-16):

> "lead feels a bit weak actually, usually lead is a bit strong."

Same round: Tamsin "Feels right now" (after 0834), the lead and Aske
doubling 10 of 14 enemies is "Fine", and the difficulty is "Right".

The lead today (stat order HP, Str, Mag, Dex, Spd, Def, Res):

| | Values |
| --- | --- |
| `lead` (`assets/data/characters.ron:38-45`) | Exile Lv1, Mov 5; 19 / 7 / 1 / 7 / 7 / 4 / 3; talent Str (+20% growth) |
| Exile growths (`classes.ron:102-108`) | 75 / 45 / 15 / 50 / 55 / 30 / 30 |
| Kit | Iron Sword (Mt 5, Hit 90, Wt 2), Steel Sword (Mt 8, Hit 75, Wt 4), Leather Vest (+1 Def); Sword D |

Against the commonest enemy, a Brigand (20 / 6 / 0 / 2 / 4 / 3 / 0, Iron
Axe Mt 8): the lead's Iron Sword does 7 + 5 − 3 = **9**, twice (the lead
attacks twice), so **18 of 20 HP**: the lead never kills a fresh Brigand in one
fight, and takes 6 + 8 − 5 = 9 per hit back.

Because the objective changes to defeating Harl (0845), the balance
moves anyway; this ticket comes after it so the bots measure the real
chapter.

**Waits for the engine carve and the Playdate port** (Nick, 2026-10-16),
so `blocked_by` has 1118 and 1050.

**Re-gated 2026-10-21 (0129):** the carve is done and 1050 was folded
into the Act 1 playtest (1064), so neither 1118 nor 1050 is in
`blocked_by` any more.

## Nick input

**Decision** (`ask-nick`), as 0834 did: each option with its bot win rate
(`cargo xtask playtest ch01`, 200 tries, same seeds), what it changes in
fights (who the lead kills in one fight), and Fire Emblem references
(e.g. a lord with a personal weapon: Lyn's Mani Katti, Eirika's Rapier;
check before citing). Options, *the agent's proposals*:

- A. **Str 7 → 8:** Iron Sword does 10 × 2 = 20, killing a fresh Brigand
  in one fight.
- B. **HP 19 → 22, Def 4 → 5:** sturdier, same damage.
- C. **A personal sword** in place of the Iron Sword (Mt 7, Hit 90,
  Wt 2; anything more is Nick's call), the Steel Sword kept.
- D. A + B.
- E. Describe your own.

Keep the bot win rate inside the 30–70% target band
(`docs/design/playtest-bots.md`); if an option leaves it, say so.

## Scope

**In:** the bot runs; the question; changing `characters.ron` (or
`items.ron` for C); `chapter-1.md` records the change as Nick's.

**Out (do not do):** other units; enemies.

## Acceptance criteria

- [x] Nick picked; recorded with the bot numbers.
- [x] Data changed; content validation and the bot run in the completion
      notes.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Content validation; existing combat tests updated only where they pin
  the lead's stats (each reviewed).

## Completion notes

**Nick chose A: Str 7 → 8.** `characters.ron` gives the lead Str 8; nothing
else changed. The table and the reasons are in `docs/design/chapter-1.md`
(*The lead a little stronger*).

- **The ticket's sums were short:** a sword's second strike does ×1.2
  (`weapons-and-items.md`) and the Leather Vest gives +2 Def, not +1. With
  them, Str 7's Iron Sword did 9 + 10 = 19 to a fresh Brigand (20 HP), and
  a Brigand hit the lead for 8, not 9. Str 8 does 10 + 12 = 22: a kill.
  The Steel Sword already killed a fresh Brigand or Raider in one fight
  (at 81% to hit); Nick saw this before choosing.
- **Bots** (`cargo xtask playtest ch01 --bot all --runs 100`, seeds
  1–100). The ticket asked for 200 tries; one run of 100 took about 42
  minutes here, so each option ran 100 (±5 is noise):

  | | Casual (30–55%) | Normal (50–75%) | Hardcore (50–75%) |
  | --- | --- | --- | --- |
  | Before | 47 ✓ | 57 ✓ | 71 ✓ |
  | **A Str 8 (chosen)** | **53 ✓** | **60 ✓** | **68 ✓** |
  | B HP 22, Def 5 | 53 ✓ | 67 ✓ | 84 ⚠ too easy |
  | C personal sword Mt 7 (simulated as `exile_sword`, lead only) | 48 ✓ | 56 ✓ | 72 ✓ |
  | D A + B | 63 ⚠ too easy | 70 ✓ | 73 ✓ |

  With A, Hardcore still finishes faster than Normal (turns median 7 vs 8).
- **Fire Emblem references shown:** Eirika's Rapier (Mt 7, Hit 95, Eirika
  only) and Lyn's Mani Katti (Mt 8, Crit 20, Lyn only) for C; I
  recommended C, Nick picked A.
- **Content validation and tests:** content validation passes; every
  gate in `run-gates` passes (4,408 tests). No test pinned the lead's
  stats. What moved, each reviewed: the title cinematic's footbridge
  fight shows the lead's hit as 10 dmg (was 9) in two snapshots
  (`combat_first_strike`, `title_cinematic_6_road_fight`), as intended;
  `ch01_winnable` to `ch03_winnable` still replay. `ch04_winnable`'s
  recording (played on the army Chapters 1–3 leave) went out of step at
  command 11 and was re-recorded
  (`cargo test -p trpg-ui --test it ch04_winnable::record -- --ignored`):
  still won on turn 5, nobody lost, as before.
- No gameplay rules decided by the agent. Left for the Act 1 playtest
  (1064): whether the lead now feels "a bit strong".
- Engine friction: none, no code (one number in a data file).
