---
id: "0452"
title: Forest tiles on the picture map show where the forest starts and ends
type: tuning
milestone: M3 Battle UI
model: opus-5.5
effort: medium
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 0452 — Forest tiles show where the forest ends

## Context

Nick, Chapter 1 playtest round 2 (0804, 2026-10-14): "seems the forest
tilemap doesn't line up well with our grid and makes it a little confusing
where the forest starts and ends visually. it overlaps with road but
becomes road, it overlaps with plain but becomes plain. Need better
differentiating here."

Why it looks that way (found while ticketing; ADR-0052, ticket 0437's
completion notes, `look-and-feel.md` *Terrain*):

- `assets-src/tilesets/tiny_tales.ron` (read by `cargo xtask
  tileset-import`): the forest's own tile is the **plain grass**
  (`"forest": ("world-map/Set_A_Standard_Fields2", 89)`), and the trees are
  a `Corners` layer (`Set_H_Trees1`, `pick: 0`).
- `crates/ui/src/map_view/sprite/ground.rs`, `paint_corners`: corner
  pictures are drawn **centred on the point where four tiles meet**, half
  a tile off the grid, and clipped only to the map. A forest tile is built
  from quarters of four tree pictures, so a lone forest tile or a one-wide
  strip shows grass inside it ("becomes plain").
- The tree art overhangs its tile by about four pixels (0437's notes) and is
  painted after `road` and `grass` (the look's layer order), so the canopy
  covers part of a road tile ("becomes road").
- Mountains had the same problem and were fixed with `pick: 1` (the tile
  "whose edge ends where the tile does"); forest never got that.

Knobs: the forest's own tile (a darker forest-floor tile), the trees'
`pick`, `sheet` / `terrain` (other tree masses in the bundle), switching
forest to a `Tiles` layer (one picture exactly on each tile, with `beside`
/ `sides` for the joins, like the bridge and pool layers), the layer
order, and, as code, clipping a corner layer to its own terrain's tiles in
`paint_corners`.

The bought art is in the private repository (ADR-0040): run `cargo xtask
private-assets --library` to have the sheets, and never commit a picture
made from them (renders go to Nick, not into the repo).

## Nick input

**Decision (look & feel is Nick's):** render Chapter 1's map with the
bought art (`cargo xtask frame-png`, private assets) around the places
where forest meets road and plain (the woods by the footbridge and along
the road), for at least three of:

- **A.** A forest-floor ground under the trees (the whole tile reads as
  forest).
- **B.** Trees drawn one picture per tile, exactly on the grid (`Tiles`
  layer).
- **C.** Another `pick` or tree mass whose edge ends at the tile.
- **D.** Today's trees clipped to the forest's own tiles.
- **E.** A thin dark line or shade along the forest's edge.
- **F.** Combinations of the above, and "describe your own".

Show today's look next to them, at 1× and 2×. Record the answer in
`look-and-feel.md` (*Terrain*). **Sign-off:** play Chapter 1: can you tell
which tiles are forest?

## Scope

**In:**
- The renders and Nick's choice, recorded.
- The change in `assets-src/tilesets/tiny_tales.ron` and/or `ground.rs`,
  the re-imported tileset in the private repository (`cargo xtask
  tileset-import`, then `cargo xtask private-assets --pin` and commit
  `assets-private.rev`), and the test tileset if the code changes.
- Thicket and burnt use the same tree layers; give them the same treatment
  if Nick's choice applies to them (say so in the question).

**Out (do not do):**
- The glyph look.
- Other terrain (roads, water, mountains) unless the chosen fix needs it.
- Buying art.

## Implementation steps

1. Fetch the bought packs (`cargo xtask private-assets --library`); try
   each option in the mapping / code; render with `frame-png`.
2. Ask Nick with the `ask-nick` skill; record the answer.
3. Make the chosen change; re-import; pin the private assets.
4. If `ground.rs` changes, add tests with the test tileset (`cargo xtask
   test-tileset`) that read the painted items, in the sprite skin's tests.

## Acceptance criteria

- [ ] `look-and-feel.md` records Nick's choice and words.
- [ ] The re-imported tileset is in the private repository and
      `assets-private.rev` points at it.
- [ ] Any code change has a skin test (test tileset): a lone forest tile
      draws forest over its whole tile and nothing over the road beside it
      (or what Nick chose).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Snapshot: the sprite skin's tests with the test tileset, if code changes.

## Completion notes

*(Filled in by the session that completes the ticket.)*
