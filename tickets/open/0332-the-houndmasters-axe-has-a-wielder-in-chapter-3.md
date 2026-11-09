---
id: "0332"
title: "The Houndmaster's Axe has a wielder in Chapter 3"
type: feature
milestone: M2 Core rules
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: blocked
blocked_by: ["0079"]
nick_input: answer-first
completed:
---

# 0332 — The Houndmaster's Axe has a wielder in Chapter 3

## Context

Nick played the Act 1 build (Chapters 1 to 4) on 2026-11-08:

> "nobody can use the houndmaster axe reward from ch2 in ch3, since none
> of the starting units have axe speciality unlocked -- we should either
> add the speciality to one of our units or add an axe unit or convert a
> unit to an axe wielding class"

What the game does today:

- **The axe.** `assets/data/items.ron:68`, id `houndmasters_axe`: Axe,
  rank D, Mt 10, Hit 70, Crit 5, Wt 7, Rng 1, Dur 25, price 900,
  `arts: ["pack_hunt"]` (Pack Hunt, `assets/data/arts.ron:192`: might +2
  for each other ally next to the target, up to +6, costs 3 durability),
  `special: true` with no personal wielder. Hendry, the Chapter 2 boss,
  carries it; `assets/battles/ch02.ron:66` (`clear_items`) puts it in the
  party's stock when Chapter 2 is won (`Campaign::apply_result`,
  `crates/core/src/campaign.rs` ~:586). Design: `docs/design/combat-arts.md`
  ~:538–552, *Special weapons*.
- **Who may wield it.** `Unit::can_wield` (`crates/core/src/item.rs`
  ~:696): the unit's **class** must list the weapon's kind
  (`class.weapon(kind)`), the unit's rank in it must be at least the
  weapon's (D), and a personal weapon only its owner. A unit's own
  `weapon_ranks` can't add a kind its class lacks.
- **The Chapter 3 army** (`assets/battles/ch03.ron` `player_slots`):
  lead (Exile, Sword), Hollis `retainer` (Guard, Spear), Tamsin
  `sergeant` (Rider, Sword and Spear), Aske `poacher` (Archer, Bow), Maud
  `keeper` (Cleric, Gauntlet), Rue `heretic` (Mage, Sword), Joss
  `defector` (Swordsman, Sword) (`assets/data/classes.ron` ~:44–116). No
  tier-1 class of theirs has axes. The tier-2 classes with axes are
  Grappler (from Brawler), Bulwark (from Guard), Iron Rider (from Guard or
  Rider) and Vanguard (from Raider); promotion is only in the debug menu
  today (0606 builds it in the between-battle menu) and no seals are
  placed in Act 1's data. The one axe class at tier 1 a player could be is
  Raider (Axe D–C).

So the axe Chapter 2 hands over is dead weight for the rest of Act 1.
**0079** asks Nick which way to fix it; this ticket builds the answer.

## Nick input

