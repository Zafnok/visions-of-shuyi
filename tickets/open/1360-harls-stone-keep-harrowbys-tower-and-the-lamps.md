---
id: "1360"
title: "Harl's stone keep, Harrowby's tower and the lamps in Rogue Adventure"
type: content
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["1318"]
blocks: ["1064"]
nick_input: none
completed:
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

- [ ] Chapter 1's fort is the stone keep: tops, pillar caps, faces
      under every wall seen from the south, paving, gates open.
- [ ] Harrowby's fort tile is a round tower; four lamps burn, moving.
- [ ] Every other map looks as before.
- [ ] Before/after pictures in the PR.
- [ ] The tileset import's tests pass; `assets-private.rev` moved.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- The importer: the wall kinds and the faces' choice on a ring of walls
  with a gate.
- `trpg_content::tileset` for any format addition; the sprite skin's
  test of its look (with the skin, ADR-0038).

## Lesser targets

The Playdate map uses the glyph look (0061) unless 0067 changes that;
the lamps' stills are their first frame. GBA: no target.

## Completion notes

