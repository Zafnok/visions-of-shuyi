//! The danger zone's edge and the cursor on the Playdate's map, through the
//! one-bit renderer.

use trpg_content::Ink;

use super::*;
use crate::glyph_buffer::Layer;
use crate::map_view::glyph::cursor::{CORNER_ARM, LARGE_CORNER_ARM, corner_arms};
use crate::map_view::glyph::units::HP_BAR_H;
use crate::map_view::playdate::tests::{c, p, painted, pd_ctx, plains, shown, unit};
use crate::map_view::scene::{CursorStyle, CursorView};
use crate::onebit::{ink_is_white, is_white};
use trpg_core::Faction;

/// The danger zone's edge painted for `danger` on plains: its rectangles.
fn edges(danger: &[(i32, i32)]) -> Vec<Rect> {
    let ctx = pd_ctx();
    let mut scene = plains(&ctx);
    scene.tint(danger.iter().map(|&(x, y)| p(x, y)), range::DANGER);
    let buf = painted(&ctx, &scene);
    for o in buf.overlays() {
        assert_eq!((o.color, o.layer), (c(Ink::Dashes), Layer::Under));
    }
    buf.overlays().iter().map(|o| o.rect).collect()
}

/// A lone tile of the zone (tile (2, 1): pixels 40..56 × 16..32) has a
/// 2-pixel edge on every side, inside the tile.
#[test]
fn the_danger_zone_has_a_dashed_edge_round_it() {
    let r = Rect::new;
    assert_eq!(
        edges(&[(2, 1)]),
        [
            r(40, 16, 16, 2),
            r(40, 30, 16, 2),
            r(40, 16, 2, 16),
            r(54, 16, 2, 16),
        ]
    );
    // Two side by side: no edge between them.
    assert_eq!(
        edges(&[(2, 1), (3, 1)]),
        [
            r(40, 16, 16, 2),
            r(40, 30, 16, 2),
            r(40, 16, 2, 16),
            r(56, 16, 16, 2),
            r(56, 30, 16, 2),
            r(70, 16, 2, 16),
        ]
    );
    assert!(edges(&[]).is_empty());
}

/// Where the zone runs on past the view, it has no edge there; where it
/// meets a visible tile off the map, it does.
#[test]
fn the_danger_zone_has_no_edge_at_the_views_edge() {
    let r = Rect::new;
    // The view's top-left tile: only its bottom and right sides.
    assert_eq!(edges(&[(0, 0)]), [r(8, 14, 16, 2), r(22, 0, 2, 16)]);
    // The bottom-right one: only its top and left sides.
    assert_eq!(edges(&[(23, 14)]), [r(376, 224, 16, 2), r(376, 224, 2, 16)]);
    let mut scene = MapScene::new(p(0, 0), (2, 1));
    scene.tint([p(0, 0)], range::DANGER);
    let rects = danger_edges(
        &scene,
        &crate::map_view::glyph::Layout::new(&scene, Rect::new(0, 0, 4, 1)).grid(),
    );
    // Its right neighbour is in the view (off the map): an edge.
    assert_eq!(rects, [r(14, 0, 2, 16)]);
}

/// On the screen the edge is dashes two pixels long, over the ranges and
/// over a unit standing on it.
#[test]
fn the_danger_edge_shows_as_dashes_over_ranges_and_units() {
    let ctx = pd_ctx();
    let mut scene = plains(&ctx);
    scene.tint([p(2, 1), p(3, 1)], range::MOVE);
    scene.tint([p(2, 1), p(3, 1)], range::DANGER);
    scene.push_unit(unit(Faction::Player, p(3, 1)));
    let frame = shown(&painted(&ctx, &scene));
    for x in 40..72 {
        for y in [16, 17] {
            let dash = ink_is_white(Ink::Dashes, x, y);
            assert_eq!(is_white(&frame, x, y), dash, "({x}, {y})");
        }
    }
    // Two on, two off.
    let row: Vec<bool> = (40..48).map(|x| is_white(&frame, x, 16)).collect();
    assert!(
        row == [true, true, false, false, true, true, false, false]
            || row == [false, false, true, true, false, false, true, true]
    );
}

/// The cursor's corner marks are the glyph skin's, in the palette's
/// cursor ink, which flips what it covers; its pulse doesn't show.
#[test]
fn the_cursor_is_the_pcs_corner_marks_flipping_what_they_cover() {
    let ctx = pd_ctx();
    let cursor = |style, brightness| {
        let mut scene = plains(&ctx);
        scene.push_unit(unit(Faction::Player, p(1, 1)));
        scene.cursor = Some(CursorView {
            pos: p(1, 1),
            brightness,
            style,
        });
        let buf = painted(&ctx, &scene);
        let flips: Vec<Rect> = buf
            .overlays()
            .iter()
            .filter(|o| o.color == c(Ink::Invert))
            .map(|o| o.rect)
            .collect();
        (flips, shown(&buf))
    };
    // Tile (1, 1): its left cell is (3, 1).
    let (flips, frame) = cursor(CursorStyle::Corners, 1.0);
    assert_flipped_once(&flips, CORNER_ARM);
    assert_eq!(cursor(CursorStyle::Corners, 0.5).0, flips);
    assert_flipped_once(&cursor(CursorStyle::LargeCorners, 1.0).0, LARGE_CORNER_ARM);
    assert_eq!(
        cursor(CursorStyle::TileGlow, 1.0).0,
        [Rect::new(24, 16, 16, 16)]
    );
    // Its top-right arm, in the unit's black tile, is white; its
    // top-left one, on the white plain beside it, is black.
    assert!(is_white(&frame, 39, 16) && is_white(&frame, 37, 16));
    assert!(!is_white(&frame, 23, 16) && !is_white(&frame, 23, 18));
    // Without the cursor those pixels are as they were.
    let mut scene = plains(&ctx);
    scene.push_unit(unit(Faction::Player, p(1, 1)));
    let plain = shown(&painted(&ctx, &scene));
    assert!(!is_white(&plain, 39, 16) && is_white(&plain, 23, 16));
}

/// `flips` are the glyph skin's corner marks, `arm` pixels long, round
/// tile (1, 1) (its left cell (3, 1)), with every pixel in one arm only:
/// each corner flips once.
fn assert_flipped_once(flips: &[Rect], arm: i32) {
    let pixels = |rects: &[Rect]| {
        let mut all: Vec<(i32, i32)> = rects
            .iter()
            .flat_map(|r| (r.y..r.y + r.h).flat_map(move |y| (r.x..r.x + r.w).map(move |x| (x, y))))
            .collect();
        all.sort_unstable();
        all
    };
    let mut theirs = pixels(&corner_arms(3, 1, arm, HP_BAR_H));
    theirs.dedup();
    assert_eq!(pixels(flips), theirs, "{arm}");
    assert_eq!(flips.len(), 8);
}
