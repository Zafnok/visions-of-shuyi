---
id: "0329"
title: "Special weapons: the first three, weapon passives, personal weapons, unsellable"
type: feature
milestone: M2 Core rules
model: opus-5.5
effort: medium
status: todo
blocked_by: ["0018", "0325"]
nick_input: none
completed:
---

# 0329 — Special weapons

## Context

Ticket 0018 (`docs/design/combat-arts.md`, *Special weapons*) decided the
first special weapons and what makes a weapon special:

| Weapon | Its art | Its passive |
| ------ | ------- | ----------- |
| Houndmaster's Axe (Axe D) | **Pack Hunt** (3): might +2 per other unit of the user's side next to the target, up to +6 | — |
| Fordwarden's Spear (Spear E, range 1–2) | **Hold the Ford** (3): stance: an enemy that steps next to the user must stop there | — |
| Moonblade (Sword C) | **Moonwave** (4): range 1–2, power `max(Str, Mag)` + might vs Res | **Moonlit**: every strike hits Res instead of Def, powered by Str |

Plus: a weapon **passive** costs nothing and applies to every strike
(attacks and counters); **personal weapons** only one character can wield
(1072's side-quest rewards); special weapons **can't be sold** and repair
for gold by the usual formula. Stats are in the design doc.

0312 already lets a weapon carry `arts` (`crates/core/src/item.rs`); arts
with `rank: None` in `arts.ron` are weapon arts.

## Nick input

None. Verified by tests; the feel goes on 1064's checklist.

## Scope

**In:**
- In `items.ron`/`item.rs`: a weapon passive field (`mitigation: Res` for
  Moonlit), a `wielder: Some(CharacterId)` lock for personal weapons, and
  a `sellable: false` (or "special") flag the shops read.
- New art effects: Pack Hunt (might per adjacent ally), Moonwave (range
  1–2 and `max(Str, Mag)` power vs Res), Hold the Ford (a stance that ends
  an enemy's move on stepping next to the user: movement and the danger
  zone in `crates/core/src/movement.rs` must account for it; the module
  says "no zones of control" today, so update its doc comment).
- The three weapons in `items.ron` and their arts in `arts.ron`.
- The forecast line `moonlit: hits Res` on every Moonblade forecast.

**Out (do not do):**
- Placing the weapons in battles: 0848 gives the Chapter 2 two; the
  Moonblade waits for its questline (0075).
- Designing personal weapons (1072).

## Implementation steps

1. Read the design doc's *Special weapons* and 0312's weapon-art code.
2. Add the item fields (serde defaults so old data loads); enforce the
   wielder lock in "can wield" and the sell flag in the shop and
   Preparations sell lists.
3. Add the art effects and the passive's mitigation switch in the combat
   formulas.
4. Hold the Ford in movement: a stance-holding unit's neighbouring tiles
   end hostile moves; the danger zone and AI paths follow it.
5. Data and forecast lines.

## Acceptance criteria

- [ ] A Moonblade strike uses the defender's Res; Moonwave uses the larger
      of Str and Mag and reaches 2.
- [ ] Pack Hunt's might counts only the user's side next to the target, up
      to +6.
- [ ] An enemy path through a tile next to a Hold the Ford user stops
      there (movement test); one starting there moves away freely.
- [ ] A personal weapon can't be equipped by anyone else; special weapons
      don't appear in sell lists.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: formulas, wielder lock, sell flag.
- Property: Hold the Ford never lets an enemy move past the user's
  neighbouring tiles in one move.
- Snapshot: the Moonblade forecast.

## Completion notes

*(Filled in by the session that completes the ticket.)*
