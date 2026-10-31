---
id: "0474"
title: "A negative stat bonus reads Res -3, not Res +-3"
type: bug
milestone: M3 Battle UI
model: sonnet-5 / gpt-6.1-terra
effort: low
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0474 — A negative stat bonus reads Res -3, not Res +-3

## Context

Found while working ticket 0443 (the Preparations Shop tab): the Iron Plate
line reads `Def +8  Res +-3`. The Iron Plate's bonus is `(def: 8, res: -3)`
(`assets/data/items.ron`, the four armour weights from 0073; a negative bonus
is a penalty, `docs/design/weapons-and-items.md`, *Armour*).
`bonus_text` in `crates/ui/src/screens/battle/info.rs` writes every bonus
as `"{stat} +{n}"`, so a negative one gets both signs. It shows wherever
the text is used: the on-map shop (0409), the Preparations Shop tab (0443),
the Loadouts tab's armour rows and the unit info panel.

## Nick input

None. Verified by tests and the snapshots; nothing to judge by playing.

## Scope

**In:**
- `bonus_text` writes a negative bonus with its minus sign only:
  `Def +8  Res -3`.

**Out (do not do):**
- Changing any item's numbers, or how penalties apply in combat.
- Colouring penalties differently (a look decision).

## Implementation steps

1. In `bonus_text` (`crates/ui/src/screens/battle/info.rs`), format the
   number with its sign (`{:+}`), so `3` reads `+3` and `-3` reads `-3`.
2. Add a unit test next to it: a bonus of `def: 8, res: -3` reads
   `Def +8  Res -3`.
3. Run the UI tests; accept the snapshots whose only change is `+-` → `-`
   (the Preparations Shop tab snapshot has the Iron Plate line), and check
   no other line changed.

## Acceptance criteria

- [ ] `bonus_text` of `(def: 8, res: -3)` is `Def +8  Res -3` (unit test).
- [ ] No snapshot shows `+-`: `grep -rn "+-" crates/ui/src --include=*.snap` is empty.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the new `bonus_text` test.
- Snapshot / integration: the snapshots that show the Iron Plate, updated.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
