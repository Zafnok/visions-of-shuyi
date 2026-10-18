//! The Playdate's map skin (ticket 1037, ADR-0070 §4,
//! `docs/design/playdate.md`, *Telling things apart without colour*): the
//! glyph skin's tiles, two 8 × 16 cells each with the PC's terrain glyphs,
//! painted for the one-bit screen in the inks of ADR-0074. Your units are
//! white letters in a black tile, enemies letters with a white edge on a
//! grey tile, a unit that has acted an outline round black letters; the
//! move range is dots, the attack range diagonal lines and the danger zone
//! a dashed edge; the cursor's corner marks flip what they cover.
//!
//! It paints the same [`MapScene`] as every skin. The inks are the
//! Playdate palette's (`assets/data/palettes/playdate.ron`), by name:
//! [`Inks`].

pub mod marks;
pub mod units;

use trpg_content::Ink;
use trpg_core::{Pos, TerrainId};

use super::glyph::{self, Layout, TILE_W_CELLS, named, tile_to_cell};
use super::grid::px_rect;
use super::path;
use super::scene::{MapScene, RangeKind};
use super::skin::MapSkin;
use super::threat;
use crate::color::{Palette, Rgb, UiColor};
use crate::glyph_buffer::{Cell, GlyphBuffer, Layer, Overlay, PxRect, Rect};
use crate::screen::Ctx;
use shuyi_ui::grid::RangeName;

/// Cells left clear at each side of the map area: on the Playdate's 50
/// cells, 24 tiles across (`docs/design/playdate.md`: 24 × 15 tiles at
/// once), with Chapter 1's 24-tile map in the middle and room for the
/// cursor's marks beside the first and last tile.
pub const SIDE_CELLS: i32 = 1;

/// How strong a flash (a terrain change's, 0410; a blow's, 0827) must still
/// be to show: one bit has no fading, so the tile is flipped for the first
/// half of the glyph skin's flash.
pub const FLASH_SHOWN: f32 = 0.5;

/// The battle map for the Playdate's one-bit screen.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PlaydateSkin;

impl MapSkin for PlaydateSkin {
    fn name(&self) -> &'static str {
        "playdate" // check-text: not player text (the skin's id)
    }

    fn view_tiles(&self, area: Rect) -> (i32, i32) {
        let across = (area.w - SIDE_CELLS - SIDE_CELLS).max(0) / TILE_W_CELLS;
        (across, area.h.max(0))
    }

    /// Terrain and the ranges filling it, the danger zone's edge, the
    /// units, the path, the cursor; over them all, the threat arrows and
    /// marks.
    fn paint(&self, ctx: &Ctx, scene: &MapScene, area: Rect, buf: &mut GlyphBuffer) {
        let layout = layout(scene, area);
        let inks = Inks::of(&ctx.palette);
        let clip = px_rect(area);
        draw_tiles(ctx, &inks, buf, scene, &layout);
        let dashes = ctx.palette.get(UiColor::DangerZone);
        for rect in marks::danger_edges(scene, &layout.grid()) {
            buf.add_overlay(Overlay::new(rect, dashes, Layer::Under));
        }
        for unit in &scene.units {
            if let Some(at) = tile_to_cell(unit.pos, &layout) {
                units::draw_unit(buf, &ctx.palette, &inks, unit, at, clip);
            }
        }
        let color = ctx.palette.get(UiColor::Path);
        for overlay in path::path_overlays(&scene.path, &layout.grid(), color) {
            buf.add_overlay(overlay);
        }
        if let Some(c) = &scene.cursor
            && let Some(at) = tile_to_cell(c.pos, &layout)
        {
            marks::draw_cursor(buf, &ctx.palette, c, at, clip);
        }
        if let Some(t) = &scene.threat {
            let grid = layout.grid();
            for o in threat::threat_overlays(t, &grid, threat::Start::Edge, inks.threat()) {
                buf.add_overlay(o);
            }
        }
    }

    fn tile_px(&self, scene: &MapScene, area: Rect, tile: Pos) -> Option<PxRect> {
        let (x, y) = tile_to_cell(tile, &layout(scene, area))?;
        Some(px_rect(Rect::new(x, y, TILE_W_CELLS, 1)))
    }
}

/// Where `scene`'s tiles go in `area`: as many as fit, in the middle of
/// it across, from its top.
fn layout(scene: &MapScene, area: Rect) -> Layout {
    let fit = PlaydateSkin.view_tiles(area);
    let margin = (area.w - TILE_W_CELLS * fit.0) / 2;
    Layout::fitted(scene, (area.x + margin, area.y), fit)
}

