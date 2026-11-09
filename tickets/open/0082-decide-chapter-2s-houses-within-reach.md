---
id: "0082"
title: "Decide: Chapter 2's houses within reach"
type: design-decision
milestone: Design decisions
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 0082 — Decide: Chapter 2's houses within reach

## Context

Nick played the Act 1 build (Chapters 1–4) on 2026-11-08:

> "ch2 houses are hard to get to for all 5. We should have more houses.
> Or introduce an early, weak flying unit"

**The rule** (Nick's, `docs/design/turn-structure.md`, *Save the
village*, ~:415): villages appear only when the story is about them, "as
a real race, never an optional extra". A house is saved by a unit's
`Evacuate` on its tile, or by still standing when the battle is won; a
raider that ends its turn on or next to it burns it. Kell's Ford asks
for **at least 3 of 5** and the defeat of Hendry (`assets/battles/
ch02.ron` `village`, `objective`); the 4th house saved gives the
Fordwarden's Spear, the 5th an Elixir. The design's own example ends
with four standing. So "all 5 is hard" is partly by design; how hard is
Nick's call. Treat his answer as a nudge.

**The map** (`assets/maps/ch02.map`, 24×16; the party starts at the
bottom by the road, x 7–10, y 13–15; `~` is the river, `:` the road and
the ford at x 9, `=` the bridge at x 19, `H` the houses):

```
TT.......:......TT..^^AA   y 0
TTT......:.......T..###A
TT.H.....:..,.......#F#^   (3,2) north-west house
.T.......:::::::::::###^
.........:..TT.....:...T   Hendry (9,5), archer (11,5)
..TT.....:.........:..TT   guards (8,6), (10,6)
.........:..H......:....   (12,6) the big house by the water
~~~~~~~~~:~~~~~~~~~=~~~~   y 7  river
~~~~~~~~~:~~~~~~~~~=~~~~   y 8  river
.........:.........:...T
....T....:.H.......:..TT   (11,10) house by the ford
..TTH....::::::::::::...   (4,11) west house
..TT.....:......,.......
.........:.......H...TT.   (17,13) south-east house
...^^....:..........TTT.
..^^^....:..........TT..   y 15
```

Water costs a foot unit 5 (a whole turn) and stops mounted and armoured
units (`terrain.md`). Mov: Lead 5, Hollis 4, Tamsin 7, Aske 5, Maud 5,
Rue 5.

**The race** (the battle file's comment, with the raiders as placed):

| House | Burns on the Hounds' turn | Movement needed, Hounds standing where they start (turns) |
|---|---|---|
| (4,11) west | 1 | Aske 5 (1), Tamsin 7 (1) |
| (11,10) by the ford | 1 | Lead 5 (1), Rue 6 (2) |
| (17,13) south-east | 2 | Lead 8, Hollis 8 (2); Tamsin 10 (2) |
| (12,6) big house, north bank | 3 | Rue 19, Lead 18 (4); Tamsin 26 by the bridge (4) |
| (3,2) north-west | after (12,6) | Aske 23, Lead 25 (5); Tamsin 39 (6) |

(Worked out with the Hounds as walls at their start tiles; the session
checks it on the real map, since the Hounds move.) Hendry and his two
guards close the ford's north end, so the north bank means the bridge
or wading. The big house needs 4 turns and burns on the 3rd, so it is
saved only by killing its raider (who starts at (23,4), far north-east)
or Hendry before then; the north-west house follows it. The three south
houses are the "save 3" the battle is built around.

**Fliers.** Nick decided fliers are late-game (0017, `docs/design/
progression.md` ~:766): "No flying class below tier 3. Nobody starts as
a flier". The only flying class is the tier-3 Flier (`classes.ron`
~:345). Bows are ×3 against Flying (`items.ron:52`); Chapter 2 has two
Hound archers (Lv 5, Iron Bow: Str 6 + 18 = 24 power against a flier).
An early flier changes 0017's rule, so say so plainly.

