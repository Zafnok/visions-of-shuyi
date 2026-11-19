---
id: "1091"
title: "Element states on tiles and the reactions of spells and elemental hits with them"
type: feature
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["0085", "1064"]
nick_input: answer-first
completed:
---

# 1091 — Element states on tiles and their reactions

## Context

Nick's search for the game's storefront hook (2026-11-16) chose legendary
beasts at the bottom of dungeon delves, on top of three battle systems
that make the enemy's turn the exciting part (0085–0088, 0772,
1091–1099, 1275–1281). This is the first: **a tile holds one element
state** (wet, burning, frozen…), and a spell or an elemental hit
**reacts with the state of the tile under its target** (fire on frozen
melts it to water, ice on wet freezes it, fire on wet makes steam).
1098 (a broken beast part falls as terrain), 1278 (light in a dark
delve) and 1280 (the frost wyrm) plug into it.

**The rules come from 0085's answers** in `docs/design/magic.md`,
*Element states on tiles* (the states, the reactions, whether a hit on a
unit reacts, innate elements, the forecast, whether Act 1 keeps today's
rules). Where they differ from the sketch here, the decision wins.

What exists today (`docs/design/magic.md`, *Elements and affinities*,
*Terrain magic*): `Element` is `Fire`, `Ice`, `None`
(`crates/core/src/magic.rs`), with `Affinity` on classes
(`ClassDef::affinities`) and an element on weapons
(`WeaponStats::element`). A tile cast (`CastTarget::Tile`) of a spell
with a `TerrainEffect` (`crates/core/src/spell.rs`) changes the battle's
own map (`BattleState::map`, `set_terrain`, `Event::TerrainChanged`);
burning tiles wait in `BattleState::burning` and burn out in
`start_phase` (`burn_out`, `crates/core/src/battle.rs`). `burning`,
`ice`, `water` and `sea` are terrains in `assets/data/terrain.ron`.
Every combat goes through `BattleState::clash` and ends with
`Event::CombatResolved`. The map is the kit's scene (`MapScene`,
`TileView`, `RangeLayer` in `crates/ui/src/map_view/scene.rs`), painted
by the glyph, sprite and Playdate skins (`map_view/skin.rs`); the
forecast (`screens/battle/forecast.rs`, `ForecastView::notes`) reads
`AttackPreview` (`crates/core/src/battle/arts.rs`).

## Nick input

**Answer first:** ticket 0085. Nothing after merge: verified by tests,
`cargo shuyi frame-png` pictures in the PR and the bots. How the
reactions feel in play goes in Completion notes for a later playtest.

## Scope

**In:**
- An element-state layer on the battle's tiles in `core`: 0085's states,
  their effects and lengths, and terrains that always hold one (rivers
  and sea count as wet, if 0085 says so).
- 0085's reactions, from data, fired by tile casts and (if decided) by
  elemental hits on a unit, with their events.
- If decided: innate elements, new elements, starting states in a
  battle file.
- The states in every map skin, the reaction in the forecast, a flash
  on a change, the unit info line. Today's terrain magic keeps working.

**Out (do not do):**
- Powder kegs (their own ticket if 0085 keeps them).
- Light as an element in dark delves (1278), a beast part falling as
  terrain (1098), forged weapons with element traits (1099).
- The AI choosing tile casts for their reactions (it scores attacks by
  the forecast, which holds a reaction's damage; more is a ticket of its
  own if the bots show a need).
- New battle notes or dialogue (dialogue writing is paused, 0766).

## Implementation steps

Expected size (ADR-0092): about **900 production lines**: `core` ~450,
`content` ~150, `ui` ~300. Comparable: 0410 (terrain magic and casting
on the battle screen). Measure with `cargo xtask code-lines`; at 3× run
`engine-friction`. If 0085 picks five states or spreading lightning,
split the look (scene, skins, forecast) into a ticket of its own before
coding (`write-ticket`).

1. Read `magic.md` (*Element states on tiles*), `terrain.md`, ADR-0004,
   ADR-0020, ADR-0039: a saved `BattleState` changes, so raise
   `SAVE_VERSION` (`crates/core/src/save.rs`) and regenerate the golden
   saves (`crates/core/tests/it/save_format.rs`).
2. **The data.** A new `assets/data/elements.ron` (loader in
   `crates/content/src/`, beside `spell.rs` and `terrain.rs`, through
   `ron_loader`): the states (id, how many rounds they last or for good,
   the effect kinds each has), the terrains that always hold a state,
   and the reactions `(element, state) → { state after, terrain after,
   damage, unit effect }`. Effects are a closed list the rules
   implement (for example root, push, block ranged attacks); data picks
   among them. The content check refuses unknown ids; a test version
   goes in `assets-test/`.
