---
id: "0450"
title: "Danger zone and shown range of an enemy that never moves: only what it reaches from its tile"
type: bug
milestone: M3 Battle UI
model: sonnet-5
effort: medium
status: todo
blocked_by: []
nick_input: sign-off
completed:
---

# 0450 — Danger zone of an enemy that never moves

## Context

Found while building the threat arrows (ticket 0448). An enemy whose AI is
`Stationary` (`crates/core/src/ai.rs`: Harl in Chapter 1's keep, the Frost
Elemental in the Quick Battle) never leaves its tile: it attacks only what
its weapon or spell reaches from where it stands.

- The **threat arrows** know this (`threats_at` in
  `crates/core/src/movement.rs`; `docs/design/turn-structure.md`, *Threat
  arrows*: "An enemy that never moves ... counts for the tiles it can
  attack from where it stands").
- The **danger zone** (`danger_zone`, `danger_tiles` in
  `crates/ui/src/screens/battle/mod.rs`) and the **range shown when the
  player picks an enemy** (`Threat` in `crates/ui/src/screens/battle/mode.rs`,
  built with `threat_area`) don't: both tint every tile the enemy could
  attack *if it walked its full Mov first*. So the danger zone round Harl is
  far bigger than what he will ever attack, and a tile inside it shows no
  arrow from him. The two displays disagree.

In Fire Emblem a boss that doesn't move shows only its attack range.

## Nick input

**Sign-off** after merge: in Chapter 1, switch the danger zone on and look
at the keep; pick Harl to see his own range. Both should be the few tiles
round him, not a wide area.

## Scope

**In:**
- `core`: one place that says which tiles a unit threatens, respecting
  `AiBehavior::Stationary`, used by `danger_zone`, by the range shown for a
  picked enemy, and by `threats_at`.
- The tests and snapshots that change.

**Out (do not do):**
- Changing how `Guard` enemies are shown: they do move to attack, so their
  whole range counts (decided in ticket 0055).
- Changing the AI, or what `Stationary` means.
- The threat arrows' look or when they show (0448).

## Implementation steps

1. In `crates/core/src/movement.rs`, make `threat_area` (or a new function
   beside it that `danger_zone` and `threats_at` both call) return, for a
   unit whose `ai` is `AiBehavior::Stationary`, only the tiles at each
   range's Manhattan distance from its own tile, plus its own tile. Every
   other unit is unchanged. Keep `threats_at` giving the same answers as
   today (its tests in `crates/core/src/movement/tests.rs` must pass
   unchanged) and remove its own `Stationary` branch once the shared
   function has it.
2. Check the callers of `threat_area` (`grep -rn threat_area crates`): the
   battle screen's picked-enemy range and the AI. The AI must not change:
   if it uses `threat_area` for its own planning, give it the unchanged
   version.
3. Update the module docs of `movement.rs` (the *Threat* rule).

## Acceptance criteria

- [ ] A `core` unit test: a `Stationary` unit with Mov 5 and a range 1–2
      weapon has a threat area of the 13 tiles within 2 of it; the same
      unit as `Guard` or `Aggressive` has the full area.
- [ ] A property test: for any scene, a tile is in the danger zone exactly
      when `threats_at` lists someone for a unit standing on it (for tiles
      the unit can stand on).
- [ ] A test in `crates/ui/tests/it/` reads `MapScene`: in the Quick Battle
      with the danger zone on, a tile 4 away from the Frost Elemental and
      out of every other enemy's reach is not tinted `RangeKind::Danger`.
- [ ] The AI soak test and `ch01_winnable` pass unchanged (the AI plays the
      same).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit + property (`core`): the threat area of a `Stationary` unit.
- Integration (`crates/ui/tests/it/`): the scene's danger tiles (reads the
  scene, not cells: ADR-0038).
- Snapshot: accept the danger-zone snapshots that change, after reading
  them.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
