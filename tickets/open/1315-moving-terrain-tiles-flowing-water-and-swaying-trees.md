---
id: "1315"
title: "Moving terrain tiles: flowing water and swaying trees"
type: feature
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: high
status: blocked
blocked_by: ["0096", "1314"]
nick_input: answer-first
completed:
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

- [ ] A test: the sprite skin draws different water cells at 0 ms and at
      600 ms, and the same cells with idle motion off.
- [ ] A test: the Playdate tier draws frame 0 for every moving row.
- [ ] `cargo shuyi bench` stays within each target's budget.
- [ ] `frame-png` strips in the PR.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the importer reads `.tsx` frames; the frame for a time.
- Look: the sprite skin's cells at fixed times (ADR-0038: tests of a look
  read cells).
- Engine: per `engine-change`.

## Lesser targets

Playdate: frame 0 (1314). GBA: no target; the `Minimal` tier's row is
frame 0 too.

## Completion notes

