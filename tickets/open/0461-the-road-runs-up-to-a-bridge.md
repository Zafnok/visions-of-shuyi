---
id: "0461"
title: "The road runs up to a bridge instead of narrowing at it"
type: bug
milestone: M3 Battle UI
model: sonnet-5
effort: low
status: todo
blocked_by: ["1118", "1050"]
nick_input: sign-off
completed:
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

## Nick input

**Sign-off:** a game screenshot of both Chapter 1 bridges, before and
after.

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

- [ ] The road keeps its width up to each Chapter 1 bridge (screenshot in
      the PR, from the running game).
- [ ] A unit test of the corner picking with the game's tileset (through
      `map_view::corners`): a road tile next to a bridge tile picks the
      "inside" corner, not an edge.
- [ ] `cargo test -p trpg-content --features private-assets --test it private_assets::`
      passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: corner selection with a bridge neighbour.

## Completion notes

*(Filled in by the session that completes the ticket.)*
