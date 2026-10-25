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

use shuyi_ui::grid::glyph::{
    CursorLook, FlashLook, GlyphSkin, GlyphStyle, RangeLook, ThreatLook, cell_of,
};
use shuyi_ui::grid::skin::{MapColors, UnitPainter};
use trpg_content::Ink;

use super::glyph::{GLYPH, MAP_COLORS};
use super::grid::{Grid, px_rect};
use super::scene::{MapScene, UnitView, range};
use crate::color::{Palette, Rgb, UiColor};
use crate::glyph_buffer::{GlyphBuffer, Layer, Overlay, PxRect, Rect};
use crate::screen::Ctx;

pub use shuyi_ui::grid::glyph::style::FLASH_SHOWN;

/// The battle map for the Playdate's one-bit screen: the glyph skin's
/// tiles and paint order, with the ranges filling their tiles (their
/// glyphs edged in white; the danger zone has none: [`marks::danger_edges`]),
/// a tile whose terrain just changed flipped while its flash is strong,
/// flat cursor marks that flip what they cover, and the threat arrows and
/// marks black edged in white. One cell is left clear at each side of the
/// map area: on the Playdate's 50 cells, 24 tiles across
/// (`docs/design/playdate.md`: 24 × 15 tiles at once), with Chapter 1's
/// 24-tile map in the middle and room for the cursor's marks beside the
/// first and last tile.
pub const ONE_BIT: GlyphSkin = GlyphSkin {
    colors: MapColors::new(MAP_COLORS.path, MAP_COLORS.cursor, "map_threat"),
    style: GlyphStyle {
        range: RangeLook::Fill {
            glyphs: "map_marked_glyph",
            skip: &[range::DANGER],
        },
        flash: FlashLook::Invert,
        cursor: CursorLook::Flat,
        threat: ThreatLook::Plain {
            edge: "map_threat_edge",
        },
        side_cells: 1,
    },
    ..GLYPH
};

/// This game's units as the Playdate's skin shows them
/// ([`units::draw_unit`]), over the danger zone's dashed edge.
#[derive(Debug, Clone, Copy)]
pub struct OneBitUnits<'a> {
    /// The colours.
    pub palette: &'a Palette,
    /// The inks it paints with besides them.
    pub inks: Inks,
    /// The map area, in console pixels: the units' items are clipped to it.
    pub clip: PxRect,
}

impl<'a> OneBitUnits<'a> {
    /// The painter of `ctx`'s palette, for a map painted into `area`.
    pub fn new(ctx: &'a Ctx, area: Rect) -> Self {
        Self {
            palette: &ctx.palette,
            inks: Inks::of(&ctx.palette),
            clip: px_rect(area),
        }
    }
}

impl UnitPainter<UnitView> for OneBitUnits<'_> {
    fn paint_under(&self, scene: &MapScene, grid: &Grid, buf: &mut GlyphBuffer) {
        let dashes = self.palette.get(UiColor::DangerZone);
        for rect in marks::danger_edges(scene, grid) {
            buf.add_overlay(Overlay::new(rect, dashes, Layer::Under));
        }
    }

    fn paint(&self, scene: &MapScene, grid: &Grid, buf: &mut GlyphBuffer) {
        for unit in &scene.units {
            if let Some(at) = cell_of(grid, unit.pos) {
                units::draw_unit(buf, self.palette, &self.inks, unit, at, self.clip);
            }
        }
    }
}

/// The inks the skin's units are painted with besides the default
/// palette's: the Playdate palette's colours of these names, or the
/// design's ink where a palette lacks one (the PC's, in a test). The
/// ranges' glyphs and the threat arrows are [`ONE_BIT`]'s, by name.
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
    /// A unit a battle note is about: what covers it.
    pub highlight: Rgb,
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
            highlight: ink("map_highlight", Ink::Invert),
            hp_lost: ink("map_hp_lost", Ink::Grey),
        }
    }
}

#[cfg(test)]
pub(crate) mod tests;
