---
id: "0510"
title: "cargo xtask skirmish: generate a balanced skirmish from a request (place, level, unit mix, chapters)"
type: feature
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: ["0509", "1007", "1008"]
nick_input: none
completed: 2026-11-01
---

# 0510 — Generate a skirmish to order

## Context

Nick (0033, `docs/design/playtest-bots.md`, *Generating a skirmish to
order*): "I will tell you I think the map is empty and I want a skirmish on
this part of the grid and you will put one that makes sense for the lv I
think of." His examples:

- "a skirmish on grid 10,10 of open world map that's lv35 scaled
  difficulty, made up of a mix of units"
- "a skirmish on grid 13,13 lv40 scaled difficulty showing up in ch2 only
  made up wholly of mages"

**"Lv 35" means a lv 35 battle** (Nick): enemies around lv 35, and the bots
play it with a lv 35 army of the units the player would have by then.
Random skirmishes may change anything when autobalanced, terrain included
(Nick, Q11); fixed ones only enemy numbers (0509).

World map nodes and their grid come from 1007; skirmish nodes, level
markers and the random-skirmish pool from 1008; autobalancing from 0509.

## Nick input

None. Nick no longer checks gameplay tickets after merge (Nick,
2026-10-21: "I will save all my checking for either design (i.e. 00
tickets) or for the full act 1 playtest itself"). Verification is by
tests, `cargo shuyi frame-png` pictures in the PR and the playtest bots
(0506, 0509). What the old sign-off asked goes on the checklist of
Nick's next playtest (1064, the Act 1 playtest, if this lands before
it):

> Nick asks for one skirmish in his own words, plays it on the
> Pages build, and says whether it felt like "a lv N battle".

## Scope

**In:**
- `cargo xtask skirmish new --at <x,y> --level <n> [--units mix|<class>|<weapon type>] [--chapters <list>] [--kind fixed|random] [--tier easy|normal]`.
- A **reference army** builder: the units recruited by a given chapter, at
  level N, with ordinary gear for that point.
- Map generation for skirmishes, and the random-skirmish knobs 0509 left
  out (positions, reinforcements, terrain).
- Adding the result to the game data (battle file + world map node).

**Out (do not do):**
- Generating skirmishes while the game runs: all generation is at authoring
  time. Random skirmishes in the game still pick from a pool (1008); this
  tool can fill that pool.
- Story battles and side quests (authored by hand; 0509 balances them).

## Implementation steps

**Size (0135, ADR-0101):** about 2,600-3,400 changed lines. The natural cut, if it runs over: the army,
the enemies, the CLI and the writer here; the map generator and the
autobalance knobs in a new ticket blocked by this one.

1. **Reference army** (`crates/bots/src/army.rs`): from the story data, the
   player units recruited by the requested chapter (earliest if several);
   each at level N using its class's average growths, promoted where the
   class rules require (`progression.md`); gear = the best ordinary weapons
   and items the shops sell by that chapter. Document the approximation in
   `docs/playtesting.md`.
2. **Map generation**: a small map (size from `world-structure.md`'s
   skirmish sizes) using the terrain of the world map region around the
   grid point (1007), from a few layout templates with random variation.
   Deterministic from `--seed`.
3. **Enemies**: 4–6 enemies (random skirmish) or the fixed-skirmish size,
   classes from `--units`, around level N, placed by the generator.
4. **Balance**: run 0509's search with the full random-skirmish knob set
   (add position, reinforcement and terrain knobs here) until the three bots
   are inside the bands for the tier (Easy by default for random, as
   `world-structure.md`).
5. **Write**: a new battle file under `assets/battles/`, and the node at
   `<x,y>` in the world map data with its kind, level marker and chapters.
   Print the 0506 report for the result.
6. **Docs:** `docs/playtesting.md` gets a *Generating skirmishes* section with
   Nick's two example requests as commands.

## Acceptance criteria

- [x] `cargo xtask skirmish new --at 10,10 --level 35 --units mix` writes a
      battle file and node that pass content validation, with enemies
      around lv 35 and all three bots inside their bands.
- [x] `--units mage --chapters 2` produces only mage enemies and a node
      shown only in Chapter 2.
- [x] Same arguments and seed → identical files.
- [x] Unit test `reference_army_levels_and_roster`: for a chapter, the army
      holds exactly the units recruited by then, all at level N.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: reference army; map generator determinism; unit-mix filter.
