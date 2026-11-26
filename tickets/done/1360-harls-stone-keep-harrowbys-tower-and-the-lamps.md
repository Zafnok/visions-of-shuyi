---
id: "1360"
title: "Harl's stone keep, Harrowby's tower and the lamps in Rogue Adventure"
type: content
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: ["1318"]
blocks: ["1064"]
nick_input: none
completed: 2026-11-26
---

# 1360 — Harl's stone keep, Harrowby's tower and the lamps in Rogue Adventure

## Context

Split from 1318 (2026-11-25), the tileset overhaul 0096 decided
(`docs/design/look-and-feel.md`, *Chapter 1's map*). 1318 gave Chapter 1
a map look of its own, drawn with ELV Games' *Rogue Adventure*
(`RA_*.png` in `assets-private/library/elv-games/tilesets/rogue-adventure/`),
and an importer that reads plain PNG sheets
(`crates/tools/src/tileset_import.rs`, `assets-src/tilesets/tiny_tales.ron`).
The fort's walls, its floor, the fort tiles and the gate still show Tiny
Tales' pictures in that look.

0096's answer:

- **Harl's fort: the stone keep.** Rogue Adventure's ruins
  (`RA_Ruins.png`): the wall's top seen from above, round pillars at the
  corners, and a tall wall face with arrow slits **on every side seen
  from the south** (outside under the south wall, inside under the north
  wall), paving inside. Its gates stay open. **Nothing covers the fort:
  hills and peaks are drawn under it.**
- **Harrowby** is a small round stone tower (no cottage: Chapter 1 has
  none).
- **Lamps** burn at the fort's gate and the bridges.
- **Rocks** are few: "the rocks are too numerous".

**How the mockups drew it** (0096's throwaway renderer; `RA_Ruins.png`
cells as (column, row) of 16 pixels; on Chapter 1's map `#` is wall, `_`
floor, `F` fort, `:` road):

- A wall tile's **kind** by its wall neighbours: across (wall left or
  right, none up or down), up-and-down (wall above and below, none
  beside), else a **pillar** (corners, ends, a lone tile).
- Its top: across (1, 8); pillar cap (0, 8); up-and-down the left half
  of (0, 4) and its mirror on the right half.
- **The face** (arrow slits) on the tile below each wall tile that isn't
  up-and-down, when that tile is neither wall nor road (a gate): (1, 9)
  under an across wall, (0, 9) under a pillar; drawn over the ground
  and sorted with the units by its bottom edge (a unit stands in front
  of it).
- The paving inside (floor and fort tiles): (13, 2); the keep's fort
  tile (20, 2) a carved stone, (13, 1).
- **Harrowby's tower** on its fort tile (2, 12): the pillar cap (0, 12)
  on the tile above and the pillar's face (0, 15) on the tile itself,
  two tiles tall, so it stands up.
- **Lamps**, `RA_Jungle_Animation.png`, 16×32, four frames 32 pixels
  apart from pixel (0, 256), 150 ms each, standing on (19, 5) and
  (21, 5) beside the gate and (10, 6) and (12, 6) beside the road
  bridge, reaching a tile above.

**The format may not hold this yet:** the face's kind depends on the
wall's own neighbours (diagonals of the face's tile); the tower and the
lamps are taller than a tile; lamps stand at places no terrain names.
Choose the smallest addition (a map's decorations by place, a picture
taller than its tile), with an ADR if it changes a format's shape;
check 1219 (the engine's tileset format) and 1359 (hills, which may add
an eight-neighbour layer) first and reuse what they added. Lamps placed
by hand on Chapter 1's map go in its private map file, not in this
repository (`content-change`).

## Nick input

None: 0096 decided it. Nothing to check after merge: `frame-png`
pictures of the fort, Harrowby and a bridge before and after in the PR
(cropped, ADR-0118); what's left to judge in play goes on 1064's
checklist.

## Scope

**In:**
- Chapter 1's look: the keep's walls, faces, paving and keep stone;
  Harrowby's tower; the four lamps, moving (`frames`, with their stills
  for lesser tiers, ADR-0131); few or no rocks.
- Layer order: the fort over hills and peaks (1359; either order works:
  if 1359 hasn't landed, put the fort's layers last so they stay on top).
- The art change pushed to the private repository and
  `assets-private.rev` moved.

**Out (do not do):** hills and peaks (1359); trees (1356); doors and
houses elsewhere (0471, 0491); map layout changes; new terrain rules;
other chapters' maps.

## Implementation steps

1. Add the ruins' and lamps' tiles to `assets-src/tilesets/tiny_tales.ron`
   (and what the importer needs for a picture taller than a tile).