/// The inks the skin paints with besides the default palette's: the
/// Playdate palette's colours of these names, or the design's ink where a
/// palette lacks one (the PC's, in a test).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Inks {
    /// Your unit's tile, and its letters.
    pub player: (Rgb, Rgb),
    /// An enemy's tile, and its letters.
    pub enemy: (Rgb, Rgb),
    /// A unit that has acted: its tile, its letters and its outline.
    pub acted: (Rgb, Rgb, Rgb),
    /// An ally's or a neutral unit's letters, on the terrain's tile.
    pub other: Rgb,
    /// A terrain glyph on a tile a range fills.
    pub marked: Rgb,
    /// A unit a battle note is about, or a tile flashing: what covers it.
    pub highlight: Rgb,
    /// The threat arrows and marks, and their edge.
    pub arrows: (Rgb, Rgb),
    /// The HP lost on the HP line of your units and allies.
    pub hp_lost: Rgb,
}

impl Inks {
    /// The inks of `palette`.
    pub fn of(palette: &Palette) -> Self {
        let ink = |name: &str, fallback: Ink| {
            palette
                .lookup(name)
                .unwrap_or_else(|| Rgb::from(fallback.code()))
        };
        Self {
            player: (
                ink("map_player_tile", Ink::Black),
                ink("map_player_letters", Ink::White),
            ),
            enemy: (
                ink("map_enemy_tile", Ink::Grey),
                ink("map_enemy_letters", Ink::BlackEdged),
            ),
            acted: (
                ink("map_acted_tile", Ink::White),
                ink("map_acted_letters", Ink::Black),
                ink("map_acted_outline", Ink::Black),
            ),
            other: ink("map_other_letters", Ink::BlackEdged),
            marked: ink("map_marked_glyph", Ink::BlackEdged),
            highlight: ink("map_highlight", Ink::Invert),
            arrows: (
                ink("map_threat", Ink::Black),
                ink("map_threat_edge", Ink::White),
            ),
            hp_lost: ink("map_hp_lost", Ink::Grey),
        }
    }

    /// The threat arrows' colours.
    pub const fn threat(&self) -> threat::Colors {
        let (arrow, edge) = self.arrows;
        threat::Colors {
            arrow,
            mark: arrow,
            edge,
        }
    }
}

/// Draws every tile of `layout`: its terrain in its palette colours (tiles
/// off the map left as they are); the fill of the range laid on last
/// (the danger zone has none: [`marks::danger_edges`]), its glyphs edged in
/// white; a tile a spell would change as the terrain it would become, as on
/// the glyph skin; and a tile whose terrain just changed flipped while its
/// flash is strong.
fn draw_tiles(ctx: &Ctx, inks: &Inks, buf: &mut GlyphBuffer, scene: &MapScene, layout: &Layout) {
    let grid = layout.grid();
    let tints = scene.tints();
    for dy in 0..grid.tiles.1 {
        for dx in 0..grid.tiles.0 {
            let Some(tile) = scene.tile_at(dx, dy) else {
                continue;
            };
            let at = (layout.cells().x + TILE_W_CELLS * dx, layout.cells().y + dy);
            if let Some(id) = tile.terrain {
                draw_terrain(ctx, buf, id, at);
            }
            if let Some(fill) = fill(tints.at(dx, dy)) {
                let fill = ctx.palette.get(glyph::range_color(fill));
                for x in at.0..at.0 + TILE_W_CELLS {
                    if let Some(&cell) = buf.get(x, at.1) {
                        buf.set(x, at.1, Cell::new(cell.glyph, inks.marked, fill));
                    }
                }
            }
            if let Some(id) = tile.becomes {
                draw_terrain(ctx, buf, id, at);
            }
            if tile.flashes.iter().any(|&s| s >= FLASH_SHOWN) {
                buf.add_overlay(Overlay::new(
                    grid.rect_at(dx, dy),
                    inks.highlight,
                    Layer::Over,
                ));
            }
        }
    }
}

/// The range whose fill a tile in `ranges` shows: the last one laid on,
/// the danger zone aside.
fn fill(ranges: &[RangeName]) -> Option<RangeKind> {
    ranges
        .iter()
        .rev()
        .filter_map(|&name| RangeKind::named(name))
        .find(|&kind| kind != RangeKind::Danger)
}

/// Draws terrain `id` on the tile whose left cell is `(x, y)`: its two
/// glyphs in its palette colours. Nothing for a terrain the content lacks.
fn draw_terrain(ctx: &Ctx, buf: &mut GlyphBuffer, id: TerrainId, (x, y): (i32, i32)) {
    let Some(t) = ctx.game.content.terrain.display.get(id) else {
        return;
    };
    let fg = named(&ctx.palette, &t.fg, UiColor::Text);
    let bg = named(&ctx.palette, &t.bg, UiColor::Black);
    for (i, glyph) in (0..).zip(t.glyphs) {
        buf.set(x + i, y, Cell::new(glyph, fg, bg));
    }
}

#[cfg(test)]
pub(crate) mod tests;
