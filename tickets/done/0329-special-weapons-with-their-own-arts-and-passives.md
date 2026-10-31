---
id: "0329"
title: "Special weapons: the first three, weapon passives, personal weapons, a warning before selling"
type: feature
milestone: M2 Core rules
model: reasoning
effort: medium
status: done
blocked_by: ["0018", "0325"]
nick_input: none
completed: 2026-10-28
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
(1072's side-quest rewards); selling a special weapon **asks first** (`Sell Moonblade for 1500? No
shop sells it, so it can't be bought back.`, No / Yes, cursor on No) and
no shop stocks one; they repair
for gold by the usual formula. Stats are in the design doc.

0312 already lets a weapon carry `arts` (`crates/core/src/item.rs`); arts
with `rank: None` in `arts.ron` are weapon arts.

## Nick input

None. Verified by tests; the feel goes on 1064's checklist.

## Scope

**In:**
- In `items.ron`/`item.rs`: a weapon passive field (`mitigation: Res` for
  Moonlit), a `wielder: Some(CharacterId)` lock for personal weapons, and
  a `special: true` flag the shops read (never stocked; selling asks
  first, through the engine's yes/no question screen).
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
   wielder lock in "can wield"; in the shop and Preparations sell lists,
   selling a special weapon opens the yes/no question first.
3. Add the art effects and the passive's mitigation switch in the combat
   formulas.
4. Hold the Ford in movement: a stance-holding unit's neighbouring tiles
   end hostile moves; the danger zone and AI paths follow it.
5. Data and forecast lines.

## Acceptance criteria

- [x] A Moonblade strike uses the defender's Res; Moonwave uses the larger
      of Str and Mag and reaches 2.
- [x] Pack Hunt's might counts only the user's side next to the target, up
      to +6.
- [x] An enemy path through a tile next to a Hold the Ford user stops
      there (movement test); one starting there moves away freely.
- [x] A personal weapon can't be equipped by anyone else; selling a special
      weapon shows the question and only Yes sells it (screen test).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: formulas, wielder lock, the special flag.
- Property: Hold the Ford never lets an enemy move past the user's
  neighbouring tiles in one move.
- Snapshot: the Moonblade forecast.

## Completion notes

Done as planned:

- **Data**: `items.ron` has the Houndmaster's Axe, the Fordwarden's Spear
  and the Moonblade with the design's stats, `special: true`, their arts
  and the Moonblade's `passive: Some((name: "Moonlit", mitigation:
  Some(Res)))`; a weapon may also carry `wielder: Some("character")`.
  A wielder or passive on a weapon that isn't special is a load error.
  `arts.ron` has Pack Hunt (`pack_might: Some((2, 6))`), Hold the Ford
  (`stance: Some((stops_enemies: true))`) and Moonwave (`max_range:
  Some(2), power: Some(Higher), mitigation: Some(Res)`).
- **Combat**: `CombatMods` gained `power` (Str, Mag or the higher) and
  `mitigation` (Def or Res). A weapon's passive goes into its wielder's
  mods in `Unit::combat_input`, so it counts on attacks, counters, the
  AI's forecasts and previews alike. Pack Hunt counts the other units of
  the user's side next to the target when the battle builds the combat.
- **Movement**: tiles next to a hostile unit whose stance stops enemies
  end a move stepped onto them. The engine's Dijkstra has no "enter but
  don't leave" tile, so `movement::reachable` runs it with those tiles
  closed and then adds each one entered from its cheapest reached
  neighbour (game-side, small; no engine change). `path_cost` refuses a
  drawn path that leaves one (`PathError::Stopped`). The danger zone,
  threat arrows, legal moves and the AI all read `reachable`.
- **Shop**: no shop stocks a special weapon (`ShopKind::stocks`, also the
  map content check). Selling one in the battle shop opens the engine's
  question (`Ask`): `Sell Moonblade for 1 500? No shop sells it, so it
  can't be bought back.`, No / Yes, the cursor on No; only Yes sells.
  Preparations has no sell list yet: 0443 (its shop tab) now says to keep
  this question. Special weapons repair by the usual formula (unchanged
  code; tested).
- **Forecast**: a line `moonlit: hits Res` whenever a side striking holds
  the Moonblade (attacker, or the defender when it counters). The
  passive's name is translatable (`passives.<item id>` in the language
  packs). The arts menu describes the new effects (`+2 might per ally
  beside target, max +6`, `stance: enemies stepping next to it stop`,
  `reaches 2; higher of Str/Mag; hits Res`).
- The balance bots' weapon knob skips special weapons when it moves an
  enemy's weapon a tier up or down (no shop tier; it had started picking
  the Houndmaster's Axe as the step above the Steel Axe).

Engine friction: estimate 350, measured 354 (1.0×), under 2×.

*the agent's starting rules* (where the design is silent), for the Act 1
playtest (1064):

1. **Hold the Ford lasts like every stance**: from the attack until the
   start of the user's next phase.
2. **A tile next to the holder that another unit stands on can't be
   passed** by an enemy: it would have to stop there and can't.
3. **Pack Hunt counts green (allied) units** next to the target as the
   user's side, as well as the user's own faction; the user itself never
   counts.
4. **The `moonlit: hits Res` line also shows when the Moonblade's holder
   is the one countering.**
5. The sell question shows the gold as the shop does (`1 500`).

