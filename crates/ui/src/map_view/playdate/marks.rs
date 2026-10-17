//! The Playdate's marks over the map (`docs/design/playdate.md`): the
//! danger zone's dashed edge and the cursor's corner marks, which flip the
//! pixels they cover so they show on white, grey and black tiles alike.

use crate::prelude::*;

use crate::color::{Palette, UiColor};
use crate::glyph_buffer::{GlyphBuffer, Layer, Overlay, PxRect, Rect};
use crate::map_view::glyph::TILE_W_CELLS;
use crate::map_view::glyph::cursor::{CORNER_ARM, LARGE_CORNER_ARM, corner_arms};
use crate::map_view::grid::{CELL_PX, Grid};
use crate::map_view::scene::{CursorStyle, CursorView, MapScene, RangeKind};

/// How thick the danger zone's edge is, in pixels: as in the mockup Nick
/// chose (`docs/design/playdate/q3r2-e-grey.png`). Two rows of the dashes'
/// 2 × 2 checker make dashes two pixels long.
pub const DANGER_EDGE: i32 = 2;

/// The danger zone's edge on `grid`: on each tile of it, a line
/// [`DANGER_EDGE`] pixels thick along each side whose neighbour is a
/// visible tile outside it. Where the zone goes on past the view's edge
/// nothing is drawn: the edge is where it ends, not where the screen does.
pub fn danger_edges(scene: &MapScene, grid: &Grid) -> Vec<PxRect> {
    let danger = |dx: i32, dy: i32| {
        scene
            .tile_at(dx, dy)
            .map(|t| t.tints.contains(&RangeKind::Danger))
    };
    let (tw, th) = grid.tile;
    let mut out = Vec::new();
    for dy in 0..grid.tiles.1 {
        for dx in 0..grid.tiles.0 {
            if danger(dx, dy) != Some(true) {
                continue;
            }
            let tile = grid.rect_at(dx, dy);
            let sides = [
                ((0, -1), Rect::new(tile.x, tile.y, tw, DANGER_EDGE)),
                (
                    (0, 1),
                    Rect::new(tile.x, tile.y + th - DANGER_EDGE, tw, DANGER_EDGE),
                ),
                ((-1, 0), Rect::new(tile.x, tile.y, DANGER_EDGE, th)),
                (
                    (1, 0),
                    Rect::new(tile.x + tw - DANGER_EDGE, tile.y, DANGER_EDGE, th),
                ),
            ];
            for ((ox, oy), side) in sides {
                if danger(dx + ox, dy + oy) == Some(false) {
                    out.push(side);
                }
            }
        }
    }
    out
}

/// The glyph skin's corner marks round the tile whose left cell is
/// `(x, y)`, each `arm` pixels long, with no pixel in two arms: flipped
/// twice, the corner would show as if not flipped at all. Each corner's
/// upright arm gives its flat one the pixel they share.
fn flipped_arms(x: i32, y: i32, arm: i32) -> Vec<Rect> {
    let arms = corner_arms(x, y, arm);
    let mut out = Vec::with_capacity(arms.len());
    for &[flat, upright] in arms.as_chunks::<2>().0 {
        let from = i32::from(upright.y == flat.y);
        out.push(flat);
        out.push(Rect::new(upright.x, upright.y + from, 1, upright.h - 1));
    }
    out
}

/// Draws `cursor` on the tile whose left cell is `(x, y)`, clipped to
/// `clip`, in the palette's cursor ink (which flips what it covers): the
/// glyph skin's corner marks (`flipped_arms`), or for the tile glow the
/// whole tile. Its pulse isn't shown: one bit has no dimming.
pub fn draw_cursor(
    buf: &mut GlyphBuffer,
    palette: &Palette,
    cursor: &CursorView,
    (x, y): (i32, i32),
    clip: PxRect,
) {
    let ink = palette.get(UiColor::Cursor);
    let rects = match cursor.style {
        CursorStyle::Corners => flipped_arms(x, y, CORNER_ARM),
        CursorStyle::LargeCorners => flipped_arms(x, y, LARGE_CORNER_ARM),
        CursorStyle::TileGlow => {
            let (cw, ch) = CELL_PX;
            vec![Rect::new(x * cw, y * ch, TILE_W_CELLS * cw, ch)]
        }
    };
    for rect in rects {
        if let Some(rect) = rect.intersect(&clip) {
            buf.add_overlay(Overlay::new(rect, ink, Layer::Over));
        }
    }
}

#[cfg(test)]
mod tests;
