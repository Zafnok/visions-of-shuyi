---
id: "0864"
title: "Tamsin a little stronger, after 0834 took too much"
type: tuning
milestone: M7 Chapter 1 & game flow
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: []
blocks: ["1064"]
nick_input: decision
completed: 2026-11-17
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

- [x] Nick's pick is recorded in `chapter-1.md` with the tables.
- [x] Tamsin's data is what he picked (the content tests pass).
- [x] The bots are in their bands on Chapters 1 and 2 (the playtest
      command's report in the PR).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the winning-replay tests of Chapters 1 and 2.
- The bots' playtest runs (reported in the PR).

## Completion notes

**Nick chose Spd +1 and Def +1** ("spd + 1, def + 1", A and half of B):
Tamsin is Spd 6, Def 6 (were 5, 5) in `characters.ron` (the private
content). The tables, the options and his words are in `chapter-1.md`,
*Tamsin a little stronger*; her sheet holds no stat numbers, so it is
unchanged.

- **Doubling.** Her attack speed is 2 (was 1): in Chapter 1 she still
  doubles only the Guard, with 2 more Avoid and 1 less damage per hit.
  The Iron Sword (D) would have doubled 10 of 14 again.
- **Bots, Chapter 1** (seeds 1–100): 53 ✓ / 51 ✓ / 69 ✓ (was 53 / 60 /
  68). Options A, B and C barely moved how often the bots lose her: they
  send her in first on her Mov 7.
- **Chapter 2 retuned (Nick: "Retune Ch2 in this PR"; "I'd want to keep
  3/5 houses"; "figure out positioning so they don't burn it the first
  turn").** Spd 6 broke Chapter 1's recorded win, and the new recording
  leaves Chapter 2 another army (Tamsin lv 6, the lead lv 5, where the old
  one had Tamsin 8 and the lead 1). Chapter 2 fell to 15 / 15 / 12
  (blocked), and a recording with no change to Tamsin did the same (14 /
  37 / 46): its bands leaned on one recording, which is ticket **0531**.
  The fix in `ch02.ron`: the west house's raider starts at (1, 15), two
  turns away, and 0524's second west raider (1, 13) and the ford's raider
  (7, 9) are gone; 13 Hounds (was 15). Bots: 48 / 50 / 65 on seeds 1–100,
  36 / 58 / 63 on 101–200, all in band. Hardcore isn't faster than Normal
  there (5 turns vs 4); it wasn't on `main` either (5 vs 5).
- **What was tried for Chapter 2 and dropped:** the second west raider
  removed (18 / 35 / 8); the south-bank raiders levels 3–4 (10–17 /
  18–30 / 46–62); the ford raider a plain fighter, or on the north bank
  (where it burned the north-west house on turn 1), or at (9, 9) (where
  it waded the ford and burned the big house); the autobalance, 71 and
  107 evaluations (best: every Hound 4 levels down, 34 / 45 / 76); "save
  2 of 5" (37 / 60 / 70, which Nick turned down).
- **Recordings.** Every winning playthrough from Chapter 1 to Chapter 5
  (with Aldwater and the Salt Road) was recorded again; each wins with
  nobody lost: ch01 turn 10, ch02 turn 2 (5 of 5 houses), ch03 turn 7,
  aldwater 3, ch04 12, salt_road 3, ch05 10. `ch02_winnable` counts 13
  Hounds (was 15).
- **Later chapters moved** with the new armies (seeds 1–100): ch03 68 ⚠ /
  70 / 42 ✗, aldwater 100 / 50 ✗ / 80, ch04 50 / 32 ✗ / 31 ✗, salt_road
  100 / 94 / 96 ⚠, ch05 51 / 15 ✗ / 11 ✗. None is blocked. Ticket
  **0876** brings them back (both follow-ups are in 1064's `blocked_by`
  and the roadmap).
- **The agent's starting rules** (for the Act 1 playtest, 1064): the
  Chapter 2 placement and the two raiders removed are the agent's, to
  Nick's "positioning" ask; the ford house is no longer threatened on the
  first turn either.
- Engine friction: none, no Rust code (data, recordings and one test's
  count).
