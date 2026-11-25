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
use crate::settings::{MapZoom, RangeStyle, ThreatArrows};
use shuyi_ui::grid::glyph::GlyphSkin;
use shuyi_ui::grid::ranges::RangeShow;

/// How strongly the picture look tints ranges, in percent of the glyph
/// look's strength. Nick picked 150% from renders (ticket 1343,
/// 2026-11-23): at 100% the red attack tint over the bought tiles' bright
/// green grass blends to rust; at 150% it reads red.
pub const SPRITE_RANGE_STRENGTH: u16 = 150;

/// Calls `paint` with what the kit's skins paint with in `ctx`: its
/// palette, this game's terrains and its lights.
pub fn with_looks<R>(ctx: &Ctx, paint: impl FnOnce(&Looks) -> R) -> R {
    let terrains = TerrainLooks(&ctx.game.content.terrain.display);
    paint(&Looks::new(
        &ctx.palette,
        &terrains,
        &ctx.game.content.lighting,
    ))
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
    /// A sprite skin's glyph skin (its colours and marks), to paint a
    /// frame of its moving tiles with ([`moving`](super::sprite::moving)).
    sprite_glyph: Option<GlyphSkin>,
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
        Self {
            grid,
            units,
            sprite_glyph: None,
        }
    }

    // engine-friction: engine#2359, moves (game 1357 deletes it)
    /// The same skin, painting a frame of its tileset's moving tiles
    /// with `glyph`'s colours and marks.
    pub(super) const fn with_sprite_glyph(mut self, glyph: GlyphSkin) -> Self {
        self.sprite_glyph = Some(glyph);
        self
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
        Some(Self {
            grid,
            ..self.clone()
        })
    }

    /// The same skin, at the same zoom, drawing the threat arrows as
    /// `arrows` (ticket 0908) and the ranges as `ranges` (ticket 0923).
    #[must_use]
    pub fn with_marks(&self, arrows: ThreatArrows, ranges: RangeStyle) -> Self {
        let (shape, show) = (arrows.shape(), self.range_show(ranges));
        let glyph = |skin: GlyphSkin| skin.with_threat_shape(shape).with_range_show(show);
        let skin = match &self.units {
            Units::Glyph => Self::new(Rc::new(glyph(GLYPH)), Units::Glyph),
            Units::OneBit => Self::new(Rc::new(glyph(ONE_BIT)), Units::OneBit),
            Units::Sprite(tileset) => Self::sprite_on(Tileset::clone(tileset), glyph(GLYPH)),
        };
        match self.zoom() {
            Some(zoom) => skin.at_zoom(zoom).unwrap_or(skin),
            None => skin,
        }
    }

    /// How this kind of skin shows ranges drawn as `ranges`: the picture
    /// look tints them [`SPRITE_RANGE_STRENGTH`] percent as strongly as
    /// the glyph and one-bit looks.
    fn range_show(&self, ranges: RangeStyle) -> RangeShow {
        match self.units {
            Units::Sprite(_) => ranges.show().with_strength(SPRITE_RANGE_STRENGTH),
            Units::Glyph | Units::OneBit => ranges.show(),
        }
    }

    /// Paints `scene` into the cells of `area`, its units by the painter of
    /// this frame's `ctx`. Touches nothing outside it.
    pub fn paint(&self, ctx: &Ctx, scene: &MapScene, area: Rect, buf: &mut GlyphBuffer) {
        // engine-friction: engine#2359, moves (game 1357 deletes it)
        // The frame of the tileset's moving tiles at the scene's clock, if
        // any moves now (ticket 1315).
        let moved = match (&self.units, self.sprite_glyph) {
            (Units::Sprite(tileset), Some(glyph)) => {
                let zoom = self.grid.zoom().unwrap_or(1);
                super::sprite::moving::frame_skin(ctx, scene, tileset, glyph, zoom)
            }
            _ => None,
        };
        let grid: &dyn GridSkin<UnitView> = match &moved {
            Some(skin) => skin,
            None => self.grid.as_ref(),
        };
        with_looks(ctx, |looks| {
            let mut paint = |units: &dyn UnitPainter<UnitView>| {
                grid.paint(looks, scene, area, units, buf);
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
    use crate::screen::tests::ctx;

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

    #[test]
    fn the_picture_look_tints_ranges_at_150_percent_and_the_others_at_100() {
        let sprite = MapSkin::sprite(ctx().game.content.tilesets["test"].clone());
        for ranges in RangeStyle::ALL {
            let percent = |skin: &MapSkin| skin.range_show(ranges).strength_percent;
            assert_eq!(percent(&sprite), 150, "{ranges:?}");
            assert_eq!(percent(&MapSkin::glyph()), 100, "{ranges:?}");
            assert_eq!(percent(&MapSkin::playdate()), 100, "{ranges:?}");
            // The rest of the look is the range style's own.
            assert_eq!(sprite.range_show(ranges).with_strength(100), ranges.show());
        }
    }
}
