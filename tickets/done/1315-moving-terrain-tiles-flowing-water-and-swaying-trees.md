---
id: "1315"
title: "Moving terrain tiles: flowing water and swaying trees"
type: feature
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: ["0096", "1314"]
blocks: ["1064"]
nick_input: answer-first
completed: 2026-11-25
---

# 1315 — Moving terrain tiles: flowing water and swaying trees

## Context

Nick (2026-11-22) wants "swaying trees, flowing water" on the battle
map. 0096 decides which tiles move and how fast; 1314 gives each target a
graphics tier and requires a still fallback for every moving look.

Today the tileset format (`assets/tilesets/README.md`: `terrain`,
`layers`, `looks`) has no frames, and nothing in `crates/ui/src/map_view/`
reads time except unit idle and the edge arrows
(`map_view/sprite/units.rs`). The Tiny Tales bundle already has frame
loops in Tiled `.tsx` files (coasts, beaches, hills, dark woods, desert,
snow, mines; e.g. four frames at 600/200/400/200 ms). The engine's parked
tickets 2359 (animated tiles in the grid kit) and 2360 (a tile-layer
painter with animated tiles) cover the engine half. Motion is drawn from
the time alone, never stepped (`shuyi-animate` skill; `SH2103`: the rules
never read a clock).

**From 1314 (done, ADR-0131):** a skin reads the tier from
`ctx.platform.graphics` (`GraphicsTier::{Minimal, Reduced, Full}`; the
Playdate is `Reduced`). Each moving row carries a
`trpg_content::motion::Motion` (`lowest`, and a `stills` entry per lower
tier: `(show: FirstFrame)`, with `todo: "..."` for a picture still to
make) and the skin draws `motion.drawn(tier)`. Collect the moving rows
from the content (a function of `trpg_content`, tested) and hand them to
`motion::check` in the content test `every_moving_look_names_its_stills`
(its `TODO(1315)`), which then checks them. Preview with
`cargo shuyi frame-png --graphics reduced`.

## 0096's answer (2026-11-24)

`docs/design/look-and-feel.md`, *Moving tiles and effects* and *Chapter
1's map*. For this ticket:

- **Motion is drawn art, never made by the skin** (no code sway, no colour
  cycling, no stand-in glints). Drop the "skin-made sway" step below.
- **Chapter 1 is Rogue Adventure** (ELV Games, `assets-private/library/
  elv-games/tilesets/rogue-adventure/`): its animated water
  (`RA_Animated_Water.png`, 8 frames), its swaying trees
  (`animated-trees/`, 12 frames of 64×80, green, blue and autumn), grass
  tufts and lamps (`RA_Jungle_Animation(s).png`). Not the Tiny Tales
  `.tsx` loops (their frames are all the same picture).
- **Trees by footprint:** a big tree's trunk on one forest tile, its crown
  over the tiles to its left and right and the two above (3×3); big trees
  wherever a 3×3 of forest is free, then half-size trees (the frames
  halved: one tile wide, two tall) on what's left, each tree with its own
  phase. Halving the frames is a change to ELV's art: the credits say it
  was modified (ELV's licence).
- The mock that settled it is a throwaway script, not in the repository.

## Nick input

**Answer first:** 0096. Nothing to check after merge: verified by tests
and `frame-png` pictures at several times; what's left to judge in play
goes on 1064's checklist.

## Scope

**In:**
- Frames on a tileset row: a list of cells with durations, read from
  the bundle's `.tsx` animations by `cargo xtask tileset-import` where
  they exist.
- Every moving row names its fallback per tier (1314): Playdate and any
  lower tier draw frame 0. A missing still asset is a `TODO` row the
  1314 check lists.
- The sprite map skin draws the frame for the scene's clock
  (`clock_ms`), so a `frame-png` at a given time is reproducible.
- With idle motion off (`ctx.idle_motion`, 0093) tiles show frame 0.
- Tree sway, if 0096 picked a skin-made sway: a small time-based offset
  per tree, with a per-tile phase so a forest doesn't move as one.
- Engine half via the `engine-change` skill (2359/2360 or new tickets),
  if the grid kit's ground painter must learn frames.

**Out (do not do):** drifting effects (1316); the tile re-pick (1318);
moving objects (torches, smoke) unless 0096 asked for them here.

## Implementation steps

1. Engine: frames in the grid kit's tile painter (or confirm what's
   there at the pin), with tests at fixed times.
