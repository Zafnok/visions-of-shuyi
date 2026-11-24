---
id: "0527"
title: "The bots report units that can't hurt the enemy or fall behind"
type: feature
milestone: M4 Enemy AI
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: []
blocks: ["1064"]
nick_input: none
completed: 2026-11-22
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
(0333 armour, 0610 EXP for a level gap, 0870 Joss) can show what it did
to each unit, and so a unit falling behind is seen before Nick plays.

**The report proposes; it never tunes.** Player units, classes and
growths are Nick's (`docs/design/playtest-bots.md`, *Autobalancing*
~:232–245: the autobalancer changes only the battle, and its knobs,
`crates/bots/src/balance.rs` `knobs` ~:112, are enemy numbers only).
Nick asked about bots that tune the player's side too; that is a design
question for 0080 (how the army gets through armour), not this ticket.

**0080 answered it (2026-11-22):** asked whether the bots should also
try player-side changes, Nick picked **"Report only"**: the bots try
player-side changes and report which ones put each chapter in its band,
and he picks; they never apply one (`docs/design/playtest-bots.md`,
*The player's side: proposed, never applied*). That search is ticket
**0533**, which starts from this ticket's flags. This ticket builds the
per-unit report only, as below.

**0080 also changed the numbers** (built by 0333): bow hits deal at
least 2 and spear hits at least 3. **Either order with 0333:** `can_hurt`
reads `trpg_core::forecast`, so it follows whichever rule is in; if 0333
lands first, Aske can hurt every guard (the first acceptance criterion
then checks a unit that still can't, e.g. Maud: her Iron Gauntlets'
power 4 against Def 15, and no attack spell), and if not, write the criterion as it stands and add a line to
0333 to update it.

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
- Trying player-side changes and reporting the ones that put a chapter in
  its band (Nick's "Report only", 0080): ticket **0533**.
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
   playtest` prints (`crates/tools/src/game_tools.rs` ~:300).
2. Add the per-unit counts to `BattleMeasures` and the named measures to
   `BattleMeasure::finish`.
3. Write `can_hurt` and the level-by-chapter table from the arrival army.
4. Print the table and the flags.
5. Run Chapters 1 to 4 with `--bot all` and paste the tables in
   `docs/playtesting.md`.

**Expected size** (ADR-0101): about 300 production lines (measures,
`can_hurt`, the table) and 300 of tests, plus docs.

## Acceptance criteria

- [x] `can_hurt` on Chapter 2's start says Aske (Iron Bow) can't hurt
      the two `hound_guard`s (Def 15) and Rue can (test).
- [x] A test battle with a scripted kill gives the killer `kills 1`, its
      `dealt` the HP taken and its `exp` the event's amount.
- [x] `cargo xtask playtest ch02 --bot normal --runs 5` prints a line per
      player unit with the numbers above, and a flag for a unit under a
      threshold (a test of the printing, and the output in the PR).
- [x] The first run of Chapters 1 to 4 is in `docs/playtesting.md`.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `can_hurt`; the per-unit counts from a list of events; the flags
  at and around each threshold.
- Integration: a short `playtest` run whose report has the per-unit lines.

## Completion notes

Done 2026-11-22.

- `BattleMeasures::units` (`UnitTally`: dealt, kills, exp, levels per
  player unit), from `CombatResolved`'s strikes (HP before minus after,
  so a last strike's overkill doesn't count), `Pushed` and
  `CollisionDamage` (credited to the unit acting in that command:
  `CombatResolved`'s attacker, `SpellCast`, `SkillUsed`, `ArtUsed`),
  `UnitFell`/`UnitDowned` (a kill for the last unit to hurt it in that
  command), `ExpGained` and `LeveledUp`. Only damage to `Faction::Enemy`
  units counts.
- `BattleMeasure::finish` adds `dealt <name>`, `kills <name>`, `exp
  <name>`, `levels <name>` through the engine's runner, **a 0 left
  out** (read with `trpg_bots::unit_measure`), so the try lines stay
  short and the `--json` report and history keep them.
- `crates/bots/src/units.rs`: `can_hurt` / `enemies_hurt` (each weapon
  slot and known attack spell equipped on a copy of the start, then
  `BattleState::threat_forecast` against each enemy on its tile, Combat
  Arts and actives included) and `flags` with the three thresholds as
  constants.
- `cargo xtask playtest` prints a units table per bot after the bot
  lines (no `--units` flag: about 7 lines per bot); `Record::units` keeps
  each unit's arrival level and can-hurt count (`#[serde(default)]` for
  older history), and `--merge` checks parts agree on it.
- Docs: `docs/playtesting.md` *Units that fall behind* with the first
  run of Chapters 1–4; `docs/design/playtest-bots.md` *What the report
  shows*. Lines added to 1180 (the measures' names) and 1207 (the table)
  so their move carries them.

**Deviations.** The first run plays Chapter 1 over 100 tries and
Chapters 2 to 4 over 30: with other sessions' builds on the machine, a
Hardcore try of Chapter 3 took about 18 minutes of one core, and 100
tries of all four would have taken hours. "Level by chapter" is the `lv`
column of each chapter's table (the arrival army) and `levels med`
(the median gain in the chapter), not a separate cross-chapter table.
0080 was answered while this was built (Nick's "Report only"); the
player-side search is 0533, so this report only reports. 0333 (bow and
spear minimums) isn't done: a line in it says to update this ticket's
Chapter 2 test when the bow's minimum lands.

**The agent's starting rules** (bots tooling, not gameplay; tunable):
`can hurt < 2/3`, `lv 3+ below the army` (the army's lower median),
`exp < half a share` (under half of 1/n of the army's EXP over every
try); a kill is the last unit to hurt the enemy in the same command;
push and collision damage go to the unit acting; a Combat Art counts
toward "can hurt".

**What the first run says** (for 0080, 0333, 0610, 0870): Aske deals the
least of the fighting units but Joss in every chapter and can't hurt
Chapter 2's two hound guards; Rue takes a fifth to a third of the EXP;
the bots barely use Joss, so they can't confirm or refute "Joss is a bit
too strong"; Maud the cleric is always flagged `can hurt < 2/3` by
design. The bands of Chapters 2–4 have moved since their last
documented runs (ch02 Casual and Hardcore over the band, ch03 Hardcore
under, ch04 Normal and Hardcore under, over 30 tries); not this
ticket's to tune.
