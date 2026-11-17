---
id: "0334"
title: "Riders get off and back on their horse, as 0083 decided"
type: feature
milestone: M2 Core rules
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["0083"]
nick_input: none
completed:
---

# 0334 — Riders get off and back on their horse

## Context

Nick played the Act 1 build (Chapters 1 to 4) on 2026-11-08:

> "We should allow manual dismount and remount for mounted class to avoid
> effectiveness (strategy) -- however, treat like fire emblem does, after
> you make an action you can't take a free action of dismounting or
> remounting."

What the game does today: there is no dismount anywhere. A unit's
movement type and tags come from its **class** (`ClassDef.movement_type`,
`tags`, `assets/data/classes.ron`; Rider ~:85: `movement_type:
"mounted"`, Mov 7, `tags: [Mounted]`). They decide:

- **Moving:** `mover_info` (`crates/core/src/movement.rs` ~:240–257) reads
  the class's movement type; terrain costs per movement type are in
  `assets/data/terrain.ron` (mounted: water impassable, floor costs 1,
  `docs/design/terrain.md` ~:102–106).
- **Being hit:** spears are effective against `Mounted` (×2 might,
  `items.ron:50`; applied in `side`, `crates/core/src/combat.rs` ~:583–595),
  read from the defender's tags.
- **The enemy AI's flow field** (`crates/core/src/ai.rs`, `decide` ~:465,
  `self.flow(unit.faction, class.movement_type)`), the danger zone and
  threat arrows (`movement.rs` ~:508–531), and the persona bots' threats
  (`crates/bots/src/persona.rs` ~:403).

Act 1's mounted units: Tamsin (`sergeant`, Rider) on the player's side;
`hound_rider`s among the enemies (Chapters 2 to 4, and Chapter 4's
turn-7 riders).

**0083 decided it (2026-11-17)**; the rules are in `terrain.md`,
*Getting off and back on a horse*. In short:

- **Who:** every class with movement type `mounted` (Rider, Lancer,
  Outrider, Iron Rider, High Lancer, Juggernaut, Windrunner); never
  fliers.
- **On foot:** movement type `foot` (Iron Rider and Juggernaut:
  `armored`, keeping `Armored`); **Mov −2** (Tamsin 7 → 5); **Spd +2**
  (Tamsin 5 → 7); **no `Mounted` tag**; weapons, other stats and skills
  (Lance Rush, Charge 1) unchanged.
- **When:** a **free action** (it doesn't use the unit's action, like
  `Command::Equip`, `crates/core/src/battle.rs` ~:325, ~:859), **before
  moving** (then the move uses the new Mov and costs) **or after moving**
  (no more moving, but it can still act), **not both** in one turn, and
  **never after the unit has acted**. Mounting follows the same rules;
  no mounting on a tile a horse can't enter, such as a river (*the
  agent's starting rule*).
- **Where:** indoors changes nothing. **Enemies never do it.** **Every
  battle starts mounted.**
- **Control:** an action-menu line, **Dismount** / **Mount**, plus an
  **optional rebindable action with no default key** (Nick).
- **Look:** deferred (Nick: "dont need to block mechanic on it"): the
  glyph mark and the combat picture on foot are 0089's; MattWalkden's
  mounted map sprites are 0499's. This ticket puts the state in the map
  scene and paints nothing new.

## Nick input

None (0083 answered it). Nothing is asked here. Verified by tests, `cargo shuyi
frame-png` pictures in the PR and the bots; what is left to judge by
playing goes on the Act 1 playtest's checklist (1064).

## Scope

**In:**

