---
id: "0081"
title: "Decide: a bigger EXP cut for a unit above its enemy's level"
type: design-decision
milestone: Design decisions
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 0081 — Decide: a bigger EXP cut for a unit above its enemy's level

## Context

Nick played the Act 1 build (Chapters 1–4) on 2026-11-08:

> "Mages (i.e. Rue) are too necessary (and thus get used too often) bc
> most enemies are far too armored, i.e. armored knights are literally
> impossible to kill without mages so she gets too much combat and thus
> snowballs"

> "On the snowball point, we should penalize it, reduce XP depending on
> lv disparity without going to 0. Fire Emblem has a system like this"

**The game already has Fire Emblem's cut, gently.** `docs/design/
progression.md` (*EXP formulas*, ~:257–300) uses the GBA Fire Emblems'
shape, doubled for Nick's faster pace (`exp_for_combat`,
`crates/core/src/progression.rs` ~:100). With `d = enemy level − unit
level`:

- no damage: **2**;
- damage: `2 × ((31 + d) / 3)`, at least 2;
- kill: `2 × ((31 + d) / 3 + max(0, 20 + 3d))` (+40 for a boss), at most
  100.

100 EXP is a level. Then `paced_exp` (~:124) scales every award by the
unit's own level (`exp_pace` in `classes.ron` ~:31: 100 % to Lv 39), which
does nothing in Act 1. So the cut by level gap exists and never reaches 0
(the floor is 2), as Nick asks; it is only gentle: a kill of an enemy 5
levels below still gives 26 (a quarter of a level).

**What the army earns today** (the bots' recorded army, `docs/
playtesting.md` ~:310–330: at Chapter 3 Rue Lv 8, Aske Lv 5; at Chapter 4
Rue Lv 12, Aske Lv 6; guards Lv 6 in Chapter 3, Lv 7 in Chapter 4):

| Case | d | Hit | Kill |
|---|---|---|---|
| Rue Lv 8, Ch 3 guard Lv 6 | −2 | 18 | **46** |
| Rue Lv 12, Ch 4 guard Lv 7 | −5 | 16 | **26** |
| Aske Lv 5, Ch 3 guard Lv 6 (she does 0 damage) | +1 | **2** | — |
| Aske Lv 5, if she could hurt it | +1 | 20 | 66 |
| Equal levels (for scale) | 0 | 20 | 60 |

**Honest read, to tell Nick first:** Rue snowballs mostly because she is
the only one who can hurt a guard (0080), so she takes every guard kill.
A steeper cut slows her a little; it doesn't make anyone else able to take
those kills. Aske's side is that 0 damage gives 2 EXP; that is 0080's to
fix. This ticket is the "penalize the snowball" half.

**What other Fire Emblems do** (from the Fire Emblem Wiki's EXP formulas;
check each again before showing it):
- **GBA (FE6–8):** our shape (`(31 + d) / 3` for damage, plus a kill
  bonus by level gap). The big cut there is that a promoted unit counts
  as **20 levels higher**, so promoted units earn little from unpromoted
  enemies.
- **Path of Radiance:** damage EXP `(21 + d) / 2`, at least 1; a kill
  adds the gap and a mode bonus. Promoted units count 20 levels higher
  here too. It also hands out **bonus EXP** after chapters, which the
  player gives to any unit between battles (Path of Radiance and Radiant
  Dawn).
- **Awakening:** when the unit is 2+ levels above, the kill bonus falls
  3 a level but never below **7**: a floor, "without going to 0".
- **Three Houses** scales a kill's EXP by the level gap too (the factor
  depends on the difficulty; don't quote numbers for it).

Act 1 has no promotion yet (0606), so the "+20 levels when promoted" rule
matters later; say so and ask whether to keep it in mind for tier 2.

## Nick input

**Decision** (`ask-nick`), each option with the table above redone:

- **A. A steeper slope, like today's but sharper.** Damage
  `2 × ((31 + 2d) / 3)`, kill bonus `max(0, 20 + 5d)`. Rue Lv 8 on a
  Lv 6 guard: kill 46 → **38**; Rue Lv 12 on a Lv 7 guard: 26 → **14**.
  A unit below its enemy earns more (Aske hitting a Lv 6 guard: 20 →
  22). Equal levels unchanged.
- **B. A cut for each level above, with a floor.** The award loses 10 %
  for each level the unit is above its enemy, never below 20 % (and never
  below 2). Rue Lv 8 on a Lv 6 guard: 80 %, kill 46 → **36**; Rue Lv 12 on
  a Lv 7 guard: 50 %, kill 26 → **13**. Units at or below the enemy's
  level: unchanged.
- **C. A floor on the kill bonus, like Awakening**, with a steeper fall
  above it (for example A's slope, but the kill bonus never below 7, so
  a kill always gives at least `2 × (that hit + 7)`).
- **D. Catch-up instead of (or with) a cut**: Path of Radiance's bonus
  EXP. Each battle won puts some EXP in a pool (for example 100), and the
  player gives it out between battles, so Aske can be brought up.
- **E. Combinations, or describe your own** (for example B with D).

Ask too: later, should a promoted unit count as higher-level for EXP, as
in the GBA games and Path of Radiance (yes / no / ask again at tier 2)?

Check every game claim before showing it; claim only what the game
really does.

## Scope

**In:** the question, the tables, the answer with Nick's words in
`progression.md` (*EXP formulas*, with new worked examples), the
`docs/design/README.md` row, 0610's text updated with what to build.

**Out (do not do):**
- Building it: ticket **0610** (it also reruns the bots, since the
  army's levels at each chapter change).
- How non-mages hurt armour (0080). Either order: if 0080 is answered,
  redo the table with its rule (Aske's row gets real damage EXP); if not,
  use today's and note in 0080 that this ticket's formula changes its
  EXP numbers.
- The EXP pace by the unit's own level (`exp_pace`, settled by 0608,
  done).

## Implementation steps

1. Read `progression.md` (*Character level and unit EXP*, *EXP
   formulas*), `progression.rs` (`exp_for_combat`, `paced_exp`),
   `docs/playtesting.md`.
2. Check the other games' formulas on the Fire Emblem Wiki
   (<https://www.fireemblemwiki.org/wiki/Exp>) before quoting them.
3. Work out each option's table (and a whole Chapter 3 for Rue: about
   how many levels she gains with each option, from the bots' runs or by
   hand).
4. Ask Nick; record the formula, his words and worked examples (numbers
   marked his or *tunable*).
5. Update 0610's text.

## Acceptance criteria

- [ ] `progression.md` has the new formula, Nick's words and worked
      examples; `docs/design/README.md` has the row.
- [ ] 0610's text says exactly what to build.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

None (a decision); `cargo xtask ticket-lint`.

## Completion notes

*(Filled in by the session that completes the ticket.)*
