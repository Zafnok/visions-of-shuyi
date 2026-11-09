---
id: "0334"
title: "Riders get off and back on their horse, as 0083 decided"
type: feature
milestone: M2 Core rules
model: opus-5.5 / gpt-6.1-sol
effort: high
status: blocked
blocked_by: ["0083"]
nick_input: answer-first
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

**0083** asks Nick how getting off and on works. His rule above is
fixed: it is a **free action** (it doesn't end the unit's action, like
`Command::Equip`, `crates/core/src/battle.rs` ~:325, ~:859), and only
**before the unit acts**. 0083 settles the rest (whether "acts" includes
moving; what changes on foot: movement type, Mov, the Mounted tag, stats,
weapons, Lance Rush and Charge; whether enemies do it; whether it lasts
past the battle; how a rider on foot looks).

## Nick input

**Answer first:** ticket 0083 (with its mockups of a rider on foot in
both looks). Nothing else is asked here. Verified by tests, `cargo shuyi
frame-png` pictures in the PR and the bots; what is left to judge by
playing goes on the Act 1 playtest's checklist (1064).

## Scope

**In:**

- **Core (ADR-0004: state changes only through a `Command` and its
  `Event`s).** A unit's mount state in the battle (e.g. `Unit.on_foot:
  bool`, serialised with the battle so a suspend and a rewind keep it).
  The likely shapes, by 0083's answer on *when*:
  - **Before moving only:** a new free command, e.g. `Command::Dismount
    { unit }` / `Command::Mount { unit }` (or one `SetMount { unit,
    on_foot }`), accepted for a ready unit of the current phase that
    hasn't moved or acted, refused otherwise with a `CommandError`;
    its event, e.g. `Event::Dismounted { unit }` / `Event::Mounted
    { unit }`. Movement then uses the new state.
  - **After moving, before the action** (the action menu's moment): the
    move and the action are one `Command::Act { unit, dest, action }`, so
    a dismount there has to ride on it (a field such as `mount:
    Option<MountChange>` applied after the move and before the action),
    and the move is still checked with the state the unit started in.
  - What the state changes is one place that the rest asks, e.g. a
    `Unit::movement_type(&ClassTable)` and `Unit::tags(&ClassTable)` that
    swap in `foot` and drop `Mounted` when on foot (plus Mov, stats or
    weapons if 0083 says so). `mover_info`, `combat_input`'s tags, the
    AI's flow field (keyed by movement type), the danger zone and threat
    arrows, and the bots' threats all go through it, never the class
    directly.
  - Whether it lasts: reset to mounted at battle start (on the unit the
    campaign builds) unless 0083 says it carries over.
- **The enemy side:** as 0083 says. If enemies never do it, nothing
  changes in `ai.rs` but reading the mount state through the one place
  above. If they do, the AI's rule for when (*the agent's starting rule*,
  written in `ai.rs`'s module doc with an example) and its tests.
- **The persona bots:** they may ignore it in this ticket (they play as
  before; say so in `docs/playtesting.md`). If the bands move because the
  enemy does it, the bots' threat model reads the mount state.
- **UI (`crates/ui`):**
  - The action menu entry: a `MenuEntry` (e.g. `Dismount` / `Mount`, one
    entry whose text follows the state) in
    `crates/ui/src/screens/battle/mode.rs` ~:156–210, shown only for a
    unit whose class is mounted and only at the moment 0083 allows,
    with its help line (`battle.help.*`, `crates/ui/src/screens/battle/mod.rs`
    ~:2089) and its text in `assets/lang/en/ui.ron`. If the answer is
    "before moving", the entry must be reachable before a destination is
    picked (for instance when the unit is chosen as its own destination,
    if that opens the menu: check how `mode.rs` handles it), and the move
    range redraws after it. Input is actions only (the `keyboard-input` skill): no new
    action unless 0083's answer needs one; then it gets a default key on
    every layout, a pad button and a Playdate button or chord.
  - **The map (ADR-0038):** the map scene's `UnitView`
    (`crates/ui/src/map_view/scene.rs` ~:86) gets the mount state (e.g.
    `on_foot: bool`); every map skin paints it as 0083 picked: the glyph
    skin (`map_view/glyph`), the sprite skin (`map_view/sprite/units.rs`,
    `Tileset::unit_picture`, `crates/content/src/tileset.rs` ~:99, which
    may need an on-foot picture per character or class in the tileset
    file) and the Playdate skin (`map_view/playdate`). If the on-foot
    picture comes from the bought art, the private repository gets it
    and `assets-private.rev` moves in this PR (`cargo shuyi
    private-assets --pin`; AGENTS.md, *Bought art*).
  - The unit info screen shows the unit's Mov and movement as on foot
    (it reads the unit, so this is a test that it does).
  - The forecast's `!` (effective) goes away by itself for a rider on
    foot; a test says so.
- `docs/design/terrain.md` (*Mounted*) and the battle rules doc
  (`turn-structure.md` or wherever 0083 recorded it) say the rule; the
  rules' doc in `battle.rs` lists the new command.

**Expected size** (ADR-0101): about 1,600–2,000 changed lines: core
command, state and the one place that answers movement type and tags
(~250 production, ~400 tests), the menu entry and help (~120 production,
~250 tests), the scene field and three skins (~150 production, snapshot
tests), docs (~100). If 0083's answer adds on-foot stats or weapons, or
enemies that dismount, it won't fit: **split before coding**
(`write-ticket`): this ticket keeps the core and the AI, and a Battle UI
ticket (04xx, `blocked_by` this one) takes the menu entry, the scene and
the skins. 1064's `blocked_by` then names both.

**Out (do not do):**
- Dismounting forced by terrain (indoors) unless 0083 asks for it.
- Flying units (none in Act 1; `progression.md` ~:766, fliers are late).
- New mounted classes or changes to Rider's numbers.

## Implementation steps

1. Read 0083's answer, `movement.rs` (`mover_info`, `reachable`,
   `threats_at`), `combat.rs` (`side`'s tags), `ai.rs` (`decide`,
   `flow`), `battle.rs` (`Command`, `Event`, how `Equip` is accepted and
   applied, the save format and rewind), and the action menu
   (`mode.rs`, `mod.rs`).
2. Core: the state, the command(s) and event(s), the one place for
   movement type and tags; route every reader through it (`grep -rn
   "movement_type\|\.tags" crates/core crates/bots`).
3. Core tests (below); then the AI per 0083.
4. UI: the menu entry and help, the scene field, each skin; snapshots and
   `frame-png` pictures.
5. Docs.
6. Run the winnable tests of Chapters 1 to 4 (they shouldn't move: the
   recorded playthroughs never dismount) and the bots on Chapters 2 to 4
   if the enemy side changed.

## Acceptance criteria

- [ ] A ready Tamsin who hasn't acted can get off (the command is
      accepted, its event is emitted), and then a Hound's spear forecast
      against her has no ×2 (`effective` false) and her move range is a
      foot unit's (tests).
- [ ] She can get back on under the same rule, and neither is accepted
      after she has acted (tests: a `CommandError` on each).
- [ ] At the moment 0083 allows, the action menu shows the entry for a
      mounted unit only (tests through the battle screen).
- [ ] The map scene's unit says it is on foot and each skin paints it as
      0083 picked (`frame-png` pictures of glyph, sprite and Playdate in
      the PR).
- [ ] A suspend and resume, and a rewind past the command, keep the right
      state (tests).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the command's acceptance and refusals, movement and tags on
  foot, combat effectiveness, the AI reading the state (`crates/core`).
- Property: getting off then on (when both are allowed) leaves the unit
  as it was; a refused command changes nothing.
- Snapshot / integration: the action menu with the entry; the map in each
  skin with a rider on foot; save and rewind round trips.

## Completion notes

*(Filled in by the session that completes the ticket.)*
