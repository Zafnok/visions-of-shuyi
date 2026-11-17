---
id: "1279"
title: "Beast hunts on the world map: rumors, tracks and the rival column"
type: feature
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["0087", "1277", "1281", "1095"]
nick_input: answer-first
completed:
---

# 1279 — Beast hunts on the world map: rumors, tracks and the rival column

## Context

Part of the hook chain Nick settled on 2026-11-16 after five research
rounds: legendary beasts hidden at the bottom of dungeon delves, fought
as a raid (0085–0088, 0772, 1091–1099, 1275–1281). This ticket is the
hunt, the part on the world map that leads to a beast's delve. The sketch
Nick agreed to (0087's *Context*):

- a rumor in town puts a mark on the world map;
- skirmishes nearby turn up tracks; after enough tracks the delve
  entrance appears;
- a rival enemy column marches to the same place, but moves only when
  you finish a story battle, so random fights never cost you the beast;
- the beast waits on the delve's last floor.

**The rules come from 0087's answers in `docs/design/beasts.md`. Where
they differ from the sketch here, the decision wins.** In particular
0087 decides what happens if the column gets there first (bound and met
later, fled to another region, or gone), what happens to a hunt left
unfinished when its act locks behind you, and how many beasts there are
and where. The story's framing is decided by 0087 and recorded in the
private repository's `story/design/` (ticket 0772); nothing of it goes
in this public repository.

What exists to build on:

- **The world map's rules** (`crates/core/src/worldmap.rs`):
  `NodeCondition` (when a place opens), `WorldNode::hidden` (a secret
  place stays out of sight until it opens), `WorldState` (saved), and
  `Campaign::clear_node`, which already does two of the hunt's three
  triggers' work: it removes a won random group (`world.skirmishes`,
  `crates/core/src/worldmap/skirmish.rs`) and, after a won story battle,
  calls `spawn_skirmishes`. Tracks and the column hook in there. Winning
  the map's final story battle replaces the map (`WorldEvent::NewAct`):
  the act locks.
- **Towns**: `WorldMapScreen::open_town` (`crates/ui/src/screens/world_map.rs`
  ~:470) lists a town's shops in `Open::Town`, or opens the one shop
  straight away; a rumor needs the list even in a one-shop town.
- **Marks and the panel**: `PlaceMark` and `PanelView`
  (`world_map/view.rs`), painted by `glyph.rs`, `picture.rs` and
  `playdate.rs`; the panel's lines in `WorldMapScreen::panel` (~:728).
  The random skirmish's mark is already a `?` on a grey plate
  (`docs/design/look-and-feel.md`, *The world map*), so the hunt's mark
  must look different.
- **Delves** (1277): `NodeKind::Delve`, `DelveDef` with an authored
  `last` floor, and `Campaign::start_delve`.
- **The beast army** (1095): a third army hostile to everyone with its
  own Beast Phase. The hunt's last floor holds a beast of that army, so
  the chain plays end to end before the real lair exists (1280).

## Nick input

**Answer first:** ticket 0087 (its questions 3, 7 and 13, and the
hunt's numbers: how many tracks, how far "nearby" is, how fast the
column marches). The hunt's mark and the column's mark on the map are a
look choice: if 0087 didn't settle them, show Nick mockups before
building, in the glyph look and the picture look (`ascii-art`). Verified
by tests, `cargo shuyi frame-png` pictures and the bots; what is left to
judge by playing goes in the Completion notes for a later playtest.

## Scope

**In:**
- A hunt in the world map's data: the town whose rumor starts it, the
  delve place it leads to, the places whose skirmishes give tracks, how
  many tracks open the entrance, and the column's start, path and pace.
- Hearing the rumor in its town (an entry in the town's list; its line
  is plain UI text by lang key, naming the place).
- Tracks from won skirmishes near the hunt's place; the delve entrance
  opening at enough tracks.
- The rival column: marching on each won story battle, its mark on the
  map, and what 0087 says happens when it arrives first or the act ends.
- The hunt's delve ends on a last floor with a beast of the Beast army
  (a test beast here).
- Test data on the test world map.

**Out (do not do):**
- The frost wyrm, its lair and its parts (1280; 1096–1098).
- Real hunts on the game's acts (1280 places the first; others are
  content tickets after 0087).
- Rumor dialogue or scenes: dialogue writing is paused (0766). The
  rumor is one UI line; a scene, if 0087 wants one, is a writer's
  ticket.
- The column fighting battles on its way, or battles against it, unless
  0087 asks for them (then write a ticket for it).

