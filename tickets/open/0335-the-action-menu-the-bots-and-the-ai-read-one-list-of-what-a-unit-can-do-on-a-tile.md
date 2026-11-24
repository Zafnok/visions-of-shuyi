---
id: "0335"
title: "The action menu, the bots and the AI read one list of what a unit can do on a tile"
type: infra
milestone: M2 Core rules
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
blocks: ["1064"]
nick_input: none
completed:
---

# 0335 — The action menu, the bots and the AI read one list of what a unit can do on a tile

## Context

From the friction sweep of 2026-11-17
([report](../../docs/research/friction-2026-11-17.md), *A new unit
action fans out to 6-8 files and 4 crates*; bucket B7, code structure,
with 0259, and the amendments to 0334, 0257, 1291 and 1236).

`UnitAction` (`crates/core/src/battle.rs:718`, 12 variants) is matched
in 19 non-test files across 4 crates (core, ui, bots, content):
`battle.rs` 34, `ui/screens/battle/rewind.rs` 13, `core/legal.rs` 13,
`ui/screens/battle/mode.rs` 11, `core/ai.rs` 9, `art_list.rs` 8,
`bots/persona.rs` 6, `shop.rs` 4, `bots/measures.rs` 4, and 10 files
with one or two. Whether an action is on offer at a tile is said in
three places:

- **The menu** (`menu_entries`, `mode.rs:343`–`380`) asks core for
  almost every entry (`can_seize`, `can_escape`, `can_evacuate`,
  `talk_targets`, `attacks.check(&UnitAction::Open)`), but the Shop
  entry restates core's rule: `player && state.map().shop(dest).is_some()`
  (`mode.rs:361`–`366`) against `plan_shop` (`battle.rs:3004`–`3008`).
  The UI keeps its own `MenuEntry` enum (`mode.rs:156`, 13 variants),
  each with a text key (`mode.rs:193`–`205`) and a mapping back to
  `UnitAction` (`mode.rs:1111`).
- **The bots' candidates** (`legal.rs`, `actions` ~:190–270) list
  `Seize`, `Escape`, `Open` and `Evacuate` at every tile and a shop's
  visits wherever `map().shop(dest)` is some, then let `apply` refuse.
- **The AI** restates a rule too: `evacuation` (`ai.rs:1030`) checks
  "player" and "a standing house" itself, which is `plan_evacuate`
  (`battle/village.rs:155`).

The drift today is small (one restated rule in the UI, one in the AI),
but the next action pays the whole fan-out. 0334 (getting off and back
on a horse) is about to add one, and its scope already plans a new
`MenuEntry`. This ticket comes first so 0334 adds one row.

This is the `engine-friction` skill's outcome 3, a game-side structure
(ADR-0092): the rules are the game's, and nothing names an engine gap.

## Nick input

None. Nothing a player sees changes: the same menu lines in the same
order, the same bot and AI choices. Verified by tests, the winnable
tests of Chapters 1 to 4 and the bots' records replaying unchanged.

## Scope

**In:**
- `crates/core/src/battle/actions.rs` (new, an `impl BattleState` block
  as `arts`, `skills` and `village` already are):
  - `pub enum ActionKind`: the actions whose offer at a tile is a yes or
    no rule of core: `Seize`, `Escape`, `Shop`, `Open`, `Evacuate`,
    `Talk`, `Wait`. Doc comment: the kinds that pick a target from a
    list (attack, arts, magic, skills, items, equip) keep their own
    queries and are not here.
  - `pub fn actions_at(&self, unit: UnitId, dest: Pos) -> Vec<ActionKind>`,
    in the menu's order, each kind decided by the same check `apply`
    runs (`check_seize`, `check_escape`, the shop's place check,
    `plan_evacuate`, the open check, `talk_targets`). Like `can_seize`
    today, it doesn't check that the unit can reach `dest` or still
    act.
  - The shop's place check (player, and a shop at `dest`) pulled out of
    `plan_shop` into one private function both use.
