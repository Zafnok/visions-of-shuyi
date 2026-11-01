---
id: "0856"
title: Battle files mark a unit as non-combat
type: feature
milestone: M7 Chapter 1 & game flow
model: sonnet-5 / gpt-6.1-terra
effort: low
status: done
blocked_by: []
nick_input: none
completed: 2026-11-01
---

# 0856 — Battle files mark a unit as non-combat

## Context

`combat-arts.md` (line ~397): **non-combat** green units (villagers, wild
beasts) are "marked so in chapter data" and never use Combat Arts or
actives. Core has the marker (`Role::Noncombatant`, ticket 0803's
`Unit::role`), but a battle file (`assets/battles/*.ron`, loaded by
`crates/content/src/battle.rs`) can only set `boss: true`; nothing sets
`Noncombatant`. Ticket 0513 added green units to battle files
(`faction: Ally`, `items: [...]`) and its test villager
(`assets-test/battles/test_green.ron`) could only be made "non-combat" by
taking its weapons away.

## Nick input

None.

## Scope

**In:**
- A field on an `enemies`/`reinforcements` entry that sets
  `Role::Noncombatant` (e.g. `noncombatant: true`), with an error when it
  is combined with `boss: true`.
- `assets/battles/README.md` documents it; `test_green.ron`'s villager uses it.

**Out (do not do):**
- Deciding which Act 1 units are non-combat (each chapter's battle ticket).

## Implementation steps

1. `RawEnemy` gets the field; `Checker::enemy` sets the role; the error.
2. Content tests: it loads, and the error is reported.
3. README row and the test villager.

## Acceptance criteria

- [x] A battle file can make a unit `Role::Noncombatant`; `boss` with it is an error.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the content loader (loads; the error).
- Property: none.
- Snapshot / integration: none.

## Completion notes

- `noncombatant: true` on an `enemies` or `reinforcements` entry sets
  `Role::Noncombatant`; with `boss: true` it is an error ("a boss can't be
  non-combat"). Documented in `assets/battles/README.md`; `test_green.ron`'s
  villager uses it.
- Random skirmish pools refuse an entry with the field, as they do `boss`.
- The field is not limited to green units: the ticket asks only for the boss
  error, and an enemy with the role simply stays without arts, as a regular
  enemy does.
- Deviation: the test villager keeps `loadout: Some((weapons: []))`. The role
  stops arts and actives, not plain attacks, and the bots' baseline test needs
  the villager to drink a Potion with no kill to make.
- Gameplay rules decided: none. No follow-up tickets.
