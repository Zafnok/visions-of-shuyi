---
id: "1277"
title: "Dungeon delves: a world-map node, generated floors, the Way Down and the Way Out"
type: feature
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["0088", "1064"]
nick_input: answer-first
completed:
---

# 1277 — Dungeon delves: a world-map node, generated floors, the Way Down and the Way Out

## Context

Nick's storefront-hook search (five research rounds, 2026-11-16) settled
the game's hook: legendary beasts hidden at the bottom of dungeon delves,
fought as a raid. The chain is 0085–0088, 0772, 1091–1099 and 1275–1281.
This ticket builds the delve itself, "Torchlight", with no beast and no
darkness yet: those are 1278 (dark floors), 1279 (the hunt that leads to
a delve) and 1280 (the frost wyrm's lair).

The sketch Nick agreed to: a delve is a world-map node; a party of 3–6
goes down; each floor is a generated grid battle; the **Way Down** starts
collapsing once someone steps on it, and anyone left behind starts the
next floor alone (a rescue, not a death); a **Way Out** every third
floor; loot is kept only if you leave by a Way Out. **The rules come from
0088's answers in `docs/design/delves.md`. Where they differ from the
sketch here, the decision wins.**

What exists to build on:

- **The world map** (1007, 1008; `docs/design/world-structure.md`):
  `NodeKind`, `WorldNode`, `WorldState`, `WorldCommand`, `WorldEvent` and
  `Campaign::world_command` / `Campaign::clear_node` in
  `crates/core/src/worldmap.rs`; random groups in
  `crates/core/src/worldmap/skirmish.rs`. The map files are
  `crates/content/src/worldmap.rs` (`WorldMapDef`). The screen is
  `crates/ui/src/screens/world_map.rs` with `PlaceMark` and `FindKind`
  in `world_map/view.rs`, painted by `glyph.rs`, `picture.rs` and
  `playdate.rs`. `world-structure.md` lists "Grinding tower / dungeon
  (E)" as deferred; 0088 takes it up.
- **Escape maps** (0069, 0319): `Objective::Escape { area }`,
  `UnitAction::Escape`, `BattleState::can_escape`, `BattleState::escaped`
  and `Event::UnitEscaped` in `crates/core/src/battle.rs`. The AI and the
  bots head for the exit through `escape_area` and `flow_field` in
  `crates/core/src/ai.rs` (~:1051). The map scene always shows the exit
  area (`RangeName` `EXIT`, `crates/ui/src/map_view/scene.rs`). The Way
  Down and the Way Out are exit areas with more rules.
- **A battle built at run time from data**: a saved random group becomes
  an ordinary `BattleDef` through `trpg_content::skirmish::battle` and
  `battle::random_skirmish` (`crates/content/src/battle.rs`), so every
  placement, template and loadout check runs on it (ADR-0104). A floor is
  built the same way.
- **Generation**: `cargo xtask skirmish new` (0510,
  `crates/bots/src/skirmish.rs`) generates maps at **authoring time
  only**, as dev tooling (`trpg-bots` never ships). A delve generates its
  floors **while the game runs**, so the floor generator goes in `core`,
  pure and deterministic from a saved seed (ADR-0004, ADR-0019). That is
  a new pattern: record it in an ADR (`write-adr`).
- **Who fights**: battles place named characters by `PlayerSlot`
  (`crates/core/src/campaign.rs`, `Campaign::battle_setup`). The game has
  no screen to pick who deploys; a delve needs one.
- **Loot**: chests are `TileFeature::Chest(Loot)` on `BattleMap`
  (`crates/core/src/map.rs`); `Campaign::apply_result` puts what a won
  battle found into the stock.
- **Terrain** for dungeon floors exists: `floor`, `wall`, `door` in
  `assets/data/terrain.ron`, drawn from the bought *Dungeons* set in the
  picture look.

## Nick input

**Answer first:** ticket 0088 (its questions 1, 2, 4, 5, 7 and 8: the
party and whether the rest wait, floors and their size, the Way Down,
the Way Out and loot, rewind and Retry in a delve, coming back; dark
and light and lingering are 1278's). If `delves.md` leaves open whether
HP carries between floors, whether Preparations come between floors or
whether the player can save between floors, the session's starting rule
is the plainest (HP restored as after any battle, Preparations only at
the entrance, only the suspend inside a delve), written in the
Completion notes for the playtest. Nothing else is asked here. Verified by
tests, `cargo shuyi frame-png` pictures and the bots; what is left to
judge by playing goes in the Completion notes for a later playtest.

## Scope

**In** (the rules half; the screens are 1281, split off when this
ticket was written because the whole was 2,700–3,200 changed lines):
- `NodeKind::Delve` on the world map in the rules (entering it gives
  `WorldEvent::DelveAhead`).
- The party as a rule: `start_delve` refuses a party outside the size
  0088 set (3–6 in the sketch).
- Floors generated at run time from the delve's data and a saved seed:
  rooms of `floor`, `wall` and `door`, enemies from generic templates,
  chests, the Way Down, and the Way Out on every Nth floor.
- An optional authored last floor (`last: Some(battle id)`), so 1279 and
  1280 can put a beast's lair at the bottom.
- The Way Down's collapse, the units left behind starting the next floor
  alone, the haul kept only through a Way Out, and leaving the delve.
- The delve's progress saved with the campaign; the suspend works on a
  floor as on any battle.
- A test delve on the test world map (`assets-test/`).

**Out (do not do):**
- The screens (1281): the delve's mark in the world-map skins, its panel
  lines, the party picker, the flow's stages, snapshots and frame-png
  pictures.
- Darkness, light and lingering (1278). Beasts, hunts and lairs (1279,
  1280, 1095–1098).
- Real delves in the game's acts (content tickets after 0087 and 0088,
  in the private repository's `content/`).
- Running the generator from `cargo xtask skirmish` or sharing code with
  0522's layout templates (either order works: if 0522 lands first, its
  layouts stay authoring-time tooling; nothing here waits on it).
- The run mode (0050): it may reuse this generator, but that is its own
  ticket's call.
- A `cargo xtask playtest` stage for a whole delve. The bots play floors
  in tests here; 1276 and 1280 decide what more they need.
- Walking around freely (0060): delves are grid floors whatever it says.

**The rooms are generic** (the engine's redundancy sweep of 2026-11-28): carving rooms of floor, wall and
door and repairing their connectivity is parked engine#2119's BSP rooms
and connectivity repair. Its enemies, chests, the Way Down and the Way
Out stay the game's rules. Ask through the `engine-change` skill for
Nick to unpark 2119 with this ticket as the asker. If 2119 is still
parked when this starts, write the room carver over `Pos` and the core's
random source, under 300 lines, so the pin bump after 2119 replaces it.

## Implementation steps

**Size (ADR-0092, ADR-0101):** about 900 production lines (comparable:
1008's world-map skirmishes, 428 measured, plus a room generator a few
times 0510's `map_text`); about 1,800–2,100 changed lines with tests,
the test data, the ADR and the archived ticket. The screens were split
off to 1281 when this ticket was written.

1. Read `docs/design/delves.md` (0088), `world-structure.md`,
   `death-and-difficulty.md` (*Saving*), ADR-0104, and the code above.
2. **The delve's data** (`crates/core/src/delve.rs`, new): `DelveDef`
   (id, floors, the party's size range, the Way Out's spacing, each
   floor's enemy level and size, the template pool, the chest loot table,
   the collapse length, an optional authored `last` battle) and
   `DelveState` (the delve and its node, the floor, the seed, the party
   by `CharacterId`, who is left behind, the haul of gold and items).
   Materials (1099) are either order: if 1099 is done, the haul holds
   them too and pays them out through `Campaign::gain_material`; if not,
   add a line to 1099 that its materials join the delve's haul.
   `Campaign` gains `delve: Option<DelveState>`, left out of the save
   while `None` and read as `None` from a save without it, as
   `Campaign::world` is, so `SAVE_VERSION` stays (`crates/core/src/save.rs`
   module docs).
3. **Delve commands** on `Campaign`, each returning `DelveEvent`s
   (ADR-0004): `start_delve(def, node, party)`, `finish_floor(def,
   state)` (who went down, who was left behind, the floor's loot into the
   haul, the next floor or the end) and `leave_delve(def, by)` (through a
   Way Out: the haul goes into the stock and gold; any other way, as
   0088 says). The seed of each floor comes from the campaign RNG
   (`Campaign::rng`, as `spawn_skirmishes` uses it) once, at the start,
   and is saved. On the world map: `NodeKind::Delve { delve: String }`,
   not a battle (`is_battle` stays false), and `WorldCommand::Enter` on
   it gives `WorldEvent::DelveAhead(node)`.
4. **The floor's objective** in `crates/core/src/battle.rs`: a new
   `Objective` variant for the Way Down (its area, the Way Out's area if
   the floor has one, the collapse length). `UnitAction::Escape` works
   on either area; the first unit through the Way Down starts the
   collapse; when it ends, the floor is over (victory if anyone went
   down, as 0088 says). Record which way each escaped unit left. Extend
   `escape_area` and `flow_field` (`crates/core/src/ai.rs`) so the AI on
   the player side and the bots head for the Way Down. The module docs'
   *Rules* list the new rules.
5. **The floor generator** (`crates/core/src/delve/floor.rs`):
   `plan_floor(def, floor, seed) -> FloorPlan` (tiles, the party's
   tiles, tiles apart for whoever was left behind, the Way Down, the Way
   Out, enemies as template, level and tile, chests), deterministic.
   `trpg_content::delve` (new, `crates/content/src/delve.rs`) loads
   `delves/*.ron`, validates them (templates exist and are generic, a
   `last` battle exists and fits), and turns a plan into an ordinary
   `BattleDef` through the same `build` path `random_skirmish` uses, with
   player slots from the party. The floor's music is the
   `dungeon_tense` cue (`audio.md`) unless 0088 names others.
   Write the ADR: the game generates delve floors while it runs, in
   `core`, from a saved seed; authoring-time generation (0510) stays for
   skirmishes.
6. **Test data**: a three-floor delve with a Way Out on floor 3 on
   `assets-test/worldmaps/test_world.ron`, in `assets-test/delves/`.

## Acceptance criteria

- [ ] `plan_floor` gives the same plan for the same delve, floor and seed
      (unit test), and every floor it makes passes content validation,
      with every party tile able to reach the Way Down (property test).
- [ ] The first unit through the Way Down starts the collapse; when it
      ends, the units still on the floor are left behind and start the
      next floor on the tiles apart, alive (unit tests in
      `crates/core/src/delve/tests.rs`).
- [ ] Leaving by a Way Out puts the haul into the stock and gold; the
      other ways do what 0088 says (unit tests).
- [ ] A save made with a delve in progress loads to the same delve and
      floor, and the same seed gives the same next floor (determinism
      test); a save without `delve` still loads.
- [ ] A headless test plays the test delve's three floors with the
      `BaselineBot` on the player side and leaves by the Way Out.
- [ ] `start_delve` refuses a party outside the size range (unit test).
- [ ] The ADR exists and the ADR index lists it.
- [ ] What is left to judge by playing is in the Completion notes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: delve commands and events; the collapse; left-behind units; the
  haul; the generator's determinism; content validation of delve files.
- Property: generated floors are valid and connected for many seeds and
  floors; the party size stays in its range.
- Integration: a whole test delve played headless; save and load in a
  delve.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
