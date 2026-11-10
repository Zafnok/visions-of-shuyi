---
id: "1088"
title: "The world map's cursor goes where the key points"
type: bug
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-11-09
---

# 1088 — The world map's cursor goes where the key points

## Context

Nick's playtest of the Act 1 build (2026-11-08):

> "The world map cursor movement is janky and probably bugged it kinda
> just goes wherever it wants"

**Today's rule** (`WorldMapScreen::step_cursor`,
`crates/ui/src/screens/world_map.rs` ~:268–294; *the agent's starting
rule* from 1007, `docs/design/world-structure.md` ~:224 and
`docs/design/controls.md` ~:100): among **every shown place**
(`open_places` = `WorldMap::shown_nodes`, open ones and the greyed ones a
road away), score each by its pixel offset (a cell is 8 × 16): `along =
x·dx + y·dy`, `across = |x·dy| + |y·dx|`; a candidate if `along > 0` and
`across ≤ 2·along` (about ±63°); the lowest `along + 2·across` wins; none →
the Denied sound. It ignores roads, treats greyed places as stops, and
isn't reversible.

**Repro** on Act 1's map at Chapter 4 (`assets/worldmaps/act1.ron`, before
Veyne Hall is won; shown: Greywater, The Valley Road, Millhaven, Millhaven
Road, The Company Ledger, Aldwater Toll Bridge, Veyne Hall, The Captain's
Oath, The Salt Road), worked out from the code above:

- On **Millhaven**, Right goes to **Veyne Hall** (no road joins them;
  Millhaven's roads go to The Valley Road, Millhaven Road and The Company
  Ledger). Left from there goes to **The Captain's Oath**, not back.
- On **Aldwater Toll Bridge**, Up goes to **Millhaven** and Down to
  **Greywater** (no road to either); Right goes to **The Salt Road**
  (greyed, not built) instead of Veyne Hall, where Aldwater's road leads.
- On **Veyne Hall**, Left goes to **The Captain's Oath** (greyed), not
  back along the road to Aldwater.

The test `the_cursor_jumps_to_the_nearest_place_that_way`
(`crates/ui/src/screens/world_map/tests.rs` ~:143) pins today's rule on
the test map.

The engine has an open ticket for this kind of pick: engine#2355 (FocusMap:
the d-pad moves between widgets by where they sit, with neighbours set by
hand, and a check that every widget can be reached).

## Nick input

None. *The agent's starting rule* below replaces the 1007 one in
`world-structure.md` and `controls.md`; it goes on the Act 1 playtest's
checklist (1064). Verified by the tests below.

**The agent's starting rule: the cursor follows the roads.**

1. A direction key looks only at the shown places joined to the cursor's
   place by a road the map draws (open or not yet open).
2. It picks the one whose straight line from here is closest in angle to
   the key's direction (in pixels, a cell 8 × 16), if it is less than 90°
   from it; none that way → the cursor stays, with the Denied sound.
3. The opposite key, pressed right after a step, goes back to the place
   the cursor came from.
4. Greyed places are stops like any other: each one shown has a road to an
   open place (`WorldMap::is_shown`), so the cursor can always leave it.

Example, the repro's spots: on Millhaven, Right has no road that way
(stays); Down goes to The Valley Road. On Aldwater, Right and Up go to
Veyne Hall (the road east and north), Left to The Valley Road; Down stays.
On Veyne
Hall, Left goes back to Aldwater, Up to The Captain's Oath, Down to The
Salt Road. A quick simulation of this rule on `act1.ron` with every place
shown reached every place from Greywater; without step 3, four key pairs
(e.g. Aldwater Up → Veyne Hall, then Down → The Salt Road) didn't undo
each other, which step 3 fixes.

Previous / Next unit keep going through every place in the map's order
(`next_place` ~:319), and Find keeps jumping anywhere.

## Scope

**In:**
- `step_cursor` by the rule; the screen remembers the last step (from
  where, which direction) and forgets it on any other move.
