---
id: "0079"
title: "Decide: who wields the Houndmaster's Axe in Act 1"
type: design-decision
milestone: Design decisions
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 0079 — Decide: who wields the Houndmaster's Axe in Act 1

## Context

Nick played the Act 1 build (Chapters 1–4) on 2026-11-08:

> "nobody can use the houndmaster axe reward from ch2 in ch3, since none
> of the starting units have axe speciality unlocked -- we should either
> add the speciality to one of our units or add an axe unit or convert a
> unit to an axe wielding class"

**The axe.** `houndmasters_axe` (`assets/data/items.ron:68`): Axe, rank
D, Mt 10, Hit 70, Crit 5, Wt 7, Rng 1, Dur 25, price 900, special, with
its art **Pack Hunt** (`arts.ron` ~:192: this combat, might +2 for each
other ally next to the target, up to +6; 3 durability). Hendry carries it
in Chapter 2; winning gives it to the party (`assets/battles/ch02.ron`
`clear_items`, `crates/core/src/campaign.rs` ~:586). Design:
`docs/design/combat-arts.md` (*Special weapons*, ~:538–560).

**Why nobody can use it.** `Unit::can_wield` (`crates/core/src/item.rs`
~:696) needs the unit's *current class* to list the Axe kind and the
unit's Axe rank to be at least D. A personal `wielder` on a special
weapon narrows who may use it; it never adds a kind to a class.

| Unit | Class (Mov) | Weapon kinds | Axe? |
|---|---|---|---|
| Lead | Exile (5) | Sword D–C | no (lord line gets axes only at tier 3, Grand Marshal) |
| Hollis (`retainer`) | Guard (4), Armored | Spear D–C | no; **Bulwark** (tier 2) has Axe D–B |
| Tamsin (`sergeant`) | Rider (7), Mounted | Sword E–C, Spear D–C | no; **Iron Rider** (tier 2) has Axe D–B |
| Aske (`poacher`) | Archer (5) | Bow D–C | no |
| Maud (`keeper`) | Cleric (5) | Gauntlet E–D | no |
| Rue (`heretic`) | Mage (5) | Sword E–D, spells | no |
| Joss (`defector`, joins after Ch2) | Swordsman (5) | Sword D–C | no |

Tier-2 classes with axes (`assets/data/classes.ron`): Grappler (from
Brawler), Bulwark (from Guard), Iron Rider (from Guard or Rider), Vanguard
(from Raider). Promotion needs a mastered class (class level 10) and a
Tier 2 Seal; in Act 1 it is reachable only from the debug menu (0606 is
open) and no seal is placed in Act 1's data. So promotion can't put the
axe in anyone's hands by Chapter 3.

The tier-1 **Raider** (`classes.ron` ~:62: Axe D–C, Mov 5, up to Chain
armour, base 22/7/0/3/4/4/0) is playable, not enemy-only. A **Reclass
Seal** (`items.ron` ~:97) can take any unit to any tier-1 class between
battles (`progression.md`, *Reclass*), but it is "a rare resource, never
sold in Chapter 1", and none is placed in Act 1 either.

**The first axe user in the story** is Hedda Ravn (`shieldbearer`,
Raider), who joins in **Chapter 5** (`docs/story/characters/
shieldbearer.md`; 0851).

**Why it matters beyond the reward.** An axe is this game's answer to
armour: every axe hit does at least 5 (`AxeMinDamage(5)`, `items.ron:51`),
and an axe user at rank D knows **Armor Cleave** (`arts.ron` ~:122: ×2
might against Armored, 4 durability). A Chapter 3 guard (Lv 6, HP 24,
Def 15) shrugs off every weapon the party has today (ticket 0080). With
this axe, a Str 8 unit does `8 + 10 − 15 = 3`, raised to **5**; with
Armor Cleave `8 + 20 − 15 =` **13**. So who holds it is half of 0080's
question; the two tickets may be asked together.

Nick paused AI story writing on 2026-11-08 until a human writer is hired
(ticket 0766). Any option that needs new dialogue or a new named
character says so: its words wait for the writer.

## Nick input

**Decision** (`ask-nick`), at most three questions per message. Show
each option as a player situation, with the axe's numbers against a
Chapter 3 guard for the unit who would hold it.

**1. Who gets the axe?**

