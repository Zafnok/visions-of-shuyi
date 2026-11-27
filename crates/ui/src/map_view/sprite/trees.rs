// engine-friction: 1356 (the kit's sprite skin paints nothing between the ground's pictures and its tint wash, so a tree takes the wash of each tile it covers over its own pixels)
//! Trees by footprint on the sprite skin (ticket 1356,
//! `docs/design/look-and-feel.md`, *Woods: trees by footprint*): each
//! tree of the look's [`Trees`] at its frame for the scene's clock, from
//! its own phase, standing where [`Trees::place`] puts it on the **whole
//! map** (so a tree doesn't move as the view scrolls).
//!
//! **Draw order.** A tree is painted over the ground (its pictures and
//! layers) and under everything else: the path, the units, their marks,
//! the cursor. A crown reaches over the tiles above its trunk, so it must
//! not hide the ranges there: the kit paints its tint wash over each
//! tile's own picture with the ground, and has no step between the two.
//! So a tree takes, over its own pixels, the wash of each tile it covers
//! (the wash's colour and strength, read from what the ground painted),
//! and the ranges' outlines on those tiles are painted again over it:
//! the tree reads as under the wash, as the ground does.
//!
//! Like the moving tiles ([`super::moving`]), a tree stands still on its
//! first frame below the lowest graphics tier it moves at (the Playdate is
//! `Reduced`) and while idle motion is off.

use crate::prelude::*;

use trpg_content::Tileset;
use trpg_content::tileset::Trees;
use trpg_core::{Grid, Pos, TerrainId};

use crate::glyph_buffer::{GlyphBuffer, Item, Layer, Overlay, Paint, PxRect, Rect, Sprite};
use crate::map_view::grid::Grid as TileGrid;
use crate::map_view::scene::MapScene;
use crate::screen::{Ctx, IdleMotion};

use super::{side, src_rect};

/// What the trees are painted with: the frame's context, the tileset, the
/// whole map's terrain (`None`: the scene's own tiles are the map), and
/// the first of the buffer's items the ground painted.
#[derive(Debug, Clone, Copy)]
pub struct TreePainter<'a> {
    /// The context: its graphics tier, idle motion and lights.
    pub ctx: &'a Ctx,
    /// The tileset whose look has the trees.
    pub tileset: &'a Tileset,
    /// The whole map's terrain, if the screen knows it.
    pub map: Option<&'a Grid<TerrainId>>,
    /// The index in the buffer's items of the ground's first.
    pub ground_from: usize,
}

/// The tint wash the ground painted over one tile.
#[derive(Debug, Clone, Copy)]
struct Wash {
    tile: PxRect,
    paint: Paint,
    opacity: u8,
}