- **Core (ADR-0004: state changes only through a `Command` and its
  `Event`s).** A unit's mount state in the battle (e.g. `Unit.on_foot:
  bool`), and whether it changed this turn (once a turn), both serialised
  with the battle so a suspend and a rewind keep them. Both of 0083's
  moments are needed:
  - **Before moving:** a new free command, e.g. `Command::SetMount
    { unit, on_foot }`, accepted for a ready unit of the current phase
    whose class is mounted and that hasn't moved, acted or already
    changed this turn; refused otherwise with a `CommandError`. Its
    event, e.g. `Event::MountChanged { unit, on_foot }`. The move that
    follows uses the new state.
  - **After moving, before the action** (the action menu's moment): the
    move and the action are one `Command::Act { unit, dest, action }`, so
    the change rides on it (a field such as `mount: Option<bool>`,
    applied after the move and before the action), and the move is
    checked with the state the unit started in. Refused if it already
    changed this turn. "Mount" is also refused on a tile a mounted unit
    can't enter (a river; *the agent's starting rule*, 0083).
  - What the state changes is one place that the rest asks, e.g.
    `Unit::movement_type(&ClassTable)`, `Unit::tags(&ClassTable)`,
    `Unit::mov(..)` and the Spd the stats read: on foot, movement type
    `foot` (`armored` for a class tagged `Armored`), no `Mounted` tag,
    **Mov −2** and **Spd +2** (*tunable*: two named constants, with
    `terrain.md`'s numbers in their doc). Weapons, skills and the other
    stats don't change. `mover_info`, `combat_input`'s tags and Spd, the
    AI's flow field (keyed by movement type), the danger zone and threat
    arrows, and the bots' threats all go through it, never the class
    directly.
  - It doesn't last: every battle starts mounted (the units the campaign
    builds have `on_foot: false`).
- **The enemy side:** enemies never do it. `ai.rs` only reads the mount
  state through the one place above (a player rider on foot is a foot
  unit to its flow field and its targeting).
- **The persona bots:** they don't dismount in this ticket (they play as
  before; say so in `docs/playtesting.md`). Their threat model reads the
  mount state through the one place.
- **UI (`crates/ui`):**
  - The action menu entry: one `MenuEntry` (`Dismount`, or `Mount` when
    on foot) in `crates/ui/src/screens/battle/mode.rs` (`menu_entries`
    ~:343, `item` ~:156–210), shown only for a unit whose class is
    mounted and that hasn't changed this turn, with its help line
    (`battle.help.*`, `crates/ui/src/screens/battle/mod.rs` ~:2089) and
    its text in `assets/lang/en/ui.ron`. **Before moving:** choosing it
    with the unit's own tile as the destination sends `SetMount` and goes
    back to choosing a destination, with the move range redrawn for the
    new state (check how `mode.rs` opens the menu on the unit's own
    tile). **After moving:** it rides on the `Act` and the menu stays
    open (the unit can still attack, use an item or wait).
  - **A new input action** (e.g. `Action::ToggleMount`) with **no default
    key** on any layout, pad or Playdate (Nick: "optional, not bound by
    default key"), listed on the key-binding screen so a player can bind
    it; it does what the menu line does for the selected unit. Follow the
    `keyboard-input` skill (an unbound action must pass `check-keys`; if
    the keymap can't express "no default", make the smallest engine-side
    or game-side change, per `engine-change`).
  - **The map (ADR-0038):** the map scene's `UnitView`
    (`crates/ui/src/map_view/scene.rs` ~:86) gets `on_foot: bool`. The
    skins paint **nothing new** (0089 decides the glyph mark and the
    combat picture), but for one thing 0499 left to this ticket: **the
    sprite skin must pick the picture on foot**. 0499 made the mounted
    classes' map sprites MattWalkden's riders and named the Tiny Tales
    knight on foot for each of them in the tileset's `units.on_foot`
    (`Tileset::on_foot_picture(&ClassId)`,
    `crates/content/src/tileset.rs`); `paint_pictures`
    (`crates/ui/src/map_view/sprite/units.rs`) uses it in place of
    `unit_picture` for a unit whose `on_foot` is true (a named
    character's own picture first, as `unit_picture` does), with a
    test that reads the scene and the tileset.
  - The side panel shows the unit's Mov and Spd on foot (it reads the
    unit, so this is a test that it does).
  - The forecast's `!` (effective) goes away by itself for a rider on
    foot; a test says so.
- The rules' doc in `battle.rs` lists the new command and the `Act`
  field; `terrain.md` already says the rule (0083).

**Expected size** (ADR-0101): about 1,700–2,000 changed lines: core
command, state and the one place that answers movement type, tags, Mov
and Spd (~300 production, ~450 tests), the menu entry, help and the
unbound action (~150 production, ~300 tests), the scene field (~20),
docs (~100), this file (~300). The skins paint nothing new, which makes
room for the on-foot stats. If the estimate passes 2,000, **split
before coding** (`write-ticket`): this ticket keeps the core, and a
Battle UI ticket (04xx, `blocked_by` this one) takes the menu entry, the
action and the scene field. 1064's `blocked_by` then names both.

**Out (do not do):**
- Dismounting forced by terrain (indoors): 0083 said no.
- Enemies dismounting: 0083 said no.
- Flying units (none in Act 1; `progression.md` ~:766, fliers are late).
- New mounted classes or changes to Rider's numbers.
- How a rider on foot looks (0089) and MattWalkden's riders (0499).

## Implementation steps

1. Read `terrain.md` (*Getting off and back on a horse*), `movement.rs`
   (`mover_info`, `reachable`, `threats_at`), `combat.rs` (`side`'s
   tags), `ai.rs` (`decide`, `flow`), `battle.rs` (`Command`, `Event`,
   how `Equip` is accepted and applied, the save format and rewind), and
   the action menu (`mode.rs`, `mod.rs`).
2. Core: the state, the command, the `Act` field and the event, the one
   place for movement type, tags, Mov and Spd; route every reader
   through it (`grep -rn "movement_type\|\.tags\|\.mov\b" crates/core
   crates/bots`).
3. Core tests (below).
4. UI: the menu entry and help, the unbound action, the scene field;
   snapshots and `frame-png` pictures of the menu.
5. Docs.
6. Run the winnable tests of Chapters 1 to 4 (they shouldn't move: the
   recorded playthroughs never dismount).

## Acceptance criteria

- [ ] A ready Tamsin who hasn't moved can get off (`SetMount` accepted,
      its event emitted); then a Hound's spear forecast against her has
      no ×2 (`effective` false), her move range is a foot unit's with
      Mov 5, and her Spd is 7 (tests).
- [ ] She can instead ride, then get off as part of `Act` and still
      attack in the same command (test), but not both before and after
      moving in one turn (a `CommandError`).
- [ ] Getting back on follows the same rules; neither is accepted after
      she has acted, nor "Mount" in a river (tests: a `CommandError` on
      each).
- [ ] An Iron Rider on foot moves as `armored`, Mov 4, keeping `Armored`
      (test).
- [ ] Enemy riders never change; a new battle starts every rider mounted
      (tests).
- [ ] The action menu shows Dismount / Mount for a mounted class only,
      before and after moving, and not once it has changed this turn
      (tests through the battle screen); the new action has no default
      key and `check-keys` passes.
- [ ] The map scene's unit says it is on foot (test).
- [ ] A suspend and resume, and a rewind past the change, keep the right
      state (tests).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the command's and the `Act` field's acceptance and refusals,
  movement, tags, Mov and Spd on foot, combat effectiveness, the AI
  reading the state (`crates/core`).
- Property: getting off on one turn and back on the next leaves the unit
  as it was; a refused command changes nothing.
- Snapshot / integration: the action menu with the entry; the scene with
  a rider on foot; save and rewind round trips.

## Completion notes

*(Filled in by the session that completes the ticket.)*