## Implementation steps

**Size (ADR-0092, ADR-0101):** about 650 production lines (comparable:
1008's random skirmishes, 428 measured, which hook `clear_node` the same
way, plus two marks and a town entry); about 1,800–2,100 changed lines
with tests and the test data. If it passes 2,500, stop and split: the
rules and data stay here; the marks, panel lines and town entry go to a
new ticket blocked by this one.

1. Read `docs/design/beasts.md` (0087), `world-structure.md`, ADR-0104,
   1277's delve code and 1095's Beast army.
2. **The hunt's rules** (`crates/core/src/worldmap/hunt.rs`, new):
   `Hunt` (the data above) on `WorldMap` (`hunts: Vec<Hunt>`), and
   `HuntState` (heard, tracks, where the column is, the outcome) in
   `WorldState` (`hunts: BTreeMap<String, HuntState>`, left out of the
   save while empty, as `skirmishes` is). `NodeCondition::HuntFound(id)`
   opens the delve place (marked `hidden` in the map file). New
   `WorldEvent`s: `RumorHeard`, `TrackFound`, `ColumnMarched`, and the
   arrival and lock outcomes 0087 names.
3. **Triggers**: `WorldCommand::HearRumor` in the rumor's town. In
   `Campaign::clear_node`: a won skirmish (fixed or random) within the
   hunt's reach adds a track once the rumor is heard; a newly won story
   battle moves each live column by its pace, next to `spawn_skirmishes`
   (`Campaign::rng` if the pace rolls); the final story battle settles
   every live hunt as 0087 says before `NewAct`. A column at the delve
   before the player has won it ends the hunt as 0087 says.
4. **Content** (`crates/content/src/worldmap.rs`): the hunt's fields in
   the world map file, validated (its town is a town, its delve place a
   `NodeKind::Delve` whose delve has a `last` floor, the column's path is
   along the map's roads, track places exist). `assets/worldmaps/README.md`
   documents them.
5. **The map** (`world_map/view.rs` and the three skins): the hunt's
   mark on the delve's place once the rumor is heard (shown though the
   place is hidden; the panel says how many tracks of how many), the
   column's mark on its place with its panel line, the delve's mark once
   it opens. A `FindKind` row if 0087 asks for one. If 1082 is done, add
   their icons; if not, add them to 1082's list. Every text by lang key;
   help text through the keymap (`keyboard-input`).
6. **The town**: `open_town` lists `Rumors` beside the shops while the
   town has an unheard rumor (the list shows even for one shop).
7. **The last floor**: the test hunt's delve's `last` battle holds one
   unit of the Beast army (`Faction::Beast`, as in 1095's
   `assets-test/battles/test_beast.ron`), and winning it settles the
   hunt (`WorldEvent` and the place cleared, as 0087 says). A beast that
   retreats deeper ends its battle as a victory whose `fled()` names it
   (1095's starting rule): the delve then does what 0087 says (one more
   floor and the lair again, or the hunt left open for a later visit),
   not a clear. Whatever the hunt itself gives (a track's find, the
   beast's drop) goes through `Campaign::gain_material` (1099; either
   order: if 1099 isn't done, leave that reward out and add a line to
   1099).
8. **Test data** on `assets-test/worldmaps/test_world.ron`: a hunt from
   one of its towns, two track places, a three-floor test delve and a
   column two story battles away.

## Acceptance criteria

- [ ] Before the rumor no tracks count; after it, a won skirmish in reach
      adds one and one out of reach doesn't (unit tests).
- [ ] At the set number of tracks the delve place opens and shows (unit
      test through `is_open` and `is_shown`).
- [ ] The column moves only on a newly won story battle, never on a
      skirmish, a side quest or a retry (property test over random win
      sequences).
- [ ] A column arriving first, and the act locking, give 0087's outcomes
      (unit tests).
- [ ] The same save and the same actions give the same hunt
      (determinism test); a save without hunts still loads.
- [ ] The marks show in the glyph, picture and Playdate looks (snapshot,
      frame-png pictures in the PR); the town lists `Rumors` (screen test).
- [ ] An integration test hears the rumor, wins two track skirmishes,
      plays the test delve with the `BaselineBot` and wins the last floor.
- [ ] What is left to judge by playing is in the Completion notes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: rumor, tracks, reach, opening, the column's march and outcomes.
- Property: the column never moves except on a new story win; tracks
  never exceed the number needed.
- Snapshot / integration: the world map with a hunt's marks; the whole
  hunt played headless.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
