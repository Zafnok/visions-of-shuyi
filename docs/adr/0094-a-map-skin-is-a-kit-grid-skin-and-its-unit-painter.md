# ADR-0094: A map skin is a kit grid skin and the painter of its units

- **Status:** Accepted
- **Date:** 2026-10-20
- **Related tickets:** 1167, 1153; engine ticket 2313

## Context

ADR-0088 §6 kept the game's `MapSkin` trait as a façade over the tactics
kit's `GridSkin`: the trait repeated `GridSkin` method for method, adding
zoom, a third walk speed and a `paint` taking the game's `Ctx`, and its
glyph and sprite skins only forwarded to the kit's. The Playdate's skin
copied the kit glyph skin's paint order, tiles and terrain, differing in
how ranges, flashes, the cursor and the threat marks look and a side
margin. ADR-0088 §2 kept a `RangeKind` enum converted to the kit's
`RangeName`s and back. The first engine-friction sweep (1153) counted
about 190 lines of this glue.

Engine ticket 2313 gave `GridSkin` zoom (`zoom_levels`, `zoom`,
`at_zoom`), documented that a game builds its `UnitPainter` in its own
`draw` each frame, and gave the kit's `GlyphSkin` a `GlyphStyle` (ranges
`Tint` or `Fill`, flashes `Blend` or `Invert`, a `Pulse` or `Flat`
cursor, `Highlighted` or `Plain` threat marks, side cells).

## Decision

1. **`MapSkin` is a struct, not a trait** (`map_view/skin.rs`): an
   `Rc<dyn GridSkin<UnitView>>` and `Units`, which of the game's painters
   goes with it (`Glyph`, `Sprite(tileset)`, `OneBit`). It derefs to the
   kit skin, so `view_tiles`, `tile_px`, `tile_cells`, `grid`,
   `tile_size` and `walk` are the kit's. Its own methods are the game's:
   `paint(ctx, scene, area, buf)` builds the painter from that frame's
   context and calls the kit skin's `paint`; `zoom` and `at_zoom` speak
   the player's `MapZoom`; `tileset`, `tileset_id`; and `name`, the kit
   skin's except `"playdate"` for the one-bit look. It is cheap to clone
   and `Ctx` holds it by value.
2. **The skins are constructors**: `MapSkin::glyph()` (the kit's glyph
   skin as `GLYPH`), `MapSkin::sprite(tileset)` (the kit's sprite skin,
   zooming to every `MapZoom` when its terrain tiles paint the ground),
   `MapSkin::playdate()` and `MapSkin::new(grid, units)` for any other
   kit skin (a test's).
3. **The Playdate's map skin is the kit's glyph skin in a style**
   (`playdate::ONE_BIT`): ranges fill their tiles with their glyphs in
   `map_marked_glyph` (the danger zone skipped: its dashed edge is the
   one-bit painter's `paint_under`), flashes inverted, the flat cursor,
   the threat arrows in `map_threat` edged in `map_threat_edge`, one side
   cell. Its painter (`OneBitUnits`) draws the units' tiles, letters,
   outlines and HP lines and the danger zone's edge.
4. **Ranges are the kit's names.** `map_view::range` holds this game's
   four as `RangeName` constants (`range::MOVE`…, each its colour's name)
   and `range::ALL`; there is no enum and no conversion. This supersedes
   ADR-0088 §2's `RangeKind`.

This supersedes ADR-0088 §6 (the façade trait). Everything else in
ADR-0088 stands.

## Consequences

- A new map look is a kit skin (or a `GlyphStyle`) and, if its units look
  different, a `Units` variant and its painter; nothing forwards.
- Code outside a skin names the kit's methods for what the trait used to
  repeat: `skin.walk().held` instead of `held_walk_tiles_per_s()`.
- The Playdate's map changes only through the kit's style options; a look
  the style can't give needs an engine change (or a kit skin of the
  game's own).

## Alternatives considered

- **Keep the trait, implemented once for a wrapper**: the forwarding
  stays, the sweep's glue with it.
- **An enum of the game's skins**: every method a `match`, and a test's
  own kit skin (the battle tests' narrow view) would need a variant.
- **Keep `RangeKind` for its exhaustive `match`**: one test listed the
  ranges that way; `range::ALL` does the same job.
