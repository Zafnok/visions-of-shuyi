---
id: "0846"
title: "The lead a little stronger in Chapter 1"
type: tuning
milestone: M7 Chapter 1 & game flow
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: ["0845"]
nick_input: decision
completed:
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

- [ ] Nick picked; recorded with the bot numbers.
- [ ] Data changed; content validation and the bot run in the completion
      notes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Content validation; existing combat tests updated only where they pin
  the lead's stats (each reviewed).

## Completion notes

*(Filled in by the session that completes the ticket.)*
