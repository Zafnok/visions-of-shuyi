---
id: "0074"
title: "Decide: the Steel armour sold from Chapter 5"
type: design-decision
milestone: Design decisions
model: reasoning
effort: medium
status: todo
blocked_by: ["0073"]
nick_input: decision
completed:
---

# 0074 — Decide: the Steel armour

## Context

Ticket 0073 (2026-10-21) settled four armour weights (Cloth, Leather,
Chain, Plate) and a starter piece of each, all sold by the Preparations
shop from Chapter 2 (`docs/design/weapons-and-items.md`, *Armour* and
*The Preparations shop*). Nick found it odd that Iron Plate waited for
the Steel weapons, and chose: **Iron Plate from Chapter 2, a Steel Plate
later**, "maybe Steel versions of the other armour too", numbers set
later. The Steel weapons arrive at **Chapter 5** (Saltmere, 0044).

Starter pieces for comparison (all *tunable*): Linen Robe (Cloth, Res
+4, wt 0, 300), Leather Vest (Def +2 Res +2, wt 1, 400), Chain Mail (Def
+4 Res +1, wt 3, 800), Iron Plate (Def +8 Res −3, wt 6, 1500). The Steel
weapons are about +3 might and +2 weight over Iron (`assets/data/items.ron`).

## Nick input

**Decision**, with the `ask-nick` skill, at most three questions per
message, with examples (Fire Emblem's Iron → Steel → Silver weapons,
Dark Souls / Elden Ring armour sets that trade weight for defense, FFT's
shop tiers of hats, armour and robes by chapter):

1. **Which weights get a Steel piece** at Chapter 5: only Plate, Plate and
   Chain (the metal ones), or all four (a better robe and leather too,
   under other names since they aren't steel)?
2. **How much better**, and what it costs: show each proposed piece's Def,
   Res, weight and price beside the starter, with what it does against a
   Chapter 5 enemy (a Steel Axe hit, a Fire spell).
3. The names (Steel Plate, Steel Mail, …) if they aren't obvious.

## Scope

**In:**
- The decision in `weapons-and-items.md` (*Armour* and the basic shop's
  Chapter 5 row), with Nick's words and numbers marked Nick's or
  *tunable*; the `docs/design/README.md` row.
- `write-ticket` for the data change (the pieces in `items.ron` and the
  basic shop's Chapter 5 row), added to 1064's `blocked_by`.

**Out (do not do):**
- Any code or data.
- Armour beyond Chapter 5's tier (Silver, rare pieces).

## Implementation steps

1. Read `weapons-and-items.md` (*Armour*, *Attack speed*, *The
   Preparations shop*) and `assets/data/items.ron`.
2. Ask with `ask-nick`, the proposed pieces in a table.
3. Record; write the follow-up data ticket (blocked by 0321 and, for the
   shop row, 0443).

## Acceptance criteria

- [ ] `weapons-and-items.md` lists the Steel armour pieces with their numbers and Nick's words.
- [ ] `docs/design/README.md` row added.
- [ ] A follow-up data ticket exists and is in 1064's `blocked_by`.

## Tests required

- None (documentation).

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
