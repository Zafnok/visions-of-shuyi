---
id: "1278"
title: "Dark floors: light as an element, and lingering wakes something"
type: feature
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["1277", "1091"]
nick_input: answer-first
completed:
---

# 1278 — Dark floors: light as an element, and lingering wakes something

## Context

Part of the hook chain Nick settled on 2026-11-16 after five research
rounds (legendary beasts at the bottom of dungeon delves; 0085–0088,
0772, 1091–1099, 1275–1281). 1277 builds the delve and its generated
floors in full light. This ticket makes them dark, as the sketch Nick
agreed to says: **light behaves like an element** (fire lights tiles,
water puts it out), and **lingering too long on a floor wakes
something**. **The rules come from 0088's answers in
`docs/design/delves.md`, and the element reactions from 0085's in
`docs/design/magic.md` (*Element states on tiles*). Where they differ
from the sketch here, the decisions win.**

What exists to build on:

- **Element states on tiles** (1091, from 0085): a tile holds one
  element state, and a spell or an elemental hit reacts with it. Light is
  one more thing a tile holds, changed by the same reactions (fire on a
  dark tile lights it, water or ice on a lit one puts it out, as 0088
  says), so it goes beside 1091's `BattleState::tile_states`
  (`TileState`, `StateId`) and through its `ReactionTable`, not in a
  system of its own. A tile holds one element state, so light is a
  second layer (lit or dark) that the same reactions change. Today's fire is a terrain cast:
  `Burning` and `Event::TerrainChanged` in `crates/core/src/battle.rs`
  (*Terrain*); a burning tile is a light while it burns.
- **Delve floors** (1277): `plan_floor` in `crates/core/src/delve/floor.rs`
  and `trpg_content::delve`, which build each floor's `BattleDef`; the
  delve's data, `DelveDef`. Torches, braziers or whatever 0088 names are
  placed by the generator.
- **Lights today are a look only**: `crates/content/src/lighting.rs`
  (ADR-0058, ticket 0438): noon, dusk, night, each a recipe for shading
  the ground's pictures; "the rules never see it". A dark floor is a
  rule now, so it can't be only a map's `light`. The picture look's
  shading recipes are still how a dark tile looks.
- **The map scene** is the engine kit's `shuyi_ui::grid::MapScene`
  (`crates/ui/src/map_view/scene.rs`, built by `BattleScreen::scene` in
  `crates/ui/src/screens/battle/mod.rs`). Its `TileView` has terrain,
  flashes and `becomes`, no light; its `ranges` are named tile layers
  (`RangeLayer`) the game's skins colour. Darkness is either a range
  layer the game's glyph, sprite and Playdate skins paint
  (`crates/ui/src/map_view/glyph.rs`, `sprite.rs`, `playdate.rs`), or a
  channel added to the kit by the `engine-change` skill; the
  `engine-friction` skill decides (ADR-0092).
- **What a side knows**: the enemy AI (`crates/core/src/ai.rs`) and the
  bots' list of legal commands (`crates/core/src/legal.rs`, 0504) read
  the whole state. The threat arrows (`ThreatView`, 1251, 1259), the
  Danger Zone (`Action::DangerZone`), `NextUnit`/`PrevUnit`, the info
  panel and the forecast all show every unit. None of them know about
  hidden units.
- **Reinforcements** (`Reinforcement`, `Event::UnitsArrived`) arrive on a
  set turn: a floor's waker can be one, placed by the generator, so
  lingering needs no new rule of arrival.

## Nick input

**Answer first:** ticket 0088 (its question 3: how far each unit sees,
how long fire's light lasts, whether water puts light out, whether ice
glows, and how enemies outside the light show, not at all or as marks
such as fading footprints; and its question 6: what wakes, when, and
whether the player is warned), and 0085 for the reactions light joins. Nothing else is asked here. Verified by tests, `cargo shuyi
frame-png` pictures and the bots; what is left to judge by playing
goes in the Completion notes for a later playtest.

## Scope

**In:**
- Light on each tile of a delve floor, in 1091's tile states: lit or
  dark (or the levels 0088 sets), with light sources the generator
  places, and the reactions 0088 and 0085 name.
- Seeing: which tiles and units each side sees, from the light and its
  units, as one function in `core`. A side can't target what it can't
  see; the enemy AI and the bots follow the same rule.
