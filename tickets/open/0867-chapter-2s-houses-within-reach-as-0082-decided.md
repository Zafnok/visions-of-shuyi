---
id: "0867"
title: "Chapter 2's houses within reach, as 0082 decided"
type: content
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: blocked
blocked_by: ["0082"]
nick_input: answer-first
completed:
---

# 0867 — Chapter 2's houses within reach

## Context

Nick played the Act 1 build (Chapters 1 to 4) on 2026-11-08:

> "ch2 houses are hard to get to for all 5. We should have more houses.
> Or introduce an early, weak flying unit"

Chapter 2, Kell's Ford (`assets/maps/ch02.map`, `assets/battles/ch02.ron`,
built by 0848; beat sheet `docs/story/chapters/ch02.md`):

- **Five houses:** (3, 2) and (12, 6) on the north bank; (11, 10),
  (4, 11) and (17, 13) on the south bank. The river is rows 7–8; the
  crossings are the ford road at x 9 and the bridge at x 19. Water costs
  a foot unit 5 and stops mounted and armoured units.
- **The rule:** save at least 3 (Evacuate on the house's tile, or keep
  the raiders off until Hendry falls) **and** defeat Hendry
  (`ch02.ron` ~:67, ~:70: `objective: DefeatUnit(...)`, `village:
  (minimum: 3, rewards: [Item("fordwardens_spear"), Item("elixir")])`:
  the 4th house saved gives the Fordwarden's Spear, the 5th an Elixir).
- **The raiders' race** (the file's header): the two south-bank raiders
  burn the west house and the house by the ford on the Hounds' first
  turn unless evacuated; the bridge-road one reaches the south-east house
  on their second; the north-east one the big house by the water on their
  third, then the north-west one.
- **The party** starts at the south end by the ford road (x 7–10, y
  13–15): the lead (Mov 5), Hollis (4), Tamsin (7), Aske (5), Maud (5),
  Rue (5). There are no fliers in Act 1: the Flier class is tier 3
  (`classes.ron` ~:345–353; `docs/design/progression.md` ~:766, *Fliers
  are late-game*).
- Scenes tied to a house: `ch02_house_keeper`, Maud at the house by the
  ford (`UnitEntersArea` at (11, 10), `ch02.ron` ~:93).

**0082** asks Nick how to put the houses within reach; this ticket
builds the answer.

## Nick input

**Answer first:** ticket 0082. Nothing else is asked here. Verified by
the bots, the Chapter 2 winnable test and `cargo shuyi frame-png`
pictures of the map in the PR; what is left to judge by playing goes on
the Act 1 playtest's checklist (1064).

## Scope

**In:** what 0082 decided. Where each likely shape goes:

- **More houses, or houses moved nearer:** `H` tiles in `ch02.map` (the
  map's picture look draws a `house` tile from the tileset with no other
  work; check the glyph and sprite pictures), the `village` minimum and
  `rewards` in `ch02.ron` (a reward per house past the minimum, as 0082
  says), and the raiders: their starting tiles or count so the race still
  reads as the file's header says (rewrite the header for the new race).
  The raider rule (`crates/core/src/ai.rs`, rule 5 in the module doc)
  finds every standing house, so it needs no change.
- **A lower minimum** (e.g. 3 → 2): the `village` minimum alone.
- **Raiders later or slower:** their tiles or the number of raiders.
- **An early, weak flier.** A new playable flier is a story and class
  decision (0082's): a new tier-1 flying class (`classes.ron`, `tags:
  [Flying]`, bows ×3 against it) and a new named character, or a green
  ally. If 0082 picks it, this ticket does **the battle side only**: the
  class data and the unit's slot in Chapter 2. The character's join
  scene, sheet, face and map sprite are their own tickets
  (story-writing, art), listed in the Completion notes and in
  `blocked_by` if the battle needs the scene first; `progression.md`'s
  *Fliers are late-game* is rewritten with Nick's words.
- In every case:
  - `ch02_house_keeper`'s area still on the house by the ford, or moved
    with it; every trigger in `ch02.ron` still fires in the recorded
    playthrough.
  - The beat sheet (`docs/story/chapters/ch02.md`, *The battle*) and the
    script don't name a count of houses today (`ch02.dlg`: "those
    houses"); keep it so.
  - **Re-balance:** the Chapter 2 playthrough re-recorded on purpose
    (`crates/ui/tests/it/ch02_winnable.rs`, its header says how), then
    Chapters 3 and 4's (the army they get changes), and the bots on
    Chapter 2, both seed ranges, inside their bands (`cargo xtask
    playtest ch02 --bot all`; `cargo shuyi autobalance --stage ch02`
    if it leaves them), and on Chapters 3 and 4 after the new arrival
    army. Tables in `docs/playtesting.md`.

**Out (do not do):**
- **0481** (the objective shown when a battle starts, with AND / OR):
  either order. It reads the minimum and the objective from the battle
  file, so a new minimum shows by itself; if 0481 is done first, check
  its Chapter 2 snapshot and refresh it on purpose.
- **0528** (enemies close in): either order; whichever lands second
  re-runs Chapter 2's bands with the other in.
- The house's look in the picture look (0471, art Nick buys).

## Implementation steps

1. Read 0082's answer, 0848's done ticket, `ch02.ron`, `ch02.map`, the
   village rules (`docs/design/turn-structure.md`, *Save the village*),
   and the Chapter 2 winnable test.
2. Change the map and the battle file as 0082 decided; rewrite the
   battle file's header to say the new race.
3. Re-record the playthroughs (2 → 4), run the bots, autobalance if
   needed, record the tables.
4. `frame-png` pictures of the new map (glyph and picture looks) for the
   PR.

## Acceptance criteria

- [ ] The map and battle file are what 0082 decided.
- [ ] A test: from the party's start, the houses 0082 asked to be
      reachable are (e.g. the turns a unit of the party needs to stand on
      each house, by `movement::reachable`, under the number 0082 gave).
- [ ] The Chapter 2 to 4 winnable tests pass with re-recorded
      playthroughs; the bots are in their bands on Chapters 2 to 4
      (tables in the PR).
- [ ] `frame-png` pictures of the new map in the PR.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the reach test above; the winnable tests of Chapters 2
  to 4; the content tests (the map loads, the village has its houses).
- The bots' playtest runs (reported in the PR).

## Completion notes

*(Filled in by the session that completes the ticket.)*
