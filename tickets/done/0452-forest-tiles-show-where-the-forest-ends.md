---
id: "0452"
title: Forest tiles on the picture map show where the forest starts and ends
type: tuning
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: []
nick_input: decision
completed: 2026-10-14
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

- [x] `look-and-feel.md` records Nick's choice and words.
- [x] The re-imported tileset is in the private repository and
      `assets-private.rev` points at it.
- [x] Any code change has a skin test (test tileset): a lone forest tile
      draws forest over its whole tile and nothing over the road beside it
      (or what Nick chose). *No skin code changed; see the notes.*
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Snapshot: the sprite skin's tests with the test tileset, if code changes.

## Completion notes

- **Renders shown to Nick** (Chapter 1 opening, zoom 2× and 1×, local
  only, never committed): today's; B one tree per tile (`Set_G_Trees1`);
  B+A the same on a darker grass (`Set_A_Standard_Fields3` 233); C
  `pick: 1`, and the bigger tree mass (`Set_G_Trees1` "Tree1"); D today's
  trees cut off at the forest's tiles (a throwaway switch in
  `ground.rs`, removed). A alone (dark floor under today's trees) looked
  the same as today: the trees cover it. E (an edge line) wasn't
  rendered.
- **Nick chose** D without the half trees: the wood keeps to its own
  tiles and ends a little inside them, in whole trees (his words are in
  `look-and-feel.md`).
- **How:** the tree sheets have a 3×3 block (tiles 5–7 / 21–23 / 37–39,
  or from 13 for the second mass) drawing a patch of trees whose edge is
  4 px inside the block, every edge tree whole. A new mapping layer,
  `Block(of, sheet, block, tint)`, makes the importer join each tile from
  that block's quarters for the sides where the terrain ends (top edge
  when the tile above isn't forest, the edge all round for a lone tile),
  and writes it as an ordinary `Tiles` layer with a picture for each of
  the 16 mixes of sides (`beside` = every other terrain). So the game's
  tileset format and the skin's code are unchanged, which is why there is
  no new skin test: the existing `Tiles` tests cover it
  (`a_tile_layers_picture_goes_by_the_neighbours_it_looks_for`). The
  importer's tests cover the joining and the layer on a made-up sheet.
- Forest, thicket, burnt and burning all use it (same art layout).
- *the agent's starting rules* (also in `look-and-feel.md`): thicket, burnt
  and burning get the same treatment as forest; at an inward corner of a
  wood (forest above and beside a tile but not on the diagonal) the trees
  run on to the tile's corner, as the side mixes don't look at the
  diagonal.
- The burnt ground under the bare trees (`ash`, a corner layer) still
  spreads half a tile like before; out of the ticket's scope (not
  trees), and Chapter 1 has none.
- Private repository commit `cec0924`, pinned in `assets-private.rev`.
- No ADR: the mapping file is the importer's input and its header
  explains `Block`; the game's file format didn't change.
