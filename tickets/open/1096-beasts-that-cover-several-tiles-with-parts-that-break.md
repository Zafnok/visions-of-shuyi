---
id: "1096"
title: Beasts that cover several tiles, with parts that break
type: feature
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["1095"]
nick_input: answer-first
completed:
---

# 1096 — Beasts that cover several tiles, with parts that break

## Context

Part of the legendary-beasts chain from Nick's storefront-hook search
(2026-11-16; 0085–0088, 0772, 1091–1099, 1275–1281). 1095 gives the game
a beast army and its Beast Phase, with single-tile beasts. This ticket
makes a beast **cover several tiles, each body tile a part** (head, ice
claw, tail…) **with its own HP; each beast attack belongs to a part, and
breaking the part removes that attack**. 1097 then shows the player which
part is about to attack and what breaking one does; 1098 says what a
broken part leaves on the map.

**The rules come from 0087's answers** in `docs/design/beasts.md`
(question 2: each body tile is a part, as with Three Houses' demonic
beasts; question 8: whether damage to a part also comes off a total,
whether the part you hit strikes back). Where they differ from the
sketch here, the decision wins. The beasts' story framing is 0087's and
0772's, in the private repository; none of it goes here.

**What the game does today:**

- **One unit, one tile.** `Unit.pos` (`crates/core/src/unit.rs` ~:182)
  is a single tile, and the battle compares positions directly in many
  places: occupancy in `BattleState` (`crates/core/src/battle.rs`
  ~:2951, ~:3437, `arrive` ~:3737), distances (`Pos::manhattan` in
  `battle.rs` ~:2730, ~:2804, `battle/arts.rs` ~:535, ~:596,
  `battle/skills.rs`, `ai.rs` ~:570), movement (`movement.rs`:
  `reachable` ~:301 and `search` ~:313 put every other unit on its one
  tile; `mover_info` ~:239 gives the mover's movement type and Mov;
  0334 routes the movement type through one place on `Unit`).
- **Acting** (`Command::Act { unit, dest, action }`, `battle.rs` ~:863)
  moves one unit along `plan_move`'s path (~:2518) and makes it act; the
  unit is then `acted`.
- **Falling**: a unit at 0 HP leaves the map (`Event::UnitFell`, ~:3634)
  or goes down (`Defeat`, `unit.rs` ~:78).
- **The map scene** (`crates/ui/src/map_view/scene.rs`): one `UnitView`
  per unit, at `pos`, painted by the glyph skin
  (`map_view/glyph/units.rs`, `draw_unit` ~:57), the picture skin
  (`map_view/sprite/units.rs`, its picture from
  `Tileset::unit_picture`, `crates/content/src/tileset.rs` ~:99, by
  character then class) and the Playdate's (`map_view/playdate/units.rs`).
  The engine's kit scene (`shuyi_ui::grid::scene::MapUnit`) knows one
  `pos` per unit. While a unit walks, the screen draws it at
  `Mode::drawn_pos` (`screens/battle/mod.rs` `drawn_pos` ~:1665,
  `add_units` ~:2550; `walk.rs`).

**The model this ticket recommends** (a technical choice: the session
confirms it in an ADR with the `write-adr` skill): **each part is a
`Unit`**, on its own tile, with its own HP, stats, loadout (its attack)
and map label, and the parts of one beast form a **body**. Then
targeting, the forecast, combat, counters, EXP, the danger zone, threat
arrows and the map's units all work per part with no change, and what is
new is only what makes the body one creature: it moves as one, acts once
per phase, and loses a part when the part breaks. The other model, one
`Unit` with a footprint and a part list, would change every target from
`UnitId` to a part and every HP read; the ADR says why it wasn't picked.

## Nick input

**Answer first:** ticket 0087. Nothing else is asked here. Verified by
tests, `cargo shuyi frame-png` pictures in the PR and the bots; what is
left to judge by playing goes in the Completion notes for the playtest of
the first beast fight (1280).

## Scope

**In:**

- **Core** (ADR-0004: only `Command`s and `Event`s change the state; a
  new module, e.g. `crates/core/src/beast.rs`):
  - `BodyId`, `Body { id, name, parts: Vec<UnitId>, core: UnitId }` in
    `BattleState` (e.g. `bodies: Vec<Body>`, from `BattleSetup`), and
    `Unit::body: Option<BodyId>` (left out of a save when `None`,
    ADR-0039). Every part is `Faction::Beast` (1095).
  - **Moving as one.** A part's reach is its body's: a step is allowed
    only if every other part's tile, moved the same way, can be entered
    by the body's movement type, isn't blocked and holds no unit but its
    own parts; it can stop only where every moved tile is free. The step
    costs the most of its tiles' costs (*Claude's starting rule*).
    `movement::reachable` answers this for a part, so `plan_move`, the
    AI, `legal_commands`, the danger zone and threat arrows all get it.
  - **Acting once.** `Command::Act` with a part as `unit` moves the
    whole body by `dest − unit.pos` (one `Event::UnitMoved` per part, the
    acting part first, then body order) and makes that part act; every
    part is then `acted`. A sibling's own `Act` that phase is refused
    (`CommandError::AlreadyActed`). The AI needs nothing new: each part
    "decides" and the best-scoring one acts, its siblings then done.
  - **Breaking.** A part at 0 HP **breaks**: `Event::PartBroken { body,
    part }`, then it leaves the map as a fall does; its attack goes with
    it. The body goes on with the parts left (*starting rule*: a broken
    part comes off; 0087 question 8 may say it stays as a broken tile).
    When the `core` part breaks (or the last part, or a total HP pool if
    0087 has one: `Body::hp`), the beast is defeated and every part
    leaves (`Event::UnitFell` each; `is_defeated(core)` true), so a
    `DefeatUnit` objective names the core part. A part doesn't counter
    if 0087 says the part you hit doesn't strike back (a flag per part).
  - 1095's retreat moves the whole body to `fled`.
