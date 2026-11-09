---
id: "0527"
title: "The bots report units that can't hurt the enemy or fall behind"
type: feature
milestone: M4 Enemy AI
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0527 — The bots report units that can't hurt the enemy or fall behind

## Context

Nick played the Act 1 build (Chapters 1 to 4) on 2026-11-08:

> "Aske the archer is too weak to even get XP so she snowballs inversely
> and becomes useless almost immediately. Her STR + wep are not enough to
> go thru the heavily armored units. Some things that might make this
> better are more thorough bots automated balancing (i.e. tweaking not
> just the enemy but also the player classes and units and levelups) OR
> introducing piercing mechanic with a guaranteed min hit i.e. 2 or 3 min
> hit for piercing damage"

> "Mages (i.e. Rue) are too necessary ... armored knights are literally
> impossible to kill without mages so she gets too much combat and thus
> snowballs"

> "Joss is tuned to be a bit too strong"

The bots measure a battle, never a unit: each try gives `won`, `items`,
`turns`, `deaths`/`retreats` and which unit fell
(`crates/bots/src/playtest.rs`, `BattleMeasure::finish` ~:150–205, from
`BattleMeasures`, `crates/bots/src/measures.rs`). Nobody saw that Aske can
hurt almost none of Chapter 2's guards or that Rue does most of the
killing. The only per-unit numbers are the army's levels at each chapter
(`docs/playtesting.md` ~:310–330: at Chapter 3 Aske lv 5, Joss lv 3, Rue
lv 8; at Chapter 4 Aske lv 6, Joss lv 3, Rue lv 12), and those come from
the **recorded playthroughs** (`crates/bots/src/arrival.rs`,
`PLAYTHROUGHS`, the `ch0N_winning.ron` fixtures), not from the bots.

This ticket adds a report per playable unit, so that a balance change
(0333 armour, 0610 EXP for a level gap, 0866 Joss) can show what it did
to each unit, and so a unit falling behind is seen before Nick plays.

**The report proposes; it never tunes.** Player units, classes and
growths are Nick's (`docs/design/playtest-bots.md`, *Autobalancing*
~:232–245: the autobalancer changes only the battle, and its knobs,
`crates/bots/src/balance.rs` `knobs` ~:112, are enemy numbers only).
Nick asked about bots that tune the player's side too; that is a design
question for 0080 (how the army gets through armour), not this ticket.

**0080's question 2** asks Nick whether the bots should also try
player-side changes (report only, or apply them within limits he sets).
**Either order with 0080:** if 0080 is done and he picked the second,
this report grows to list the change it would make for each flagged
unit; if not, build the report only (player stats stay his).

## Nick input

None. Verified by tests and by the report's first run on Chapters 1 to 4,
pasted into `docs/playtesting.md` and the PR.

## Scope

**In:**

- **Per unit, per try (the bots' battle runs):** for each player unit,
  new named measures in `BattleMeasure::finish`, gathered in
  `BattleMeasures::observe` from the events: `dealt <name>` (HP taken
  off enemies, from `Event::CombatResolved`'s outcome and the other
  damage events), `kills <name>` (enemies it felled), `exp <name>`
  (`Event::ExpGained`), `levels <name>` (`Event::LeveledUp` count). They
  go through the engine's runner like `fell <name>` does today, so the
  `--json` report and the history keep them.
- **Per unit, at the battle's start (no play needed):** the share of the
  map's enemies it **can hurt**: for each enemy, whether any weapon or
  attack spell the unit can use gives `damage > 0` in
  `trpg_core::forecast` against it on its starting tile (terrain
  included). A pure function in `crates/bots` (e.g. `can_hurt(state,
  unit) -> (hurt, of)`), so a test can call it.
- **Level by chapter:** each unit's level arriving at each chapter (the
  arrival army, `arrival.rs`) and, from the bots' tries, its median level
  gain in that chapter.
- **The table:** one line per unit per battle, e.g. (the shape only;
  the numbers are made up):

  ```text
  ch02 · units (normal, 100 tries)
  Aske       lv 2  can hurt 9/15   dealt med 14  kills med 0  exp share 6%   ⚑ can hurt < 2/3
  Rue        lv 3  can hurt 15/15  dealt med 58  kills med 4  exp share 34%
  ```

  printed after the battle's report by `cargo xtask playtest` (where the
  game's own lines go, beside the engine's table), and a `--units` flag
  if it makes the default report too long.
- **Flags** (*the agent's starting values*, *tunable*, as constants with
  a comment): a unit that can hurt **under 2/3** of the battle's enemies;
  a unit whose arrival level is **3 or more below the army's median**;
  a unit with **under half an even share** of the EXP the army earned.
  A flag is a line in the report, never a failure (as a band missed is
  only reported).
- `docs/playtesting.md`: a section (*Units that fall behind*) on how to
  read it, and the first run of Chapters 1 to 4 with the persona bots.
- `docs/design/playtest-bots.md`, *What the report shows*: one paragraph
  that the report shows each unit and that it proposes, never tunes.

**Out (do not do):**
- Changing any unit, class, growth or weapon, or letting the autobalancer
  do so.
- Changing the bands or the personas.
- **1180** (the bots on the engine's playtest pieces) and **1207** (the
  CLI and history on the engine's). **Either order:** if they are done
  first, the per-unit lines go where they put the game's own report
  lines; if not, add a line to 1180 naming the new measures and the
  table so the move carries them. If the engine's runner can't carry
  per-unit lines without a change, keep them in the game's report (the
  `engine-friction` skill if that code grows past three times its
  estimate).

## Implementation steps

1. Read `measures.rs`, `playtest.rs`, `arrival.rs` and how `cargo xtask
   playtest` prints (`crates/xtask/src/game_tools.rs` ~:300).
2. Add the per-unit counts to `BattleMeasures` and the named measures to
   `BattleMeasure::finish`.
3. Write `can_hurt` and the level-by-chapter table from the arrival army.
4. Print the table and the flags.
5. Run Chapters 1 to 4 with `--bot all` and paste the tables in
   `docs/playtesting.md`.

**Expected size** (ADR-0101): about 300 production lines (measures,
`can_hurt`, the table) and 300 of tests, plus docs.

## Acceptance criteria

- [ ] `can_hurt` on Chapter 2's start says Aske (Iron Bow) can't hurt
      the two `hound_guard`s (Def 15) and Rue can (test).
- [ ] A test battle with a scripted kill gives the killer `kills 1`, its
      `dealt` the HP taken and its `exp` the event's amount.
- [ ] `cargo xtask playtest ch02 --bot normal --runs 5` prints a line per
      player unit with the numbers above, and a flag for a unit under a
      threshold (a test of the printing, and the output in the PR).
- [ ] The first run of Chapters 1 to 4 is in `docs/playtesting.md`.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `can_hurt`; the per-unit counts from a list of events; the flags
  at and around each threshold.
- Integration: a short `playtest` run whose report has the per-unit lines.

## Completion notes

*(Filled in by the session that completes the ticket.)*
