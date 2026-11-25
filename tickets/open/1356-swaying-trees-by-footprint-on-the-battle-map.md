---
id: "1356"
title: "Swaying trees by footprint on the battle map"
type: feature
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["1315", "1318"]
blocks: ["1064"]
nick_input: none
completed:
---

# 1356 — Swaying trees by footprint on the battle map

## Context

Split from 1315 (2026-11-24). 1315 gave tileset cells frames (`frames`
in `assets/tilesets/README.md`, `trpg_content::tileset::TileFrames`)
and the sprite skin draws them at the scene's clock, still on a lesser
tier (ADR-0131) and with idle motion off. It moved no real tile: Chapter
1's art is Rogue Adventure, whose tileset is 1318's, and Tiny Tales'
own loops are all one picture.

The trees are more than a moving cell. 0096's answer
(`docs/design/look-and-feel.md`, *Chapter 1's map*, *Woods: trees by
footprint*):

- A **big tree** (Rogue Adventure's `animated-trees/`, 12 frames of
  64×80, green, blue and autumn) stands with its trunk on one forest
  tile and its crown over the tiles to its left and right and the two
  above (3×3). Big trees go wherever a 3×3 of forest is free.
- The rest of the forest gets **half-size trees**: the same frames
  halved, one tile wide and two tall, so 2×2 and 1×2 woods have trees
  of their own ("all need sway").
- **Each tree with its own phase**, so a forest doesn't move as one.
- Halving the frames changes ELV's art: the credits say it was
  modified (ELV's licence; the `work-ticket` skill's credits rule).
- Motion is drawn art, never made by the skin (no code sway).

## Nick input

None: 0096 decided it. Nothing to check after merge: `frame-png` strips
of Chapter 1's woods at several times in the PR; what's left to judge in
play goes on 1064's checklist.

## Scope

**In:**
- Where the trees stand, from the map's forest tiles alone (a pure
  function of the terrain grid, tested): big trees first wherever a 3×3
  of forest is free, then half-size ones on what's left, the same on
  every run.
- The halved frames, made by a tool from the bought sheets (into the
  private repository's `game/`, never this one: ADR-0040), and the
  credit marked as modified.
- The sprite skin draws each tree's frame for the scene's clock, each
  tree its own phase (from its place, never a random number), still on
  a lesser tier and with idle motion off, as 1315's tiles.
- **Draw order, decided and tested:** a crown reaches over the tiles
  above its trunk, so it must not hide the ranges on those tiles, nor a
  unit standing there. The kit's ground painter paints its tint wash
  over the tiles' own pictures (`shuyi_ui::grid::sprite::ground`); a
  tree painted after the ground covers it. If trees must go under the
  wash, that is an engine change (`engine-change` skill, with Nick's OK
  before an engine PR), beside engine#2359.
- Each tree's `motion` with its stills (`Reduced` and `Minimal` its
  first frame), collected for `every_moving_look_names_its_stills`.

**Out (do not do):** the grass tufts and lamps of `RA_Jungle_Animation`
(a later ticket if Chapter 1's map wants them); drifting leaves (1316);
the tile re-pick (1318).

## Implementation steps

1. Read 1315's `crates/ui/src/map_view/sprite/moving.rs` and
   `crates/content/src/tileset/frames.rs`, and 1318's tileset.
2. The placement function and its tests (3×3 free, then 1×2, then
   what a 1×1 forest tile gets).
3. The halved frames tool and the tileset rows for big and half trees.
4. The skin: the trees at their frames and phases, in the decided draw
   order.
5. `frame-png` strips of Chapter 1's woods at 0, 200, 400 and 600 ms.
6. `cargo shuyi bench` on the biggest battle stays in budget.

## Acceptance criteria

- [ ] A test: a 3×3 forest gets one big tree; a 2×2 and a 1×2 forest
      get half-size trees; no tree stands on a tile that isn't forest.
- [ ] A test: two trees of one forest show different frames at one
      clock, and each the same frame at that clock on every run.
- [ ] A test: the Playdate's tier draws every tree's first frame.
- [ ] A test: a range on a forest tile under a crown still shows.
- [ ] The credits mark ELV's trees as modified.
- [ ] `cargo shuyi bench` stays within each target's budget.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the placement; the phase per tree.
- Look: the sprite skin's items at fixed times (ADR-0038).

## Completion notes

*(Filled in by the session that completes the ticket.)*