impl TreePainter<'_> {
    /// Paints the trees of `scene`'s look in `grid`, over the ground
    /// already in `buf`.
    pub fn paint(&self, scene: &MapScene, grid: &TileGrid, buf: &mut GlyphBuffer) {
        let Some(trees) = self.tileset.trees(&scene.look.tiles) else {
            return;
        };
        let Some(image) = self.tileset.terrain.as_ref().map(|t| t.image) else {
            return;
        };
        let (placed, map_px) = self.placed(trees, scene, grid);
        let view = grid.bounds();
        let shown = map_px.map_or(Some(view), |m| view.intersect(&m));
        let Some(shown) = shown else {
            return;
        };
        let t_ms = match self.ctx.idle_motion {
            IdleMotion::On => Some(scene.clock_ms),
            IdleMotion::Off => None,
        };
        let tier = self.ctx.platform.graphics;
        let paint = match self.ctx.game.content.lighting.get(&scene.look.light) {
            Some(light) if !light.is_plain() => Paint::Shaded(light),
            _ => Paint::Image,
        };
        let (washes, outlines) =
            ground_tints(&buf.items()[self.ground_from.min(buf.items().len())..]);
        let art = grid.art_tile();
        let mut covered: Vec<PxRect> = Vec::new();
        for tree in placed {
            let Some((kind, frame)) = trees.picture(tree, tier, t_ms) else {
                continue;
            };
            let (fx, fy) = (side(kind.foot.0), side(kind.foot.1));
            let at = Rect::new(art.w / 2 - fx, art.h - fy, side(frame.w), side(frame.h));
            let dest = grid.zoomed(grid.tile_px(tree.trunk), at);
            let Some(clip) = dest.intersect(&shown) else {
                continue;
            };
            let mut sprite = Sprite::new(image, src_rect(frame), dest, Layer::Under);
            sprite.clip = clip;
            sprite.paint = paint;
            buf.add_sprite(sprite);
            for wash in &washes {
                if let Some(part) = clip.intersect(&wash.tile) {
                    let mut tinted = sprite;
                    tinted.clip = part;
                    tinted.paint = wash.paint;
                    tinted.opacity = wash.opacity;
                    buf.add_sprite(tinted);
                }
            }
            covered.push(clip);
        }
        for outline in outlines {
            if covered.iter().any(|c| c.intersect(&outline.rect).is_some()) {
                buf.add_overlay(outline);
            }
        }
    }

    /// Where the trees stand, top row first, and the pixels of the whole
    /// map (`None` when the map is the scene's own tiles).
    fn placed(
        &self,
        trees: &Trees,
        scene: &MapScene,
        grid: &TileGrid,
    ) -> (Vec<trpg_content::tileset::Tree>, Option<PxRect>) {
        if let Some(map) = self.map {
            let (w, h) = (i32::from(map.width()), i32::from(map.height()));
            let placed = trees.place(w, h, |p| map.get(p).copied());
            let (x0, y0) = grid.tile_px(Pos::new(0, 0));
            let (x1, y1) = grid.tile_px(Pos::new(w, h));
            (placed, Some(Rect::new(x0, y0, x1 - x0, y1 - y0)))
        } else {
            let o = scene.origin;
            let terrain = |p: Pos| scene.tile(Pos::new(o.x + p.x, o.y + p.y))?.terrain;
            let mut placed = trees.place(scene.size.0, scene.size.1, terrain);
            for tree in &mut placed {
                tree.trunk = Pos::new(o.x + tree.trunk.x, o.y + tree.trunk.y);
            }
            (placed, None)
        }
    }
}

/// The ground's tint washes (each a tile's own picture as a silhouette in
/// one colour) and its outlines (the ranges' edges and relief) among
/// `items`, in the order painted.
fn ground_tints(items: &[Item]) -> (Vec<Wash>, Vec<Overlay>) {
    let mut washes = Vec::new();
    let mut outlines = Vec::new();
    for item in items {
        match item {
            Item::Sprite(s) if matches!(s.paint, Paint::Solid(_)) => washes.push(Wash {
                tile: s.clip,
                paint: s.paint,
                opacity: s.opacity,
            }),
            Item::Rect(o) => outlines.push(*o),
            _ => {}
        }
    }
    (washes, outlines)
}

#[cfg(test)]
mod tests {
    use trpg_content::Tileset;
    use trpg_content::motion::{GraphicsTier, Motion, Show, Still};
    use trpg_content::tileset::{ImageRect, TreeKind, Trees};
    use trpg_core::{Grid, Pos, TerrainId};

    use crate::glyph_buffer::{GlyphBuffer, Item, Paint, PxRect, Rect, Sprite};
    use crate::map_view::MapSkin;
    use crate::map_view::scene::range::MOVE;
    use crate::map_view::scene::{MapScene, RangeLayer};
    use crate::map_view::sprite::src_rect;
    use crate::map_view::sprite::tests::{blank, brigand};
    use crate::screen::tests::ctx;
    use crate::screen::{Ctx, IdleMotion};

    /// The look the test scenes name, which has the trees.
    const LOOK: &str = "rogue_adventure";

    /// `count` frames of `size` px along a row of the test image from
    /// `(left, top)`, 100 ms each, its foot the middle of its bottom edge.
    fn kind((left, top): (u32, u32), (w, h): (u32, u32), count: u32) -> TreeKind {
        let rect = |k: u32| ImageRect {
            x: left + w * k,
            y: top,
            w,
            h,
        };
        TreeKind {
            foot: (w / 2, h),
            frames: (0..count).map(|k| (rect(k), 100)).collect(),
        }
    }

    /// A big tree of 48 × 48 px (3 × 3 tiles) and a half-size one of 16 ×
    /// 32, each twelve frames, on the forest.
    fn trees(c: &Ctx) -> Trees {
        let forest = c.game.content.terrain.display.id_of("forest").unwrap();
        let still = Still {
            show: Show::FirstFrame,
            todo: None,
        };
        let stills = [GraphicsTier::Reduced, GraphicsTier::Minimal].map(|t| (t, still.clone()));
        Trees {
            of: [forest].into_iter().collect(),
            big: vec![kind((0, 200), (48, 48), 12)],
            half: vec![kind((0, 300), (16, 32), 12)],
            motion: Motion {
                lowest: GraphicsTier::Full,
                stills: stills.into_iter().collect(),
            },
        }
    }

