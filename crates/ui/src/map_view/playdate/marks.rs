//! The Playdate's marks over the map (`docs/design/playdate.md`): the
//! danger zone's dashed edge. (The cursor's corner marks, which flip the
//! pixels they cover so they show on white, grey and black tiles alike,
//! are the kit's flat cursor: [`ONE_BIT`](super::ONE_BIT).)

use crate::prelude::*;

use crate::glyph_buffer::{PxRect, Rect};
use crate::map_view::grid::Grid;
use crate::map_view::scene::{MapScene, range};

/// How thick the danger zone's edge is, in pixels: as in the mockup Nick
/// chose (`docs/design/playdate/q3r2-e-grey.png`). Two rows of the dashes'
/// 2 × 2 checker make dashes two pixels long.
pub const DANGER_EDGE: i32 = 2;

/// The danger zone's edge on `grid`: on each tile of it, a line
/// [`DANGER_EDGE`] pixels thick along each side whose neighbour is a
/// visible tile outside it. Where the zone goes on past the view's edge
/// nothing is drawn: the edge is where it ends, not where the screen does.
pub fn danger_edges(scene: &MapScene, grid: &Grid) -> Vec<PxRect> {
    let tints = scene.tints();
    let danger = |dx: i32, dy: i32| {
        let zone = range::DANGER;
        scene
            .tile_at(dx, dy)
            .map(|_| tints.at(dx, dy).contains(&zone))
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

#[cfg(test)]
mod tests;
