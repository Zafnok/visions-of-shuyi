---
id: "0609"
title: "A spell or skill learned in battle comes with its full uses"
type: bug
milestone: M5 Progression
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-11-09
---

# 0609 — A spell or skill learned in battle comes with its full uses

## Context

A friend's playtest of the Act 1 build, passed on by Nick (2026-11-07):

> "Characters leveling up i.e. Rue leveling and learning Frost skill
> shouldn't give it with 0 charges that battle, instead probably just
> give full usage of it right there and then"

**Nick's decision, 2026-11-07** (in the same message): a spell or active
skill learned in the middle of a battle has its full uses at once.

Why it has 0 today:

- Uses live in `SpellState.uses_left` (`crates/core/src/spell.rs`
  ~:186–216); a missing entry is 0. They refill only in
  `Unit::prepare_for_battle` (`crates/core/src/item.rs` ~:936–944:
  `SpellState::full`, `SkillUses::full`).
- Level-up calls `learn_spells` (`crates/core/src/progression.rs`
  ~:382–391) at ~:306 (level up) and ~:359 (class level up). It adds the
  spell to `unit.learned` and emits `Event::SpellLearned`, but never adds
  a `uses_left` entry. The doc comment at `spell.rs` ~:256–261 says it:
  "New spells have no uses until the next battle starts."
- Rue is a `mage` (`assets/data/characters.ron` ~:88); the class teaches
  Frost at class level 5 (`classes.ron` ~:97); Frost has `uses: 10`
  (`spells.ron` ~:30).
- The 0-uses rule was the agent's, never Nick's:
  `docs/design/combat-arts.md` ~:676–679 ("*Claude's starting rule*;
  nothing in the game teaches an active mid-battle yet").
  `docs/design/magic.md` ~:129–131 covers only the refill at a battle's
  start.

## Nick input

None: decided in his message above. Verified by core tests.

## Scope

**In:**
- When a unit learns a spell in battle (level up or class level up), its
  `uses_left` entry is set to the spell's full `uses`.
- The same for an active skill with `Uses(n)` learned in battle
  (`SkillUses`).
- `refresh_spells` (`spell.rs` ~:262; used on promotion and reclass,
  `class_change.rs` ~:224): if it can run during a battle, a newly gained
  spell or skill gets its full uses too; a spell the unit already had
  keeps what is left (no free refill). If it runs only between battles,
  say so in a doc comment and leave it.
- Docs: `magic.md` (a spell learned in battle has its full uses at once,
  Nick 2026-11-07) and `combat-arts.md` ~:676 replaced by the same rule
  for actives; the comment at `spell.rs` ~:256.

**Out (do not do):**
- Refilling spells the unit already knew when it levels up.
- The level-up screen's look.

## Implementation steps

1. Read `learn_spells`, `SpellState`, `SkillUses`, `prepare_for_battle`,
   `refresh_spells`.
2. Give `learn_spells` what it needs to know each spell's `uses` (the
   `SpellTable`, or the uses passed in by the caller); insert the entry
   for each spell it adds. The same for actives.
3. Tests; docs.
4. The state hash changes for records where a unit learns a spell
   mid-battle: run the replay and golden tests and update only the
   values this change explains (say which in the PR).

## Acceptance criteria

- [x] A unit that learns Frost from a level-up in battle has Frost's full
      uses at once and can cast it the same battle (core test, Rue-like
      test mage).
- [x] A spell the unit already knew keeps its uses left across the
      level-up (test).
- [x] An active skill with `Uses(n)` learned in battle has `n` (test), if
      any class teaches one by level; if none does, a unit test on the
      learning function.
- [x] `magic.md` and `combat-arts.md` say the new rule with Nick's date.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `learn_spells` and the skill equivalent.
- Property: after any sequence of level-ups in a battle, every learned
  spell has `0 ≤ uses_left ≤ uses`, and a newly learned one starts full.
- Integration: a battle where the test mage levels up and casts the new
  spell in the same battle.

## Completion notes

- New `Unit::fill_new_uses` (`crates/core/src/item.rs`): every learned
  spell and usable non-attack active with no uses entry yet gets its full
  uses; one with an entry (even 0) keeps it. The battle calls it after
  each EXP / CP award (`award`) and after the EXP pool is shared.
- Deviation from step 2: `learn_spells` and `grant_exp` are unchanged.
  They also run between battles (the campaign's bonus EXP), where uses
  don't matter, so the battle fills the uses after the award instead of
  progression taking the spell table. A missing entry means "learned since
  the battle started", since `prepare_for_battle` gives every spell and
  active one.
- `refresh_spells` runs only between battles (the camp's class change
  screen and the bots' army builder), so it only got a doc comment.
- No class teaches an active by level in battle (mastering the current
  class makes nothing new usable), so the skill side is a unit test on
  `fill_new_uses` (a Guard that masters Exile gets Rally full, Brace
  keeps 1 use).
- Tests: a battle where a test mage learns Frost on a level up, has 10
  uses at once while Bolt keeps 1, and casts Frost the next turn; unit
  tests for spells and actives; a property test over level ups, class
  level ups and casts.
- Replay and golden hashes: none changed (no recorded battle has a unit
  learning a spell mid-battle).
- Docs: `magic.md` and `combat-arts.md` state Nick's rule (2026-11-07);
  the old starting rule in `combat-arts.md` is gone.
- Starting rules decided: none.
