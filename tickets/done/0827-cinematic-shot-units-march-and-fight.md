---
id: "0827"
title: "Cinematic shots where units march and fight"
type: feature
milestone: M7 Chapter 1 & game flow
model: reasoning
effort: high
status: done
blocked_by: ["0817"]
nick_input: none
completed: 2026-10-13
---

# 0827 — Cinematic shots where units march and fight

## Context

Nick picked the "Trailer cuts" storyboard for the title's intro cinematic
(ticket 0036, `docs/design/title-screen.md`, *Intro cinematic*). In the
loud part of the song its shots show units **walking across the
battlefield and fighting**: a march toward the enemy (3×), two close-ups
where units trade blows and an enemy falls (4×), and the whole field with
the rest of the party advancing (2×).

0817's `MapPan` shot only shows a battle's units standing at their
starting places. This ticket lets a map shot carry a small script of moves
and strikes.

The fights are **authored pictures, not played battles**: the cinematic
does not run the combat rules (`core` is not involved), so nothing here
can drift when stats are rebalanced, and every frame is still drawn from
the time `t` alone (0817's rule, its ADR).

Graphics are a skin (ADR-0038): units and the map are painted by the map
skin from a map scene. This ticket changes what is **in the scene** at
time `t` (where each unit is, its HP bar, whether it is flashing or
gone); it draws nothing itself.

## Nick input

None. The storyboard is decided (0036). Nick sees the result at 0820's
sign-off.

## Scope

**In:**
- `MapPan` gets an optional `action` list in the cinematic file format:
  timed `Walk` and `Strike` entries (below), and an `action_from` time so
  several shots can continue one fight across cuts.
- The player builds the map scene for time `t` from the battle's starting
  places plus the actions that have happened by then.
- Validator rules, `assets/cinematics/README.md`, a short action in
  `assets-test/cinematics/test.ron`.
- An addendum to 0817's ADR (or a new ADR that extends it): actions are
  authored, drawn from `t` alone, and never use the combat rules.

**Out (do not do):**
- Running `core` combat, the forecast, the combat scene (0413) or battle
  sounds. The cinematic makes no sounds of its own (`title-screen.md`).
- EXP, level-ups, items, spells, terrain effects.
- The real title cinematic's action (0820 writes it on the Chapter 1 map).
- Camera changes inside a shot beyond 0817's pan: one size per shot
  (0036).

## Implementation steps

1. **Format** (extend 0817's; keep its naming style):
   ```ron
   (at: 68.0, shot: MapPan(battle: "ch01", from: (6.0, 12.0), to: (10.0, 8.0), zoom: 3,
       action_from: 0.0,
       action: [
           (at: 1.0, do: Walk(unit: "sergeant", path: [(6, 12), (11, 12), (11, 8)], secs: 6.0)),
           (at: 18.5, do: Strike(unit: "lead", target: "bandit_3", hp_left: 40)),
           (at: 22.5, do: Strike(unit: "lead", target: "bandit_3", hp_left: 0)),
       ])),
   ```
   - `action_from` is the action clock's value at the shot's start; the
     clock then runs in real seconds. A later shot with the same `action`
     list and a later `action_from` shows the same fight further on, from
     another camera. To avoid repeating the list, allow a top-level
     `actions: { "<name>": [ … ] }` table and `action: "<name>"` in shots.
   - `Walk`: the unit moves along `path` (tiles; each step to a
     neighbouring tile or a straight run) at an even speed over `secs`,
     and stays at the end.
   - `Strike`: at `at` the unit lunges a few pixels toward the target and
     back (about 0.4 s); halfway through, the target flashes and its HP
     bar drops to `hp_left` percent. At `hp_left: 0` the target fades out
     over about 0.5 s and is gone afterwards.
   - `unit` / `target` are the battle file's unit ids (player slots by
     character id, enemies by their `id`).
2. **Validator:** units and targets exist in the battle; paths start where
   the unit is at that moment (its starting place or the end of its last
   walk), stay on the map and on tiles its movement type can enter; no
   action on a unit after a strike left it at 0; `hp_left` is 0–100 and
   never rises; entries sorted by `at`. All errors at once with the file
   name.
3. **Scene at time `t`:** a pure function
   `cinematic::action_state(battle, actions, clock) -> Vec<UnitView>`
   (position in map pixels, HP fraction, flash, opacity). The player puts
   those into the map scene it hands to the map skin (0432's `MapScene`;
   if the scene's unit entries can't yet carry a pixel offset, a flash or
   an opacity, add those fields there and make both skins paint them).
4. `test.ron`: one walk and one strike that fells a unit on the test
   battle; update 0817's snapshots.

## Acceptance criteria

- [x] Unit: `action_state` before, during and after a walk (including a
      corner of the path), at each stage of a strike, and after a unit
      falls.
- [x] Unit: two shots with the same action and different `action_from`
      show the same unit positions at the same action-clock value.
- [x] Each validator error has a test.
- [x] Snapshot: the test cinematic mid-walk and mid-strike; the same `t`
      drawn twice gives the same buffer.
- [x] The tests of what happened read the map scene, not cells (ADR-0038);
      both map skins paint a moving, flashing and fading unit.
- [x] The ADR text is written and listed in `docs/adr/README.md`.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `action_state`; the validator.
- Property: for any clock value, every unit's position lies on its path or
  at an end of it, and HP never rises as the clock grows.
- Snapshot / integration: the test cinematic through the debug "Play test
  cinematic" tool.

## Completion notes

**Done.**

- **Format** (`assets/cinematics/README.md`): `MapPan` takes an optional
  `action` (a list of `(at, do: Walk(…) | Strike(…))` written in place,
  or the name of one in the file's new top-level `actions` table) and
  `action_from`. Loaded as `ShotAction { from_ms, actions }` with unit
  ids, whole milliseconds and a path of every tile
  (`crates/content/src/cinematic/action.rs`).
- **Validator**: every rule in step 2, each with a test, all reported at
  once with the file name and the entry's number.
- **Scene at time `t`**: `trpg_ui::cinematic::action_state` (pure), and
  `CinematicPlayer::scene(t)`, the `MapScene` the map skin is handed. A
  shot with an action is painted each frame; one without, once as before.
- **Skins**: `UnitView` got one field, `flash`. The glyph skin lights the
  struck unit's tile; the sprite skins draw its shape in the text colour
  over its picture. Walks and falls use what the scene already had
  (`offset`, `facing`, `frame`, `fade`).
- **Test cinematic**: the lead walks round a corner to the brigand and
  fells it in two strikes, through a named list. 0817's snapshots
  updated (`cinematic_pan_middle` replaced by `cinematic_mid_walk` and
  `cinematic_mid_strike`).
- **ADR-0061** (extends ADR-0060), listed in `docs/adr/README.md`.

**Deviations from the plan**

- `action_state(start, actions, clock_ms)` takes the battle's units as
  they start (scene entries) rather than the battle, and returns scene
  entries: a tile plus an offset in tiles, not map pixels. Pixels belong
  to the skin (ADR-0038); both skins already paint tile + offset.
- An enemy's `id` isn't kept in `BattleDef`, so `battle::unit_names`
  reads the ids from the battle files (as `load_lights` does the light).
  A unit is named by a slot's character, an enemy's `id`, or an enemy's
  character (Chapter 1's captain has no `id`).
- Extra checks the ticket didn't list, each so the list can be drawn:
  a unit's walk can't start before its walk before has ended; a unit
  can't strike in the middle of its own walk, or itself; a walk takes
  more than 0 seconds and has at least two tiles; `action_from` needs an
  `action`; a named list no shot uses is an error (it would never be
  checked).

**Look choices where the ticket was silent** (*the agent's starting
choices*; nothing here is a game rule, all are tunable constants in
`crates/ui/src/cinematic/action.rs`, to judge at 0820's sign-off):

- A struck unit flashes for 0.2 seconds, brightest as the blow lands.
- A striking unit leans a quarter of a tile toward its target (4 pixels
  on the glyph map's tiles, 6 on the 24-pixel sprite tiles) and faces it.
  From further away (an archer) it leans the same little, along the line
  to the target.
- `hp_left` is a percentage of the unit's **full** HP, rounded up, so 1
  never shows an empty bar on a unit still standing.
- Units the list doesn't name stand still: they don't step on the spot
  as units do in a battle.
- On the glyph map a walking unit goes from tile to tile and the lean of
  a strike doesn't show (its units stand in character cells); the flash,
  the HP bar and the fade do. The sprite map shows all of it.

**Not done here**: I did not watch it in a running build; the snapshots
and the scene tests are the check. 0820 writes the real action on the
Chapter 1 map.

**Follow-up tickets**: none.
