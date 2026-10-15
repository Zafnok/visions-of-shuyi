//! Where a scene's tiles go on the console, in pixels: what every map skin
//! works out from its own tile size, and the shared drawing (the path)
//! works from.

use trpg_core::Pos;

use crate::console::ConsoleSpec;
use crate::glyph_buffer::{PxRect, Rect};

/// A console cell in pixels, across × down. The map skins lay the map out
/// in [`ConsoleSpec::PC`]'s cells: the battle screen's layout is the PC's.
pub const CELL_PX: (i32, i32) = (ConsoleSpec::PC.cell_w as i32, ConsoleSpec::PC.cell_h as i32);

/// `cells` in console pixels.
pub fn px_rect(cells: Rect) -> PxRect {
    let (cw, ch) = CELL_PX;
    Rect::new(cells.x * cw, cells.y * ch, cells.w * cw, cells.h * ch)
}

/// The tiles of a scene laid out on the console.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Grid {
    /// The map tile in the top-left corner.
    pub origin: Pos,
    /// The top-left pixel of that tile.
    pub corner: (i32, i32),
    /// A tile's size in pixels, across × down.
    pub tile: (i32, i32),
    /// Tiles drawn, across × down.
    pub tiles: (i32, i32),
    /// Console pixels per pixel of the art (the skin's zoom): 1 unless
    /// everything on the map is drawn bigger. `tile` already counts it.
    pub scale: i32,
}

impl Grid {
    /// Where `tile` is, as tiles right of and below the origin, if drawn.
    pub fn offset(&self, tile: Pos) -> Option<(i32, i32)> {
        let dx = tile.x.checked_sub(self.origin.x)?;
        let dy = tile.y.checked_sub(self.origin.y)?;
        let inside = (0..self.tiles.0).contains(&dx) && (0..self.tiles.1).contains(&dy);
        inside.then_some((dx, dy))
    }

    /// The top-left pixel of `tile`, drawn or not (it may lie outside the
    /// grid).
    pub fn tile_px(&self, tile: Pos) -> (i32, i32) {
        let (dx, dy) = (tile.x - self.origin.x, tile.y - self.origin.y);
        (
            self.corner.0 + self.tile.0 * dx,
            self.corner.1 + self.tile.1 * dy,
        )
    }

    /// The pixels of the tile `dx` right of and `dy` below the origin.
    pub fn rect_at(&self, dx: i32, dy: i32) -> PxRect {
        let (w, h) = self.tile;
        Rect::new(self.corner.0 + w * dx, self.corner.1 + h * dy, w, h)
    }

    /// The pixels of `tile`, or `None` if it isn't drawn.
    pub fn rect(&self, tile: Pos) -> Option<PxRect> {
        let (dx, dy) = self.offset(tile)?;
        Some(self.rect_at(dx, dy))
    }

    /// The skin's zoom, at least 1.
    pub fn zoom(&self) -> i32 {
        self.scale.max(1)
    }

    /// A tile at the art's own size, its corner at (0, 0): what a mark's
    /// place on a tile is worked out on, whatever the zoom
    /// ([`zoomed`](Self::zoomed)).
    pub fn art_tile(&self) -> PxRect {
        let zoom = self.zoom();
        Rect::new(0, 0, self.tile.0 / zoom, self.tile.1 / zoom)
    }

    /// The same tiles at the art's own size, laid from (0, 0): what a
    /// drawing across tiles (the path) is worked out on, whatever the
    /// zoom, before [`zoomed`](Self::zoomed) from this grid's corner
    /// places it.
    #[must_use]
    pub fn at_art_size(&self) -> Self {
        let tile = self.art_tile();
        Self {
            corner: (0, 0),
            tile: (tile.w, tile.h),
            scale: 1,
            ..*self
        }
    }

    /// `art`, pixels worked out on [`art_tile`](Self::art_tile), on the
    /// tile whose top-left pixel is `corner`: every art pixel a square of
    /// the zoom's console pixels.
    pub fn zoomed(&self, corner: (i32, i32), art: PxRect) -> PxRect {
        let zoom = self.zoom();
        Rect::new(
            corner.0 + art.x * zoom,
            corner.1 + art.y * zoom,
            art.w * zoom,
            art.h * zoom,
        )
    }

    /// The pixels every drawn tile covers.
    pub fn bounds(&self) -> PxRect {
        let (w, h) = self.tile;
        let (x, y) = self.corner;
        Rect::new(x, y, w * self.tiles.0, h * self.tiles.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(x: i32, y: i32) -> Pos {
        Pos::new(x, y)
    }

    #[test]
    fn tiles_are_laid_from_the_corner_in_their_size() {
        let g = Grid {
            origin: p(5, -2),
            corner: (4, 8),
            tile: (24, 20),
            tiles: (3, 2),
            scale: 1,
        };
        assert_eq!(g.offset(p(5, -2)), Some((0, 0)));
        assert_eq!(g.offset(p(7, -1)), Some((2, 1)));
        for out in [
            p(8, -2),
            p(4, -2),
            p(5, 0),
            p(5, -3),
            p(i32::MIN, 0),
            p(0, i32::MIN),
        ] {
            assert_eq!(g.offset(out), None, "{out:?}");
            assert_eq!(g.rect(out), None, "{out:?}");
        }
        assert_eq!(g.tile_px(p(5, -2)), (4, 8));
        assert_eq!(g.tile_px(p(7, -1)), (52, 28));
        // Outside the grid too.
        assert_eq!(g.tile_px(p(4, -3)), (-20, -12));
        assert_eq!(g.rect(p(6, -1)), Some(Rect::new(28, 28, 24, 20)));
        assert_eq!(g.rect_at(2, 0), Rect::new(52, 8, 24, 20));
        assert_eq!(g.bounds(), Rect::new(4, 8, 72, 40));
        // At the art's own size, a mark is where it was worked out.
        assert_eq!(g.art_tile(), Rect::new(0, 0, 24, 20));
        let mark = Rect::new(3, -2, 5, 7);
        assert_eq!(g.zoomed((28, 28), mark), Rect::new(31, 26, 5, 7));
    }

    #[test]
    fn a_zoomed_grid_draws_every_art_pixel_as_a_square() {
        let g = Grid {
            origin: p(0, 0),
            corner: (8, 0),
            tile: (48, 40),
            tiles: (3, 2),
            scale: 2,
        };
        assert_eq!(g.art_tile(), Rect::new(0, 0, 24, 20));
        // From the tile's corner: twice as far, twice as big.
        let mark = Rect::new(3, -2, 5, 7);
        assert_eq!(g.zoomed((56, 40), mark), Rect::new(62, 36, 10, 14));
        assert_eq!(g.zoomed((56, 40), g.art_tile()), g.rect_at(1, 1));
        // The same tiles unzoomed, from (0, 0); already so at 1×.
        let art = Grid {
            corner: (0, 0),
            tile: (24, 20),
            scale: 1,
            ..g
        };
        assert_eq!(g.at_art_size(), art);
        assert_eq!(art.at_art_size(), art);
        // A scale below 1 counts as 1.
        let flat = Grid { scale: 0, ..g };
        assert_eq!(flat.art_tile(), Rect::new(0, 0, 48, 40));
        assert_eq!(flat.zoomed((1, 2), mark), Rect::new(4, 0, 5, 7));
    }
}
