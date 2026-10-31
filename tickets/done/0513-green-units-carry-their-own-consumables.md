---
id: "0513"
title: "Green units carry their own consumables and use them"
type: feature
milestone: M4 Enemy AI
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-10-22
---

# 0513 — Green units carry their own consumables

## Context

Nick, ticket 0044 (2026-10-21): green units **carry their own
consumables, set per battle**, never drawn from the player's pack
(`docs/design/weapons-and-items.md`, *Battle pack*). Today only player
units use items (`UnitAction::UseItem { pack_index, target }` in
`crates/core/src/battle.rs`, from the shared pack), and the AI that plays
the Other phase (`crates/core/src/ai.rs`, ticket 0501) never uses one.
Enemies still carry none (Nick, 0501).

The rules (0044; the *the agent's starting rules* are marked in the doc):
the chapter data gives a green unit its items; it uses one like a player
unit does (itself or an adjacent green or player unit, and its action
ends); its items can't be traded or taken; unused ones are gone when the
battle ends. Non-combat green units may carry items too.

## Nick input

None. Verified by tests and the bots. When a green unit uses an item is a
*the agent's starting rule* (step 3) for the Act 1 playtest (1064) to judge.

## Scope

**In:**
- An optional item list on a green unit in the battle file
  (`assets/battles/*.ron`), loaded and validated by `trpg-content`
  (consumables only; an item on a player or enemy unit is an error).
- A per-unit carried-items list in `core`'s battle state, and a way to use
  one (a new `UnitAction` variant or a source field on `UseItem`; pick the
  smaller change and say why in the module docs).
- The Other-phase AI uses them (step 3).
- `assets/battles/README.md` documents the field.

**Out (do not do):**
- Enemies with items (Nick, 0501: never).
- Trading with or looting green units.
- Deciding which Act 1 green units carry what: each chapter's battle
  ticket does that in its data.

## Implementation steps

1. `trpg-content`: the field on a unit in the battle file, e.g.
   `items: ["potion", "potion"]`, default empty; errors for an unknown
   item, a non-consumable, or items on a non-green unit (ADR-0005: report
   every error).
2. `core`: the green unit keeps its list in the battle state; using one
   heals as the pack's items do (`ConsumableEffect`), removes it, emits the
   same event as a pack item (with whose item it was) and ends the action.
   It never touches the pack or the stock; nothing carries over after the
   battle. Replays and saves include the list.
3. `ai.rs`: *the agent's starting rule* (FE allied NPCs): a green unit
   that holds an item uses it on itself when below half its max HP and no
   attack it could make would kill a foe; otherwise it acts as today. It
   does not heal others with items. Write the rule in the module docs.
4. A test battle in `assets-test/` with a green unit carrying a Potion.

## Acceptance criteria

- [x] `trpg-content` rejects items on a player or enemy unit and a non-consumable item, in one report.
- [x] Core unit test: a green unit uses its Potion, heals by its amount (capped at max HP), the item leaves its list, the player's pack is unchanged, its action ends.
- [x] AI unit test: a wounded green unit with a Potion and no kill uses it; one with a kill attacks.
- [x] A save/replay round trip keeps a green unit's remaining items.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: content validation, the use in core, the AI choice.
- Property: a green unit's item use never changes the player's pack or stock.
- Snapshot / integration: none needed (the battle log line is the existing
  item-use line).

## Completion notes

Done.

- **Battle files** (`trpg-content`): an entry of `enemies` or
  `reinforcements` takes `faction: Ally` (a green unit; `Enemy` is the
  default) and `items: [...]`. Errors, all in one report: `items` on a
  player slot or an enemy, an unknown item, a non-consumable; also a green
  `boss`, a green unit with a `defeat`, and `faction: Player`/`Neutral`.
  `DefeatUnit` now names only enemies (it accepted any non-player unit).
- **Core**: `Unit::carried` (empty for everyone else, left out of saves
  when empty so older saves load) and a new `UnitAction::UseOwnItem
  { index, target }`, green units only. It shares the pack item's checks
  and effect, emits the same `ItemUsed` and `Healed`, ends the action and
  never touches the pack or stock. A new action rather than a source field
  on `UseItem`: every `UseItem` already written (menus, bots, play records,
  tests) stays as it is. The legal-action list and the rewind log know it.
- **AI**: the starting rule below, written in `ai.rs`'s module docs.
- **Test battle** `assets-test/battles/test_green.ron`: an unarmed
  villager with two Potions next to a raider; the bots' rule-battle test
  plays it and sees it drink (3 of 5 seeds).

**Deviations.**
- Battle files had no way to place a green unit at all, so this ticket
  added `faction: Ally` on an `enemies` entry (the list was already "the
  other units on the map").
- `ItemUsed` has no new "whose item" field: only green units carry items
  and only player units use the pack, so its `unit` already says whose it
  was, and the log line stays the existing one.
- The save/replay criterion is a save round trip of the battle state
  (`a_green_units_items_survive_a_save`) plus the bots' replay of
  `test_green` (same commands, same events, same end).

**the agent's starting rules** (for the Act 1 playtest, 1064):
- A green unit drinks one of its items, on itself, where it stands
  (no move first), when it is **below half its max HP** and **no attack it
  could make this turn has any chance to kill** a foe; else it plays as
  before. With several items it uses the first in its list.
- It never uses items on others (the battle allows it; the AI doesn't).
- Battle files can't yet mark a green unit as a non-combat unit (`Role`, ticket 0856);
  the test villager is simply unarmed.

Engine friction: estimate 150, measured 148 (1.0×), under 2×.
Follow-up: 0856 (battle files mark a unit as non-combat).
