---
id: "0522"
title: "Skirmish generator: layout templates, and the position, reinforcement and terrain knobs"
type: feature
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["0510"]
nick_input: none
completed:
---

# 0522 — Skirmish generator: layouts and the full knob set

## Context

Split from 0510 before coding (ADR-0101: that ticket planned 2,600–3,400
lines; its own *natural cut* is this one). `cargo xtask skirmish new`
(0510) generates one plain layout and balances with the enemy numbers
only. `docs/design/playtest-bots.md`, *Autobalancing* (Nick, Q11): a random
skirmish may change "anything: enemy levels, count, classes, gear,
positions, reinforcements, and the terrain".

A pool battle keeps only some of that. 1008's `pool_fits`
(`crates/content/src/battle.rs`) refuses reinforcements and authored
loadouts, and a spawned group overwrites every level and may use fewer
enemies than the file lists. So `--kind random` has only the class knob
today, which can't balance a one-class pool at all: positions and terrain
are what it needs.

## Nick input

None. Verified by tests and the bots (`tickets/README.md`, *No checks after
merge on gameplay tickets*).

## Scope

**In:**
- A few layout templates with random variation in `crates/bots/src/skirmish.rs`
  (0510 has one: scattered ground, a clear row across, the army on the left,
  the enemies on the right), deterministic from `--seed`.
- Knobs for generated battles: enemy positions and terrain for both kinds;
  reinforcements for a fixed skirmish only (the pool refuses them).
- A terrain knob changes the map, so the search's stage must carry the map
  with it (`TunedBattle` today holds one `Content`, with the map inserted).

**Out (do not do):**
- Generating while the game runs (0510's *Out*).
- Story battles and side quests.

## Implementation steps

**Size (0135, ADR-0101):** about 900 changed lines.

1. Layout templates, picked from the seed and the world map's ground.
2. `TunedBattle` carries its map; `BattleText`'s writer learns reinforcements.
3. The knobs, in `crates/bots/src/balance.rs`'s `generated_knobs`.
4. `docs/playtesting.md`: the knob table and *Generating skirmishes*.

## Acceptance criteria

- [ ] `--kind random --units mage` lands in its bands.
- [ ] Every step of every new knob leaves a battle that passes content
      validation, and a pool battle still passes `pool_fits`.
- [ ] Same arguments and seed → identical files.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: each layout's determinism; each knob's steps.
- Property: generated battles of every layout pass content validation.

## Completion notes

