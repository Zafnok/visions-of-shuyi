---
id: "1094"
title: "Steal the Turn: a perfect parry freezes the enemy phase for one move of yours"
type: feature
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["1093"]
nick_input: answer-first
completed:
---

# 1094 — Steal the Turn

## Context

Nick's search for the game's storefront hook (2026-11-16) chose
legendary beasts fought as a raid, on top of three battle systems that
make the enemy's turn the exciting part (0085–0088, 0772, 1091–1099,
1275–1281). The third is **Steal the Turn**: a perfect parry freezes the
current enemy (or beast) phase, the world drains to grey, one of your
units gets one move, then the phase resumes from the changed board.
Nick: "this sounds more interesting"; on the grey freeze, "yea this
sounds ok"; of a rough animation of it, "works quite well". 1098 (a part
broken during a stolen move gives the rare material) and 1280 use it.

**The rules come from 0086's answers** in `docs/design/timing.md`,
*Steal the Turn*: when (any perfect parry or only against bosses and
beasts; how many per phase), who moves (the parrier, any unit, or the
parrier or an ally next to them), what the move is (full action, act
only, move only; arts, spells, items; the parried unit as a target), a
unit that already acted, and the look (grey, banner, reduced flash).
Where they differ from the sketch here, the decision wins.

What it builds on:
- 1092: a strike records its press (`Strike::press` in
  `crates/core/src/combat.rs`), so a perfect parry is in the command's
  `Event::CombatResolved`; 1093 shows it and its flash.
- Phases (`turn-structure.md`; `Phase`, `end_phase`, `start_phase`,
  `check_ready` in `crates/core/src/battle.rs`): a unit acts only in its
  own phase, once. A move after an attack is a `PendingMove` answered by
  `Command::MoveAfter`, which `legal_commands` (`crates/core/src/legal.rs`)
  and `next_command` (`crates/core/src/ai.rs`) handle first.
- The engine's bot loop and history ask `Sides::side_to_move` (the
  phase today, `impl Sides for BattleState`). The game's own loops ask
  `state.phase()`: `crates/bots/src/arrival.rs` (enemy phases until
  `Phase::Player`, `EndPhase` as fallback), `testkit.rs`, `measures.rs`,
  `playtest.rs`; the battle screen's `ai_phase()`
  (`crates/ui/src/screens/battle/mod.rs`).
- The map's look: units are `UnitView`s
  (`crates/ui/src/map_view/scene.rs`) painted by `GlyphUnits`,
  `SpriteUnits` and `OneBitUnits`; the ground by the kit's skins through
  `MapSkin::paint` and `with_looks` (`crates/ui/src/map_view/skin.rs`);
  a map's light greys and tints the ground's pictures
  (`assets/data/lighting.ron`, ADR-0058) but not the glyph map.

## Nick input

**Answer first:** ticket 0086 (through 1093). Nothing after merge:
verified by tests and `cargo shuyi frame-png` pictures; how a steal
feels goes in Completion notes for a later playtest.

## Scope

**In:**
- The steal in `core`: when it starts, who may move and how, the
  command that uses it (or passes), the resume, one per phase (or
  0086's count), its events, and saving it with the battle.
- Everyone who asks whose move it is asks `side_to_move`.
- The battle screen hands control to the player mid-phase and back,
  with the banner, the grey world in every map skin and the help bar.
- A reduced-flash option, if 0086 decides one.
- **Either order with 1095** (the Beast Phase): the steal is written
  for any AI phase. If 1095 is done, a parried beast blow steals too
  (`a_perfect_parry_in_the_beast_phase_steals_the_turn` here); if not,
  1095 adds that test.
- **Either order with 1098:** if 1098 is done, its "broke during a
  stolen move" query answers from `stolen()` (a test here); if not,
  1098 reads `stolen()` itself.

**Out (do not do):**
- The Beast Phase itself (1095); rare materials (1098).
- Bots that parry perfectly (they press Good, so never steal; tests
  script the steal).

## Implementation steps

Expected size (ADR-0092): about **650 production lines**: `core` ~300,
`bots` ~20, `ui` ~330 (control, banner, the grey world, help).
Comparable: 0326's Overwatch (an action in the enemy's phase) with its
look on the map. Measure with `cargo xtask code-lines`; at 3× run
`engine-friction`.

1. Read `docs/design/timing.md`, `turn-structure.md`, ADR-0004, ADR-0020,
   ADR-0039 (a saved `BattleState` changes: raise `SAVE_VERSION` and
   regenerate the golden saves) and 1092's ADR.
