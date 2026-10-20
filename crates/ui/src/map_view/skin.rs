//! A map skin: how a [`MapScene`] looks (ADR-0038). Each is one of the
//! tactics kit's grid skins (`shuyi_ui::grid`, tickets 1115 and 2313) and
//! which of this game's unit painters goes with it ([`Units`]), built each
//! frame from the frame's context; [`with_looks`] gives the kit's skins
//! this game's palette, terrains and lights.

use alloc::rc::Rc;

use shuyi_ui::grid::skin::{GridSkin, Looks, UnitPainter};
use trpg_content::Tileset;

use super::glyph::{GLYPH, GlyphUnits};
use super::playdate::{ONE_BIT, OneBitUnits};
use super::scene::{MapScene, TerrainLooks, UnitView};
use super::sprite::SpriteUnits;
use crate::glyph_buffer::{GlyphBuffer, Rect};
use crate::screen::Ctx;
use crate::settings::MapZoom;

/// Calls `paint` with what the kit's skins paint with in `ctx`: its
/// palette, this game's terrains and its lights.
pub fn with_looks<R>(ctx: &Ctx, paint: impl FnOnce(&Looks) -> R) -> R {
    let terrains = TerrainLooks(&ctx.game.content.terrain.display);
    paint(&Looks {
        palette: &ctx.palette,
        terrains: &terrains,
        lights: &ctx.game.content.lighting,
    })
}

/// Paints battle maps: a kit skin and the painter of this game's units
/// that goes with it. Its tile size, its view, where its tiles are and its
/// walk speeds are the kit skin's (it derefs to it). A skin never changes
/// the game: only the frame, how many tiles are on screen, and how fast a
/// walk is shown.
#[derive(Debug, Clone)]
pub struct MapSkin {
    grid: Rc<dyn GridSkin<UnitView>>,
    units: Units,
}

/// Which of this game's painters paints the units on a skin.
#[derive(Debug, Clone, PartialEq)]
pub enum Units {
    /// Two-letter labels and HP bars ([`GlyphUnits`]).
    Glyph,
    /// The pictures of a tileset ([`SpriteUnits`]).
    Sprite(Rc<Tileset>),
    /// The Playdate's labels in one bit, and the danger zone's edge
    /// ([`OneBitUnits`]).
    OneBit,
}

impl MapSkin {
    /// The kit's skin `grid`, its units painted by `units`.
    pub fn new(grid: Rc<dyn GridSkin<UnitView>>, units: Units) -> Self {
        Self { grid, units }
    }

    /// The battle map as coloured glyphs: today's look.
    pub fn glyph() -> Self {
        Self::new(Rc::new(GLYPH), Units::Glyph)
    }

    /// The battle map for the Playdate's one-bit screen.
    pub fn playdate() -> Self {
        Self::new(Rc::new(ONE_BIT), Units::OneBit)
    }

    /// A stable `snake_case` name for the kind of skin: the kit skin's, or
    /// `"playdate"` for the one-bit one (the kit's glyph skin, styled).
    pub fn name(&self) -> &'static str {
        match self.units {
            Units::OneBit => "playdate", // check-text: not player text (the skin's id)
            _ => self.grid.name(),
        }
    }

    /// The tileset it paints from, if it paints from one.
    pub fn tileset(&self) -> Option<&Tileset> {
        match &self.units {
            Units::Sprite(tileset) => Some(tileset),
            Units::Glyph | Units::OneBit => None,
        }
    }

    /// The id of the tileset it paints from, if it paints from one.
    pub fn tileset_id(&self) -> Option<&str> {
        self.tileset().map(|t| t.id.as_str())
    }

    /// The zoom it draws the map at, or `None` if it can't zoom.
    pub fn zoom(&self) -> Option<MapZoom> {
        self.grid.zoom().and_then(MapZoom::from_factor)
    }

    /// The same skin drawing at `zoom`, or `None` if it can't zoom.
    pub fn at_zoom(&self, zoom: MapZoom) -> Option<Self> {
        let grid = self.grid.at_zoom(zoom.factor())?;
        Some(Self::new(grid, self.units.clone()))
    }

    /// Paints `scene` into the cells of `area`, its units by the painter of
    /// this frame's `ctx`. Touches nothing outside it.
    pub fn paint(&self, ctx: &Ctx, scene: &MapScene, area: Rect, buf: &mut GlyphBuffer) {
        with_looks(ctx, |looks| {
            let mut paint = |units: &dyn UnitPainter<UnitView>| {
                self.grid.paint(looks, scene, area, units, buf);
            };
            match &self.units {
                Units::Glyph => paint(&GlyphUnits {
                    palette: &ctx.palette,
                }),
                Units::Sprite(tileset) => paint(&SpriteUnits { ctx, tileset }),
                Units::OneBit => paint(&OneBitUnits::new(ctx, area)),
            }
        });
    }
}

impl core::ops::Deref for MapSkin {
    type Target = dyn GridSkin<UnitView>;

    fn deref(&self) -> &Self::Target {
        self.grid.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_sprite_skin_has_a_tileset_and_only_its_terrain_zooms() {
        for skin in [MapSkin::glyph(), MapSkin::playdate()] {
            assert_eq!(skin.tileset_id(), None);
            // A skin can't zoom unless it says it can.
            assert_eq!(skin.zoom(), None);
            assert!(skin.at_zoom(MapZoom::X2).is_none());
        }
        assert_eq!(MapSkin::glyph().name(), "glyph");
        assert_eq!(MapSkin::playdate().name(), "playdate");
    }
}
