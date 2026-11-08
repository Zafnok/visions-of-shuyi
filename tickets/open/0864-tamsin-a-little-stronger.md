---
id: "0864"
title: "Tamsin a little stronger, after 0834 took too much"
type: tuning
milestone: M7 Chapter 1 & game flow
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 0864 — Tamsin a little stronger

## Context

A friend's playtest of the Act 1 build, passed on by Nick (2026-11-07):

> "Tamsin was overnerfed in our attempt to nerf her. She used to be OP but
> is now dogshit. Needs a slight buff."

Tamsin Rook (`sergeant`, `assets/data/characters.ron` ~:57–65): Rider,
level 2, talent Spd, base `(21, 7, 0, 6, 5, 5, 1)` (HP, Str, Mag, Dex, Spd,
Def, Res), Sword E and Spear D, carries only the Iron Spear (weight 4),
wears the Leather Vest. Rider class (`classes.ron` ~:85–91): Mov 7, growths
`(75, 45, 5, 45, 45, 35, 15)`, `lance_rush`, `charge_1`.

**What took her down, twice:**

1. **0834** (Nick's choice, round 2, 2026-10-15): Spd 7 → 5 and no Iron
   Sword. She went from doubling 10–11 of Chapter 1's 14 enemies to 1
   (the table in `docs/design/chapter-1.md`, *Tamsin toned down*). Sim:
   30 % won, the band's bottom edge. Round 3: "Feels right now".
2. **An accidental second cut:** 0321 / 0855 (the four armour weights)
   took the Leather Vest from weight 0 to 1 (`items.ron` ~:80). Burden is
   weapon + armour − Str/5 (`weapons-and-items.md` ~:216), so her attack
   speed fell from the 2 that 0834's table assumed to 1. Nobody chose
   that for her.

The bots agree she is now weak: the Casual bot's most-retreated unit is
"Tamsin Rook 93×" (`docs/playtesting.md` ~:202); the Hardcore bot's
most-fallen unit is Tamsin (12× and 10×, 0855's Completion notes).

Her role (`docs/story/characters/sergeant.md` ~:5–9, ~:69: "she stays
fast"): the fast cavalry sergeant.

## Nick input

**Decision (tuning)**, with the `ask-nick` skill, as 0834 was asked:
first the table of whom she doubles in Chapter 1 and Chapter 2 today
(her attack speed against each enemy's) and how often she falls or
retreats in the bots' runs, then each option with the same table after
the change and the bots' numbers (Casual, Normal, Hardcore win rates
on Chapters 1 and 2, inside or outside their bands):

- A. **Spd 5 → 6**: gives back the attack speed of 2 that 0834 tuned
  her at (the vest's weight took 1); it still doubles almost no one in
  Chapter 1, and adds Avoid.
- B. **Def +1 or HP +2**: she survives more, without doubling more.
- C. **A lighter vest for her only, or her talent's growth**, so the
  change is smaller at first and grows with her.
- D. **The Iron Sword back** (0834's "no Iron Sword" undone): doubles
  most of Chapter 1 again (11 of 14 before); probably too much, say so.
- E. Combinations, or describe your own.

Keep "slight" in mind: Nick asked for a slight buff, one notch, not a
return to round 2's Tamsin.

## Scope

**In:**
- The tables, the question, the answer, recorded in `chapter-1.md`
  (*As built*, after *Tamsin toned down*) and her sheet if her stats are
  written there.
- The data change in `characters.ron` (or `items.ron` / `classes.ron`
  only if Nick picks a change for everyone).
- The bots in their bands on Chapters 1 and 2 after the change (the
  playtest command, as 0855 ran it); the winning-replay tests and
  goldens updated only for what this change explains.

**Out (do not do):**
- The lead a little stronger (0846). Either order: if 0846 is done,
  the tables and the bots' numbers include it; if not, 0846 re-runs
  them with this change in.
- Changing the doubling rule or the armour weights.

## Implementation steps

1. Read 0834's and 0855's done tickets, `chapter-1.md`, `weapons-and-items.md`
   (*Attack speed*), the playtest docs (`docs/playtesting.md`).
2. Work out the tables (today, and after each option), run the bots on
   Chapters 1 and 2 for each option.
3. Ask Nick; record the answer.
4. Make the data change; run the bots again; update tests.

## Acceptance criteria

- [ ] Nick's pick is recorded in `chapter-1.md` with the tables.
- [ ] Tamsin's data is what he picked (the content tests pass).
- [ ] The bots are in their bands on Chapters 1 and 2 (the playtest
      command's report in the PR).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the winning-replay tests of Chapters 1 and 2.
- The bots' playtest runs (reported in the PR).

## Completion notes

*(Filled in by the session that completes the ticket.)*