- **A. An existing unit's class gets axes now.** For example the
  **Guard** class gains Axe (D–C) beside its spears, so Hollis can take
  the axe in Chapter 3 (Str 8: 5 a hit on a guard, 13 with Armor Cleave).
  Like Fire Emblem Engage, whose armour-knight class comes in a sword, a
  lance and an axe version. Downside: it changes every
  Guard, enemies included (they keep their spears unless a battle gives
  them axes), and Hollis already holds the line, so the anti-armour job
  stays on the slowest unit (Mov 4).
- **B. One unit gets an axe rank outside its class**, like Three Houses,
  where nearly any unit can train and equip nearly any weapon whatever its
  class. For example Joss (an ex-Hound; the Hounds are raiders with axes)
  or Tamsin. Downside: a new rule (a personal weapon kind), which no other
  unit has.
- **C. Reclass a unit into Raider** with a Reclass Seal given with the
  axe (or placed in Chapter 2). Who fits: Joss (keeps his stats, Spd 9,
  loses swords while a Raider; Str 6: 5 a hit, 11 with Armor Cleave) or
  Hollis (gains Mov 5 but loses his spears, the Armored tag and the
  right to wear Plate later; he keeps his Chain Mail). Nick also finds
  Joss "a bit too strong" (0866): the axe on Joss adds to that. Like
  Fire Emblem's Second Seal (Awakening, Fates), an item that changes a
  unit's class. Downside: a Reclass Seal in Chapter 2 is far
  earlier than "rare".
- **D. A new axe recruit by Chapter 3**, or Hedda earlier. A new named
  character, or moving Hedda's join from Chapter 5, is a **story
  decision**, and its scenes wait for the writer (0766).
- **E. Change the reward instead**: Hendry's axe stays his, or stays in
  the stock until Hedda joins in Chapter 5 (with a line in its
  description saying an axe user is needed), and Chapter 2 gives
  something the party can use now.
- **F. Combinations, or describe your own.**

**2. When must the axe user arrive?** By Chapter 3 (the next battle),
or is Chapter 5 (Hedda) fine if the reward says so?

**3. Only if he picks A or B:** which unit, and at what Axe rank (D lets
them use this axe and Armor Cleave at once; E would need the rank raised
first).

Check every game claim before showing it; claim only what the game
really does.

## Scope

**In:** the question, the worked numbers, the answer recorded with
Nick's words in `docs/design/combat-arts.md` (*Special weapons*), and in
`progression.md` if a class's weapon kinds change; the `docs/design/
README.md` row; 0332's text updated with what to build.

**Out (do not do):**
- Building it: ticket **0332**.
- Any new dialogue or a new named character's scenes: they wait for the
  writer (0766).
- How the army gets through armour in general (0080). Either order: if
  0080 is answered first, show its pick beside these numbers; if not, say
  that the axe is one of 0080's answers and note Nick's pick in 0080.
- Promotion and reclass in the between-battle menu (0606). If Nick picks
  a reclass (C) and 0606 isn't done, 0332 gives the class change in data
  or a scene step instead of the menu; say so in 0332's text.
- Joss's tuning (0866). Either order: show Joss's numbers as they are
  when asked; if 0866 is done, use its values.

## Implementation steps

1. Read `combat-arts.md` (*Special weapons*), `progression.md`
   (*Reclass*, the class tree), `weapons-and-items.md` (weapon kinds),
   `classes.ron`, `characters.ron`, the character sheets of Hollis,
   Tamsin, Joss and Hedda.
2. Work out each option's numbers for the unit who would hold the axe:
   damage a hit and with Armor Cleave against a Chapter 3 guard (Lv 6:
   HP 24, Def 15; Def 18 on the gate) and against Chapter 3's other
   enemies, and whether it strikes twice (attack speed, weight 7).
3. Ask Nick; iterate; record his words verbatim and the rule set.
4. Update 0332's text (what to build, which data, which tests).

## Acceptance criteria

- [ ] Nick's answer and words are in `combat-arts.md` (and
      `progression.md` if a class changes), with numbers marked his or
      *tunable*.
- [ ] `docs/design/README.md` has the row.
- [ ] 0332's text says exactly what to build.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

None (a decision); `cargo xtask ticket-lint`.

## Completion notes

*(Filled in by the session that completes the ticket.)*
