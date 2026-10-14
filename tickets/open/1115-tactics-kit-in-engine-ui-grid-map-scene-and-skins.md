---
id: "1115"
title: "Tactics kit in engine-ui: a grid map scene generic over the game's pieces, skins, ground layers, cursor and tile camera"
type: infra
milestone: "Engine: carve"
model: opus-5.5
effort: xhigh
status: todo
blocked_by: ["1113", "1114"]
nick_input: none
completed:
---

# 1115 — Tactics kit in engine-ui: grid map scene and skins

## Context

ADR-0038 made the battle map a `MapScene` painted by a `MapSkin`.
`docs/engine/boundary.md`, coupling 6 and *ui*:

- **Generic for a grid game:** tiles, rim, ranges, path, cursor, threat,
  autotiled ground and light; `map_view/{skin,grid,corners,path,threat}.rs`;
  `screens/battle/{camera,cursor,path,walk}.rs`.
- **Bound to this game:**
  - `UnitView` carries `Faction`, class, character, HP and effects
    (`map_view/scene.rs:131-168`).
  - `RangeKind` is fixed to Danger, Move, Attack and Heal.
  - Skins take `trpg_core::Pos` and the whole `Ctx`.
  - Generic code reaches into `screens::battle` for walk speeds and unit
    colors.
  - `GAME_TILESET = "tiny_tales"` (`map_view.rs:37`).

## Nick input

None. The battle map must look identical in the glyph and sprite looks.

## Scope

**In:**
- `engine_ui::grid::MapScene<P>`, where `P` is the game's piece view
  (this game: today's `UnitView`).
- Ranges become `RangeLayer { kind: name, tiles }`, and the skin colors
  them by name.
- `trait GridSkin<P>` with the tile size, `view_tiles`, and paint for
  ground, ranges, path, cursor and threat. Pieces are painted by a
  `PiecePainter<P>` that the game gives.
- Moved with their tests:
  - the corner autotiling rule and ground layers with light (ADR-0052,
    ADR-0058);
  - grid math (with a `Projection` hook left for 1126, ADR-0064 §6);
  - path and threat arrows;
  - the tile camera, virtual cursor, path steering and walk timing.
  Walk speeds become skin settings.
- The glyph and sprite skins split. The terrain and cursor parts move to
  the engine. The unit labels, HP bars, effect marks and unit sheets stay
  the game's, as its `PiecePainter`s.
- `Pos` comes from `engine-core` (1114).

**Out (do not do):**
- isometric (1126);
- making the battle screen itself generic (it is the game's);
- the `.map` format (1116).

## Implementation steps

1. Make the scene and skins generic in place, with aliases.
2. Move the modules.
3. A toy grid game in the engine's tests: a 10×8 map, two pieces, a move
   range, a path. Harness test plus a `frame-png` snapshot, in both
   skins (a test tileset from `xtask test-tileset`).
4. Check the behavior tests in `ui/tests/it` (map skin, map zoom,
   threat). They read the scene, so they should pass unchanged.

## Acceptance criteria

- [ ] The toy grid game passes in `engine-ui` alone, in both skins.
- [ ] No snapshot of the game changes in either look.
- [ ] Every battle test passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: toy grid game Harness tests and snapshots.
- Snapshot: the game's unchanged.

## Completion notes