    /// The test tileset, its look [`LOOK`] with [`trees`].
    fn skin(c: &Ctx) -> MapSkin {
        let mut tileset: Tileset = c.game.content.tilesets["test_auto"].clone();
        tileset.trees.insert(LOOK.to_owned(), trees(c));
        MapSkin::sprite(tileset)
    }

    fn moving_ctx(idle: IdleMotion, tier: GraphicsTier) -> Ctx {
        let mut c = ctx();
        c.idle_motion = idle;
        c.platform = c.platform.clone().with_graphics(tier);
        c
    }

    /// A map drawn as rows: `#` forest, `.` grass.
    fn map(c: &Ctx, rows: &[&str]) -> Grid<TerrainId> {
        let id = |name| c.game.content.terrain.display.id_of(name).unwrap();
        let cells = rows
            .iter()
            .flat_map(|r| r.chars())
            .map(|ch| if ch == '#' { id("forest") } else { id("plain") })
            .collect();
        let (w, h) = (rows[0].len(), rows.len());
        Grid::from_cells(u16::try_from(w).unwrap(), u16::try_from(h).unwrap(), cells).unwrap()
    }

    /// A view of `size` tiles of `map` from `origin`, `clock_ms` into its
    /// clock.
    fn view(map: &Grid<TerrainId>, origin: Pos, size: (i32, i32), clock_ms: u64) -> MapScene {
        let mut scene = MapScene::new(origin, size);
        scene.set_terrain(|pos| map.get(pos).copied());
        LOOK.clone_into(&mut scene.look.tiles);
        scene.clock_ms = clock_ms;
        scene
    }

    /// The cells of a view of `size` tiles of 16 px (two cells across, one
    /// down) at the buffer's corner.
    const fn area((w, h): (i32, i32)) -> Rect {
        Rect::new(0, 0, w * 2, h)
    }

    fn paint(c: &Ctx, scene: &MapScene, map: Option<&Grid<TerrainId>>) -> GlyphBuffer {
        let mut buf = blank();
        skin(c).paint_on(c, scene, map, area(scene.size), &mut buf);
        buf
    }

    /// The trees painted (not their washes), in order: each one's place
    /// among the buffer's items and its sprite.
    fn tree_sprites(c: &Ctx, buf: &GlyphBuffer) -> Vec<(usize, Sprite)> {
        let t = trees(c);
        let frames: Vec<PxRect> = t
            .big
            .iter()
            .chain(&t.half)
            .flat_map(|k| k.frames.iter().map(|&(r, _)| src_rect(r)))
            .collect();
        let tree = |s: &Sprite| frames.contains(&s.src) && !matches!(s.paint, Paint::Solid(_));
        let items = buf.items().iter().enumerate();
        items
            .filter_map(|(i, item)| match item {
                Item::Sprite(s) if tree(s) => Some((i, *s)),
                _ => None,
            })
            .collect()
    }

    /// The frames of the trees of `map` drawn `clock_ms` into the clock.
    fn frames_at(c: &Ctx, map: &Grid<TerrainId>, clock_ms: u64) -> Vec<PxRect> {
        let scene = view(map, Pos::new(0, 0), (6, 5), clock_ms);
        let drawn = tree_sprites(c, &paint(c, &scene, Some(map)));
        drawn.iter().map(|(_, s)| s.src).collect()
    }

    /// The pixels of tile `pos` of a view from (0, 0).
    const fn tile(pos: Pos) -> PxRect {
        Rect::new(16 * pos.x, 16 * pos.y, 16, 16)
    }

    const WOOD: [&str; 5] = [".....", ".###.", ".###.", ".###.", "....."];
    const WOODS: [&str; 5] = ["######", "######", "######", "##.###", "##.###"];

    #[test]
    fn a_big_tree_stands_on_its_trunk_and_crowns_the_three_by_three_above() {
        let c = moving_ctx(IdleMotion::Off, GraphicsTier::Full);
        let m = map(&c, &WOOD);
        let buf = paint(&c, &view(&m, Pos::new(0, 0), (5, 5), 0), Some(&m));
        let drawn = tree_sprites(&c, &buf);
        assert_eq!(drawn.len(), 1);
        let (at, tree) = drawn[0];
        assert_eq!(tree.dest, Rect::new(16, 16, 48, 48));
        assert_eq!(tree.src, src_rect(trees(&c).big[0].frames[0].0));
        // Over the ground's pictures.
        assert!(
            buf.items()[..at]
                .iter()
                .any(|i| matches!(i, Item::Sprite(_)))
        );
    }

