---
id: "0084"
title: "Decide: Chapter 3's throne and the enemies around it"
type: design-decision
milestone: Design decisions
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 0084 — Decide: Chapter 3's throne and the enemies around it

## Context

Nick played the Act 1 build (Chapters 1–4) on 2026-11-08:

> "ch3 seize throne is too similar to just asking the player to kill
> everybody due to the placement of the throne"

> "Too many AI placements and behaviors don't make much sense like the
> random far away archer in ch3 and a lot like to stand still even when
> it doesn't make sense. I can give a pass for when they want to stand in
> advantageous terrain but overall they should advance toward the player
> more often"

**The map** (`assets/maps/ch03.map`, Greywater Abbey; `K` the prior's
seat, the Seize point; `G` the gate; `F` the gate towers; `~` the lake,
which only fliers cross; `w` shallows a foot unit can wade at 5):

```
          x 0         1         2
            0123456789012345678901234567
y  1        TT..~~~~~~~~~~~#############
   2        T...w~~~~~~~~~~#______#____#   scriptorium x 16-21; reliquary x 23-26
   3        ....w~~~~~~~~~~#K_____#____#   K = (16,3), the prior on it
   4        ....w~~~~~~~~~~#______#____#
   5        ....~~~~~~~~~~~###_#####_###   stair (18,5); reliquary door (24,5)
   6        ....~~~~~::~~~~F__.........#
   7        ....:::::::::::G__.........#   causeway y 7-8, gate (15,7)-(15,8)
   8        ....:::::::::::G__..____...#
   9        ....~~~~~::~~~wF__.._TT_...#
  10        ....~~~~~~~~~ww#...._TT_...#
  11        ....w~~~~~~~~~w.....____...#   breach (15,11)
  12        ....w~~~~~~~~~~#.........TT#
```

The party starts on the shore (x 1–3, y 6–8). `assets/battles/ch03.ron`:
`Seize(pos: (16,3), by_lord: true)`; the battle note says "Only the exile
can seize the prior's seat, and he sits on it."

**Why it plays as "kill everybody".** The seat is the farthest tile from
the start (22 movement for the lead with nobody in the way: 5 turns), at
the end of the only route: through the two-tile gate (held by Ned at
(15,7) and a guard at (15,8), Def 18 on the gate), across the courtyard,
up a one-tile stair (18,5) whose top (18,4) a guard holds, past a
swordsman at (17,3), and onto the prior (Mage Lv 6, HP 24, Res 12,
**Stationary** on the seat). The only other way in is wading the
shallows to the breach (15,11). **Every other enemy is `Guard`** (attacks
only what it can reach this turn, else holds; `crates/core/src/ai.rs`
~:218), except the two tower archers (15,6) and (15,9) (`Stationary`).
So nobody comes out to meet the party; it walks into each group in turn
and kills it.

**The far archers.** `warden_archer` at (22,12), the courtyard's
south-east corner (23 movement from the start, farther than the seat),
and at (25,3), in the reliquary by its chests. Both `Guard`. The route to
the stair passes about 9 tiles from (22,12), out of its reach (Mov 5 +
range 2), so it may never act: Nick's "random far away archer". The
reliquary one at least guards something (the chests).

**Story limits.** The beat sheet (`docs/story/chapters/ch03.md`, *The
prior dies*): the prior dies at his desk, still talking, burning the
register; Maud takes it from the desk. Moving him or his seat is a
staging change to check against that; new or changed dialogue waits for
the writer (Nick paused AI story writing on 2026-11-08 until one is
hired, ticket 0766).

**Not this ticket:** how the AI decides to advance in general is 0528's
(Nick's "overall they should advance toward the player more often").
This ticket decides Chapter 3's own map: where the seat is, who stands
where, and which units here hold or come.

## Nick input

**Decision** (`ask-nick`), with `frame-png` pictures of Greywater for each
option, in both the glyph look and the picture look (the private art is
fine in screenshots), each enemy's behaviour marked, and the seat's
distance in turns. Check every game claim before showing it; claim only
what the game really does.

**1. The seat and the route.**

- **A. A seat reachable without clearing the map.** Move the Seize point
  nearer the gate (for example the scriptorium over the gate, as the beat
  sheet first describes it, with its stair inside the gate passage), so
  a fast push can take it while the courtyard is still held.
- **B. A second way up.** Open the scriptorium to the reliquary (a door
  in the wall at x 22) or a back stair by the breach, so there are two
  routes and the player picks one.
- **C. The prior off the seat.** He stands at the hearth or by the stair
  and comes at the party; the seat is empty, so killing him isn't the
  only way. Downside: the beat sheet has him die at his desk; check with
  Nick, and changed scenes wait for the writer.
- **D. Pressure, so a slow sweep costs something.** Reinforcements (more
  Hounds over the causeway behind the party from a set turn, as many Fire
  Emblem chapters send on set turns; the game's rule is `turn-structure.md`,
  *Reinforcements*), or a turn limit (`Seize within N turns`, also
  already a rule there). Into the Breach puts every mission on a turn
  count, for the feel.
- **E. Combinations, or describe your own.**

**2. Who comes, and who holds** (this map's units only):

- The courtyard's units: `Aggressive` (they come out over the causeway),
  hold until the gate falls then come, or keep `Guard`?
- The far archer at (22,12): move it somewhere it matters (the stair's
  head, a wall over the courtyard), make it come, or drop it?
- The reliquary archer (25,3): keeps guarding the chests?
- The tower archers (forts beside the gate): stay put (Nick's pass for
  "advantageous terrain")?

## Scope

**In:** the pictures, the question, Nick's answer and words in the
Chapter 3 beat sheet's battle section (`ch03.md`, *3. The battle*) and,
if a rule changes (a Seize rule, reinforcements), `turn-structure.md`;
the `docs/design/README.md` row; 0868's text updated with what to build.

**Out (do not do):**
- Building it (map, battle file, bots): ticket **0868**.
- The general AI rule for advancing: ticket **0528**. Either order: if
  0528 is decided first, show its rule here; if not, this ticket's
  choices per unit stand and 0528 must not undo them.
- The throne's picture (0471) and the seize objective's name (0331).
- New or changed dialogue: waits for the writer (0766).

## Implementation steps

1. Read `ch03.md` (beat sheet), `ch03.map`, `ch03.ron`, `turn-structure.md`
   (*Reinforcements*, *Turn limits and objectives*), `terrain.md`
   (gate, throne and Seize, ~:146–166), `ai.rs` (module doc, `AiBehavior`).
2. Make the pictures: throwaway copies of `ch03.map`/`ch03.ron` per
   option (never committed), `cargo shuyi frame-png` in both looks;
   optionally a bots run per option to show turns and win rates.
3. Ask Nick; iterate; record his words and the result.
4. Update 0868's text (and tell 0528 what this map decided).

## Acceptance criteria

- [ ] Nick's answer and words are recorded (`ch03.md`, and
      `turn-structure.md` if a rule changes); `docs/design/README.md` row.
- [ ] 0868's text says exactly what to build.
- [ ] No throwaway map or battle file is committed.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

None (a decision); `cargo xtask ticket-lint`.

## Completion notes

*(Filled in by the session that completes the ticket.)*
