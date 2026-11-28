---
id: "0450"
title: "Danger zone and shown range of an enemy that never moves: only what it reaches from its tile"
type: bug
milestone: M3 Battle UI
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-11-17
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

None. Nick no longer checks gameplay tickets after merge (Nick,
2026-10-21: "I will save all my checking for either design (i.e. 00
tickets) or for the full act 1 playtest itself"). Verification is by
tests, `cargo shuyi frame-png` pictures in the PR and the playtest bots
(0506, 0509). What the old sign-off asked goes on the checklist of
Nick's next playtest (1064, the Act 1 playtest, if this lands before
it):

> In Chapter 1, switch the danger zone on and look
> at the keep; pick Harl to see his own range. Both should be the few tiles
> round him, not a wide area.

## Scope

**In:**
- `core`: one place that says which tiles a unit threatens, respecting
  `AiBehavior::Stationary`, used by `danger_zone`, by the range shown for a
  picked enemy, and by `threats_at`.
- The persona bots' copy of the rule (`crates/bots/src/persona.rs`,
  `threats`: "an enemy that never moves threatens only from its tile",
  added by 0506 after this ticket was written) calls the same place, so
  the bots and the danger zone agree. **Either order with 1180:** if 1180
  is done, the bots build the engine's `Threats` and take stationary
  enemies from this place; if not, `persona.rs`'s `threats` calls it.
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

- [x] A `core` unit test: a `Stationary` unit with Mov 5 and a range 1–2
      weapon has a threat area of the 13 tiles within 2 of it; the same
      unit as `Guard` or `Aggressive` has the full area.
- [x] A property test: for any scene, a tile is in the danger zone exactly
      when `threats_at` lists someone for a unit standing on it (for tiles
      the unit can stand on).
- [x] A test in `crates/ui/tests/it/` reads `MapScene`: in the Quick Battle
      with the danger zone on, a tile 4 away from the Frost Elemental and
      out of every other enemy's reach is not tinted `RangeKind::Danger`.
- [x] The AI soak test and `ch01_winnable` pass unchanged (the AI plays the
      same).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit + property (`core`): the threat area of a `Stationary` unit.
- Integration (`crates/ui/tests/it/`): the scene's danger tiles (reads the
  scene, not cells: ADR-0038).
- Snapshot: accept the danger-zone snapshots that change, after reading
  them.

## Completion notes

- `core` has one place that says what a unit threatens:
  `threat_tiles` in `crates/core/src/movement.rs`. For a `Stationary`
  unit it is its own tile plus every tile at a distance one of its ranges
  covers (nothing with no ranges); for every other unit it is
  `threat_area`, unchanged. `danger_zone`, `threats_at`, the threat
  planner and the battle's threat areas (1251, 1259) all use it, so
  `threats_at` lost its own `Stationary` branch.
- The range shown for a picked enemy (`threat_of` in
  `crates/ui/src/screens/battle/mode.rs`) and the persona bots' `threats`
  (`crates/bots/src/persona.rs`; 1180 isn't done) call `threat_tiles`.
- **Deviation: the AI keeps the old rule.** The AI's healer uses the
  danger zone to pick a safe tile, so it now calls a crate-private
  `ai_danger_zone`, the old union of `threat_area`s (an enemy that never
  moves counted as if it walked). The AI soak test and `ch01_winnable`
  pass unchanged.
- One edge changed in `threats_at`: a unit asked about the tile a
  stationary enemy stands on is now threatened by it (its own tile is
  in what it threatens, as a moving enemy's is). No unit can stand there,
  so nothing a player sees changes. A tile off the map is never
  threatened by it now, as for a moving enemy (the property test against
  the planner found the two disagreeing there; its seed is kept in
  `proptest-regressions`).
- Tests: two unit tests and a property test in `movement/tests.rs`
  (danger zone holds a free tile exactly when `threats_at` lists someone
  for the unit standing on it, with stationary units and random ranges);
  `the_frost_elemental_threatens_only_from_its_tile` in
  `crates/ui/tests/it/battle.rs` reads the scene's tints for the danger
  zone and for the picked elemental. The Quick Battle danger-zone
  snapshot changed: the tiles round the Frost Elemental beyond its
  range lost the danger tint, read and accepted.
- Pictures: `docs/screenshots/0450-danger-zone.png`,
  `docs/screenshots/0450-picked-elemental.png`.
- No starting rules: the ticket and Fire Emblem's boss ranges settled it.

### For the Act 1 playtest (1064)

- For the Act 1 playtest (1064): in Chapter 1, switch the danger zone on
  and look at the keep; pick Harl to see his own range. Both should be the
  few tiles round him, not a wide area.