3. **The layer.** In `crates/core/src/magic.rs` (or a new `element.rs`):
   `StateId`, `TileState { state, phase, rounds_left }` and
   `ReactionTable`. `BattleState` gets `tile_states: BTreeMap<Pos,
   TileState>` and `pub fn tile_state(&self, pos) -> Option<StateId>`
   (the stored state, else the terrain's own). If a state changes moving
   or fighting, give the battle one `effective_terrain(pos)` that
   movement (`crate::movement`) and combat (`BattleState::combatant`)
   read instead of `map.tiles`, so nothing else learns about states.
4. **Reactions.** `fn react(&mut self, pos, element, by: Phase, events)`
   applies the table: `Event::ElementReacted { pos, element, state }`,
   `Event::TileStateChanged { pos, from, to }`, `Event::TerrainChanged`
   if the terrain changes (`ice` melting to `water`), then damage and
   unit effects with their existing events (push: Shove's rule in
   `battle/skills.rs`; timed effects: `TimedMods`, `EffectExpired`). A
   tile cast (`plan_tile_cast`, `cast_on_tile`) may target a tile by its
   state as well as by `TerrainEffect::from`; an elemental hit reacts
   once after its `CombatResolved` if any elemental strike hit. Putting
   out a tile in `BattleState::burning` takes it off that list.
5. **Expiry.** At phase start, beside `burn_out` in `start_phase`,
   states whose rounds are up end (`TileStateChanged` to `None`), in
   `(y, x)` order, before reinforcements arrive. Write the order in the
   module docs of `battle.rs` (*Terrain*).
6. **If 0085 decides them:** innate elements (`element:
   Option<Element>` on `ClassDef` or the character, taken by the strikes
   0085 names in `Unit::combat_input`, `crates/core/src/item.rs`); new
   `Element` variants (lightning); starting states in a battle file
   (`assets/battles/README.md`), checked against the map.
7. **The look** (ADR-0038): in the map scene, painted by every map skin.
   First the kit's range layers: a `RangeName` per state with its
   palette colour (`crates/ui/src/color.rs`, every palette in
   `assets/data/palettes/`, the Playdate's pattern map), laid under the
   move and attack ranges. If 0085's look needs pictures under the
   units, the kit has no per-tile mark yet: follow `engine-change` (ask
   first, as it says) and ship the tint meanwhile. A change flashes
   like a terrain change (`TerrainFlash`).
8. **The forecast.** `AttackPreview` gets the reaction its attack would
   cause; `forecast::view` adds it to `ForecastView::notes` as 0085
   words it (`Melts ice · rider pushed · 5 dmg`); words in
   `assets/lang/`. Aiming a tile cast shows the state it leaves, as
   `TileView::becomes` shows a terrain today. The unit info screen says
   a unit's innate element (`crates/ui/src/screens/battle/info.rs`).
9. **The bots.** `cargo xtask playtest` on Act 1: same bands if Act 1
   keeps today's rules (0085's last question).

## Acceptance criteria

- [ ] Battle tests (reading the events) for each reaction 0085 lists,
      for example `fire_on_a_frozen_tile_melts_it_to_water`,
      `ice_on_a_wet_tile_freezes_it_and_roots_its_unit`,
      `ice_on_a_burning_tile_puts_it_out_a_round_early`.
- [ ] `a_hit_reacts_with_the_tile_under_its_target_once_per_combat`
      (if 0085 says hits react), and a physical hit never reacts.
- [ ] `states_end_at_phase_start_before_reinforcements_arrive`.
- [ ] Today's terrain-magic tests pass unchanged (unless 0085 changed
      that rule, and the test says what changed).
- [ ] The content check refuses a reaction on an unknown state.
- [ ] Snapshot tests of the map scene's text with a wet, a burning and a
      frozen tile, and of a forecast with a reaction note.
- [ ] `cargo shuyi frame-png` pictures in the PR: each state in the
      glyph, sprite and Playdate looks, and the forecast.
- [ ] The save golden files are regenerated at the new `SAVE_VERSION`.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the table lookup; `tile_state` for a terrain's own state; expiry.
- Property: a replay gives the same events twice; at most one state per
  tile, never off the map.
- Snapshot / integration: the scene's text, the forecast, a Harness test
  casting Fire on a frozen tile from the spell menu.
- Bots: Act 1's bands hold.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
