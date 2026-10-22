---
id: "0313"
title: "Save the village: houses, Evacuate, raiders who burn them (core rules)"
type: feature
milestone: M2 Core rules
model: opus-5.5
effort: medium
status: todo
blocked_by: ["0308", "0037"]
nick_input: answer-first
completed:
---

# 0313 — Save the village: houses, Evacuate, raiders who burn them

## Context

This ticket was "Villages and the Visit action": FE GBA's village a player
unit visits for a one-time gift. Ticket 0069 (2026-10-22) replaced that
rule. Nick: villages in a battle only when the story is about them, "a real
race not an optional one... like save the village with a minimum number
houses allowed to pass, and past that get extra rewards at the end", and no
plain gift villages at all. The rule is in `docs/design/turn-structure.md`
(*Act 1's battle rules*, *Save the village*). Chapter 2, Kell's Ford (0848),
is its first and, in Act 1, only user.

In short: a map marks **houses** and a **minimum** to save. A player unit on
an unsaved house's door uses **`Evacuate`** (its action) and the house is
saved for good. An enemy marked as a **raider** that ends its turn on or next
to an unsaved house's door burns it (lost for good). A house still standing
when the battle is won counts as saved. The battle is lost the moment the
minimum can no longer be met; each house standing past the minimum adds a
reward at the end.

**Blocked** until `docs/design/terrain.md` defines the house (village) tile:
ticket **0037**. Don't invent the tile. The UI half (the action menu entry,
the objective line, the house's look on every map skin) is ticket 0409.

## Nick input

**Answer first:** ticket 0037 (the house tile: its glyph, costs and bonuses
come from that decision, not from this ticket). The rule itself is decided
(0069).

## Scope

**In:**
- A house map feature (`TileFeature::House` or as 0037 names the tile) in
  `crates/core/src/map.rs`, and the `.map` header syntax for it
  (`crates/content/src/map.rs`, `assets/maps/README.md`), validated like
  chests (tile passable).
- The battle's village goal: a minimum to save and the extra reward per
  house past it, in the battle file (`assets/battles/README.md`), validated
  (minimum ≤ number of houses, rewards' item ids known, `Gold(0)` refused).
- A unit setting that marks an enemy as a **raider**.
- `UnitAction::Evacuate` for a **player** unit on an unsaved house's door
  (reuse `CommandError::PlayerOnly`; add errors for no house and a house
  already saved or lost). `Event::HouseSaved`, `Event::HouseBurned`.
- Burning: at the end of a raider's action, every unsaved, unburned house
  whose door is the raider's tile or one of the four beside it burns.
- Loss when too many houses are lost for the minimum; on a win, standing
  houses count as saved and each one past the minimum gives its reward
  (gold to the purse, items where chest items go).
- The saved / lost counts in `BattleState` (saved with the battle, like
  opened chests), so the objective line (0409) can show
  `Save at least 3 of 5 houses (1 saved, 1 lost)`.
- The AI: a raider walks toward the nearest unsaved house and prefers
  burning it to attacking (*Claude's starting rule*, tuned by the bots).
- The bots (0504's legal moves) and play records handle `Evacuate` and the
  burning events.
- A test battle in `assets-test/` with houses and raiders.

**Out (do not do):**
- Plain "Visit for a gift" villages: Nick dropped them (0069).
- The UI (0409), the Kell's Ford battle itself (0848), playing a house's
  line (0705's scene triggers do that).

## Implementation steps

1. Read `turn-structure.md` (*Save the village*) and 0037's tile.
2. Map: the house feature, a lookup, the header syntax and its checks.
3. Battle: `Evacuate`, burning, the loss check (after every action, with the
   other loss conditions), the rewards on a win, through `Command` →
   `Event` (ADR-0004). Document the rule in the `battle.rs` module docs.
4. AI for raiders; extend `legal_commands` in `crates/core/src/battle/tests.rs`
   with `Evacuate` and add houses to `prop_map()`.
5. The test battle; the bots run it (`cargo xtask playtest <test battle>
   --bot all`) without a softlock.

## Acceptance criteria

- [ ] `Evacuate` saves a house; each error case leaves the state unchanged
      (`refused_act`).
- [ ] A raider ending its action on or next to an unsaved door burns it; a
      saved house never burns.
- [ ] The battle is lost the moment the minimum can't be met, and won with
      the extra rewards counted from the standing houses.
- [ ] Saved and lost houses survive a RON save/load; play records replay
      identically.
- [ ] Map and battle files with houses parse, print back and are validated.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: Evacuate, burning (on the door, beside it, diagonal does not), the
  loss, the rewards; parsing, printing, validation.
- Property: the random-command test in `battle/tests.rs` includes
  `Evacuate`; a saved house never becomes burned.

## Completion notes

*(Filled in by the session that completes the ticket.)*