- **The menu:** `menu_entries` takes these kinds from `actions_at`. The
  UI-only entries (Attack, Arts, Magic, Skill, Item, Equip) stay as
  they are. `MenuEntry` wraps the kind (e.g. `MenuEntry::Act(ActionKind)`)
  or maps one to one; either way the menu's text key per kind is one
  table in the UI (core knows no text), and the mapping back to
  `UnitAction` (`mode.rs:1111`) uses it.
- **The bots' candidates:** `legal.rs` builds the tile actions from
  `actions_at(unit.id, dest)` (a `Shop` kind expands to `shop_visits`
  as today) instead of listing them at every tile.
- **The AI:** `evacuation` (`ai.rs:1030`) asks `actions_at` (or
  `can_evacuate`) instead of restating the rule. Its choice of tile
  (the top-left standing house) stays. `escape_area` stays: it finds
  where to go, not whether Escape is legal there.
- The existing `can_seize`, `can_escape` and `can_evacuate` become
  one-line readers of the same checks, or go if nothing else reads them.

**Out (do not do):**
- Splitting the rest of `impl BattleState` (`battle.rs` is 4,083 lines)
  by action. A later ticket if the sweep's next round asks for it.
- Moving the target-list kinds (attack, arts, magic, skills, items,
  equip) into `actions_at`: their menus and `AttackFrom` (1247) work
  out targets once per menu, and that stays.
- The rewind's phrases per `UnitAction` (`rewind.rs`): they describe a
  past command, not what is on offer.
- Any change to what the menu offers, in what order, or to a bot's or
  the AI's choice.
- Getting off and back on a horse: 0334, which this ticket is before.

## Implementation steps

1. Read `menu_entries` and the mapping back (`mode.rs:156`–`380`,
   ~:1100–1120), `legal.rs`'s `actions`, `ai.rs`'s `evacuation` and
   `escape_area`, and `battle.rs`'s `check_seize`, `check_escape`,
   `plan_shop`, the open check and `battle/village.rs`'s
   `plan_evacuate`.
2. Write `battle/actions.rs` with `ActionKind` and `actions_at`; pull
   the shop's place check out of `plan_shop`.
3. Point `menu_entries`, `legal.rs` and `evacuation` at it.
4. Tests (below). Run the winnable tests of Chapters 1 to 4 and the
   bots' record tests: none should change.
5. Measure with `cargo xtask code-lines` and put the numbers in the PR
   (ADR-0092). **Expected size:** about 120 production lines added and
   about 60 removed, about 250 test lines, this file about 300: about
   750 changed lines.
6. A row for 0335 in `docs/engine/friction.md`, outcome 3.

## Acceptance criteria

- [ ] `grep -n "map().shop(" crates/ui/src/screens/battle/mode.rs`
      finds nothing.
- [ ] `menu_entries` builds its Seize, Escape, Shop, Open, Evacuate and
      Talk entries from `actions_at`, and the menu tests in
      `crates/ui` pass unchanged.
- [ ] `legal.rs` no longer pushes `Seize`, `Escape`, `Open` and
      `Evacuate` at every tile; `legal_commands` returns the same
      commands as before on the test battles (test).
- [ ] `ai.rs`'s `evacuation` has no faction or house-state check of its
      own.
- [ ] For every unit and every tile it can reach in the Chapter 1 to 4
      test battles, a kind is in `actions_at` exactly when `apply`
      accepts the matching command (`Command::Act`, or `Command::Talk`
      for Talk) for a place reason (property test).
- [ ] The winnable tests of Chapters 1 to 4 and the bots' records pass
      unchanged.
- [ ] A row for 0335 in `docs/engine/friction.md`.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `actions_at` for each kind, yes and no (a shop tile for a player
  and an enemy, an escape tile outside and inside the area, a standing,
  saved and burned house, an unopened and an opened chest, a talk pair
  fired and not).
- Property: `actions_at` agrees with `apply` (above); `legal_commands`
  unchanged on the test battles.
- Snapshot / integration: none new; the existing menu snapshots
  unchanged.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