    #[test]
    fn trees_of_one_forest_show_different_frames_and_the_same_on_every_run() {
        let c = moving_ctx(IdleMotion::On, GraphicsTier::Full);
        let m = map(&c, &WOODS);
        for clock_ms in [0, 250, 1_000, 7_777] {
            let frames = frames_at(&c, &m, clock_ms);
            assert!(frames.len() >= 4, "{frames:?}");
            // The two big trees side by side.
            assert_ne!(frames[0], frames[1], "{clock_ms} ms");
            assert_eq!(frames, frames_at(&c, &m, clock_ms), "{clock_ms} ms");
        }
        // And a tree moves: a moment later it shows another frame.
        assert_ne!(frames_at(&c, &m, 0)[0], frames_at(&c, &m, 100)[0]);
    }

    /// The Playdate's tier (`Reduced`) and the lowest draw every tree's
    /// first frame, and so does every tier while idle motion is off.
    #[test]
    fn the_playdates_tier_draws_every_trees_first_frame() {
        let t = trees(&ctx());
        let firsts = [t.big[0].frames[0].0, t.half[0].frames[0].0].map(src_rect);
        let m = map(&ctx(), &WOODS);
        let cases = [
            (IdleMotion::On, GraphicsTier::Reduced),
            (IdleMotion::On, GraphicsTier::Minimal),
            (IdleMotion::Off, GraphicsTier::Full),
        ];
        for (idle, tier) in cases {
            let c = moving_ctx(idle, tier);
            for clock_ms in [0, 150, 1_234, 9_999] {
                let frames = frames_at(&c, &m, clock_ms);
                assert_ne!(frames.len(), 0, "{tier}, {clock_ms} ms");
                for frame in frames {
                    assert!(firsts.contains(&frame), "{tier}, {clock_ms} ms: {frame:?}");
                }
            }
        }
    }

    /// A range on a forest tile under a crown still shows: the tree takes
    /// the tile's tint over its own pixels there, and the range's outline
    /// is painted again over it; a tile without a range gets nothing. A
    /// unit standing there is painted over the tree.
    #[test]
    fn a_range_and_a_unit_on_a_tile_under_a_crown_still_show() {
        let c = moving_ctx(IdleMotion::Off, GraphicsTier::Full);
        let m = map(&c, &WOOD);
        let mut scene = view(&m, Pos::new(0, 0), (5, 5), 0);
        let ranged = Pos::new(2, 1);
        scene.ranges.push(RangeLayer {
            kind: MOVE,
            tiles: vec![ranged],
        });
        scene.push_unit(brigand(ranged));
        let buf = paint(&c, &scene, Some(&m));
        let (at, tree) = tree_sprites(&c, &buf)[0];
        let after = &buf.items()[at + 1..];
        let solid = |s: &Sprite| matches!(s.paint, Paint::Solid(_));
        let washed: Vec<&Sprite> = after
            .iter()
            .filter_map(|i| match i {
                Item::Sprite(s) if s.src == tree.src && solid(s) => Some(s),
                _ => None,
            })
            .collect();
        assert_eq!(washed.len(), 1, "{washed:?}");
        assert_eq!(washed[0].clip, tile(ranged));
        // The wash is the one the ground painted on that tile.
        let ground = buf.items()[..at].iter().find_map(|i| match i {
            Item::Sprite(s) if solid(s) && s.clip == tile(ranged) => Some(s),
            _ => None,
        });
        let ground = ground.expect("the ground tints the tile");
        assert!(ground.opacity > 0);
        let wash = |s: &Sprite| (s.paint, s.opacity);
        assert_eq!(wash(washed[0]), wash(ground));
        // The range's outline the ground drew on the tile, each line again
        // over the tree.
        let inside = |r: PxRect| tile(ranged).intersect(&r) == Some(r);
        let outline: Vec<PxRect> = buf.items()[..at]
            .iter()
            .filter_map(|i| match i {
                Item::Rect(o) if inside(o.rect) => Some(o.rect),
                _ => None,
            })
            .collect();
        assert_ne!(outline.len(), 0, "the ground outlines the range");
        for line in &outline {
            let again = after
                .iter()
                .any(|i| matches!(i, Item::Rect(o) if o.rect == *line));
            assert!(again, "{line:?} isn't painted over the tree");
        }
        // The brigand's picture is painted after the tree.
        let on_tile = |r: PxRect| tile(ranged).intersect(&r).is_some();
        let unit = after
            .iter()
            .any(|i| matches!(i, Item::Sprite(s) if s.image != tree.image && on_tile(s.clip)));
        assert!(unit, "the unit is painted over the tree");
    }