- The road list: `WorldMap`'s roads (`crates/core/src/worldmap.rs`; the
  ones the screen draws, open and locked).
- `world-structure.md` ~:224 and `controls.md` ~:100: the rule.
- **engine#2355, either order.** If it is done when this is worked and its
  focus map takes neighbours set by hand, give it each place's road
  neighbours and use its pick (`engine-change` skill, §7, for the pin); if
  not, do it game-side as above and add a line to the engine friction log
  that the world map could use it.

**Out (do not do):**
- Free movement of the cursor between places.
- Roads, places or the map's layout.
- Vix's world-map tip (1081): either order, it says the keys jump place to
  place, still true.

## Implementation steps

1. Write the failing tests first: the repro's moves on Act 1's map (the
   `act1_from_a_save` helper in `crates/ui/tests/it/world_map.rs` builds
   it at a cleared state).
2. The new `step_cursor`, its last-step memory.
3. Rewrite `the_cursor_jumps_to_the_nearest_place_that_way` for the rule
   on the test map.
4. Docs.

## Acceptance criteria

- [x] On Act 1's map at Chapter 4, Millhaven Right stays; Aldwater Right
      → Veyne Hall; Veyne Hall Left → Aldwater (Harness test).
- [x] For every shown place and direction that moves the cursor, the
      opposite key right after goes back (property test over the test
      maps and Act 1's map at two states).
- [x] From the army's place, every shown place can be reached with the
      direction keys alone (test on the same maps).
- [x] A direction with no road that way leaves the cursor and plays
      Denied (test).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `step_cursor` on the test map (`world_map/tests.rs`).
- Property: reversibility and reach, as above.
- Integration: the Act 1 map test.

## Completion notes

Done game-side: engine#2355 is still open, so `WorldMapScreen::step_cursor`
picks the place itself, and `docs/engine/friction.md` has a row saying the
world map can hand engine#2355 its road neighbours once it lands.

- **The pick**: the places joined to the cursor's by a road the screen
  draws (`WorldMap::open_roads` and `locked_roads`); the one whose straight
  line is closest in angle to the key (compared as the cosine squared, in
  whole numbers, so the Playdate's build needs no square root), under 90°;
  on a tie, the first road in the map's order. None: the cursor stays and
  Denied plays.
- **The way back**: the screen remembers the last step a cursor key made
  (from where, which way); the opposite key right after goes back there.
  Any other move forgets it: Previous / Next unit, Your army, Find, the
  end of a walk, and coming back from a screen above.
- **Tests**: `the_cursor_follows_the_roads_that_way` replaces
  `the_cursor_jumps_to_the_nearest_place_that_way`;
  `no_road_that_way_leaves_the_cursor_with_denied`;
  `the_cursors_keys_undo_each_other_and_reach_every_place` tries every
  shown place and key (exhaustive, not sampled) and searches every place
  the keys reach, on both test maps (the first at its start and after
  Stone Mill) and on Act 1's map at Chapter 4's start, after Veyne Hall,
  and with every place shown; `act1_the_cursor_goes_where_the_roads_go`
  plays the repro on Act 1 at Chapter 4 through the Harness (a new
  `act1_at_chapter` helper; `act1_from_a_save` keeps the test chapter).
- **Deviation**: `win_a_story_battle_then_the_acts_final_one` now presses
  Right twice from Wolf Pass to reach Old Bridge (Wolf Pass's one road
  goes back to Stone Mill); before, Right jumped there across no road.
- On Old Bridge in the test map, Up goes to Stone Mill: its road leans a
  little up, which is under 90° from Up.

*The agent's starting rule* (for the Act 1 playtest, 1064; in
`world-structure.md` and `controls.md`): the cursor follows the roads,
as written in this ticket's *Nick input*. One detail the ticket didn't
say: when two roads are exactly as close to the key, the first in the
map file's order wins. Left to judge by playing: whether the roads feel
right where a road bends far from its straight line (Aldwater's road to
Veyne Hall runs east, then north; Up and Right both take it).
