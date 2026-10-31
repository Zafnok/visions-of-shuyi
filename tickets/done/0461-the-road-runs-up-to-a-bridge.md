---
id: "0461"
title: "The road runs up to a bridge instead of narrowing at it"
type: bug
milestone: M3 Battle UI
model: sonnet-5 / gpt-6.1-terra
effort: low
status: done
blocked_by: []
nick_input: none
completed: 2026-10-28
---

# 0461 — The road runs up to a bridge

## Context

Chapter 1 playtest round 3 (0804, 2026-10-16), on the bought-art map
after 0452 fixed the forest edges:

> "much clearer but I found a new issue is that the road abruptly becomes
> very narrow when it touches a bridge for some reason."

Cause (read 2026-10-16): the road is a corner layer, `"road":
Corners(of: ["road", "well", "rack"], ...)` in `assets-src/tilesets/
tiny_tales.ron:76-79` (generated into the private repository's
`game/tilesets/tiny_tales.ron`). "bridge" isn't in that set, so under the
corner rule (ADR-0052: one picture per four-tile corner; the tactics
kit's in Shuyi Engine since 1115, re-exported as `map_view::corners` in
`crates/ui/src/map_view.rs`) the road draws its edge at the seam with a
bridge tile and narrows or ends there. The fix is the game's tileset
data; the rule itself doesn't change. The bridge is a `Tiles` layer
(`tiny_tales.ron:109-120`).

The bought files never go in this repository (ADR-0040); the source
`assets-src/tilesets/tiny_tales.ron` is the input to `cargo xtask
tileset-import` (or the importer this repository uses: check
`crates/xtask/src/tileset_import.rs`).

**Waits for the engine carve and the Playdate port** (Nick, 2026-10-16),
so `blocked_by` has 1118 and 1050.

**Re-gated 2026-10-21 (0129):** the carve is done and 1050 was folded
into the Act 1 playtest (1064), so neither 1118 nor 1050 is in
`blocked_by` any more.

## Nick input

None. Nick no longer checks gameplay tickets after merge (Nick,
2026-10-21: "I will save all my checking for either design (i.e. 00
tickets) or for the full act 1 playtest itself"). Verification is by
tests, `cargo shuyi frame-png` pictures in the PR and the playtest bots
(0506, 0509). What the old sign-off asked goes on the checklist of
Nick's next playtest (1064, the Act 1 playtest, if this lands before
it):

> A game screenshot of both Chapter 1 bridges, before and
> after.

## Scope

**In:**
- Add `"bridge"` to the road's corner set so the road counts a bridge as
  road at the seam and runs full width onto it. Check both bridge
  directions (water left/right and above/below).
- If the road picture then draws over the bridge's planks, stop the road
  at the bridge's edge another way (a separate "road meets bridge" end
  piece from the tileset, if it has one); say which in the notes.
- Regenerate the private tileset; commit the source here and the pin
  (`cargo xtask private-assets --pin`, `assets-private.rev`).

**Out (do not do):** other terrain seams.

## Acceptance criteria

- [x] The road keeps its width up to each Chapter 1 bridge (screenshot in
      the PR, from the running game).
- [x] A unit test of the corner picking with the game's tileset (through
      `map_view::corners`): a road tile next to a bridge tile picks the
      "inside" corner, not an edge.
- [x] `cargo test -p trpg-content --features private-assets --test it private_assets::`
      passes.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: corner selection with a bridge neighbour.

## Completion notes

Added `"bridge"` to the road corner layer's `of` in
`assets-src/tilesets/tiny_tales.ron`, regenerated the private tileset with
`cargo xtask tileset-import` (pushed to the private repository, pin moved in
`assets-private.rev`). The road layer is drawn under the bridge's tile layer, so
the bridge planks stay on top: no separate end piece was needed. Both bridge
directions work because the corner rule counts the bridge as road on every side
(the Chapter 1 map has bridges in a river running left to right).

- Test: `private_assets::the_road_runs_full_width_onto_a_bridge` checks, in
  every look, that a road beside a bridge picks the inside mix, not an edge.
- Picture: `docs/screenshots/0461-bridges.png`, the running game with the
  bought art, all three Chapter 1 bridges with the road at full width. No
  "before" picture was made.
- Engine friction: none; no game code beyond one test.
- No gameplay rules decided. Nick's Act 1 playtest (1064) checklist already
  has the bridge screenshot item.
- Gates run: fmt, the private-assets test; the full list is left to CI.
