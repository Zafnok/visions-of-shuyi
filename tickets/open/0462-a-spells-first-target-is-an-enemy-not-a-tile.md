---
id: "0462"
title: "Aiming an attack or spell starts on an enemy, not a tile"
type: feature
milestone: M3 Battle UI
model: sonnet-5
effort: medium
status: todo
blocked_by: ["1118", "1050"]
nick_input: sign-off
completed:
---

# 0462 — Aiming starts on an enemy, not a tile

## Context

Chapter 1 playtest round 3 (0804, 2026-10-16):

> "anything that attacks should prio an enemy over terrain for its first
> hover"

Today (read 2026-10-16):

- **Attack and Arts** already start on an enemy: `choose_attack`
  (`crates/ui/src/screens/battle/mode.rs:1163-1168`) builds targets in
  reading order and rotates the one the path was aimed at
  (`Selection.target`, `mode.rs:86-88`) to the front
  (`attack.rs:261-292`).
- **Spells** don't: `CastTargeting::new` (`crates/ui/src/screens/battle/
  magic.rs:340-368`) always starts at index 0 of `cast_targets`
  (`magic.rs:73-101`), which mixes units and tiles (tiles only for
  spells with a `terrain_effect`: Fire, Frost, Cinder Oath;
  `assets/data/spells.ron`) sorted together by `(y, x)` (line 99). So
  Fire can open on a forest tile above an enemy. It also ignores
  `sel.target`.

**Waits for the engine carve and the Playdate port** (Nick, 2026-10-16),
so `blocked_by` has 1118 and 1050.

## Nick input

**Sign-off:** cast Rue's Fire next to an enemy and a forest; the cursor
opens on the enemy.

## Scope

**In:**
- A spell's target list opens on: the enemy the path was aimed at, if
  it's a target; else the first enemy in reading order; else (no enemy in
  reach) the first tile. Up/down/left/right still move through all
  targets (enemies and tiles) as today (*Claude's starting rule*: the
  order of the list stays reading order; only the start changes).
- Heals keep their start (0451: the ally pointed at).
- Same rule anywhere else a list mixes units and tiles, if there is one.

**Out (do not do):** the order of targets; what can be targeted.

## Acceptance criteria

- [ ] Fire with an enemy and a forest tile in reach opens on the enemy
      (test, where the forest comes first in reading order).
- [ ] With a path aimed at an enemy, the spell opens on that enemy.
- [ ] With no enemy in reach, it opens on a tile.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `CastTargeting::new`'s start index for the three cases.

## Completion notes

*(Filled in by the session that completes the ticket.)*