2. Game: the tileset format's `frames` field, its README, the importer.
3. The water, sea and coast rows (and trees per 0096) with frames.
4. Fallbacks per tier; the 1314 check passes.
5. `frame-png` strips at 0, 200, 400, 600 ms for the PR.
6. Check the bench: `cargo shuyi bench` on the biggest battle stays in
   budget on desktop and web (`docs/targets.md`).

## Acceptance criteria

- [x] A test: the sprite skin draws different water cells at 0 ms and at
      600 ms, and the same cells with idle motion off.
- [x] A test: the Playdate tier draws frame 0 for every moving row.
- [x] `cargo shuyi bench` stays within each target's budget (heap and
      size locally; instructions are counted by CI, under Valgrind).
- [x] `frame-png` strips in the PR.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the importer reads `.tsx` frames; the frame for a time.
- Look: the sprite skin's cells at fixed times (ADR-0038: tests of a look
  read cells).
- Engine: per `engine-change`.

## Lesser targets

Playdate: frame 0 (1314). GBA: no target; the `Minimal` tier's row is
frame 0 too.

## Completion notes

**Done:**

- **Frames on a tileset** (`frames`, `assets/tilesets/README.md`;
  `trpg_content::tileset::{TileFrames, MovingTile}` in
  `crates/content/src/tileset/frames.rs`). A moving cell is written once,
  with its frames (a cell and its ms each, the first the cell itself)
  and its `motion`; every row that draws the cell moves with it, in
  every look (a terrain's tile, a corner piece, a tile layer's picture),
  so the rows stay as written. The loader checks the cell is drawn,
  listed once, has two frames or more from itself, none of 0 ms, all
  inside the image, and that its stills show `FirstFrame` (a tile's
  `Nothing` would leave a hole).
- **The 1314 check reads them:** `moving_rows` collects each tileset's
  moving cells, and `every_moving_look_names_its_stills` hands them to
  `motion::check` (its `TODO(1315)` gone).
- **The sprite skin** draws each moving cell's frame for the scene's
  `clock_ms` (`crates/ui/src/map_view/sprite/moving.rs`), the tint wash
  included; its first frame below the tier it moves at
  (`ctx.platform.graphics`) and while idle motion is off.
- **The importer** (`cargo xtask tileset-import`) reads Tiled's
  `<animation>` from the `.tsx`, packs the frames of each tile it packs
  that moves (in the same tint), turns a loop to start on the tile
  itself, and writes the `frames` with `Minimal`'s still marked `todo`.
  A loop of one picture is left out, and one that never shows its own
  tile too.
- **The test tileset's water flows** (`test_auto`, generated): four
  frames of 600, 200, 400 and 200 ms, a ripple line lower down in each.
  The skin's tests and the PR's strips read it.

**Deviations:**

- **No real tile moves yet.** Chapter 1's water and trees are Rogue
  Adventure's (0096), and its tileset is 1318's, not built yet; Tiny
  Tales' own loops are all one picture, so the importer finds no moving
  tile in `tiny_tales`. 1318's water writes the river's 8 frames in this
  format (a dated line in its Context).
- **Trees split off** to **1356** (blocked by this and 1318): placing
  big and half-size trees by footprint, halving ELV's frames (credited
  as modified), a phase per tree, and their draw order under the
  ranges. Step 3's "trees per 0096" is there.
- **Engine half: a marked workaround**, not an engine PR. engine#2359
  (moving tiles in the grid kit) is exactly the piece, but it is parked
  behind 2231 (animation clips), two engine PRs' worth: the game ships
  with `moving.rs` painting each frame with a copy of the kit's skin
  whose look has the moving cells swapped, marked
  `engine-friction: engine#2359, moves (game 1357 deletes it)`.
  **1357** (blocked by engine#2359) moves onto it. Unparking 2359 and
  2231 is Nick's call. 2359 predates the graphics tiers (engine 3347):
  1357 keeps the tier rule on the game's side.
- The strips crop the test map's river at 1× and enlarge it 3×; the
  times are from the end of the steps, not from the loop's start.

**Friction:** estimate 350, measured 377 (1.1×); signals: workaround yes
(outcome 1: engine#2359, follow-up 1357), third copy no, budget no. A
row in `docs/engine/friction.md`.

**Starting rules:** none: no gameplay rule changes. What's left to judge
in play (the river's motion in Chapter 1) waits for 1318 and goes on
1064's checklist then.

