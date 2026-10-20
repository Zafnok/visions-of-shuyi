# ADR-0088: The grid map kit: a scene generic over the game's units, skins that paint the ground, and the game's unit painters

- **Status:** Accepted; §6 (the game's `MapSkin` façade trait) and §2's `RangeKind` superseded by ADR-0094
- **Date:** 2026-10-17
- **Related tickets:** 1115
- **Extends:** ADR-0038 (a scene, a skin), ADR-0049 (unit pictures on
  glyph terrain), ADR-0052 (corner autotiling), ADR-0058 (a map's light),
  ADR-0063 §6 (couplings made generic before they move), ADR-0080
  (`Ctx<G>`). Engine ADR-0003 §6 (projections) gets its hook.

## Context

ADR-0038 made the battle map a `MapScene` (plain data) painted by a
`MapSkin`. Coupling 6 of `docs/engine/boundary.md` listed what bound it to
this game: the scene carried `UnitView` (faction, class, character, HP,
effects); `RangeKind` was a closed list of four; the skins took
`trpg_core::Pos` and the whole game `Ctx` (its content's terrain table and
lights); generic code reached into `screens::battle` for walk speeds and
unit colours; the cursor's lower arms sat above this game's 2 px HP bar.

Mutation testing runs a mutant against the tests of its own crate, so what
moves needs engine tests; and the game's battle map must look the same,
byte for byte, in every skin.

## Decision

1. **`shuyi_ui::grid::MapScene<U>`**: the scene of a game's units `U`. The
   scene needs three things of a unit, given by `MapUnit`: an id, the tile
   it is shown on, and its line in the scene's text. Everything else about
   a unit is the game's. The field keeps the name `units` (and
   `push_unit`, `unit_at`): the word every tactics game uses, and the
   game's code and tests read the scene unchanged.
2. **Ranges are named layers.** `scene.ranges` is a list of
   `RangeLayer { kind: RangeName, tiles }`, laid in order; `tint` adds one
   (only its visible tiles, nothing if none is), and `tints()` gives each
   visible tile's ranges in that order for a skin. A `RangeName` is the
   palette colour a skin tints with, and its first letter marks the tile
   in the scene's text. The game keeps its `RangeKind` enum, named by its
   colours (`move_range`…), with `From` and `PartialEq` against names, so
   `tint(…, RangeKind::Move)` and `tints_at(…) == [RangeKind::Move]` read
   as before.
3. **Terrain is an index.** `TerrainId` is `shuyi_core`'s (re-exported by
   `trpg_core`). A skin learns a terrain's name and glyph look from the
   game's `Terrains` table; the sprite skin's terrain tiles (`TerrainTiles`,
   looks, corner and tile layers, the mix rule) are `shuyi_assets::tileset`,
   and `ImageRect` and `Picture` `shuyi_assets::image`. How a tileset file
   is written and checked stays the game's for now.
4. **A skin paints the ground; the game paints its units.**
   `GridSkin<U>` has the tile size, `view_tiles`, the `grid` of where tiles
   go, its walk speeds, and `paint(looks, scene, area, units, buf)`. It
   paints the ground, the ranges, the path, the cursor and the threat
   arrows, and calls the game's `UnitPainter<U>` at two fixed points:
   `paint_under` after the ground and ranges (a light square under a unit
   picked out, the glyphs taken off a picture's tile), and `paint` after
   the path's line (before it on the glyph skin) and before its arrowhead.
   `Looks` is what a skin paints with besides the scene: the palette, the
   game's `Terrains` and its lights, built by the game from its context
   for each paint. A skin never sees the game's context.
5. **Skin settings, not the game's constants.** A skin is a value with its
   settings: `MapColors` (the palette's names of the path, cursor and
   threat colours; a threat's marks and every dark edge are the engine's
   `Highlight` and `Background`), `WalkSpeeds` (normal, fast, held) and the
   glyph skin's `foot_px` (the pixels at a tile's foot its cursor keeps
   clear of: a game's HP bar).
6. **The game's skins are façades.** This game keeps its `MapSkin` trait
   (the battle screen, cinematics and debug tools use it unchanged): its
   `GlyphSkin` is the kit's glyph skin with its letters and HP bars
   (`GlyphUnits`); its `SpriteSkin` is the kit's sprite skin of the
   tileset's terrain tiles with its unit pictures, bars and effect marks
   (`SpriteUnits`); its Playdate skin stays its own, over the kit's scene,
   layout, path and threat drawing.
7. **The projection hook.** `Grid` (where a scene's tiles go, in pixels)
   lays them through a `Projection`; only `TopDown` exists. Ticket 1126
   adds the isometric one where every skin already asks.
8. **The kit's stage pieces.** The tile `Camera`, the virtual `Cursor`
   (stepped by a `Dir`; the game maps its actions), `steer` (over a
   closure that gives the cheapest path) and walk timing (`walk`, with
   `Facing` and `STANDING_FRAME`) are the kit's.

## Consequences

- A second grid game makes a scene of its own unit view, a painter for
  it and a `Terrains` table, and gets both skins; the toy grid game in
  `shuyi-ui`'s tests is that game (a 10 × 8 field, two pieces, a move
  range, a path, walking at each skin's pace).
- Painting a frame builds the per-tile range lists once (`tints()`), where
  the scene used to hold them on each tile. A game's test that read
  `tile.tints` reads `tints_at(pos)`.
- Every frame of the game is what it was: a scripted Quick Battle under
  the glyph skin, both sprite tilesets, unit pictures on glyph terrain,
  both zooms, two lights and the Playdate skin gives the same snapshot,
  byte for byte, before and after.
- The tileset file format, the map pan shot and the scene camera stay the
  game's; a picture of the toy's field (`frame-png`) waits for 1117, which
  moves `frame-png` to the engine.

## Alternatives considered

- **Ranges generic over the game's enum (`MapScene<U, R>`)**: type-safe,
  but two parameters on every scene, skin and painter, and the ticket's
  "kind: name" was meant to keep the engine free of a closed list. Names,
  with the game's enum converting to them, keep one parameter.
- **Ranges kept on each tile (`TileView::tints`)**: one lookup less when
  painting, but a range is one thing with its tiles, which is what a skin
  that outlines ranges and a test reading the scene want.
- **The skins taking the game's `Ctx<G>`**: the game's terrain table and
  lights live in its own part of the context, so a skin would need a
  trait on `G` for them; a `Looks` value is simpler and keeps skins free
  of contexts.
- **Moving the game's `MapSkin` trait to the engine**: its zoom
  (`MapZoom`), tileset id and painting with the game's context are this
  game's; the kit's `GridSkin` is what a skin is to any grid game.
