//! The glyph skin: the battle map drawn with coloured glyphs (ADR-0018,
//! ADR-0024, ADR-0029, `docs/design/look-and-feel.md`): the tactics kit's
//! glyph skin (`shuyi_ui::grid::glyph`, ticket 1115), whose tile is two
//! 8 × 16 cells, with this game's units as two-letter labels and HP bars
//! ([`units`]), and an arrow glyph at each edge the map goes on past
//! ([`edges`]).

pub mod units;

use shuyi_ui::grid::glyph::GlyphSkin;
use shuyi_ui::grid::skin::{MapColors, UnitPainter};

use super::edges::{self, Side};
use super::grid::Grid;
use super::scene::{MapScene, UnitView};
use crate::color::{Palette, UiColor};
use crate::glyph_buffer::{Cell, GlyphBuffer};
use crate::screens::battle::walk::GLYPH_WALK;

pub use shuyi_ui::grid::glyph::{
    Layout, OVERLAY_BLEND, TILE_W_CELLS, cell_of, cursor, draw_tiles, tile_px, tile_to_cell,
};

/// This game's palette names of the map's path, cursor and threat
/// arrows: the path's and the cursor's colours, and the enemy's.
pub const MAP_COLORS: MapColors = MapColors {
    path: UiColor::Path.name(),
    cursor: UiColor::Cursor.name(),
    threat: UiColor::Enemy.name(),
};

/// The kit's glyph skin as this game paints it: its colours, its walk
/// speeds, and the cursor's lower arms above the 2 px HP bar. Terrain,
/// then the ranges tinting it, then the units (a highlighted one with its
/// tile's colours swapped), the path and the cursor; over them all, the
/// threat arrows and marks.
pub const GLYPH: GlyphSkin = GlyphSkin::new(MAP_COLORS, GLYPH_WALK, units::HP_BAR_H);

/// This game's units as the glyph skin shows them ([`units::draw_unit`]),
/// a highlighted one with its tile's colours swapped; over them, the edge
/// arrows ([`edge_arrow`]).
#[derive(Debug, Clone, Copy)]
pub struct GlyphUnits<'a> {
    /// The colours.
    pub palette: &'a Palette,
}

impl UnitPainter<UnitView> for GlyphUnits<'_> {
    fn paint(&self, scene: &MapScene, grid: &Grid, buf: &mut GlyphBuffer) {
        for unit in &scene.units {
            if let Some((x, y)) = cell_of(grid, unit.pos) {
                units::draw_unit(buf, self.palette, unit, x, y);
                if unit.highlight {
                    units::invert_tile(buf, x, y);
                }
            }
        }
    }

    fn paint_over(&self, scene: &MapScene, grid: &Grid, buf: &mut GlyphBuffer) {
        let color = self.palette.get(UiColor::Cursor);
        let brightness = edges::brightness(scene.clock_ms);
        for (side, tile) in edges::arrows(scene) {
            let Some((x, y)) = cell_of(grid, tile) else {
                continue;
            };
            let (glyph, x) = edge_arrow(side, x);
            if let Some(&cell) = buf.get(x, y) {
                let fg = cell.bg.lerp(color, brightness);
                buf.set(x, y, Cell { glyph, fg, ..cell });
            }
        }
    }
}

/// The arrow glyph at the `side` edge of the view and its cell, on the
/// tile whose left cell is `x`: the tile's outer cell on the right, else
/// its left cell. In the cursor's colour, fading into the cell's
/// background as [`edges::brightness`] says; the cell's glyph (terrain,
/// or a letter of a unit's label) gives way to it.
pub const fn edge_arrow(side: Side, x: i32) -> (char, i32) {
    match side {
        Side::Right => ('→', x + TILE_W_CELLS - 1),
        Side::Left => ('←', x),
        Side::Down => ('↓', x),
        Side::Up => ('↑', x),
    }
}

#[cfg(test)]
pub(crate) mod tests;
