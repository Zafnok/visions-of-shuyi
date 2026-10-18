---
id: "1154"
title: "The map picture and the map pan over the grid kit move to the engine; the scene camera goes with them"
type: infra
milestone: "Engine: carve"
model: sonnet-5
effort: high
status: todo
blocked_by: ["1115"]
nick_input: none
completed:
---

# 1154 — The map picture and the map pan move to the engine

## Context

1115 (ADR-0088) moved the battle map's scene and skins to the engine as
the tactics kit's grid (`shuyi_ui::grid`): `MapScene<U>`, `GridSkin<U>`
with the game's `UnitPainter<U>`, the camera, cursor, path and walk
timing. `docs/engine/boundary.md` still lists three things that waited on
it, all "K, until 1115":

- `crates/ui/src/cinematic.rs`: `MapPicture` (a whole map painted once
  into a buffer just big enough, to pan over: `paint`, `size_px`,
  `point_px`) and `shot_skin` (the map skin at the art's size). Nothing
  in them is this game's but the types: `Ctx`, `MapSkin`, `MapScene`,
  `UiColor::Text`/`Black` for the blank cell.
- The `MapPan` shot's frame (`Frame::MapPan { origin_px, zoom }`, from
  the engine's `Pan` over the picture) and its paint (the picture as the
  frame's backdrop at `origin_px` and `zoom`).
- `crates/ui/src/debug/scene_camera.rs`: the scene camera (ticket 0228),
  a debug tool that shows a map's `MapPicture` through a window, panned by
  the pixel and zoomed in whole steps. 1112 left it the game's because it
  paints the map scene.

The shot kinds stay the game's (1113, ADR-0060): what moves is the
picture and the pan over it, which any grid game's cinematic or debug
tool can use.

## Nick input

None. Nothing a player sees changes.

## Scope

**In:**
- `shuyi_ui::grid::picture::MapPicture`: `paint(looks, skin: &dyn
  GridSkin<U>, units: &dyn UnitPainter<U>, scene, console: ConsoleSpec,
  blank: Cell)`, `size_px`, `point_px`, with its tests on the toy's grid
  (`shuyi_ui::toy::grid`: `Field`, `Kit`, `Piece`, both skins). The
  game's `MapPicture::paint(ctx, skin, scene)` keeps its signature and
  calls it through its `MapSkin` (add a way for a `MapSkin` to paint into
  a buffer of any size if `MapSkin::paint` isn't enough as it is).
- The map pan's frame and paint as an engine helper next to `Pan`
  (`shuyi_ui::cinematic`), e.g. `PanFrame { origin_px, zoom }` with a
  `paint_pan(picture, frame, buf)`, which the game's `MapPan` painter
  calls. The shot kind itself, its action and its data stay the game's.
- The scene camera, generic over a grid skin and a picture: the engine's
  debug tool (`shuyi_ui::debug`), the game giving its test map
  (`test_small`) as a `MapPicture` and its texts; the toy opens it on its
  field in a Harness test.
- `docs/engine/boundary.md`: the rows of `cinematic.rs`, `cinematic*`,
  `debug*` and the dialogue screen no longer say "until 1115".

**Out (do not do):**
- The map shot's action (`cinematic/action.rs`: authored walks, lunges,
  strikes and falls): this game's cinematic data. Leave it.
- The dialogue screen and `Place` (the place behind a conversation):
  the game's.
- The tileset file format (`trpg_content::tileset`): its own ticket if
  1118's example needs it.
- 1131 (moving `engine/` to its own repository). Either order works: if
  1131 has landed, do the engine half in the engine's repository through
  the engine-change skill (1152) and the game half here.

## Implementation steps

1. Move `MapPicture` and its tests: `engine/crates/ui/src/grid/picture.rs`,
   generic over the unit view `U`; port the game's tests of it
   (`crates/ui/src/cinematic/tests.rs`, the ones naming `MapPicture`) to
   the toy's field, keeping the game's own tests (they paint this game's
   maps) as tests of its wrapper.
2. Add the pan frame and its paint to `shuyi_ui::cinematic`; make the
   game's `Built::MapPan` frame and paint use them. Frames must not
   change: compare `cargo xtask frame-png` of the title's map shots (the
   times 1113 used: 15.5 s to 140 s, and `--playdate`) before and after,
   byte for byte.
3. Move the scene camera to `shuyi_ui::debug`, the game giving its
   picture, caption and help text; the game's debug menu entry opens the
   engine's screen. Its game tests and snapshots stay and must not change.
4. Update `docs/engine/boundary.md` and `engine/README.md`.

## Acceptance criteria

- [ ] `shuyi_ui::grid::picture::MapPicture` paints the toy's field in both
      skins in a test in `shuyi-ui` alone.
- [ ] The game's `MapPicture` and `MapPan` shot use the engine's; the
      title's frames are byte-identical before and after (`frame-png`).
- [ ] The scene camera is the engine's; the toy opens it in a Harness test;
      the game's scene camera tests and snapshots are unchanged.
- [ ] `cargo xtask check-engine-boundary` passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the picture's size and points on the toy's field, both skins.
- Integration: the toy's scene camera through the Harness.
- Snapshot: the game's unchanged; `frame-png` comparisons in the
  Completion notes.

## Completion notes
