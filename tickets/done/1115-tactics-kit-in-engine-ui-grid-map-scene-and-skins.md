---
id: "1115"
title: "Tactics kit in shuyi-ui: a grid map scene generic over the game's pieces, skins, ground layers, cursor and tile camera"
type: infra
milestone: "Engine: carve"
model: opus-5.5 / gpt-6.1-sol
effort: xhigh
status: done
blocked_by: ["1113", "1114"]
nick_input: none
completed: 2026-10-17
---

# 1115 — Tactics kit in shuyi-ui: grid map scene and skins

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
- `shuyi_ui::grid::MapScene<P>`, where `P` is the game's piece view
  (this game: today's `UnitView`).
- Ranges become `RangeLayer { kind: name, tiles }`, and the skin colors
  them by name.
- `trait GridSkin<P>` with the tile size, `view_tiles`, and paint for
  ground, ranges, path, cursor and threat. Pieces are painted by a
  `PiecePainter<P>` that the game gives.
- Moved with their tests:
  - the corner autotiling rule and ground layers with light (ADR-0052,
    ADR-0058);
  - grid math (with a `Projection` hook left for 1126, engine ADR-0003 §6);
  - path and threat arrows;
  - the tile camera, virtual cursor, path steering and walk timing.
  Walk speeds become skin settings.
- The glyph and sprite skins split. The terrain and cursor parts move to
  the engine. The unit labels, HP bars, effect marks and unit sheets stay
  the game's, as its `PiecePainter`s.
- `Pos` comes from `shuyi-core` (1114).

**Playdate:** the Playdate port (1025's tickets, 1027–1053) builds
`core`, `content` and `ui` without `std` and adds a second front end,
`crates/playdate`. If those tickets have landed, code this ticket moves
keeps its default `std` feature and stays in the `no-std` CI build, and
`cargo xtask playdate` keeps building (ADR-0070 §6). 1037's Playdate map skin stays the game's skin over the engine's scene.

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

- [x] The toy grid game passes in `shuyi-ui` alone, in both skins.
- [x] No snapshot of the game changes in either look.
- [x] Every battle test passes.
- [x] All gates in the `run-gates` skill pass (but `cargo xtask playdate`'s
      last step, see below).

## Tests required

- Integration: toy grid game Harness tests and snapshots.
- Snapshot: the game's unchanged.

## Completion notes

**What moved** (ADR-0088):

- `shuyi_core::TerrainId` (a terrain is an index in the game's table);
  `trpg_core::TerrainId` names it, so saves and maps are unchanged.
- `shuyi_assets::image::{ImageRect, Picture}` and `shuyi_assets::tileset`
  (`TerrainTiles` with `look(name)`, `Look`, `Layer`, `CornerLayer`,
  `TileLayer`, `mix`, `parse_mix`, `mix_name`, `MIXES`), with their tests:
  what the sprite skin paints the ground from. `trpg_content::tileset`
  re-exports them and keeps the file format and its checks.
- `shuyi_ui::grid` (new):
  - `scene`: `MapScene<U>` of the game's units (`MapUnit`: id, tile,
    text), `TileView`, ranges as `RangeLayer { kind: RangeName, tiles }`
    with `tint`, `tinted`, `tints_at` and `tints()` (each visible tile's
    ranges in order), `CursorView`/`CursorStyle`, `ThreatView`,
    `to_text(&dyn Terrains)`.
  - `layout` (was `map_view/grid.rs`): `Grid` with a `Projection` field
    (`TopDown` only: the hook for 1126), `Grid::top_down`, `CELL_PX`,
    `px_rect`.
  - `corners`, `path`, `threat` (`paint` now takes the colours).
  - `skin`: `GridSkin<U>` (name, tile size, `view_tiles`, `grid`, walk
    speeds, `paint`, `tile_px`, `tile_cells`), `UnitPainter<U>`
    (`paint_under`, `paint`), `NoUnits`, `Looks` (palette, `Terrains`,
    lights), `MapColors`, `WalkSpeeds`, `TerrainGlyphs`.
  - `glyph`: the glyph skin (`GlyphSkin { colors, walk, foot_px }`),
    `Layout`, `tile_to_cell`, `cell_of`, `draw_tiles`, `draw_terrain`;
    `glyph::cursor` with `CursorInk`.
  - `sprite`: the sprite skin over `Option<TerrainTiles>` (ground in the
    map's look and light, or the glyph skin's ground), its cursor marks,
    and `sprite::ground`.
  - `camera`, `cursor` (steps by a `Dir`), `steer` (over a closure giving
    the cheapest path), `walk` (timing, `Facing`, `STANDING_FRAME`).
- The toy game's grid (`shuyi_ui::toy::grid`): four terrains, `Piece`, a
  painter for each skin (`Letters`, `Cards`), terrain tiles with layers
  and an indoor look, two lights, and `Field`, a 10 × 8 stage with your
  piece and a foe: pick up, see the move range, steer the path, walk at
  the skin's pace. `tests/it/grid.rs` plays it in both skins, with a
  snapshot of each.

**The game** keeps its names. `map_view::MapScene` is
`MapScene<UnitView>`; `UnitView`, its text and `RangeKind` (named by its
colours, `move_range`…, with `From` and `PartialEq` against names) are in
`map_view/scene.rs`, with `TerrainLooks` (its terrain table as the kit's
`Terrains`). `MapSkin` stays the game's trait: `GlyphSkin` is the kit's
glyph skin (`GLYPH`, `MAP_COLORS`, foot = the HP bar) with `GlyphUnits`;
`SpriteSkin` is the kit's sprite skin with `SpriteUnits` (the light
square or cleared glyphs under, then pictures and marks); the Playdate
skin stays its own over the kit's scene. `skin::with_looks` hands the
kit this game's palette, terrains and lights. `screens/battle/{camera,
cursor,path,walk}.rs` re-export the kit's with `cursor::dir` (the game's
actions), `path::steer` over its `Reach`, and its walk speeds as skin
settings (`GLYPH_WALK`, `SPRITE_WALK`).

**Unchanged frames.** Before any change I recorded every frame (text
snapshot and scene text) of a 22-step scripted Quick Battle (select,
steer, walk, attack, rewind, danger zone, end turn, enemy phase) under
the glyph skin, the test tileset, the tileset with layers and looks and
unit pictures on glyph terrain, each at both zooms, at dusk and at night,
and under the Playdate skin: 264 frames, 4.1 MB. After the move the file
is byte-identical. No snapshot file changed; every game test passes,
`ui/tests/it` unchanged.

**Deviations from the ticket:**

- The ticket said `MapScene<P>` and `PiecePainter<P>`. The kit says
  `units`, `MapUnit` and `UnitPainter<U>`: the scene's field keeps the
  name every tactics game uses, and the game's code and its
  `ui/tests/it` read `scene.units`, `push_unit` and `unit_at` unchanged.
- To make the scene and skins free of the game's types, more moved than
  the ticket listed: `TerrainId` (to `shuyi-core`) and the tileset's
  terrain looks with `ImageRect`/`Picture` (to `shuyi-assets`), which the
  ground painter reads. The tileset file format stays the game's.
- Skins get a `Looks` value, not the game's context; the game's
  `MapSkin` trait stays as a façade (ADR-0088 §4, §6).
- Steps 1 and 2 were done together: each module was `git mv`ed to
  `engine/` and made generic there, then the game rewired over it.
- "A `frame-png` snapshot": `frame-png` is still the game's xtask (1117
  moves it), so the toy's snapshots are the Harness's text snapshots. The
  toy's terrain tiles are described in code with their image's size, not
  made by `xtask test-tileset`: 1118 brings a glyph-made tileset to
  `shuyi-cli`.
- The ADR's `shuyi_ui::world::Projection` is `shuyi_ui::grid::Projection`
  for now: there is no `world` module yet.
- Four game test lines changed because tiles no longer hold their
  ranges: `scene_tests.rs` (two `tile.tints` → `tints_at`), `tests.rs`
  (no ranges → `scene.ranges.is_empty()`) and the Playdate's fill test
  (lays its ranges with `tint`). The game's ground tests moved to the
  engine (ported to the toy's tiles); its sprite tests kept their own
  `layered`/`with_indoor` helpers.
- `.cargo/mutants.toml` leaves out `engine/crates/ui/src/toy/**`, the toy
  game's grid, as it does `toy.rs`.

**Follow-up:** 1154 (the map picture, the map pan and the scene camera,
which `boundary.md` had waiting on this ticket, move to the engine).

**Not checked locally:** `cargo xtask playdate` builds the Rust staticlib
but the SDK's `pdc.exe` crashes on this machine (as in 1108–1112); CI's
`playdate` job runs it. Mutation testing runs in CI.

**Gameplay rules decided:** none. Nothing a player sees changes.