2. The wall kinds and the faces (the format addition above, its checks
   in `trpg_content::tileset`, its README in `assets/tilesets/`).
3. The tower and the lamps; re-pack; `frame-png` before and after.

## Acceptance criteria

- [x] Chapter 1's fort is the stone keep: tops, pillar caps, faces
      under every wall seen from the south, paving, gates open.
- [x] Harrowby's fort tile is a round tower. **The four lamps are not
      here**: they need an engine change, split to **1378** (see the
      notes).
- [x] Every other map looks as before.
- [x] Before/after pictures in the PR.
- [x] The tileset import's tests pass; `assets-private.rev` moved.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- The importer: the wall kinds and the faces' choice on a ring of walls
  with a gate.
- `trpg_content::tileset` for any format addition; the sprite skin's
  test of its look (with the skin, ADR-0038).

## Lesser targets

The Playdate map uses the glyph look (0061) unless 0067 changes that;
the lamps' stills are their first frame. GBA: no target.

## Completion notes

**Done in part (2026-11-26); the lamps are 1378.**

- **Split, with Nick's OK** (2026-11-26, "Split"): the lamps stand at
  places no terrain names, and nothing of a map's file but its terrain
  and its `look` (`MapLook { tiles, light }`) reaches the sprite skin,
  which is the engine's. **engine#3247** (Zafnok/shuyi-engine#610, a
  ticket only) adds pictures a map's look places on tiles; **1378**
  places the four lamps once the pin moves past it.
- **No format change, so no ADR:** the keep and the tower fit the
  tileset format as 1361 left it. A wall's top is a tile layer by its
  four sides; its face and the tower are corner layers drawn from the
  point at a tile's top-left (`offset: (8, 24)` onto the tile below,
  `size: (16, 32)` with `offset: (8, -8)` standing up a tile), with
  `skip`.
- **The importer** (`crates/tools/src/tileset_import.rs`): three mapping
  layers, `Walls` (across, up and down from a cell's left half and its
  mirror, else a pillar's cap: `WallTop::of`), `Faces` (the face on the
  tile below each wall tile, a pillar's where the wall goes on up:
  `face`) and `Standing` (two cells stacked, standing on each tile); a
  `Tiles` layer may leave its `tile` out (a see-through cell) and give
  `near` (its picture beside any of `beside`).
- **The mapping** (`rogue_adventure`, RA_Ruins.png): `ra_wall` across
  (1, 8), up and down (0, 4)'s left half mirrored, a pillar (0, 8);
  `ra_wall_face` (1, 9) and (0, 9), skipping walls, roads, gates and
  doors; `ra_tower` (0, 12) over (0, 15) on every fort, its top never on
  a wall or floor; `ra_keep` the carved stone (13, 1) on a fort by a
  floor, drawn over the tower's foot. The floor (and what stands
  indoors) is paved, (13, 2); a wall's own tile is the grass round a
  pillar's cap; Tiny Tales' floor rim and fort icon are gone from the
  look. The four layers are drawn last, over hills and peaks.
- **Deviations from the mockups' rule:** a corner point sees only its
  four tiles, so the face chooses by the wall above it and the one above
  that: a pillar's face only under the foot of a wall running down. A
  wall tile alone (a pillar's cap) would hang the face of a wall running
  across; Chapter 1 has none, and on its keep every face is the mockups'.
  The tower is every fort tile's in this look, and the keep's stone
  covers it where a fort is by a floor.
- **Rocks:** none in this look (1318 and 1359 drew no scattered rocks),
  so nothing to thin.
- **Credits:** the ELV Games credit is now "Rogue Adventure World
  (modified: a wall mirrored)" (the mirrored half is an edit, which the
  licence allows when the credits say so); `THIRD_PARTY_ASSETS.md` names
  the ruins sheet and the change.
- **Art:** re-packed, merged as Zafnok/visions-of-shuyi-assets#37;
  `assets-private.rev` is at its merge on `main` (`2f1bbb03`).
- **Checked:** every battle map rendered with the old and the new
  tileset: the other 13 are pixel-identical; Chapter 1 differs only at
  the keep and Harrowby.
- **Tests:** the importer's (the tops and faces on a ring of walls with
  two gates, against the mockups' rule; the mirrored and stacked cells;
  the four layers packed, listed, checked and read back by the game at
  their size and place); the private test that Chapter 1's look draws
  the keep last with its faces, tops, tower and stone.
- **Friction:** estimate 150 production lines (by 1359's `Quarters`
  layer, 118 lines), measured 215 (1.4×), under 2×.

**No gameplay rule decided.** Left to judge in play (1064): the keep's
look on the real map (a unit stands in front of a face; the faces under
the north wall cover the floor's top row); the lamps until 1378.