- **Content** (`crates/content`): a new beast table (e.g.
  `crates/content/src/beast.rs` reading `data/beasts.ron`): per beast its
  name, movement type and parts, each with an id, name, map label,
  offset from the core part, stats (a template and level, as an enemy
  entry), loadout (its attack) and whether it is the core. A battle file
  places one with `beasts: [(beast: "…", pos: (x, y), ai: …,
  retreats_on: …)]`; `BattleLoader` expands it into part units and a
  `Body` and checks every part's tile is on the map, enterable and free.
  A public stand-in `assets/data/beasts.ron` with one test beast of
  three or four parts (ADR-0121's table gets its row; if the beast
  table should rather take test entries from `assets-test/data/` as
  ADR-0057's five tables do, the ADR says so), the format in
  the stand-in's notes and the battle entry in
  `assets/battles/README.md`, and `assets-test/battles/test_beast.ron`
  using it.
- **UI** (ADR-0038: the map scene, painted by every skin): parts are
  units, so each is a `UnitView` on its tile with its own label, HP bar
  and picture (the tileset's picture for the part's class or id; the test
  tileset by `cargo xtask test-tileset`). `UnitView` gets the body
  (e.g. `body: Option<BodyId>`), its text says `body=N`, and while the
  body walks every part is drawn at its own tile plus the walking part's
  offset (`Mode::drawn_pos`). The glyph, picture and Playdate skins draw
  nothing new unless a part's look needs it.
- `docs/design/beasts.md` gets the starting rules if 0087 left them
  open; `battle.rs`'s and `movement.rs`'s module docs; the ADR.

**Expected size** (ADR-0092, ADR-0101): about 850 production lines (core
~450: the body, its reach, acting and breaking; content ~250; UI ~150)
and 800 of tests, about 1,900 changed lines with docs, data and the
ticket. Comparable: 0313 (save the village), a battle rule with its own
state, events, content and AI. If the reach or the content runs past
this, **split before coding** (`write-ticket`): this ticket keeps the
core and a test beast built in code, and a second ticket (`blocked_by`
this one, named in 1097's and 1098's `blocked_by`) takes the beast table
and the battle file.

**Out (do not do):**
- The glowing next part, the part bar in the forecast, the beast's combat
  picture, the target cycle's part names (1097).
- What a broken part leaves on the map, carving, drops (1098).
- One picture drawn across several tiles: each part has its own picture
  here. If a big picture is wanted later, the engine's `MapUnit` knows
  one `pos` per unit: that is an `engine-friction` call then, never a
  wait.
- Beasts that fly, swim or push units.
- The real beasts and their numbers (1280).

## Implementation steps

1. Read 0087's answers, 1095's code, `movement.rs` (`search`,
   `mover_info`, `threat_area`, `ThreatPlanner`), `battle.rs`
   (`plan_move`, `act` ~:3151 and the fall), `ai.rs`'s ordering,
   `legal.rs`, `crates/content/src/battle.rs` (`RawEnemy`, unit
   building) and `scene.rs`.
2. Write the ADR (parts are units in a body), then `beast.rs`, the body
   in `BattleState` and `BattleSetup`, and `Unit::body`.
3. The body's reach in `movement.rs`; route every caller through
   `reachable` (`grep -rn "reachable\|search(" crates/core`), and add a
   test that each of them (danger zone, threat arrows, the AI) sees a
   body's reach.
4. Acting and breaking in `battle.rs`; the AI's tests.
5. Content: the table, the battle entry, checks, the test data.
6. UI: the scene field, the walk; snapshots and pictures.
7. Run `cargo xtask playtest test_beast` and the Act 1 bots.

## Acceptance criteria

- [ ] In `test_beast`, the beast moves as one: after its `Act` every
      part stands moved by the same offset, and no part ever ends on a
      tile it can't enter or that another unit holds (core tests).
- [ ] A body acts once per Beast Phase; a sibling's `Act` after it is
      refused (tests).
- [ ] Breaking a part emits `Event::PartBroken`, removes its attack (the
      AI's next choice never uses it) and leaves the other parts
      fighting; breaking the core defeats the beast and wins a
      `DefeatUnit` on it (tests).
- [ ] A player unit's move range and the threat arrows treat every part's
      tile as a hostile unit's (tests).
- [ ] A suspend and resume and a rewind keep bodies, parts and broken
      parts; older saves load unchanged (tests).
- [ ] `cargo shuyi frame-png` pictures of the test beast on the map
      whole, mid-walk and with a part broken, in the glyph, picture and
      Playdate looks.
- [ ] `cargo xtask playtest test_beast` ends with each bot's result in
      the PR.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the body's reach (walls, water, units in the way, its own parts
  never blocking it), acting, breaking, defeat, the retreat of a body,
  content checks (overlapping parts, a part off the map).
- Property: for random boards and bodies, every tile of a body's reach
  keeps the whole body on enterable, free tiles; a body's parts keep
  their offsets after any command sequence (`legal.rs` random play).
- Snapshot / integration: the scene's text with a body; the three skins;
  the save round trip; the bots on `test_beast`.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
