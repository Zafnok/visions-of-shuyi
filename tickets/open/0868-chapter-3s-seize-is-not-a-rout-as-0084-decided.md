---
id: "0868"
title: "Chapter 3's seize is not a rout, as 0084 decided"
type: content
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: blocked
blocked_by: ["0084"]
nick_input: answer-first
completed:
---

# 0868 — Chapter 3's seize is not a rout

## Context

Nick played the Act 1 build (Chapters 1 to 4) on 2026-11-08:

> "ch3 seize throne is too similar to just asking the player to kill
> everybody due to the placement of the throne"

> "Too many AI placements and behaviors don't make much sense like the
> random far away archer in ch3 and a lot like to stand still even when
> it doesn't make sense."

Chapter 3, Greywater Abbey (`assets/maps/ch03.map`, `assets/battles/ch03.ron`,
built by 0849; beat sheet `docs/story/chapters/ch03.md`, *The battle
(seize the scriptorium)*):

- **The Seize point** is the prior's seat, a `throne` tile (`'K'`,
  `ch03.map` legend ~:25) at (16, 3), the west end of the scriptorium
  (x 16–21, y 2–4, the long room over the gate), reached only by the
  stair at (18, 5) from the courtyard. `objective: Seize(pos: (16, 3),
  by_lord: true)` (`ch03.ron` ~:60). Only the lead may seize.
- **The prior sits on it** (`greywater_prior`, Mage Lv 6, `Stationary`,
  boss), with a Hound swordsman at (17, 3) and a warden guard at (18, 4)
  beside him; the battle note says "Only the exile can seize the prior's
  seat, and he sits on it."
- **The way there:** the party starts on the shore (x 1–3, y 6–8), goes
  along the causeway past two Hounds at the landing (10, 6), (10, 9), then
  through the gate held by Ned Tolley (15, 7) and a warden (15, 8) with an
  archer in each tower (15, 6), (15, 9), then across the courtyard (a
  warden (18, 7), a raider (20, 9), a rider (24, 7), an archer (22, 12)),
  and up the stair. The reliquary's archer (25, 3) guards three chests.
  A wader can go round by the shallows to the breach at (15, 11).

So the seat is the farthest point from the start, behind every enemy,
and the prior on it must fall first: winning means fighting through
nearly everyone.

**What the story needs:** the beat sheet's battle is "seize the
scriptorium"; the script's after-battle scene `ch03_register`
(`assets/dialogue/ch03.dlg` ~:385–395, `@place battle 18,3`) is in the
scriptorium and has the prior dead ("Someone has covered him"), and the
script's header says "The prior dies". Triggers that must still fire
(`ch03.ron` ~:62–100): `ch03_scriptorium` (`UnitEntersArea`, x 16, y 2,
w 6, h 3), the prior's engage, half-HP and death scenes, Ned's engage and
death scenes.

**0084** asks Nick how to make the seize its own goal; this ticket builds
the answer.

## Nick input

**Answer first:** ticket 0084. Nothing else is asked here (if 0084's
answer lets the battle end with the prior alive, it also says what the
after-battle scene does; that rewrite is a story-writing ticket named in
0084, needed first). Verified by the Chapter 3 winnable test, the bots
and `cargo shuyi frame-png` pictures of the map; what is left to judge
by playing goes on the Act 1 playtest's checklist (1064).

## Scope

**In:** what 0084 decided. Where each likely shape goes:

- **The seat moved** (nearer, or on a second way in): the `K` tile in
  `ch03.map`, `Seize(pos)` in `ch03.ron`, the prior's tile, the battle
  note, and `ch03_scriptorium`'s area if the seat leaves the long room;
  the map's header comment.
- **The prior off the seat** (he stands elsewhere, or comes down to
  meet the party): his tile and `ai` in `ch03.ron`; the battle note's
  text (`battle_notes`).
- **Enemies thinned or moved** (fewer between the gate and the stair;
  the far archers at (22, 12) and (25, 3) moved or gone; reinforcements
  that make a slow, kill-everyone game cost more): the `enemies` and
  `reinforcements` lists.
- **A turn limit** ("within N turns", `turn-structure.md` ~:352–355): the
  battle file's limit, which the objective line shows.
- In every case:
  - Every trigger above still fires in the recorded playthrough (a test
    reads the events: each scene id once), and `ch03_register`'s place
    (18, 3) is still inside the room.
  - The seize point is still drawn as the throne (glyphs `♔♕`, picture
    look the chair, `assets-src/tilesets/tiny_tales.ron` ~:129–135).
  - **Re-balance:** the Chapter 3 playthrough re-recorded on purpose
    (`crates/ui/tests/it/ch03_winnable.rs`; its header says how; its test
    `the_recorded_playthrough_seizes_the_scriptorium` changes if the prior
    no longer falls first), then Chapter 4's (its army changes), and the
    bots on Chapter 3, both seed ranges, inside their bands (`cargo xtask
    playtest ch03 --bot all`; `cargo shuyi autobalance --stage ch03` if
    it leaves them; the personas' seize logic, `seize_distance` and `order` in
    `crates/bots/src/persona.rs` ~:448–530, already walks to any Seize
    point), and Chapter 4
    after the new arrival army. Tables in `docs/playtesting.md`.

**Out (do not do):**
- **0526** (enemies close in instead of waiting): **either order.** If
  0526 is done first, its rule (open-ground `Guard`s wake, bodyguards
  `Hold`) applies to the new placements: give the prior's guards `Hold`
  if they still guard him. If not, 0526's pass reads this ticket's
  placements.
- **0331** (the Seize objective names its place, "the scriptorium"):
  either order; it adds a name field and doesn't care where the tile is.
- The throne's look, the gate's, poor houses: 0471 (art Nick buys).
- The prior's stats, unless 0084 says so.

## Implementation steps

1. Read 0084's answer, 0849's done ticket, `ch03.ron`, `ch03.map`, the
   beat sheet and `ch03.dlg`'s battle and after-battle scenes.
2. Change the map and the battle file as 0084 decided; rewrite both
   files' header comments for the new approach.
3. Re-record the playthroughs (3 → 4), run the bots, autobalance if
   needed, record the tables.
4. `frame-png` pictures of the new map (glyph and picture looks) for the
   PR.

## Acceptance criteria

- [ ] The map and battle file are what 0084 decided.
- [ ] The Chapter 3 winnable test seizes with nobody lost, and a test
      checks every scene trigger above fires once in it.
- [ ] The Chapter 4 winnable test passes with the new arrival army; the
      bots are in their bands on Chapters 3 and 4 (tables in the PR).
- [ ] `frame-png` pictures of the new map in the PR.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the winnable tests of Chapters 3 and 4; the triggers
  test; the content tests (the map loads, the Seize tile is a throne).
- The bots' playtest runs (reported in the PR).

## Completion notes

*(Filled in by the session that completes the ticket.)*
