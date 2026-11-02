---
id: "0477"
title: "Danger zone and threat arrows count what arts and actives reach"
type: bug
milestone: M3 Battle UI
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: ["0521"]
nick_input: none
completed:
---

# 0477 — Danger zone and threat arrows count art and active reach

## Context

Found while working 0521, which lets a boss attack with an art that reaches
farther than any of its weapons (Long Thrust, Hurl, Moonwave). A combat active
already could (Long Shot).

Every display and check of "who can attack this tile" gets its ranges from
`Unit::attack_ranges` (`crates/core/src/item.rs`), which reads only weapons
and spells:

- the danger zone and a picked enemy's shown range
  (`crates/ui/src/screens/battle/mod.rs`, `mode.rs`);
- the threat arrows (`threats_at`, `crates/core/src/movement.rs`;
  `docs/design/turn-structure.md`, *Threat arrows*);
- the AI's own danger zone (`Planner::danger`, `crates/core/src/ai.rs`), which
  decides a healer's retreat and a boss's Brace;
- the personas' check (`crates/bots/src/persona.rs`).

So a stationary spear boss at rank C can hit a tile two away that shows no
danger and no arrow. Nick asked that the arrows be accurate
(`turn-structure.md`: "ensure the threat arrows are accurate").

## Nick input

None. It makes the displays say what the rules already do, with no new look.
Verified by tests; anything left to judge by playing goes on the Act 1
playtest's checklist (1064).

## Scope

**In:**
- The ranges above include the arts and combat actives the unit may use and
  can pay for now (`Unit::may_use_arts`, `Unit::usable_arts`,
  `Unit::usable_skills`), with an art's replacement minimum and maximum range
  and an active's range bonus.
- One function gives those ranges, so the displays and the AI can't disagree.

**Out (do not do):**
- A new look for the tiles an art reaches, or any new key or panel.
- The stationary-enemy area of 0450.
- Changing what any art or active does.

## Implementation steps

**Size:** about 60 production lines, judged against `Unit::attack_ranges` and
`legal.rs`'s `attack_reach`.

1. Add the failing tests below.
2. Give `Unit` a function returning the attack ranges with techniques, taking
   the art and skill tables. An active's bonus only counts with a weapon or
   spell it fits; check how the battle decides that (`plan_active`).
3. Move the callers listed in *Context* to it. Keep `attack_ranges` only if a
   caller truly wants weapons and spells alone.
4. Check the danger-zone planning budget still holds
   (`enemy_phase_planning_time`, `crates/core/src/ai/tests.rs`).

## Acceptance criteria

- [ ] A stationary boss with a range-1 spear at rank C puts a tile two away in
      the danger zone, in `threats_at` and in its shown range.
- [ ] The same boss with a broken spear or below rank C doesn't, and neither
      does an ordinary enemy with the same weapon and rank.
- [ ] An archer boss with Long Shot's bonus threatens one tile farther.
- [ ] A healer outside what a hostile art reaches retreats as it did; one
      inside moves out of it.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `crates/core` ranges, `threats_at` and the AI's danger zone.
- Integration: a battle screen test reads the danger tiles from the scene.
- Run a persona baseline and report changed results in the PR, since the
  personas read these ranges.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-ups and anything for 1064.)*
