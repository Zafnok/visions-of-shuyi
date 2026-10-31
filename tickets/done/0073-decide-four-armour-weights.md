---
id: "0073"
title: "Decide: four armour weights (cloth, leather, chain, plate) and who wears them"
type: design-decision
milestone: Design decisions
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: []
nick_input: decision
completed: 2026-10-21
---

# 0073 — Decide: four armour weights

## Context

While answering the Preparations shop (0044, PR #385, 2026-10-21) Nick
proposed replacing today's three armour weights (Light / Medium / Heavy,
`docs/design/weapons-and-items.md`, *Armour*) with four, by material:

> "light = cloth (low def high res low weight penalty), med = leather (mix
> def and res, low weight penalty still), heavy = chain (high def, med res,
> and med weight penalty), and ultraheavy or something = plate (nearly
> impenetrable def, critically low res, very high weight penalty)"

(Full quote in `weapons-and-items.md`, *Nick's words*.)

A first lead on question 2 (Nick, PR #385, 2026-10-21): "it doesn't make
sense a cleric would wear leather armor rather than robes... martial/agile
classes, like swordsman brawler should be wearing leather most likely".
So casters in cloth (robes), martial and agile classes in leather. Start
the class table from that and confirm it.

Today's data (`assets/data/items.ron`, `assets/data/classes.ron`):

- Armour: Leather Vest (Light, Def +1, wt 0), Chain Mail (Medium, Def +3,
  wt 2), Iron Plate (Heavy, Def +5, wt 5), Warded Robe (Light, Res +2,
  wt 0). Armour gives only one stat today, and plate gives no Res penalty
  (armour has no negative bonuses yet).
- Class limits: Swordsman, Brawler, Mage, Cleric and their promotions wear
  **Light only**; Raider, Archer, Rider, Exile wear Light + Medium; Guard
  and the armored classes Medium + Heavy; a few tier-3 classes all three.
- Chapter 1's cast wears Leather Vests, including light-only classes
  (Maud the Brawler, the Cleric). If leather moves up a weight, they lose
  it, so the class limits must move with the weights.
- The Tailor specialty store (decided in 0044) trades cloth and leather,
  so Light and Medium under the new scheme.

## Nick input

**Decision**, with the `ask-nick` skill, at most three questions per
message, with examples from real games (Dark Souls / Elden Ring weight,
Divinity's physical and magic armour, Diablo, FFT's clothes / armour /
robes, Fire Emblem having almost none):

1. **Confirm the four weights and their names** (Cloth / Leather / Chain /
   Plate, or Light / Medium / Heavy / Ultraheavy as Nick wrote), and what
   each gives: a Def and Res bonus (plate's Res can go **below zero**, a
   penalty: allowed?) and a weight that feeds `burden` (attack speed).
2. **Which classes wear which**, as a table over every class in
   `classes.ron`. Propose: cloth-only casters, cloth + leather for
   skirmishers, leather + chain for the line, chain + plate for the
   armored classes; ask about the lord.
3. **A starter piece of each weight** (name, Def, Res, weight, price), so
   the basic shop (0044) and Chapter 1's loadouts have something to use.

## Scope

**In:**
- The decision in `weapons-and-items.md` (*Armour*, his words, numbers
  marked Nick's or *tunable*) and the `docs/design/README.md` table.
- `write-ticket` for the data and code change (armour weights in `core` and
  `trpg-content`, `items.ron`, `classes.ron`, Chapter 1's loadouts, the
  Preparations reason text `can't wear heavy armour`), and add it to
  1064's `blocked_by`; update 0443 and 0320 if the armour names change.

**Out (do not do):**
- Any code or data.

## Implementation steps

1. Read `weapons-and-items.md` (*Armour*, *Formulas*, *Preparations*
   rules), `assets/data/items.ron`, `assets/data/classes.ron` and
   `assets/data/characters.ron` (who wears what in Chapter 1).
2. Ask with `ask-nick`; show the class table as a mockup.
3. Record; write the follow-up ticket; update 0443 / 0320 if needed.

## Acceptance criteria

- [x] `weapons-and-items.md` gives the four weights, their stats and the class table, with Nick's words.
- [x] `docs/design/README.md` row added.
- [x] A follow-up ticket for the data and code exists and is in 1064's `blocked_by`.

## Tests required

- None (documentation).

## Completion notes

Asked in three rounds (2026-10-21). Nick decided:

- **Four weights by material:** Cloth, Leather, Chain, Plate.
- **Each class has a max armour weight** and wears that weight and every
  lighter one (Nick's idea, replacing the proposed per-class pairs).
  Casters cloth; agile and martial lines leather; raiders, cavalry and the
  Grappler chain; the armored lines, Vanguard, Warchief and Colossus
  plate. `progression.md`'s class tables now have a *Max armour* column.
- **The lord never wears plate.** Exile leather; Blade Heir / Sovereign
  leather; Commander / Grand Marshal chain.
- **Starter pieces:** Linen Robe (Res +4, wt 0, 300), Leather Vest (Def +2
  Res +2, wt 1, 400), Chain Mail (Def +4 Res +1, wt 3, 800), Iron Plate
  (Def +8 Res −3, wt 6, 1500). Nick set cloth's Res, leather's Def/Res and
  plate's Def; he kept the rest as proposed.
- **Iron Plate from Chapter 2** with the other three; a **Steel Plate**
  (maybe Steel versions of the others) from Chapter 5, numbers later.

Also updated: the attack-speed illustration (new weights: a Fighter with a
Steel Axe in Chain Mail now strikes once there), the swap example, the
reason text example (`can't wear plate armor`), the basic shop's rows and
example, the Tailor's weights, `magic.md`'s caster attack-speed note,
`progression.md`'s promotion armour example and Commander line, 0443's
shop rows and 0320's `blocked_by`.

*the agent's starting rules:*
- A penalty can't take a stat below 0 (it follows from stats never being
  negative): Iron Plate's Res −3 only bites a wearer who has Res to lose.
- The old Warded Robe (Res +2, 700) is retired; the Linen Robe replaces it.
- The tier 4–5 flying classes' max weights follow their old letters (L →
  Leather, L M → Chain); their numbers aren't designed yet anyway.

Follow-up tickets: **0321** (the four weights in `core`, `trpg-content`
and the data; Maud and Rue in Linen Robes) and **0074** (decide the Steel
armour), both in 1064's `blocked_by`; 0320 now waits on 0321.