**Answer first:** ticket 0079 (who wields the Houndmaster's Axe). Nothing
else is asked here. Verified by tests and the bots; what is left to
judge by playing (whether the wielder feels worth fielding) goes on the
Act 1 playtest's checklist (1064).

## Scope

**In:** what 0079 decided, which is likely one of these shapes (the
session builds the one Nick picked; the others are here so it knows
where each goes):

- **A. A unit gains the axe as a speciality, outside its class.** A rule
  in `core`: a unit may wield a kind its class doesn't list when its
  character data gives it that kind (a new optional field on the
  character, e.g. `extra_weapons: [(Axe, D, C)]`, start and max rank like
  a class's `weapons`), read by `can_wield` and by weapon-rank growth
  (`WeaponExpGained` / `WeaponRankUp` in `crates/core/src/battle.rs`, the
  max rank coming from that field). Data: the unit 0079 named, in
  `assets/data/characters.ron`.
- **B. A class gains axes.** A data change to that class's `weapons` in
  `classes.ron` (every unit of that class, enemies too: say so in the
  completion notes and check the enemy loadouts still pick what they
  picked).
- **C. A unit changes class.** Either in the data (the unit starts or
  joins as Raider, or another class 0079 named, with its stats moved
  as `progression.md`, *Reclass* ~:472, says), or the player reclasses
  in the between-battle menu, which is 0606's: if 0079 picks that,
  add 0606 to `blocked_by` and this ticket only adds what 0606 doesn't
  (a seal or a class the player can reach before Chapter 3).
- **D. A new axe unit joins.** A new named character is a story decision
  (Nick's, in 0079). This ticket then does the **battle side only**: the
  character's data (class, level, stats as 0079's answer gives or as the
  agent's starting values marked *tunable*), their slot in
  `ch03.ron` and later chapters' `player_slots`, the `@joins` directive
  where 0079 says they join. The scene, the character sheet, the face and
  the map sprite are their own tickets (story-writing, ascii-art and art
  purchases); this ticket names them in its completion notes and in
  `blocked_by` if the join needs the scene first.
- In every case: `docs/design/combat-arts.md` *Special weapons* (and the
  character sheet in `docs/story/characters/` if it lists weapons) say who
  can wield the axe.
- The bots on Chapters 3 and 4 inside their bands after the change
  (`cargo xtask playtest ch03 --bot all`, then `ch04`), both seed ranges,
  as 0849 and 0850 ran them.

**Out (do not do):**
- Changing the axe's numbers or Pack Hunt.
- Promotion seals or the between-battle reclass screen (0606), unless
  0079 picks C through it (then as above).
- A new character's scene, sheet, face or sprite (D, above).
- Making the axe stronger or weaker to fit the bands: if the bands move,
  the enemies move (`docs/design/playtest-bots.md`, *Autobalancing*).

## Implementation steps

1. Read 0079's answer in `docs/design/` (where 0079 recorded it),
   `can_wield` and its callers (`grep -rn can_wield crates`), and
   `progression.md` *Reclass* if the answer is C.
2. Build the shape 0079 picked (above). For A, the rule goes in
   `crates/core/src/item.rs` (`can_wield`) and the loader in
   `crates/content/src/character.rs`; every caller (Preparations'
   `usable`, the shop, the arts list, the AI's `arms`, the bots' army)
   then follows without changes.
3. Data in `assets/data/characters.ron` or `classes.ron` as decided.
4. Add the test below; run the Chapter 2 and 3 winnable tests. If the
   change moves a recorded playthrough (a unit's stats or class changed),
   record it again on purpose as the test's header says
   (`cargo test -p trpg-ui --test it ch03_winnable::record -- --ignored`)
   and then Chapter 4's.
5. Run the bots on Chapters 3 and 4; put their tables in
   `docs/playtesting.md` and the PR.

## Acceptance criteria

- [ ] A test plays Chapters 1 and 2's recorded playthroughs, takes the
      army that arrives at Chapter 3 (`chapter_3` in
      `crates/ui/tests/it/ch03_winnable.rs`), and finds at least one unit
      in a Chapter 3 player slot for which `can_wield(houndmasters_axe)`
      is true, with the axe in the stock.
- [ ] That unit can equip it in Preparations' Loadouts (a test through
      the Preparations view, `usable` true on the axe's stock row).
- [ ] 0079's answer is what the data says; the design doc says who wields
      the axe.
- [ ] The bots are in their bands on Chapters 3 and 4 (the report in the
      PR).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: for shape A, `can_wield` with an extra kind (and refused without
  it, and refused below rank D) in `crates/core/src/item/tests.rs`.
- Integration: the Chapter 3 army test above; the winnable tests of
  Chapters 2 to 4.
- The bots' playtest runs (reported in the PR).

## Completion notes

*(Filled in by the session that completes the ticket.)*