    /// A tree is shaded by the map's light, as the ground is: the image's
    /// own colours in a plain light, the light's at dusk.
    #[test]
    fn a_tree_is_in_the_maps_light() {
        let c = moving_ctx(IdleMotion::Off, GraphicsTier::Full);
        let m = map(&c, &WOOD);
        let mut scene = view(&m, Pos::new(0, 0), (5, 5), 0);
        let paint_of = |scene: &MapScene| tree_sprites(&c, &paint(&c, scene, Some(&m)))[0].1.paint;
        assert_eq!(paint_of(&scene), Paint::Image);
        "dusk".clone_into(&mut scene.look.light);
        let dusk = c.game.content.lighting.get("dusk").expect("the dusk light");
        assert_eq!(paint_of(&scene), Paint::Shaded(dusk));
    }

    /// A tree wider than its tile, or reaching below it (as the bought
    /// ones do, by their shadows), is cut at the map's edges: nothing of
    /// it is drawn off the map.
    #[test]
    fn a_tree_is_cut_at_the_maps_edges() {
        let c = moving_ctx(IdleMotion::Off, GraphicsTier::Full);
        let m = map(&c, &["#..", "..#"]);
        let mut tileset: Tileset = c.game.content.tilesets["test_auto"].clone();
        let mut wide = trees(&c);
        wide.half = vec![TreeKind {
            foot: (16, 28),
            ..kind((0, 300), (32, 32), 2)
        }];
        tileset.trees.insert(LOOK.to_owned(), wide);
        // The map in the middle of a view a tile bigger all round.
        let scene = view(&m, Pos::new(-1, -1), (5, 4), 0);
        let mut buf = blank();
        MapSkin::sprite(tileset).paint_on(&c, &scene, Some(&m), area(scene.size), &mut buf);
        let clips: Vec<PxRect> = buf
            .items()
            .iter()
            .filter_map(|i| match i {
                Item::Sprite(s) if s.src.w == 32 && s.src.y == 300 => Some(s.clip),
                _ => None,
            })
            .collect();
        // The map's pixels: (16, 16) to (64, 48). The top-left tree, drawn
        // from (8, 4), stops at the map's left and top; the bottom-right
        // one, drawn to (72, 52), at its right and bottom.
        assert_eq!(
            clips,
            [Rect::new(16, 16, 24, 20), Rect::new(40, 20, 24, 28)]
        );
    }

    /// Where the trees stand is the whole map's: a view that shows only
    /// the lower two rows of a 3 × 3 wood still shows its big tree, as it
    /// stands on the map, where the scene alone would have half-size ones.
    #[test]
    fn trees_stand_where_the_whole_map_has_them_however_the_view_scrolls() {
        let c = moving_ctx(IdleMotion::Off, GraphicsTier::Full);
        let m = map(&c, &["###..", "###..", "###..", ".....", "....."]);
        let scrolled = view(&m, Pos::new(0, 1), (5, 4), 0);
        let on_map = tree_sprites(&c, &paint(&c, &scrolled, Some(&m)));
        assert_eq!(on_map.len(), 1);
        // The trunk on (1, 2), a tile down the view: the crown cut at the
        // view's top.
        assert_eq!(on_map[0].1.dest, Rect::new(0, -16, 48, 48));
        assert_eq!(on_map[0].1.clip, Rect::new(0, 0, 48, 32));
        // The scene alone sees two rows of forest: a half-size tree on each
        // of its lower row's three tiles (map row 2), its crown above.
        let alone = tree_sprites(&c, &paint(&c, &scrolled, None));
        let dests: Vec<PxRect> = alone.iter().map(|(_, s)| s.dest).collect();
        let half = |x| Rect::new(x, 0, 16, 32);
        assert_eq!(dests, [half(0), half(16), half(32)]);
    }
}
