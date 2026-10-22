---
id: "1076"
title: "Playdate: the exit area and a downed enemy get looks of their own, and the map's marks are checked to differ"
type: bug
milestone: Post–Chapter 1
model: sonnet-5
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1076 — Playdate: the exit area and a downed enemy get looks of their own

## Context

From the retro of the last 50 PRs (2026-10-22). 0319 (#393, the escape
objective and the boss who can't die) and the Playdate battle skin 1038
(#392) were built side by side, and 0319 gave the Playdate's map two
marks that reuse a look that already means something:

1. **The exit area looks like an enemy.** `assets/data/palettes/playdate.ron`
   sets `exit_area` to `grey`, the same 50% checker as
   `map_enemy_tile`, which is how the Playdate tells enemies apart
   without colour (`docs/design/playdate.md`, *Telling things apart
   without colour*: "Enemies: the two letters on a grey tile"). The
   Playdate snapshot `it__playdate__playdate_battle.snap` now names the
   enemy tile's background `exit_area`.
2. **A downed enemy looks like your unit that has acted.**
   `crates/ui/src/map_view/playdate/units.rs`, `look`, draws a unit that
   `is_down()` with `inks.acted` whatever its side: an outlined white
   tile with black letters, which on the Playdate means "your unit, done
   this turn". Only its empty HP line differs.

Neither look is in `playdate.md`. Nick's rule (Q3 E) is that the
one-bit map tells these things apart without colour; Chapter 6 (0852)
is an escape map, so both show in Act 1 on the device Nick plays it on.

The inks are the engine's (`shuyi_assets` `ink.rs`: white, black,
black_edged, grey, dots, diagonals, dashes, invert), and dots,
diagonals, dashes and grey are taken by the ranges, the danger zone and
the enemy tiles.

## Nick input

None before building. The two new looks are *Claude's starting looks*:
write them into `playdate.md` as such, show them in `cargo shuyi
frame-png --playdate` pictures in the PR, and list them in the
Completion notes for the Act 1 playtest's checklist (1064), where Nick
sees them on his Playdate.

## Scope

**In:**
- A look for the **exit area** that no other map mark has. Options, in
  order of preference: (a) a mark that isn't a fill, e.g. a border
  drawn round the area's edge in a style no range uses, or a glyph in
  each exit tile's corner, like the map's other marks in
  `map_view/playdate/marks.rs`; (b) a new ink, if the engine's inks
  have room for one (a small engine change: the `engine-change` skill,
  §3, done in this ticket). Pick the one that reads at 1× on the
  device's 400 × 240 screen.
- A look for a **downed unit** that keeps its side: a downed enemy keeps
  the grey tile (or its letters' edge) and adds what says "down" (the
  outline, a glyph, letters without the edge); a downed player unit (if
  any battle downs one) likewise keeps its black tile's meaning.
- A test that the map's marks differ: for every pair of map roles in the
  Playdate palette that can be on screen at once (the `map_*` entries,
  `move_range`, `attack_range`, `heal_range` aside as decided,
  `danger_zone`, `exit_area`), their inks (or tile/letters/outline
  triples, for units) are not the same. `heal_range` = `attack_range` is
  decided (1037) and stays allowed.
- `playdate.md`: both looks under *Claude's starting rules from building
  the map*, with this ticket's number.
- The snapshot changes, reviewed.

**Either order with 1178:** if 1178 (the arrows at the map's edges on
the Playdate) is done, its arrows' ink is in the distinct-marks test;
if not, 1178 adds it.

**Out (do not do):**
- The PC skins (they have colour: the exit area's tint there is fine).
- Allies' and neutral units' looks (waiting for a chapter that has them).
- The arrows at the map's edges (1178).

## Implementation steps

1. **Read** `assets/data/palettes/playdate.ron`, `map_view/playdate/units.rs`
   (`look`, `Inks`), `map_view/playdate/marks.rs`, the map scene's
   `range::EXIT`, `docs/design/playdate.md` (*Telling things apart
   without colour*, the 1037 starting rules) and the snapshots in
   `crates/ui/src/map_view/playdate/snapshots/`.
2. **The distinct-marks test first** (`map_view/playdate/tests.rs`): it
   fails today on `exit_area` = `map_enemy_tile`, and on a downed enemy's
   look = an acted player unit's.
3. Make the exit area's look; make `look` keep the side for a downed
   unit.
4. Pictures: `test_escape` (the exit area beside an enemy) and
   `test_hold` (the boss down beside one of your units that has acted),
   `cargo shuyi frame-png --playdate`.
5. `playdate.md` and the snapshots.

Expected size: about 40 production lines (the look's branch, the exit
mark), plus the test.

## Acceptance criteria

- [ ] The distinct-marks test passes, and fails if `exit_area` is set
      back to `grey`.
- [ ] A downed enemy and an acted player unit look different (a test on
      `look`).
- [ ] Both pictures are in the PR.
- [ ] `playdate.md` lists both looks as *Claude's starting rules*.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `look` for a downed enemy; the distinct-marks test.
- Snapshot: the Playdate map snapshots with the exit area and a downed
  unit.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