2. **The state.** `BattleState::stolen: Option<StolenMove { phase,
   parrier, movers }>` and the phase's steal count, both saved
   (`#[serde(default)]`), with `pub fn stolen(&self)`.
3. **It starts** at the end of an AI unit's action (after its
   `MoveAfter`, if it gets one), when that command's combats hold a
   perfect parry by a player unit, the battle isn't over, the phase has
   a steal left and 0086's other conditions hold:
   `Event::TurnStolen { phase, parrier }`, after `UnitActed`.
4. **During it.** `side_to_move()` is `Phase::Player`; `phase()` stays
   the AI's. `check` accepts only an `Act` (or a `Pressed` `Act`) by a
   unit in `movers`, shaped as 0086 says (a move-only steal is `Wait`; an
   act-without-moving steal has `dest` its own tile), and its
   `MoveAfter`. The stolen move ignores `acted` (`check_ready` takes the
   steal into account) and leaves the unit's flag as it was before.
   `Wait` on its own tile passes. Anything else is
   `CommandError::TurnStolen`. `legal_commands` lists only those
   commands; `next_command` returns `None` while a steal is open.
5. **It ends** after the stolen action (and its `MoveAfter`):
   `Event::TurnResumed { phase }`, then the outcome check; the AI goes
   on. A steal never starts in a stolen move. Write the rules in
   `battle.rs`'s module docs (*Phases*).
6. **Whose move.** `impl Sides for BattleState`: `side_to_move` is the
   player during a steal. Switch the game's loops to it:
   `crates/bots/src/arrival.rs`, `testkit.rs`, `measures.rs`,
   `playtest.rs`, and `BattleScreen::ai_phase()`.
7. **The AI's plan.** Either order with 1051: if the AI plans once per
   phase, a steal's events invalidate the plan like any other command;
   if not, nothing to do.
8. **The screen.** On `TurnStolen` (after the fight's playback): the
   banner (`banner.rs`, `Banner::for_event`; words in `assets/lang/`,
   0086's), the cursor on the parrier, the camera saved and restored as
   `player_view` does for phases, the help bar for the stolen move.
   Danger zone and threat arrows work as in the player phase. On
   `TurnResumed` the AI phase plays on.
9. **The grey world** (ADR-0038: on the map, in the scene, painted by
   every map skin). A `UnitView` flag for units drawn grey (all but the
   movers), painted by the three unit painters; the ground grey through
   `MapSkin::paint`: a greyed copy of the palette for the glyph look, a
   `stolen` light in `lighting.ron` for the picture look, and 0086's
   look on the Playdate's one bit. The drain eases in over a *tunable*
   0.3 s. If greying the ground on the game side grows past 3× its
   estimate, run `engine-friction` (a kit-wide drain is generic).
10. **Reduced flash** (if 0086 decides it): a setting and its Options
    row as 1093's Timed presses row, which turns the perfect parry's
    veil into a soft one.

## Acceptance criteria

- [ ] Battle tests (scripted `Pressed` commands):
      `a_perfect_parry_in_the_enemy_phase_steals_the_turn`,
      `only_one_steal_per_phase` (or 0086's count),
      `a_good_parry_steals_nothing`,
      `the_stolen_move_is_the_only_legal_command`,
      `waiting_passes_the_stolen_move`,
      `the_enemy_phase_resumes_from_the_changed_board`,
      `a_stolen_move_that_wins_ends_the_battle`.
- [ ] `side_to_move_is_the_player_during_a_steal`; the bots' loops end
      (a bot test plays a battle with a scripted steal to its end).
- [ ] Rewind to a point inside a steal and replay gives the same state.
- [ ] A Harness test: an enemy attacks, the parry is perfect, the
      banner shows, the player moves one unit, the enemy phase goes on.
- [ ] Snapshot tests of the map scene during a steal (grey flags), and
      `cargo shuyi frame-png` pictures of the grey world in the glyph,
      picture and Playdate looks, in the PR.
- [ ] The save golden files are regenerated at the new `SAVE_VERSION`.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the start conditions; the checks during a steal; the count per
  phase.
- Property: no command sequence leaves a steal open at a phase's end;
  replaying a battle with steals is deterministic.
- Integration: the Harness steal; rewind across a steal; the bots' loop.
- Snapshot: the scene during a steal; the banner.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
