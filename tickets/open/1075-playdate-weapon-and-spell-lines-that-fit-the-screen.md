---
id: "1075"
title: "Playdate: weapon and spell lines that fit the screen"
type: feature
milestone: Post–Chapter 1
model: opus-5.5
effort: medium
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 1075 — Playdate: weapon and spell lines that fit the screen

## Context

Ticket 1038 gave the battle screen its Playdate look
(`crates/ui/src/screens/battle/playdate.rs`): menus are windows over
the map, at most 50 cells wide, so a line has 46 cells inside the
frame. The PC's lists of weapons and spells are wider than that:

- **Equip** (`items.rs`, `equip_label`, the text `battle.equip.line`:
  `{name}  Mt{might} Hit{hit} Crit{crit} Wt{weight} Rng{range}
  {left}/{max}`): Ellery's line in Harrowby Ford is
  `* Iron Sword  Mt 5 Hit 90 Crit 0 Wt 2 Rng1 20/20`, 49 cells. A
  longer weapon name is longer still.
- **Spells** (`magic.rs`, `spell_label`, `battle.spell.line`:
  `{name}  {left}/{uses}  {numbers} Rng{range}`): the same numbers.

1038's skin cuts a line too long for its window with `…`, so on the
Playdate the **uses left** (`20/20`) are cut off the end of the Equip
line, the one number the player most needs there
(`docs/design/playdate/1038-*.png`; the PR of 1038 shows the Equip
list). What the Playdate's line keeps is a look-and-feel question, so
it is Nick's (`docs/design/playdate.md`).

## Nick input

**Decision**, asked before building with `cargo shuyi frame-png
--playdate` pictures of Harrowby Ford's Equip list, each option drawn
(the `ask-nick` skill). Options to show, from real games:

- **A. Name and uses only on the line, the numbers under a rule for the
  focused line** (Fire Emblem on the GBA shows the item list with uses,
  and the numbers in a separate window; this game's arts list already
  shows the focused line's description under a rule:
  `look-and-feel.md`, *Descriptions in lists*).
- **B. Fewer numbers on the line** (drop Crit and Wt, as Advance Wars
  shows only what decides the choice).
- **C. Two rows per weapon** (the name and uses, then the numbers).
- **D. Describe your own.**

What is left to judge by playing goes on 1064's checklist.

## Scope

**In:**
- The Playdate's weapon and spell lines in the battle's menus (Equip,
  the weapon chosen for Attack, the spell list) as Nick decides,
  recorded in `docs/design/playdate.md`.
- If the line's parts are needed apart (a name, the uses, the numbers),
  the battle's view carries them for both skins (ADR-0054); the PC's
  look doesn't change.

**Out (do not do):**
- Any change to what the PC shows.
- The forecast, the unit info screen (1039).

## Implementation steps

1. Ask Nick (above) with pictures; record the answer in
   `docs/design/playdate.md`.
2. Build it in `screens/battle/playdate.rs` (and the view, if needed).
3. Look tests in `screens/battle/playdate/tests.rs`; `frame-png
   --playdate` pictures in the PR.

## Acceptance criteria

- [ ] On the Playdate, every weapon and spell line of Chapter 1's units
      shows its uses left, inside its window.
- [ ] No PC snapshot changes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit (look, with the skin): the Equip list and the spell list at the
  Playdate's size, the uses shown.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