Nick paused AI story writing on 2026-11-08 until a human writer is hired
(ticket 0766). Any option that needs new dialogue or a new named
character says so: its words wait for the writer.

## Nick input

**Decision** (`ask-nick`), with `frame-png` pictures of Kell's Ford for
each map option, in both the glyph look and the picture look (the
private art is fine in screenshots), with the raiders and each house's
turn count marked. Check every game claim before showing it.

**1. First, how hard should "all 5" be?** Possible but a stretch (today,
roughly), reachable by a good plan, or easy?

**2. Which change?**

- **A. More houses, the same "save 3" (Nick's own idea).** For example
  7 houses, the two new ones on the south bank within two turns. The
  minimum is easier to pass; saving all is still a race. Ask what the
  6th and 7th give, if anything (today the 4th gives the Fordwarden's
  Spear and the 5th an Elixir). Like the GBA Fire Emblems' village maps, where brigands head
  for villages and a destroyed village's gift is lost.
- **B. Bring the north-bank houses into reach**: move the big house to
  the ford's south end or the north-west house to the south bank, so the
  route doesn't run past Hendry.
- **C. Fewer houses**: 4, still "save 3" (drop the north-west house).
- **D. A crossing for Tamsin.** A second bridge or a horse ford at about
  x 14: then the big house is 3 turns for Tamsin, the Lead and Rue, just
  in time. The battle's first scene already has Tamsin say she's the
  fastest and takes the houses.
- **E. Slower raiders**: the north-east raider waits a turn before
  walking (one notch), so the big house burns on turn 4.
- **F. An early, weak flying recruit (Nick's other idea)**, like
  Vanessa, the pegasus knight who joins in Chapter 2 of Fire Emblem: The
  Sacred Stones. It needs: Nick to change 0017's rule (a tier-1 flying
  class), a new named character (a **story decision**; her scenes wait
  for the writer, 0766), and a look at the archers (one Chapter 2 arrow
  at ×3 does about 20 to a weak flier).
- **G. Combinations, or describe your own** (for example A with D).

If 0083 (dismounting) lets a rider on foot wade, say that Tamsin on foot
could cross the river at 5 too.

## Scope

**In:** the turn table checked on the real map, the pictures, the
question, Nick's answer and words in `turn-structure.md` (*Save the
village*) and the Chapter 2 beat sheet's battle section (`docs/story/
chapters/ch02.md`, *3. The battle*) if the map's story changes; the
`docs/design/README.md` row; 0867's text updated with what to build.

**Out (do not do):**
- Building it (the map, the battle file, the bots): ticket **0867**.
- New dialogue or a new named character: waits for the writer (0766).
- The dismount rule (0083). Either order: if 0083 is answered, use its
  rule in the turn table; if not, note D/F against it there.
- The house tile's look (0471).

## Implementation steps

1. Read `turn-structure.md` (*Save the village*), `progression.md`
   (*Fliers are late-game*), `terrain.md`, `ch02.map`, `ch02.ron`, the
   beat sheet `ch02.md`.
2. Check the table: walk each unit to each house on the real map
   (pathfinding with the Hounds' first moves, or the bots' turn log).
3. Make the pictures: throwaway copies of `ch02.map`/`ch02.ron` per
   option (never committed), `cargo shuyi frame-png` in both looks.
4. Ask Nick; iterate; record his words and the rule set.
5. Update 0867's text.

## Acceptance criteria

- [ ] Nick's answer and words are recorded (`turn-structure.md`, and
      `ch02.md` if the map's story changes); `docs/design/README.md` row.
- [ ] 0867's text says exactly what to build.
- [ ] No throwaway map or battle file is committed.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

None (a decision); `cargo xtask ticket-lint`.

## Completion notes

*(Filled in by the session that completes the ticket.)*
