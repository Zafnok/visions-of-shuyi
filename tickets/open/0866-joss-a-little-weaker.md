---
id: "0866"
title: "Joss a little weaker"
type: tuning
milestone: M7 Chapter 1 & game flow
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 0866 — Joss a little weaker

## Context

Nick played the Act 1 build (Chapters 1 to 4) on 2026-11-08:

> "Joss is tuned to be a bit too strong"

Joss Pellam (`defector`, `assets/data/characters.ron` ~:106–112, set by
0848): Swordsman, level 3, talent Dex, base `(21, 6, 0, 9, 9, 4, 2)` (HP,
Str, Mag, Dex, Spd, Def, Res), Sword D, Iron Sword (Mt 5, Hit 90, Wt 2)
and Leather Vest (+2 Def, +2 Res, Wt 1). He joins in the scene after
Chapter 2 (`assets/dialogue/ch02.dlg` ~:428, `@joins defector`) and first
fights in Chapter 3.

Swordsman class (`classes.ron` ~:44–51): Mov 5, base `(18, 5, 0, 7, 8, 3,
1)`, growths `(70, 40, 10, 55, 60, 25, 20)`, active Keen Edge (+30 hit,
+10 crit on a sword attack, costs 3 durability, `skills.ron` ~:57), passive Sword
Focus 1 (+10 crit with a sword, ~:61). Swords' follow-ups deal ×6/5
(`items.ron:49`). His growths are the class's plus 20 on Dex (his
talent): a unit has no growths of its own (`progression::growth`,
`crates/core/src/progression.rs` ~:155), so "his Spd growth 60 → 55"
isn't a data change today; only his base, level, talent or kit are.

What likely makes him strong (the session checks it with the tables
below): Spd 9 (one above the class base) with a 2-weight sword gives him
a high attack speed, so he doubles many Hounds, each follow-up at ×6/5,
and Dex 9 plus Sword Focus and Keen Edge give him crits (×3). In the recorded playthroughs he stays Lv 3 through Chapters 3 and
4 (`docs/playtesting.md` ~:310–330), so the bots' army hardly used him;
Nick's play did.

His role (`docs/story/characters/defector.md`): an 18-year-old Hound who
defects, a sword-and-shield swordsman, "a level behind where the party
will be" (`characters.ron`'s comment).

**Either order with 0079:** one of 0079's options gives Joss the axe
as a Raider. If 0079 is done and picked that, tune him in his new class
and keep the same one notch; if not, tune the Swordsman as below and
0332 re-checks him if he changes class.

## Nick input

**Decision (tuning)**, with the `ask-nick` skill, as 0864 is asked:
first the table of whom Joss doubles and how often he can crit in
Chapters 3 and 4 today (his attack speed and crit against each enemy's),
and how he does in the bots' runs, then each option with the same table
after the change and the bots' numbers (Casual, Normal, Hardcore win
rates on Chapters 3 and 4, inside or outside their bands):

- A. **Spd 9 → 8** (the class's own base): attack speed down 1, so he
  doubles fewer Hounds and has less Avoid. *The agent's pick*: one notch
  at what Nick most likely felt, his doubling.
- B. **Str 6 → 5**: every hit 1 less, doubling unchanged.
- C. **Dex 9 → 8**: hit −2 (crit is Dex / 2, so 4 either way), and
  his Dex grows from 8: a small cut now, more later.
- D. **His talent Dex → another stat** (HP or Def): the same at first,
  less as he levels (his Dex growth 75 → 55).
- E. **One level lower** (3 → 2), with his base kept: smaller numbers in
  Chapter 3, but more EXP per fight (the gap, `progression.md`); probably
  a swing, not a notch, say so.
- F. Combinations, or describe your own.

Keep "a bit" in mind: one notch (Nick, *nudges, not swings*), not a
return to an ordinary swordsman.

## Scope

**In:**
- The tables, the question, the answer, recorded in his sheet
  (`docs/story/characters/defector.md`, *Role and class*: "Level, stats
  and talent: 0848" gains the new values and why) and in
  `docs/playtesting.md` with the bots' numbers.
- The data change in `characters.ron` (or `classes.ron` / `items.ron`
  only if Nick picks a change for every Swordsman or every Iron Sword).
- The bots in their bands on Chapters 3 and 4 after the change (`cargo
  xtask playtest ch03 --bot all`, then `ch04`, both seed ranges, as 0849
  and 0850 ran them); Chapter 3's and 4's recorded playthroughs recorded
  again on purpose if they move (each winnable test's header,
  `crates/ui/tests/it/ch03_winnable.rs`, `ch04_winnable.rs`).

**Out (do not do):**
- Per-unit growths (a new rule; only if Nick asks for one, then its own
  ticket).
- **0333** (armour can be got through, after 0080): **either order.** It
  changes every unit's damage, Joss's included. If 0333 is done first,
  the tables and the bots' numbers include it; if not, 0333 re-runs them
  with this change in.
- **0610** (EXP for a level gap) and **0526** (the AI): either order, the
  same way.
- Tamsin (0864) and the lead (0846).

## Implementation steps

1. Read 0848's and 0864's tickets, `combat.rs` (*Attack speed and number
   of strikes*: `burden`, `AS`, `strikes`), `stats-and-combat.md`, and
   the Chapter 3 and 4 battle files for the enemies he meets.
2. Work out the tables (today, and after each option); run the bots on
   Chapters 3 and 4 for each option.
3. Ask Nick; record the answer.
4. Make the data change; run the bots again; re-record the playthroughs
   that moved.

## Acceptance criteria

- [ ] Nick's pick is recorded in Joss's sheet and `docs/playtesting.md`
      with the tables.
- [ ] Joss's data is what he picked (the content tests pass).
- [ ] The bots are in their bands on Chapters 3 and 4 (the playtest
      command's report in the PR).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the winnable tests of Chapters 3 and 4.
- The bots' playtest runs (reported in the PR).

## Completion notes

*(Filled in by the session that completes the ticket.)*
