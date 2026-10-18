//! The glyph skin: the battle map drawn with coloured glyphs (ADR-0018,
//! ADR-0024, ADR-0029, `docs/design/look-and-feel.md`): the tactics kit's
//! glyph skin (`shuyi_ui::grid::glyph`, ticket 1115), whose tile is two
//! 8 × 16 cells, with this game's units as two-letter labels and HP bars
//! ([`units`]).

pub mod units;

use shuyi_ui::grid::glyph::GlyphSkin as KitGlyphSkin;
use shuyi_ui::grid::skin::{GridSkin, MapColors, UnitPainter};
use trpg_core::Pos;

use super::grid::Grid;
use super::scene::{MapScene, RangeKind, UnitView};
use super::skin::{MapSkin, with_looks};
use crate::color::{Palette, Rgb, UiColor};
use crate::glyph_buffer::{GlyphBuffer, PxRect, Rect};
use crate::screen::Ctx;
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
/// speeds, and the cursor's lower arms above the 2 px HP bar.
pub const GLYPH: KitGlyphSkin = KitGlyphSkin {
    colors: MAP_COLORS,
    walk: GLYPH_WALK,
    foot_px: units::HP_BAR_H,
};

/// The battle map as coloured glyphs: today's look (ADR-0038).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GlyphSkin;

/// This game's units as the glyph skin shows them ([`units::draw_unit`]),
/// a highlighted one with its tile's colours swapped.
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
}

impl MapSkin for GlyphSkin {
    fn name(&self) -> &'static str {
        "glyph" // check-text: not player text (the skin's id)
    }

    fn view_tiles(&self, area: Rect) -> (i32, i32) {
        GridSkin::<UnitView>::view_tiles(&GLYPH, area)
    }

    /// Terrain, then the ranges tinting it, then the units (a highlighted
    /// one with its tile's colours swapped), the path and the cursor; over
    /// them all, the threat arrows and marks.
    fn paint(&self, ctx: &Ctx, scene: &MapScene, area: Rect, buf: &mut GlyphBuffer) {
        let units = GlyphUnits {
            palette: &ctx.palette,
        };
        with_looks(ctx, |looks| GLYPH.paint(looks, scene, area, &units, buf));
    }

    fn tile_px(&self, scene: &MapScene, area: Rect, tile: Pos) -> Option<PxRect> {
        GLYPH.tile_px(scene, area, tile)
    }
}

/// The palette colour of a range.
pub(super) const fn range_color(kind: RangeKind) -> UiColor {
    kind.color()
}

/// The palette colour called `name`, or `fallback` (content validation
/// makes sure terrain colours exist, so this only guards against a bug).
pub(super) fn named(palette: &Palette, name: &str, fallback: UiColor) -> Rgb {
    palette
        .lookup(name)
        .unwrap_or_else(|| palette.get(fallback))
}

#[cfg(test)]
pub(crate) mod tests;