- Property: generated battles always pass content validation.
- Snapshot / integration: one generated skirmish end to end.

## Completion notes

Done: `cargo xtask skirmish new` generates a small map and a battle, lands the
three bots in their bands with the engine's search, and writes the battle, its
map and a place on the world map, or an entry in the random pool. The reference
army is `crates/bots/src/army.rs`; battles keep it in a new `bots_army` field,
so `playtest` and `autobalance` judge them the same way.

**Measured with the full 100 tries**, on the test world map, then removed (they
are not committed):

| Request | Seed | Casual | Normal | Hardcore | Search |
| ------- | ---: | -----: | -----: | -------: | ------ |
| `--at 10,10 --level 35 --units mix` | 2 | 94 | 68 | 78 | 35 min |
| `--at 13,13 --level 40 --units mage --chapters 2` | 1 | 71 | 65 | 76 | 12 min |

The written files passed `trpg-content`'s tests after a rebuild, and
`cargo xtask playtest` gave the same three lv 35 numbers. A seed can miss:
lv 35 seed 1 ended with Hardcore at 83, over its band, and wrote nothing.
Same arguments and seed gave identical files in the end-to-end test.

**Nick's decision while working** (`docs/design/playtest-bots.md`, *Skirmishes
have their own bands*): the first lv 35 battle lasted about 3 turns, with Casual
at 95 and Normal at 55, because winning is far easier than nobody dying in so
short a fight. Enemies that hold position didn't help (Casual 78, Normal 32,
Hardcore 38, on a smaller search of 40 tries). Nick picked a Casual band of its
own for skirmishes and added "normal and HC can also be slightly easier for
skirmishes". This applies to every fixed and random skirmish, not only generated
ones.

**The agent's starting rules:**

- Skirmish bands: Casual wins 70–100%. Normal and Hardcore are 5 points easier
  than the same tier's usual band (Normal tier 55–80%, Easy tier 70–95%).
- A generated map is 14 by 10 tiles, with a clear row across its middle. Six
  enemies start within a level of the level asked for, and the search may move
  each by at most 3 levels more, so a lv 35 battle stays about lv 35.
- `--units mix` picks classes of the tier the army has reached by that level.
  `--units mage` means the tier-1 Mage at any level. Healers are left out because
  they can't win a fight.
- The army promotes at lv 13 and lv 33, into the first class on its list. A
  player's real timing and class picks will differ.
- A generated fixed skirmish defaults to Normal tier, is optional (it never
  blocks a road) and is named "Skirmish" on the map unless `--name` says
  otherwise. Its road comes from the nearest place.
- The new road never starts at a place 1070 marks `NotBuilt`, since such a road
  would never open.
- A place limited to some chapters stays out of sight outside them. Once won, it
  stays on the map, as every won place does.
- `--kind random` adds to the map's pool and makes no place, because random
  groups appear on cleared places; `--chapters` is refused with it.

**Deviations:**

- Split before coding (ADR-0101), as the ticket's own cut says: layout templates
  and the position, reinforcement and terrain knobs are ticket 0522. This ticket
  has one layout and balances with level, count, class and weapon.
- `--kind random` has only the class knob (the pool refuses or overwrites the
  rest), so it can write only when a battle's classes alone land it in its bands.
  It was exercised end to end with `--force`, not balanced at full tries. A
  one-class random battle has nothing to turn until 0522.
- `--map` is new. The default is the story's first world map (Act 1's, from 1070,
  which landed during this ticket). Chapter 2 isn't on that map (it opens at
  Chapter 4, `world-structure.md`), so Nick's "Chapter 2" example works in the
  tool and its tests but such a place can't be reached in real play; a later
  chapter number can.
- The enemy templates are `skirmish_<class>` rows added to `characters.ron`,
  because the data had no generic mage or any tier 2 or 3 generic.
- `test_too_hard` became a side quest, with the same knobs, so its test keeps the
  usual bands.
- No `frame-png` picture: the tool has no screen of its own and `frame-png` can't
  open a named battle directly.

**Known flags:** the report's "Hardcore faster than Normal" line reads ✗ on the
lv 35 battle (median 3 turns each), because fights this short leave little to
save. It is reported, not part of the bands. Generating two skirmishes in a row
needs `cargo run` for each, so the build picks up the first one's files; a
stale binary refuses the second.

Follow-up: 0522. For the Act 1 playtest (1064): the starting rules above, and
Nick's one request in his own words.
