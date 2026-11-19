---
id: "1098"
title: A broken part falls as terrain or is carved into a material
type: feature
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["1096", "1091", "1099"]
nick_input: answer-first
completed:
---

# 1098 — A broken part falls as terrain or is carved into a material

## Context

Part of the legendary-beasts chain from Nick's storefront-hook search
(2026-11-16; 0085–0088, 0772, 1091–1099, 1275–1281). 1096 lets a beast's
parts break. This ticket is what a broken part **leaves**: it **falls as
terrain** (an ice claw's shards freeze the river tiles around it) and
**drops a material**; a unit can **spend an action to carve it** instead
of using the terrain. A part broken during a stolen move (Steal the Turn,
1094) gives the **rare** material. The materials feed forging (1099,
1275) and the grinder bot's balance check (1276).

**The rules come from 0087's answers** in `docs/design/beasts.md`
(question 9: carve *or* use, a choice per broken part, or both
automatically like Monster Hunter's end-of-hunt rewards; question 10:
the rare carve only from a part broken during a stolen move, or from any
break). Where they differ from the sketch here, the decision wins. What
the materials are called and what they forge into are 0087's and 1099's.

**What the game does today, and what this builds on:**

- **Terrain changes in battle** (`crates/core/src/battle.rs`, module doc
  *Terrain* ~:70): the battle's own map copy changes only through
  `Event::TerrainChanged { pos, from, to }` (~:953); a tile cast
  (`cast_on_tile` ~:3379) and a raider's burning (`battle/village.rs`)
  make it. Movement and combat read the changed tile at once. Terrains
  and their glyphs are `assets/data/terrain.ron` (public); their pictures
  are the tilesets' (the test tileset made by `cargo xtask
  test-tileset`). Every map skin already paints any terrain, and the
  scene's `TileView::flashes` mark a change.
- **Element states on tiles** (1091, from 0085's answers in
  `docs/design/magic.md`, *Element states on tiles*): a tile holds one
  element state (`StateId`, kept in `BattleState::tile_states` as a
  `TileState`, read with `tile_state(pos)`) and an element meeting it
  reacts through its `ReactionTable` (`Event::TileStateChanged`, then
  `Event::TerrainChanged` if the terrain changes). A falling part's
  effect on the tiles around it (the shards freezing the river) is one of
  those states, set through 1091's code and events, never a second way
  of changing tiles.
- **Loot** (`battle.rs` `gain_loot` ~:3353): a chest's or a house's
  reward goes to the gold, the pack or the stock (`crates/core/src/item.rs`
  `Stock` ~:625); the stock carries into the campaign after the battle.
  **Materials** are 1099's item kind (`ItemDef::Material(MaterialDef
  …)`), counted in the party `Stock`; a reward outside a battle goes
  through 1099's `Campaign::gain_material`.
- **The action menu** (`crates/ui/src/screens/battle/mode.rs`,
  `MenuEntry` ~:156: `Open`, `Evacuate`…) and the message line after a
  chest (`screens/battle/loot.rs`, `Got Potion.`).
- **Every legal command** (`crates/core/src/legal.rs`): candidates per
  action kind, kept if `BattleState::check` accepts them; the bots and
  random-play tests use it.

## Nick input

**Answer first:** ticket 0087 (questions 9 and 10). Nothing else is
asked here. Verified by tests, `cargo shuyi frame-png` pictures in the PR
and the bots; what is left to judge by playing goes in the Completion
notes for the playtest of the first beast fight (1280).

## Scope

**In:**

- **Part data** (1096's beast table): per part, what it falls as (e.g.
  `falls_as: Some((terrain: "ice_shards", state: Some((state: "frozen",
  radius: 1))))`, the state a 1091 `StateId`), what it drops (a material
  id) and its rare drop. Checked at load: known terrains, states and
  materials.
- **Core** (ADR-0004; the new rules in `battle.rs`'s module doc):
  - **Falling as terrain.** Right after `Event::PartBroken` (1096), the
    part's tile becomes its `falls_as` terrain (`Event::TerrainChanged`)
    and its element state is laid around it through 1091's rules
    (`Event::TileStateChanged` and any reaction it sets off). The battle remembers the remains:
    `BattleState::remains: BTreeMap<Pos, Remains { part, under:
    TerrainId, material }>` (left out of a save when empty, ADR-0039).
  - **Carving** (if 0087 says a choice): a new `UnitAction::Carve`, for a
    player unit standing on the remains or next to them (*starting
    rule*: next to, as Open needs standing on a chest; 0087 wins). It
    gives the material (`Event::Carved { unit, pos, material }`, into
    the stock as 1099 says), turns the tile back to `under`
    (`Event::TerrainChanged`) and ends the action; the element state laid
    by the fall stays (*starting rule*). Refused off the remains
    (`CommandError::NoRemains`), for other factions
    (`CommandError::PlayerOnly`) and on remains already carved. The
    enemy and beast AIs never carve.
  - **Both automatically** (if 0087 says so instead): no action; the
    material is given at the break (`Event::PartDropped { part,
    material }`) and the terrain stays.
  - **The rare material** (0087 question 10): given instead of (or with)
    the common one when the part broke during a stolen move. **Either
    order with 1094:** if 1094 is done, read its stolen-move state; if
    not, keep the rule behind a `BattleState` query that answers
    `false`, and 1094 makes it answer (a line in 1094 says so).
  - `legal.rs` lists `Carve`; the random-play tests cover it.
- **Content:** the remains terrains in `assets/data/terrain.ron` (name,
  glyphs, colours, movement costs and bonuses: *starting rule*, rough
  ground that costs 2 and gives no bonus), their pictures in the test
  tileset; the test beast's parts in the stand-in `assets/data/beasts.ron`
  fall and drop test materials (1099's stand-in).
- **UI:** a `MenuEntry::Carve` shown next to remains, with its help line
  (`battle.help.carve`) and text in `assets/lang/en/ui.ron`; the message
  line after a carve or a drop (`loot.rs`: `Got Rime Shard.`); 1097's
  part bar gets its "Drops …" line if 1097 is done (else 1097 adds it).
  Input is actions only (the `keyboard-input` skill): Carve is a menu
  entry, no new key. The map shows the remains as terrain, so every skin
  paints them with no new skin code (ADR-0038); the Playdate's terrain
  look comes from the same tileset.
- **The bots:** the persona bots may ignore `Carve` (say so in
  `docs/playtesting.md`); the grinder bot that carves everything is 1276.
- `docs/design/beasts.md` gets the starting rules if 0087 left them open.

**Expected size** (ADR-0092, ADR-0101): about 450 production lines (core
~250: remains, the fall, carving, drops; content ~80; UI ~120) and 500 of
tests, about 1,300 changed lines with data and the ticket. Comparable:
0313 (save the village: an action, a tile's state, a terrain change and
a reward) at about half its size.

**Out (do not do):**
- Forging, prices, where materials can be spent (1099, 1275).
- Delve loot and keeping it only through a Way Out (1277).
- The grinder bot and its farming (1276).
- The real beasts' parts, terrains, materials and pictures (1280; the
  frost wyrm's in the private repository's content, `content-change`).
- A part's fall pushing or hurting units (none in 0087's sketch; if
  0087 asks for it, it is its own ticket).

## Implementation steps

1. Read 0087's answers, 1091's element-state code and events, 1096's
   beast table and `PartBroken`, 1099's material kind, `battle.rs`
   (`gain_loot`, `cast_on_tile`, the `Open` action's checks), `legal.rs`,
   `mode.rs` and `loot.rs`.
2. Part data and its checks.
3. Core: the fall, the remains, then carving or the automatic drop as
   0087 says, the rare rule; module docs.
4. `legal.rs`; content (terrains, test tileset, test beast's data).
5. UI: the menu entry, help, message line, the part bar's drop.
6. Snapshots, `frame-png` pictures and `cargo xtask playtest
   test_beast`.

## Acceptance criteria

- [ ] In `test_beast`, breaking a part turns its tile into its remains
      terrain and lays its element state on the tiles 0087 says, through
      1091's events (core tests).
- [ ] Carving next to remains gives the material into the stock, turns
      the tile back and ends the action; it is refused away from remains
      and a second time (tests). Or, if 0087 chose "both", the break
      gives the material and no `Carve` exists (tests).
- [ ] A part broken during a stolen move gives the rare material (a test,
      if 1094 is done; else a test of the query's `false` and a line in
      the Completion notes).
- [ ] After a won battle, the carved materials are in the campaign's
      stock (an integration test through `crates/core/src/campaign.rs`).
- [ ] A suspend and resume and a rewind keep the remains; older saves
      load unchanged (tests).
- [ ] `cargo shuyi frame-png` pictures of remains on the map (glyph,
      picture and Playdate looks), the action menu with Carve, and the
      message line.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: part data checks, the fall, remains, carving and its refusals,
  the drops, the rare rule, `legal.rs` listing Carve only where legal.
- Property: over random `test_beast` play, every remains tile is carved
  at most once and every carve adds exactly one material to the stock.
- Snapshot / integration: the menu entry and message line, the remains in
  three skins, the save round trip, the stock after a won battle.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
