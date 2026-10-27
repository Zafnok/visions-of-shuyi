---
id: "0325"
title: "Arts C–S, part 1: the arts that change the combat's numbers"
type: feature
milestone: M2 Core rules
model: sonnet-5
effort: medium
status: todo
blocked_by: ["0018"]
nick_input: none
completed:
---

# 0325 — Arts C–S, part 1: the arts that change the combat's numbers

## Context

Ticket 0018 decided the arts for weapon ranks C, B, A and S
(`docs/design/combat-arts.md`, *Ranks C–S*). This ticket adds the ones that
only change one combat's numbers or put a debuff on the target, on top of
0312's `ArtEffect` (`crates/core/src/art.rs`) and `assets/data/arts.ron`:

| Art | Kind | Rank | Cost | What's new in `ArtEffect` |
| --- | ---- | ---- | ---- | ------------------------- |
| Long Thrust | Spear | C | 3 | `max_range: Some(2)` (with `min_range: Some(1)`), `hit: -10` |
| Hurl | Axe | B | 4 | same as Long Thrust |
| Skypiercer | Spear | A | 5 | none: `effective: [(Flying, 2)]` |
| Skull Splitter | Axe | A | 5 | `crit: 30` (new field) |
| Felling Blow | Axe | S | 8 | `axe_min_damage_max_hp_div: Some(3)` or similar (minimum = `max(5, max_hp / 3)`) |
| Cascade | Sword | A | 6 | `cascade: Some(3)`: the n-th **hit** deals `+3 × (n − 1)` after the sword follow-up, before crit (misses don't count) |
| Rending Arrow | Bow | C | 4 | none: `on_first_hit: (stat: Def, amount: 5)` |
| Off Balance | Gauntlet | C | 3 | a debuff on **avoid** (avoid isn't a `Stat`: a combat-mod debuff) |
| Vital Point | Gauntlet | S | 8 | `on_first_hit` takes several stats (Str, Mag, Def, Res, Spd −4) |

The exact rules are in the design doc; read it, not this table, for them.

## Nick input

None. Verified by tests; what's left to judge by playing (the numbers)
goes on the Act 1 playtest's checklist (1064).

## Scope

**In:**
- The new `ArtEffect` fields above, their place in `ArtEffect::apply` /
  the combat formulas, and their forecast notes (`ArtNote`).
- `on_first_hit` growing from one debuff to a list (keep old data
  loading: Pinning Shot and Pressure Point).
- The nine arts in `arts.ron`, and the file's header comment.
- The forecast lines the design doc names (`rends: Def −5`, `off balance:
  avoid −20`, …) in the UI's art notes.

**Out (do not do):**
- Stances and reactions (Riposte, Spear Wall, Deflect, Overwatch): 0326.
- Strikes at several units (Wide Swing, Comet Lance, Arrow Rain, Chain
  Strike): 0327.
- Disarm, Marked Shot, Unbroken Stride: 0328. Special weapons: 0329.
- Teaching the boss AI to value debuffs (0511).

## Implementation steps

1. Read `combat-arts.md` (*Using an art*, *Ranks C–S*), `crates/core/src/art.rs`,
   the combat formulas in `crates/core/src/combat.rs` and 0312's tests in
   `crates/core/src/battle/tests/art.rs`.
2. Add the fields to `ArtEffect` (all `#[serde(default)]`), apply them in
   `apply` or in the combat strike loop (Cascade needs a count of the hits so far;
   Felling Blow needs the target's max HP), and add `ArtNote`s.
3. Make `on_first_hit` accept a list; avoid as a debuff target.
4. Add the nine arts to `assets/data/arts.ron` with the design doc's ranks
   and costs; update the comment that says C–S "come with ticket 0018".
5. UI: the forecast spells out the new notes (`crates/ui/src/screens/battle/forecast*`,
   `notes.rs`).

## Acceptance criteria

- [ ] A unit at rank C/B/A/S of a kind knows that rank's arts from this
      list (content test over `arts.ron`).
- [ ] Unit tests in `core` for each new effect, with the design doc's
      numbers (Cascade's order: base → ×6/5 → +3(n−1) → crit, with hit, miss,
      hit, hit giving +0, —, +3, +6; Felling
      Blow's `max(5, max_hp / 3)`; Long Thrust/Hurl at distance 2 and the
      defender's counter only if it reaches 2).
- [ ] Old `arts.ron` entries load unchanged.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: each effect's formula; debuff lists.
- Property: an art never lowers a stat below 0; Cascade adds nothing with
  one strike.
- Snapshot / integration: one forecast snapshot with a new note.

## Completion notes

*(Filled in by the session that completes the ticket.)*
