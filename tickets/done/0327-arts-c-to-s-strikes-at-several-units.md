---
id: "0327"
title: "Arts C–S, part 3: strikes at several units (Wide Swing, Comet Lance, Arrow Rain, Chain Strike)"
type: feature
milestone: M2 Core rules
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: ["0018", "0325"]
nick_input: none
completed: 2026-11-08
---

# 0327 — Arts C–S, part 3: strikes at several units

## Context

Ticket 0018 (`docs/design/combat-arts.md`, *Ranks C–S*, *Exact rules*)
decided four arts that strike more than one unit, generalising 0312's Line
Pierce (`ArtEffect::line_pierce`, `crates/core/src/battle.rs`):

- **Wide Swing** (Axe C, 4): no normal combat; one strike each at the
  enemies on the tiles in front of, left of and right of the user (the
  picked adjacent enemy sets the facing). No counters.
- **Comet Lance** (Spear S, 8): a straight row or column, 3 tiles; one
  strike at every enemy on it, nearest first; allies passed over.
- **Arrow Rain** (Bow S, 8): an enemy at distance 2–3; one strike at it
  and at every enemy on the four tiles next to it.
- **Chain Strike** (Gauntlet A, 6): after the combat, each ally next to the
  target whose weapon reaches it strikes it once (no durability, no EXP
  for them).

Each struck enemy is its own combat for unit EXP and class points; weapon
EXP is one award with all the damage in `dealt` (as Line Pierce).

## Nick input

None. Verified by tests; the feel goes on 1064's checklist.

## Scope

**In:**
- A shared "strikes at these tiles" step in `core` (one strike, no
  counter, no extra strikes, the 2RN and crit rolls in strike order), used
  by the four arts and, if it fits, by Line Pierce.
- The targeting each art needs (facing for Wide Swing, a straight line for
  Comet Lance, 2–3 for Arrow Rain), in `art.rs` and the battle's attack
  options.
- A forecast with one row per unit struck (the design doc's Arrow Rain
  mockup) in `crates/ui/src/screens/battle/forecast*`.
- The four arts in `arts.ron`.

**Out (do not do):**
- 0325, 0326, 0328's arts. AI valuing several targets beyond the forecast
  sum.

## Implementation steps

1. Read the design doc's rules and Line Pierce's code and tests.
2. Build the shared step; add each art's targeting and effect fields.
3. Award EXP as the design doc says; Chain Strike's allies get none.
4. Forecast rows and playback for several units.

## Acceptance criteria

- [x] Wide Swing never strikes the tile behind the user, and has no normal
      combat (no counters, no extra strikes).
- [x] Comet Lance skips allies and only goes in rows or columns.
- [x] Arrow Rain hits the target and its four neighbours that are enemies.
- [x] Chain Strike stops once the target falls; allies lose no durability.
- [x] Forecast snapshot with three rows.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit / battle: each art's tiles; EXP awards; deterministic roll order.
- Property: no art ever strikes a unit friendly to the user.
- Snapshot: the multi-row forecast.

## Completion notes

Done as planned.

- **Core.** `ArtEffect` gains `area: Option<ArtArea>` (`Cleave`,
  `Line(n)`, `Burst`) and `chain: bool`. One shared step
  (`battle/arts.rs`, `plan_strikes`) lists an art's strikes at units
  (`ArtStrike`: who strikes, who is struck, one-strike numbers with no
  counter) and Line Pierce now uses it too. An area art fights no combat:
  `fight` plays its strikes in order. Each strike needs the user, the
  striker and the unit struck still standing, so Chain Strike stops when
  the target falls. Each strike of the user's is its own combat for unit
  EXP and class points. Weapon EXP is one award with all damage in
  `dealt`, the allies' too. Allies' strikes get no EXP and spend no
  durability (normal attacks never do).
- **Targeting.** Wide Swing has range 1–1, Comet Lance 1–3 and Arrow Rain
  2–3, all set in data. A Comet Lance target off its row or column is
  `OutOfRange`, so the arts menu dims it as "out of range".
- **Preview and forecast.** `AttackPreview::pierce` became `strikes:
  Vec<StrikePreview>`, one row per strike with the struck unit's HP
  before and after a hit. Both skins paint those rows after the totals
  (PC: `Brigand   Hit  91  20 → 12`; Playdate too). An area art's main
  block shows the target's one strike and `no counter`.
- **AI.** `AttackOutlook::pierce` became `others`: Line Pierce's and an
  area art's strikes at units other than the target, summed as before.
  Chain Strike's allies aren't counted (out of scope).
- **Data.** The four arts are in `arts.ron` with the design doc's ranks
  and costs. The content check refuses `Line(0)` and an area art that
  also pierces or chains.

Starting rules (*the agent's starting rules*, for the Act 1 playtest, 1064):

- The strikes at units other than the target use the numbers at the
  target's distance (as Line Pierce already did). For example, Arrow
  Rain's strike at an enemy right next to the archer still lands.
- A struck unit's Deflect turns the strike into a miss but doesn't strike
  back: the strike has no counter, as Line Pierce's already worked (0326).
  So nobody who strikes can be hurt by these strikes.
- Chain Strike: an ally strikes whether or not it has acted this turn. It
  must have a weapon equipped that reaches 1 and isn't broken. Its damage
  counts toward the user's weapon EXP, but its hits don't count as the
  user's hits for the hit/miss base.
- Chain Strike's forecast rows name the ally striking (the HP is the
  target's). The other arts' rows name the unit struck.

Feel for 1064: whether the four arts are worth their 4–8 durability.
