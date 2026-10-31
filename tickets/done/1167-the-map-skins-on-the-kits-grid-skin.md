---
id: "1167"
title: "The map skins on the kit's grid skin: the game's MapSkin trait and the one-bit skin's copied paint order go"
type: feature
milestone: "Engine: carve"
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: ["engine#2313"]
nick_input: none
completed: 2026-10-20
---

# 1167 — The map skins on the kit's grid skin

## Context

The first engine-friction sweep (1153, `docs/engine/friction.md`)
measured the map view (`crates/ui/src/map_view*`) at 1266 production
lines against an estimate of 750 (1.7×). About 190 lines are glue:

- `map_view/skin.rs`'s `MapSkin` repeats the kit's `GridSkin` method
  for method, adding zoom, a third walk speed and a `paint` that takes
  the game's `Ctx`; `map_view/glyph.rs` and `map_view/sprite.rs` only
  forward to the kit;
- `map_view/playdate.rs` copies the kit glyph skin's paint order, its
  `draw_tiles` and `draw_terrain`, differing in how ranges and flashes
  look and a side margin;
- `map_view/scene.rs` converts the game's range kinds to the kit's
  names and back.

Engine ticket 2313 (`Zafnok/shuyi-engine`) gives `GridSkin` zoom, a
held walk speed and a documented per-frame painter, and `GlyphSkin` a
`GlyphStyle` (range fill or tint, flash blend or invert, a margin).

## Nick input

None.

## Scope

**In:**
- Move the pin past 2313 (`engine-change` skill, step 7).
- The game's skins are kit `GridSkin`s with the game's unit painters;
  `MapSkin` and its forwarding impls go. Where the battle screen needs
  the game's context to paint units, build the painter in `draw`.
- The one-bit map skin is the kit's `GlyphSkin` with a `GlyphStyle`
  and the game's one-bit unit painter; its copied paint order goes.
- The range-kind conversion goes if the kit's names can be used
  directly; otherwise say why in the Completion notes.

**Out (do not do):**
- How the units look (the painters stay the game's: ADR-0038).
- Any change the player sees, on the PC or the Playdate.

## Implementation steps

1. Write the estimate: about 190 lines removed, 40 added.
2. Glyph skin first, then sprite, then one-bit; map snapshots unchanged
   after each.

## Acceptance criteria

- [x] `grep -rn "trait MapSkin" crates/ui/src` finds nothing.
- [x] Every map snapshot (glyph, sprite, one-bit) is unchanged.
- [x] `cargo xtask code-lines` shows `trpg-ui` fell by at least 120.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- The existing map skin tests and snapshots, unchanged.

## Completion notes

**Done:**

- **The pin** moved to engine `main` (a435c0e), past 2313. It also
  carries engine 2312 (made `FrameInput::dt` private), so every
  `input.dt` became `input.dt()`; ticket 1166 notes that its pin step
  is done. And engine 1129 (tool contract): no game code changes.
- **`MapSkin` is a struct, not a trait** (`map_view/skin.rs`, ADR-0094):
  a kit `Rc<dyn GridSkin<UnitView>>` and `Units` (`Glyph`,
  `Sprite(tileset)`, `OneBit`). It derefs to the kit skin (`view_tiles`,
  `tile_px`, `tile_cells`, `grid`, `walk`); its own `paint(ctx, …)`
  builds the unit painter from that frame's context. Constructors
  `MapSkin::glyph()`, `MapSkin::sprite(tileset)` (the kit's sprite skin
  with every `MapZoom`), `MapSkin::playdate()`, `MapSkin::new(grid,
  units)`. `GlyphSkin`, `SpriteSkin`, `PlaydateSkin` and their
  forwarding impls are gone; `Ctx` holds the skin by value.
- **The one-bit skin** is `playdate::ONE_BIT`: `GLYPH` with the threat
  colour `map_threat` and a `GlyphStyle` (ranges `Fill` in
  `map_marked_glyph`, the danger zone skipped; flashes `Invert`; `Flat`
  cursor; `Plain` threat edged in `map_threat_edge`; one side cell).
  `OneBitUnits` paints the units and, in `paint_under`, the danger
  zone's edge. Its copied `draw_tiles`, `draw_terrain`, `fill`, `layout`
  and the flat cursor (`marks::draw_cursor`, `flipped_arms`) are gone;
  `Inks` lost `marked` and `arrows` (the style reads those names).
- **The range-kind conversion went:** `map_view::range` holds the four
  ranges as the kit's `RangeName` constants (`range::MOVE`…, `ALL`).
  `Harness::tints_at` returns `RangeName`s.
- Battle walk speeds read the skin's `walk()` (`normal`/`fast`/`held`).
- Docs: ADR-0094 (supersedes ADR-0088 §6 and its `RangeKind`),
  `crates/ui/README.md`, `docs/engine/boundary.md`.

**Tests:** every map snapshot (glyph, sprite, sprite units, Playdate
`harrowby_fords_opening_view`) is unchanged; all trpg-ui tests pass.
Tests of deleted game code changed shape: the old `MapSkin` default-
method test (the kit tests `cells_of`) became one on the struct's
tileset and zoom; the Playdate cursor test checks the flat marks
through the skin; `the_cursor_is_clipped_to_the_area` (which called the
deleted `draw_cursor` directly) went, as the kit's style tests cover
the clipping; the battle tests' `Narrow` skin is a kit `GridSkin`.

**Friction:** estimate 40, measured 208 added (404 removed; trpg-ui
196 smaller), 5.2×: reviewed, outcome 4 (accept, no engine glue): about
100 added lines are call sites renamed one for one. Row in
`docs/engine/friction.md`.

**Note:** the new pin's checkout prints three `invalid character '{'
in package name` errors from the engine's `templates/` (1129) on every
build; they don't fail it (every gate passes).

**Gameplay rules decided:** none. **Nick:** nothing to see; the map
looks the same on the PC and the Playdate.