- Everything that shows units hides the ones the player can't see, or
  shows only the marks 0088 picked (a footprint is a scene item, not a
  unit): the map scene, the threat arrows, the Danger Zone, unit
  cycling, the info panel and the forecast.
- The dark tiles' look in every map skin (glyph, sprite, Playdate one-bit).
- Lingering: a floor's turn limit from the delve's data, after which
  what 0088 names arrives, with the warning 0088 decides.

**Out (do not do):**
- Darkness outside delves (story battles at night stay a look:
  `lighting.rs`).
- New element reactions beyond light (1091 and 0085).
- Fog of war on the world map.
- New enemy classes or art for the waker: it uses an existing template
  until a content ticket names its own.

## Implementation steps

**Size (ADR-0092, ADR-0101):** about 650 production lines (comparable:
`Burning` and the tile-cast rules in `battle.rs`, plus one filter at
each of the five places that show units); about 1,600–1,900 changed
lines with tests and pictures.

1. Read `docs/design/delves.md` (0088), `magic.md` (*Element states on
   tiles*, *Terrain magic*), 1091's tile-state code, 1277's generator,
   and ADR-0038 and ADR-0058.
2. **Light in the rules**: a light layer next to 1091's `tile_states`
   in `BattleState` (saved with it, as `burning` is: *Saving* in the
   module docs). Its changes come only from events (a new
   `Event::LightChanged`, or 1091's `Event::TileStateChanged` if it
   fits), from rows added to 1091's `ReactionTable`. `DelveDef` gains the floor's starting light and
   its light sources; `plan_floor` places them.
3. **Seeing**: `BattleState::visible_to(faction) -> TileSet` (or the shape
   0088's rule needs), pure and cheap enough to call per frame (keep it
   per state change if not; 1259 shows how the threat arrows cache).
   `BattleState::check` refuses an attack, cast or talk at a unit the
   acting side can't see (a new `CommandError`), so `legal_commands`, the
   AI and the bots follow without their own copy of the rule. In
   `crates/core/src/ai.rs`, the enemy's flow field and targets use only
   what it sees (as 0088 says the enemy sees). Keep `core` deterministic
   (ADR-0004).
4. **The player's view**: `BattleScreen::scene` leaves out units the
   player can't see and adds a darkness layer (the choice in Context);
   the threat arrows, Danger Zone, unit cycling, info panel and forecast
   use `visible_to(Faction::Player)`. Tests read the scene (ADR-0038).
5. **The look** in every skin: the glyph skin dims a dark tile's colours;
   the sprite skin shades it with `lighting.rs`'s night recipe (or a
   `dark` line 0088's look picks); the Playdate skin uses a dither from
   its pattern map (1253). If 0088 left the look
   open, show Nick mockups in both looks before building (`ascii-art`).
6. **Lingering**: `DelveDef` gains the floor's turn limit and what wakes;
   `plan_floor` adds it as a `Reinforcement` on that turn, on a tile out
   of the party's sight. The warning (a banner line, the objective page,
   or none) is what 0088 says, its text by lang key.
7. **Test data**: the test delve's floors (`assets-test/delves/`) become
   dark with a few light sources and a waker.

## Acceptance criteria

- [ ] Fire on a dark tile lights it and water puts a lit tile out, as
      0088 and 0085 say (unit tests, through `BattleState::apply`).
- [ ] A unit on a tile the player can't see is not in the map scene, the
      threat arrows or unit cycling, and can't be attacked
      (`CommandError`), in tests that read the scene and the state.
- [ ] The enemy AI and the bots never target a unit their side can't see
      (property test over random play with `legal_commands`).
- [ ] Lingering past the floor's limit brings the waker on that turn, out
      of sight (unit test); the warning shows as 0088 says.
- [ ] frame-png pictures of a dark floor in the glyph, picture and
      Playdate looks in the PR; a look test reads a dark tile's colours.
- [ ] The test delve is still won by the `BaselineBot` headless.
- [ ] What is left to judge by playing is in the Completion notes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: light reactions; `visible_to`; refused commands at hidden units;
  the waker's arrival.
- Property: over random seeds and play, no side acts on a unit it can't
  see; generated dark floors stay valid.
- Snapshot / integration: the scene of a dark floor; a whole test delve
  played headless; save and load keep the light.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
