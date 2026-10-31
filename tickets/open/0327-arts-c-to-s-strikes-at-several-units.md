---
id: "0327"
title: "Arts C–S, part 3: strikes at several units (Wide Swing, Comet Lance, Arrow Rain, Chain Strike)"
type: feature
milestone: M2 Core rules
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: ["0018", "0325"]
nick_input: none
completed:
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

- [ ] Wide Swing never strikes the tile behind the user, and has no normal
      combat (no counters, no extra strikes).
- [ ] Comet Lance skips allies and only goes in rows or columns.
- [ ] Arrow Rain hits the target and its four neighbours that are enemies.
- [ ] Chain Strike stops once the target falls; allies lose no durability.
- [ ] Forecast snapshot with three rows.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit / battle: each art's tiles; EXP awards; deterministic roll order.
- Property: no art ever strikes a unit friendly to the user.
- Snapshot: the multi-row forecast.

## Completion notes

*(Filled in by the session that completes the ticket.)*
