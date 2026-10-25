//! The sprite skin (ADR-0038, ADR-0049, ADR-0052): the battle map painted
//! from a tileset (`assets/tilesets/<id>.ron`, `trpg_content::tileset`):
//! the tactics kit's sprite skin (`shuyi_ui::grid::sprite`, ticket 1115)
//! over the tileset's terrain tiles, with this game's units as the
//! tileset's pictures ([`units`]).
//!
//! - A tileset **with terrain tiles** paints the whole map: a tile is the
//!   tileset's `tile_px` times the zoom, a size known only to the kit's
//!   skin (its `tile_size`). The ground is each tile's own picture and
//!   the tileset's layers over them, in the look the map asks for. The
//!   cursor is corner marks with a dark edge, inside its tile (ticket
//!   0444). A unit a battle note picks out stands on a light square
//!   (Claude's placeholder: nothing has decided it).
//! - A tileset **without** them paints only the units: terrain, ranges,
//!   the path and the cursor are the glyph skin's, on its tiles, and each
//!   unit's tile loses its terrain glyphs under the picture.
//!
//! Units look the same either way: see [`units`]. Over them, the arrows
//! at the edges the map goes on past ([`edges`]): the
//! pictures of [`EDGE_MARKS_PATH`] with terrain tiles, the glyph skin's
//! arrows on glyph terrain.

pub mod units;

use alloc::rc::Rc;

use shuyi_ui::grid::skin::UnitPainter;
use shuyi_ui::grid::sprite::SpriteSkin;
use trpg_content::Tileset;
use trpg_content::image::EDGE_MARKS_PATH;

use super::edges::{self, Side};
use super::glyph::units::HP_BAR_H;
use super::glyph::{GLYPH, GlyphUnits, cell_of};
use super::grid::Grid;
use super::scene::{MapScene, UnitView};
use super::skin::{MapSkin, Units};
use crate::color::{UiColor, to_channel};
use crate::glyph_buffer::{GlyphBuffer, Layer, Overlay, PxRect, Rect, Sprite};
use crate::screen::Ctx;
use crate::screens::battle::walk::SPRITE_WALK;
use crate::settings::MapZoom;

pub use shuyi_ui::grid::sprite::{side, src_rect, tile_sprite};

impl MapSkin {
    /// The battle map painted from `tileset`, at the art's own size: named
    /// `"sprite"` when the tileset paints the whole map (and then it zooms
    /// to every [`MapZoom`]), `"sprite_units"` when only the units. Its
    /// walk is slower than the glyph skin's: a sprite is seen to walk.
    pub fn sprite(tileset: Tileset) -> Self {
        let zooms = MapZoom::ALL.map(MapZoom::factor);
        let grid = SpriteSkin::new(tileset.terrain.clone(), GLYPH, SPRITE_WALK).with_zooms(&zooms);
        Self::new(Rc::new(grid), Units::Sprite(Rc::new(tileset)))
    }
}

/// This game's units as the sprite skin shows them: under them, a light
/// square for a unit a battle note picks out (with terrain tiles), or the
/// terrain's glyphs taken off a standing unit's tile (on glyph terrain);
/// then their pictures, and their HP bars and marks ([`units`]).
#[derive(Debug, Clone, Copy)]
pub struct SpriteUnits<'a> {
    /// The context: its palette, its images, the lead.
    pub ctx: &'a Ctx,
    /// The tileset the pictures come from.
    pub tileset: &'a Tileset,
}

impl UnitPainter<UnitView> for SpriteUnits<'_> {
    fn paint_under(&self, scene: &MapScene, grid: &Grid, buf: &mut GlyphBuffer) {
        if self.tileset.terrain.is_some() {
            let light = self.ctx.palette.get(UiColor::Text);
            for unit in scene.units.iter().filter(|u| u.highlight) {
                if let Some(tile) = grid.rect(unit.pos) {
                    buf.add_overlay(Overlay::new(tile, light, Layer::Under));
                }
            }
        } else {
            // A unit between two tiles walks over the glyphs of both, so
            // the ground it leaves isn't blank behind it.
            for unit in scene.units.iter().filter(|u| !u.between_tiles()) {
                if let Some((x, y)) = cell_of(grid, unit.pos) {
                    super::glyph::units::clear_glyphs(buf, x, y, unit.fade, unit.highlight);
                }
            }
        }
    }

    fn paint(&self, scene: &MapScene, grid: &Grid, buf: &mut GlyphBuffer) {
        units::paint_pictures(self.ctx, self.tileset, scene, grid, buf);
        units::paint_marks(self.ctx, scene, grid, buf);
    }

    fn paint_over(&self, scene: &MapScene, grid: &Grid, buf: &mut GlyphBuffer) {
        if self.tileset.terrain.is_none() {
            GlyphUnits {
                palette: &self.ctx.palette,
            }
            .paint_over(scene, grid, buf);
            return;
        }
        let Some(image) = self.ctx.game.content.images.id(EDGE_MARKS_PATH) else {
            return;
        };
        let opacity = to_channel(255.0 * edges::brightness(scene.clock_ms));
        for (side, tile) in edges::arrows(scene) {
            let Some(at) = grid.rect(tile) else {
                continue;
            };
            let dest = grid.zoomed((at.x, at.y), edge_dest(side, grid.art_tile()));
            let mut sprite = Sprite::new(image, edge_src(side), dest, Layer::Over);
            sprite.opacity = opacity;
            if let Some(clip) = dest.intersect(&grid.bounds()) {
                sprite.clip = clip;
                buf.add_sprite(sprite);
            }
        }
    }
}

/// How far an edge arrow's box sits in from the view's edge, in pixels of
/// the art.
pub const EDGE_GAP: i32 = 1;

/// The box of the `side` edge's arrow in [`EDGE_MARKS_PATH`]: the right
/// and the left arrow 8 × 13, then the down and the up arrow 13 × 8.
pub const fn edge_src(side: Side) -> PxRect {
    match side {
        Side::Right => Rect::new(0, 0, 8, 13),
        Side::Left => Rect::new(8, 0, 8, 13),
        Side::Down => Rect::new(16, 0, 13, 8),
        Side::Up => Rect::new(29, 0, 13, 8),
    }
}

/// Where the `side` edge's arrow goes on its tile `tile` (at the art's
/// size): [`EDGE_GAP`] in from the view's edge, centred along it, and
/// above the HP bar, so it never covers a unit's bar.
pub fn edge_dest(side: Side, tile: PxRect) -> PxRect {
    let src = edge_src(side);
    let (w, h) = (src.w, src.h);
    let across = tile.x + (tile.w - w) / 2;
    let down = tile.y + (tile.h - HP_BAR_H - h) / 2;
    let (x, y) = match side {
        Side::Right => (tile.x + tile.w - EDGE_GAP - w, down),
        Side::Left => (tile.x + EDGE_GAP, down),
        Side::Down => (across, tile.y + tile.h - HP_BAR_H - h),
        Side::Up => (across, tile.y + EDGE_GAP),
    };
    Rect::new(x, y, w, h)
}

#[cfg(test)]
pub(crate) mod tests {
    use std::collections::BTreeSet;

    use insta::assert_snapshot;
    use proptest::prelude::*;
    use trpg_content::ImageRect;
    use trpg_content::tileset::{CornerLayer, Layer as TileLayerKind, Look, MIXES, TileLayer};
    use trpg_core::{CharacterId, ClassId, Faction, Pos, TerrainId, UnitId};

    use super::*;
    use crate::color::Rgb;
    use crate::glyph_buffer::{Cell, Item, PxRect, Rect, Sprite};
    use crate::map_view::glyph::tests::any_scene;
    use crate::map_view::grid::px_rect;
    use crate::map_view::scene::{
        CursorStyle, CursorView, Facing, STANDING_FRAME, ThreatView, TileView, UnitEffects,
        UnitView, range,
    };
    use crate::map_view::skin::with_looks;
    use crate::map_view::threat;
    use crate::screen::tests::ctx;
    use crate::screens::battle::layout::MAP_VIEW;
    use shuyi_ui::grid::{Ends, MapEdges};

    /// The picture of mix `m` in a test layer: one rectangle of the test
    /// tileset's image for each.
    fn mix_rect(m: u8) -> ImageRect {
        ImageRect {
            x: 24 * u32::from(m % 8),
            y: 96 + 24 * u32::from(m / 8),
            w: 24,
            h: 24,
        }
    }

    /// The test tileset's skin with layers: a shore between the water (and
    /// the sea and the bridge) and the rest, a picture for every mix; then
    /// a bridge that has another picture with water to its left and
    /// right.
    pub(crate) fn layered(c: &Ctx) -> MapSkin {
        let id = |name| c.game.content.terrain.display.id_of(name).unwrap();
        let mut tileset = c.game.content.tilesets["test"].clone();
        let terrain = tileset.terrain.as_mut().unwrap();
        let mut tiles = [None; MIXES];
        for m in 1..16 {
            tiles[usize::from(m)] = Some(mix_rect(m));
        }
        let shore = CornerLayer {
            of: BTreeSet::from([id("water"), id("sea"), id("bridge")]),
            tiles,
        };
        let mut sides = [None; MIXES];
        sides[0b0101] = Some(mix_rect(0));
        let bridge = TileLayer {
            of: BTreeSet::from([id("bridge")]),
            tile: mix_rect(15),
            beside: BTreeSet::from([id("water")]),
            sides,
        };
        terrain.look.layers = vec![TileLayerKind::Corners(shore), TileLayerKind::Tiles(bridge)];
        MapSkin::sprite(tileset)
    }

    /// `skin` with an indoor look: every terrain's tile another one of
    /// the image's, and no layers.
    pub(crate) fn with_indoor(skin: &MapSkin) -> MapSkin {
        let mut tileset = skin.tileset().unwrap().clone();
        let terrain = tileset.terrain.as_mut().unwrap();
        let mut indoor = Look::default();
        for (&id, rect) in &terrain.look.tiles {
            indoor.tiles.insert(id, ImageRect { y: 72, ..*rect });
        }
        terrain.looks.insert("indoor".to_owned(), indoor);
        MapSkin::sprite(tileset)
    }

    pub(crate) fn p(x: i32, y: i32) -> Pos {
        Pos::new(x, y)
    }

    /// The skin of the test tileset (24 × 24 tiles).
    pub(crate) fn skin(c: &Ctx) -> MapSkin {
        MapSkin::sprite(c.game.content.tilesets["test"].clone())
    }

    /// The skin of the test unit sheets: 16 × 20 units on glyph terrain.
    pub(crate) fn sheets(c: &Ctx) -> MapSkin {
        MapSkin::sprite(c.game.content.tilesets["test_units"].clone())
    }

    /// [`skin`] with `w × h` tiles.
    fn sized(c: &Ctx, w: u32, h: u32) -> MapSkin {
        let mut tileset = c.game.content.tilesets["test"].clone();
        if let Some(terrain) = &mut tileset.terrain {
            terrain.tile_px = (w, h);
        }
        MapSkin::sprite(tileset)
    }

    /// `skin` drawn at `zoom`, or as it is if it can't zoom (only the
    /// units painted from the tileset).
    pub(crate) fn zoomed(skin: MapSkin, zoom: MapZoom) -> MapSkin {
        skin.at_zoom(zoom).unwrap_or(skin)
    }

    /// `skin` with a picture for the test lord: the fallback's, moved to
    /// the left edge of its image.
    pub(crate) fn with_lord(skin: &MapSkin) -> MapSkin {
        let mut tileset = skin.tileset().unwrap().clone();
        let mut lord = tileset.fallback;
        lord.rect.x = 0;
        tileset
            .characters
            .insert(CharacterId("test_lord".into()), lord);
        MapSkin::sprite(tileset)
    }

    /// `skin` with the brigand's picture in a walking sheet: the test unit
    /// sheets' (the test tileset's own pictures don't walk).
    fn with_walking_brigand(c: &Ctx, skin: &MapSkin) -> MapSkin {
        let sheets = &c.game.content.tilesets["test_units"];
        let brigand = ClassId("brigand".into());
        let picture = sheets.classes[&brigand];
        let mut tileset = skin.tileset().unwrap().clone();
        tileset.classes.insert(brigand, picture);
        tileset.walking.insert(picture.image);
        MapSkin::sprite(tileset)
    }

    /// A buffer the size of the console, every cell `fill()`.
    pub(crate) fn blank() -> GlyphBuffer {
        GlyphBuffer::new(100, 32, fill())
    }

    pub(crate) fn fill() -> Cell {
        Cell::new('x', Rgb::new(1, 2, 3), Rgb::new(4, 5, 6))
    }

    pub(crate) fn brigand(pos: Pos) -> UnitView {
        UnitView {
            id: UnitId(4),
            pos,
            faction: Faction::Enemy,
            label: "Br".into(),
            class: ClassId("brigand".into()),
            character: None,
            acted: false,
            fight: crate::map_view::scene::Fight::In,
            hp: (30, 30),
            effects: UnitEffects::default(),
            fade: 0.0,
            highlight: false,
            facing: Facing::Down,
            frame: STANDING_FRAME,
            offset: (0.0, 0.0),
            flash: 0.0,
            marks: true,
        }
    }

    fn threat(tile: Pos, enemies: &[Pos]) -> ThreatView {
        ThreatView {
            tile,
            enemies: enemies.to_vec(),
        }
    }

    /// The threat arrows and marks are the last things painted, over the
    /// units and the cursor, on the skin's own tiles at its zoom: under a
    /// tileset with terrain tiles and under one without.
    #[test]
    fn threat_arrows_and_marks_are_painted_last_on_the_skins_tiles() {
        let c = ctx();
        let colors = with_looks(&c, |looks| {
            looks.threat(&crate::map_view::glyph::MAP_COLORS)
        });
        // 192 × 128 px: the whole 3 × 2 view even at twice the size.
        let area = Rect::new(2, 1, 24, 8);
        let skins = [
            skin(&c),
            zoomed(skin(&c), MapZoom::X2),
            sheets(&c),
            zoomed(sheets(&c), MapZoom::X2),
        ];
        for skin in skins {
            let mut scene = plains(&c);
            scene.push_unit(brigand(p(1, 1)));
            scene.cursor = Some(CursorView::new(p(1, 1), 1.0, CursorStyle::Corners));
            let mut buf = blank();
            skin.paint(&c, &scene, area, &mut buf);
            let before = buf.items().len();
            scene.threat = Some(threat(p(0, 0), &[p(1, 1)]));
            let mut buf = blank();
            skin.paint(&c, &scene, area, &mut buf);
            let grid = skin.grid(&scene, area);
            let arrows = threat(p(0, 0), &[p(1, 1)]);
            let expect: Vec<Item> =
                threat::threat_overlays(&arrows, &grid, threat::Start::Middle, colors)
                    .into_iter()
                    .map(Item::Rect)
                    .collect();
            assert!(expect.len() > 4, "{}", skin.name());
            assert_eq!(&buf.items()[before..], expect, "{}", skin.name());
            // The mark is over the brigand's tile, in the highlight colour.
            let tile = grid.rect(p(1, 1)).unwrap();
            let marks: Vec<_> = buf
                .overlays()
                .into_iter()
                .filter(|o| o.color == colors.mark)
                .collect();
            assert_eq!(marks.len(), 2, "{}", skin.name());
            for mark in marks {
                let r = mark.rect;
                assert!(r.x > tile.x && r.x + r.w < tile.x + tile.w, "{r:?}");
                assert!(r.y + r.h <= tile.y, "{r:?}");
            }
        }
    }

    /// A 3 × 2 view from (0, 0): plains, but a forest at (1, 0).
    pub(crate) fn plains(c: &Ctx) -> MapScene {
        let display = &c.game.content.terrain.display;
        let mut scene = MapScene::new(p(0, 0), (3, 2));
        for tile in &mut scene.tiles {
            tile.terrain = display.id_of("plain");
        }
        scene.tile_mut(p(1, 0)).unwrap().terrain = display.id_of("forest");
        scene
    }

    /// `scene` painted by `skin` into a blank console, in the 6 × 4 cells
    /// from (2, 1): 48 × 64 px from (16, 16), so 2 × 2 tiles of 24 px with
    /// 8 px to spare above and below.
    pub(crate) fn painted(c: &Ctx, skin: &MapSkin, scene: &MapScene) -> GlyphBuffer {
        let mut buf = blank();
        skin.paint(c, scene, Rect::new(2, 1, 6, 4), &mut buf);
        buf
    }

    /// The item at `i` as a rectangle.
    pub(crate) fn rect_at(buf: &GlyphBuffer, i: usize) -> Overlay {
        match buf.items()[i] {
            Item::Rect(o) => o,
            Item::Sprite(s) => panic!("item {i} is a sprite: {s:?}"),
        }
    }

    #[test]
    fn view_tiles_and_the_margin_follow_the_tile_size() {
        let c = ctx();
        let s = skin(&c);
        assert_eq!((s.name(), s.tile_size()), ("sprite", (24, 24)));
        assert_eq!(s.tileset().unwrap().id, "test");
        // The map area is 560 × 480 px.
        let fit = |w, h, area| {
            let s = sized(&c, w, h);
            let scene = MapScene::new(p(0, 0), s.view_tiles(area));
            let g = s.grid(&scene, area);
            (g.tiles, g.corner)
        };
        // 552 × 480 px of 24 px tiles: 4 px to spare each side.
        assert_eq!(fit(24, 24, MAP_VIEW), ((23, 20), (4, 0)));
        assert_eq!(fit(16, 16, MAP_VIEW), ((35, 30), (0, 0)));
        assert_eq!(fit(64, 64, MAP_VIEW), ((8, 7), (24, 16)));
        // 9 × 13: 2 px to spare across, 12 down.
        assert_eq!(fit(9, 13, MAP_VIEW), ((62, 36), (1, 6)));
        // One cell: no tile across; the odd pixel goes right.
        assert_eq!(fit(9, 13, Rect::new(0, 0, 1, 1)), ((0, 1), (4, 1)));
        // An area away from the corner: 56 × 32 px from (32, 48).
        assert_eq!(fit(24, 24, Rect::new(4, 3, 7, 2)), ((2, 1), (36, 52)));
        // An empty area shows nothing.
        let empty = Rect::new(3, 3, -4, -1);
        assert_eq!(s.view_tiles(empty), (0, 0));
        assert_eq!(s.grid(&MapScene::new(p(0, 0), (5, 5)), empty).tiles, (0, 0));
        // Where the skin has a tile, for menus beside it.
        let scene = MapScene::new(p(-3, 2), s.view_tiles(MAP_VIEW));
        let px = |tile| s.tile_px(&scene, MAP_VIEW, tile);
        assert_eq!(px(p(-3, 2)), Some(Rect::new(4, 0, 24, 24)));
        assert_eq!(px(p(0, 3)), Some(Rect::new(76, 24, 24, 24)));
        assert_eq!(px(p(20, 2)), None);
        let cells = s.tile_cells(&scene, MAP_VIEW, p(0, 3));
        assert_eq!(cells, Some(Rect::new(9, 1, 4, 2)));
        // A scene smaller than the area: only its tiles.
        let small = s.grid(&MapScene::new(p(0, 0), (2, 1)), MAP_VIEW);
        assert_eq!(small.tiles, (2, 1));
    }

    #[test]
    fn the_zoom_multiplies_the_tile_and_divides_the_view() {
        let c = ctx();
        let s = skin(&c);
        assert_eq!(s.zoom(), Some(MapZoom::X1));
        let twice = zoomed(s.clone(), MapZoom::X2);
        assert_eq!(
            (twice.zoom(), twice.tile_size()),
            (Some(MapZoom::X2), (48, 48))
        );
        assert_eq!((twice.name(), twice.tileset_id()), ("sprite", Some("test")));
        // 560 × 480 px of 48 px tiles: 11 × 10, 16 px to spare each side.
        assert_eq!(twice.view_tiles(MAP_VIEW), (11, 10));
        let scene = MapScene::new(p(0, 0), (11, 10));
        let grid = twice.grid(&scene, MAP_VIEW);
        assert_eq!((grid.corner, grid.tile, grid.scale), ((16, 0), (48, 48), 2));
        assert_eq!(s.grid(&scene, MAP_VIEW).scale, 1);
        assert_eq!(
            twice.tile_px(&scene, MAP_VIEW, p(2, 1)),
            Some(Rect::new(112, 48, 48, 48))
        );
        // A menu beside a tile: the cells it touches (8 × 16 px each).
        assert_eq!(
            twice.tile_cells(&scene, MAP_VIEW, p(2, 1)),
            Some(Rect::new(14, 3, 6, 3))
        );
        // 16 px tiles, the bought art's size: 35 × 30 and 17 × 15.
        let small = sized(&c, 16, 16);
        assert_eq!(small.view_tiles(MAP_VIEW), (35, 30));
        assert_eq!(zoomed(small, MapZoom::X2).view_tiles(MAP_VIEW), (17, 15));
        // The trait's way: the same skin at another zoom, and back.
        let dynamic = s.at_zoom(MapZoom::X2).unwrap();
        assert_eq!(dynamic.zoom(), Some(MapZoom::X2));
        assert_eq!(dynamic.view_tiles(MAP_VIEW), (11, 10));
        let back = dynamic.at_zoom(MapZoom::X1).unwrap();
        assert_eq!(
            (back.zoom(), back.view_tiles(MAP_VIEW)),
            (s.zoom(), (23, 20))
        );
        // Unit pictures on glyph terrain stay on font cells: no zoom.
        let mixed = sheets(&c);
        assert_eq!(mixed.zoom(), None);
        assert!(mixed.at_zoom(MapZoom::X2).is_none());
        let forced = zoomed(mixed.clone(), MapZoom::X2);
        assert_eq!((forced.zoom(), forced.tile_size()), (None, (16, 16)));
        assert_eq!(forced.view_tiles(MAP_VIEW), mixed.view_tiles(MAP_VIEW));
        assert_eq!(forced.grid(&scene, MAP_VIEW).scale, 1);
        // The glyph skin never zooms.
        assert_eq!(MapSkin::glyph().zoom(), None);
        assert!(MapSkin::glyph().at_zoom(MapZoom::X2).is_none());
    }

    /// A scene with one of everything, painted at 1× into 2 × 2 tiles from
    /// the console's corner and at 2× into an area twice the size: the
    /// same items in the same order, every one twice as far from the
    /// corner and twice as big (tiles, units and their outlines, HP bars,
    /// effect arrows, the path and the ranges), each showing the same part
    /// of the same image. (Not the cursor: its marks have a size for each
    /// zoom.)
    #[test]
    fn at_twice_the_zoom_everything_is_twice_the_size() {
        let c = ctx();
        let s = with_walking_brigand(&c, &with_lord(&skin(&c)));
        let twice = zoomed(s.clone(), MapZoom::X2);
        let double = |r: PxRect| Rect::new(2 * r.x, 2 * r.y, 2 * r.w, 2 * r.h);
        for clock_ms in [0, units::BOUNCE_MS, units::TURN_MS] {
            let mut scene = MapScene::new(p(0, 0), (2, 2));
            scene.set_terrain(|_| c.game.content.terrain.display.id_of("plain"));
            scene.clock_ms = clock_ms;
            scene.tint([p(0, 0)], range::MOVE);
            scene.tile_mut(p(1, 0)).unwrap().flashes.push(0.5);
            let mut lord = brigand(p(0, 1));
            lord.faction = Faction::Player;
            lord.character = Some(CharacterId("test_lord".into()));
            lord.hp = (5, 30);
            lord.effects.bonus = true;
            lord.highlight = true;
            scene.push_unit(lord);
            // Half a tile into its walk, under the unit above it.
            let mut enemy = brigand(p(1, 1));
            enemy.acted = true;
            enemy.effects = UnitEffects {
                bonus: true,
                penalty: true,
            };
            enemy.offset = (-0.5, 0.0);
            enemy.facing = Facing::Left;
            enemy.frame = 0;
            scene.push_unit(enemy);
            scene.push_unit(brigand(p(1, 0)).fading(0.25));
            scene.path = vec![p(0, 0), p(0, 1), p(1, 1)];
            scene.more = every_edge();
            let paint = |skin: &MapSkin, area| {
                let mut buf = blank();
                skin.paint(&c, &scene, area, &mut buf);
                buf
            };
            // 48 × 48 px, and 96 × 96 px.
            let small = paint(&s, Rect::new(0, 0, 6, 3));
            let big = paint(&twice, Rect::new(0, 0, 12, 6));
            assert!(small.items().len() > 20, "{}", small.items().len());
            assert_eq!(small.items().len(), big.items().len());
            for (one, two) in small.items().iter().zip(big.items()) {
                let expected = match *one {
                    Item::Rect(o) => Item::Rect(Overlay {
                        rect: double(o.rect),
                        ..o
                    }),
                    Item::Sprite(sprite) => Item::Sprite(Sprite {
                        dest: double(sprite.dest),
                        clip: double(sprite.clip),
                        base: sprite.base.map(double),
                        ..sprite
                    }),
                };
                assert_eq!(*two, expected, "at {clock_ms} ms");
            }
        }
    }

    #[test]
    fn a_tile_is_its_terrain_sprite() {
        let c = ctx();
        let s = skin(&c);
        let buf = painted(&c, &s, &plains(&c));
        let display = &c.game.content.terrain.display;
        let tile = |name| {
            src_rect(
                s.tileset()
                    .unwrap()
                    .tile(display.id_of(name).unwrap())
                    .unwrap()
                    .rect,
            )
        };
        let sprites = buf.sprites();
        // 2 × 2 tiles fit, from (16, 24).
        assert_eq!(sprites.len(), 4);
        let at = |x, y| Rect::new(x, y, 24, 24);
        let expect = [
            (tile("plain"), at(16, 24)),
            (tile("forest"), at(40, 24)),
            (tile("plain"), at(16, 48)),
            (tile("plain"), at(40, 48)),
        ];
        for (sprite, (src, dest)) in sprites.iter().zip(expect) {
            assert_eq!((sprite.src, sprite.dest, sprite.clip), (src, dest, dest));
            assert_eq!(sprite.image.path(), "tilesets/test.png");
            let look = (sprite.layer, sprite.opacity, sprite.flip_x);
            assert_eq!(look, (Layer::Under, 255, false));
        }
        assert_ne!(tile("plain"), tile("forest"));
        // No cell changes.
        let cells = |b: &GlyphBuffer| {
            let all = (0..32).flat_map(|y| (0..100).map(move |x| (x, y)));
            all.map(|(x, y)| *b.get(x, y).unwrap()).collect::<Vec<_>>()
        };
        assert_eq!(cells(&buf), cells(&blank()));
        // Tiles off the map, or of a terrain the tileset lacks: nothing.
        let mut scene = plains(&c);
        scene.tiles[0].terrain = None;
        scene.tiles[1].terrain = Some(TerrainId(999));
        assert_eq!(painted(&c, &s, &scene).sprites().len(), 2);
    }

    /// Ticket 0444: thick marks with a dark edge at the art's own size,
    /// thin ones with a dark edge zoomed in; a pixel longer as large
    /// corners; the glow style is the corners.
    #[test]
    fn the_cursor_is_dark_edged_corner_marks_inside_its_tile() {
        let c = ctx();
        let s = skin(&c);
        let cursor = c.palette.get(UiColor::Cursor);
        let black = c.palette.get(UiColor::Black);
        let painted_with = |skin: &MapSkin, pos, style, brightness| {
            let mut scene = plains(&c);
            scene.cursor = Some(CursorView::new(pos, brightness, style));
            painted(&c, skin, &scene)
        };
        let buf = painted_with(&s, p(1, 1), CursorStyle::Corners, 1.0);
        let r = Rect::new;
        let rects = |buf: &GlyphBuffer| buf.overlays().iter().map(|o| o.rect).collect::<Vec<_>>();
        // Tile (1, 1): pixels 40..64 × 48..72. The dark edges, then the
        // marks a pixel inside them.
        assert_eq!(
            rects(&buf),
            [
                r(40, 48, 6, 4),
                r(40, 48, 4, 6),
                r(58, 48, 6, 4),
                r(60, 48, 4, 6),
                r(40, 68, 6, 4),
                r(40, 66, 4, 6),
                r(58, 68, 6, 4),
                r(60, 66, 4, 6),
                r(41, 49, 4, 2),
                r(41, 49, 2, 4),
                r(59, 49, 4, 2),
                r(61, 49, 2, 4),
                r(41, 69, 4, 2),
                r(41, 67, 2, 4),
                r(59, 69, 4, 2),
                r(61, 67, 2, 4),
            ]
        );
        let looks = |buf: &GlyphBuffer, range: std::ops::Range<usize>, color| {
            let look = |o: &Overlay| (o.color, o.layer);
            buf.overlays()[range]
                .iter()
                .all(|o| look(o) == (color, Layer::Over))
        };
        assert!(looks(&buf, 0..8, black));
        assert!(looks(&buf, 8..16, cursor));
        // Large corners: a pixel longer. The marks pulse; the edge doesn't.
        let large = painted_with(&s, p(1, 1), CursorStyle::LargeCorners, 0.5);
        assert_eq!(large.overlays()[2].rect, r(57, 48, 7, 4));
        assert_eq!(large.overlays()[8].rect, r(41, 49, 5, 2));
        assert!(looks(&large, 0..8, black));
        assert!(looks(&large, 8..16, cursor.scale(0.5)));
        // The glow style: the corners, and nothing tints the tile.
        let glow = painted_with(&s, p(1, 1), CursorStyle::TileGlow, 1.0);
        assert_eq!(glow, buf);
        // At 2×, one 48 px tile from (16, 24): thin marks, 1 px of the
        // art thick and 3 long, their edge 1 px of the art.
        let twice = zoomed(s.clone(), MapZoom::X2);
        let zoomed = painted_with(&twice, p(0, 0), CursorStyle::Corners, 1.0);
        let all = rects(&zoomed);
        assert_eq!(all.len(), 16);
        assert_eq!((all[0], all[1]), (r(16, 24, 10, 6), r(16, 24, 6, 10)));
        assert_eq!((all[8], all[9]), (r(18, 26, 6, 2), r(18, 26, 2, 6)));
        assert_eq!((all[7], all[15]), (r(58, 62, 6, 10), r(60, 64, 2, 6)));
        let zoomed = painted_with(&twice, p(0, 0), CursorStyle::LargeCorners, 1.0);
        let all = rects(&zoomed);
        assert_eq!((all[0], all[8]), (r(16, 24, 12, 6), r(18, 26, 8, 2)));
        // A cursor off the view: nothing.
        let off = painted_with(&s, p(2, 0), CursorStyle::Corners, 1.0);
        assert_eq!(off, painted(&c, &s, &plains(&c)));
    }

    #[test]
    fn the_path_runs_under_units_and_its_arrowhead_over_them() {
        let c = ctx();
        let s = skin(&c);
        let mut scene = plains(&c);
        scene.push_unit(brigand(p(1, 1)));
        scene.path = vec![p(0, 1), p(1, 1)];
        let buf = painted(&c, &s, &scene);
        let path = c.palette.get(UiColor::Path);
        let kinds: Vec<&str> = buf
            .items()
            .iter()
            .map(|i| match i {
                Item::Sprite(_) => "sprite",
                Item::Rect(o) if o.color == path => "path",
                Item::Rect(_) => "mark",
            })
            .collect();
        // Four tiles; the line; the unit's outline (four) and picture;
        // its HP bar (full: one rectangle); the arrowhead.
        let mut expect = vec!["sprite"; 4];
        expect.push("path");
        expect.extend(["sprite"; 5]);
        expect.push("mark");
        expect.extend(["path"; 6]);
        assert_eq!(kinds, expect);
        // Through the centres of 24 px tiles: pixels 11..=13 of each.
        let line = rect_at(&buf, 4);
        assert_eq!(
            (line.rect, line.layer),
            (Rect::new(40, 59, 14, 3), Layer::Under)
        );
    }

    /// A small scene with one of everything, painted: the frame's items.
    #[test]
    fn sprite_skin_snapshot() {
        let c = ctx();
        let s = skin(&c);
        let mut scene = plains(&c);
        scene.tint([p(0, 0)], range::MOVE);
        scene.tile_mut(p(1, 0)).unwrap().flashes.push(0.5);
        let mut lord = brigand(p(0, 0));
        lord.faction = Faction::Player;
        lord.class = ClassId("exile".into());
        lord.hp = (5, 30);
        scene.push_unit(lord);
        let mut enemy = brigand(p(1, 1));
        enemy.acted = true;
        enemy.effects.penalty = true;
        scene.push_unit(enemy);
        scene.path = vec![p(0, 0), p(0, 1)];
        scene.cursor = Some(CursorView::new(p(0, 1), 1.0, CursorStyle::Corners));
        let text = painted(&c, &s, &scene).to_snapshot(&c.palette);
        let items = text.split("--- overlays ---\n").nth(1).unwrap_or_default();
        assert_snapshot!(items);
    }

    /// The map goes on past every side.
    fn every_edge() -> MapEdges {
        let both = Ends {
            before: true,
            after: true,
        };
        MapEdges { x: both, y: both }
    }

    /// The edge arrows a skin paints for `scene` in `area`: the sprites of
    /// [`EDGE_MARKS_PATH`].
    fn edge_arrows(c: &Ctx, skin: &MapSkin, scene: &MapScene, area: Rect) -> Vec<Sprite> {
        let marks = c.game.content.images.id(EDGE_MARKS_PATH);
        let mut buf = blank();
        skin.paint(c, scene, area, &mut buf);
        let sprites = buf.sprites().into_iter();
        sprites.filter(|s| Some(s.image) == marks).collect()
    }

    #[test]
    fn an_edge_arrow_sits_on_its_middle_tile_above_the_hp_bar_and_fades() {
        let c = ctx();
        let s = skin(&c);
        let area = Rect::new(0, 0, 9, 5);
        let mut scene = MapScene::new(p(0, 0), (3, 3));
        scene.set_terrain(|_| c.game.content.terrain.display.id_of("plain"));
        assert_eq!(edge_arrows(&c, &s, &scene, area), []);
        scene.more = every_edge();
        // 24-px tiles from (0, 4) (72 px centred in 80): the right arrow
        // on (2, 1), the left on (0, 1), the down arrow on (1, 2) and the
        // up arrow on (1, 0).
        let expected = [
            (Rect::new(0, 0, 8, 13), Rect::new(63, 32, 8, 13)),
            (Rect::new(8, 0, 8, 13), Rect::new(1, 32, 8, 13)),
            (Rect::new(16, 0, 13, 8), Rect::new(29, 66, 13, 8)),
            (Rect::new(29, 0, 13, 8), Rect::new(29, 5, 13, 8)),
        ];
        for (clock_ms, opacity) in [(0, 255), (500, 89)] {
            scene.clock_ms = clock_ms;
            let got = edge_arrows(&c, &s, &scene, area);
            let placed: Vec<_> = got.iter().map(|a| (a.src, a.dest)).collect();
            assert_eq!(placed, expected);
            for arrow in got {
                assert_eq!(
                    (arrow.opacity, arrow.clip, arrow.layer),
                    (opacity, arrow.dest, Layer::Over)
                );
                // Never on a tile's HP bar: its bottom 2 px.
                let bottom = ((arrow.dest.y - 4) / 24 + 1) * 24 + 4 - HP_BAR_H;
                assert!(arrow.dest.y + arrow.dest.h <= bottom, "{arrow:?}");
            }
        }
        // Only the sides the map goes on past.
        scene.more = MapEdges::of_view(scene.origin, scene.size, (5, 3));
        let got = edge_arrows(&c, &s, &scene, area);
        assert_eq!(
            got.iter().map(|a| a.src).collect::<Vec<_>>(),
            [expected[0].0]
        );
    }

    #[test]
    fn on_glyph_terrain_the_edge_arrows_are_the_glyph_skins() {
        let c = ctx();
        let mut scene = plains(&c);
        scene.more.x.after = true;
        let s = sheets(&c);
        assert_eq!(edge_arrows(&c, &s, &scene, Rect::new(2, 1, 6, 4)), []);
        let buf = painted(&c, &s, &scene);
        // The view's right column, its middle row: tile (2, 0).
        assert_eq!(buf.get(7, 1).map(|cell| cell.glyph), Some('→'));
    }

    /// One change to a scene the skin must show.
    struct Feature {
        name: &'static str,
        without: MapScene,
        with: MapScene,
    }

    /// The names of the fields at the top of `debug`, a struct's `{:#?}`.
    fn fields(debug: &str) -> Vec<&str> {
        debug
            .lines()
            .filter_map(|line| line.strip_prefix("    "))
            .filter(|line| !line.starts_with(' '))
            .filter_map(|line| line.split_once(':').map(|(name, _)| name))
            .collect()
    }

    /// Names every field of a scene, its tiles, its units and its cursor,
    /// so a new one fails (to compile, or this check) until it is listed
    /// here and given a feature in [`features`], or the reason it has none.
    /// The engine's scene and cursor can't be destructured outside it
    /// (`#[non_exhaustive]`): their fields are checked by name.
    fn every_field_is_listed(scene: &MapScene) {
        let scene_fields = [
            "origin", // where the tiles are: every feature moves with it
            "size",   // how many tiles: likewise
            "more", "tiles", "ranges",
            // Painted only by a tileset with layers, and one with other
            // looks: `the_ring_and_the_look_are_painted_by_a_tileset_
            // that_has_layers_and_looks`.
            "rim", "look", "units", "cursor", "path", "threat", "clock_ms",
        ];
        assert_eq!(fields(&format!("{scene:#?}")), scene_fields);
        let TileView {
            terrain: _,
            flashes: _,
            becomes: _,
        } = &scene.tiles[0];
        let UnitView {
            id: _,    // which unit it is: not shown
            label: _, // the glyph skin's; this skin shows the picture instead
            pos: _,
            faction: _,
            class: _,
            character: _,
            acted: _,
            fight: _,
            hp: _,
            effects:
                UnitEffects {
                    bonus: _,
                    penalty: _,
                },
            fade: _,
            highlight: _,
            facing: _,
            frame: _,
            offset: _,
            flash: _,
            marks: _,
        } = &scene.units[0];
        let cursor = CursorView::new(p(0, 0), 1.0, CursorStyle::Corners);
        assert_eq!(
            fields(&format!("{cursor:#?}")),
            ["pos", "brightness", "style"]
        );
    }

    /// Every feature of a [`MapScene`], each as a scene without it and the
    /// same scene with it. Every field of the scene is listed below, so a
    /// new one fails until it is listed here too, with a feature or the
    /// reason it has none.
    fn features(c: &Ctx) -> Vec<Feature> {
        let display = &c.game.content.terrain.display;
        let mut base = plains(c);
        base.push_unit(brigand(p(1, 1)));
        every_field_is_listed(&base);
        let feature = |name, change: &dyn Fn(&mut MapScene)| {
            let mut with = base.clone();
            change(&mut with);
            Feature {
                name,
                without: base.clone(),
                with,
            }
        };
        let unit = |change: fn(&mut UnitView)| move |s: &mut MapScene| change(&mut s.units[0]);
        let mut out = vec![
            feature("terrain", &|s| s.tiles[0].terrain = display.id_of("water")),
            feature("flash", &|s| s.tiles[0].flashes.push(0.5)),
            feature("becomes", &|s| s.tiles[0].becomes = display.id_of("water")),
            feature("unit pos", &unit(|u| u.pos = p(0, 1))),
            feature("faction", &unit(|u| u.faction = Faction::Player)),
            feature("class", &unit(|u| u.class = ClassId("mage".into()))),
            feature(
                "character",
                &unit(|u| u.character = Some(CharacterId("test_lord".into()))),
            ),
            feature("acted", &unit(|u| u.acted = true)),
            feature(
                "down",
                &unit(|u| u.fight = crate::map_view::scene::Fight::Down),
            ),
            feature("hp", &unit(|u| u.hp.0 = 10)),
            feature("bonus", &unit(|u| u.effects.bonus = true)),
            feature("penalty", &unit(|u| u.effects.penalty = true)),
            feature("fade", &unit(|u| u.fade = 0.25)),
            feature("highlight", &unit(|u| u.highlight = true)),
            feature("facing", &unit(|u| u.facing = Facing::Left)),
            feature("frame", &unit(|u| u.frame = 0)),
            feature("offset", &unit(|u| u.offset = (-0.5, 0.0))),
            feature("unit flash", &unit(|u| u.flash = 0.5)),
            feature("marks", &unit(|u| u.marks = false)),
            feature("path", &|s| s.path = vec![p(0, 0), p(1, 0)]),
            feature("threat", &|s| s.threat = Some(threat(p(0, 0), &[p(1, 1)]))),
            feature("more", &|s| s.more.x.before = true),
        ];
        // The clock moves the marks: an arrow rests, then is bounced.
        let mut clock = feature("clock", &|s| s.clock_ms = units::BOUNCE_MS);
        clock.without.units[0].effects.bonus = true;
        clock.with.units[0].effects.bonus = true;
        out.push(clock);
        for kind in range::ALL {
            out.push(feature("range", &move |s| s.tint([p(0, 0)], kind)));
        }
        let styles = [
            CursorStyle::Corners,
            CursorStyle::LargeCorners,
            CursorStyle::TileGlow,
        ];
        for style in styles {
            match style {
                CursorStyle::Corners | CursorStyle::LargeCorners | CursorStyle::TileGlow => {}
            }
            let at = move |brightness| CursorView::new(p(0, 1), brightness, style);
            out.push(feature("cursor", &move |s| s.cursor = Some(at(1.0))));
            let mut dim = feature("cursor brightness", &move |s| s.cursor = Some(at(0.5)));
            dim.without.cursor = Some(at(1.0));
            out.push(dim);
        }
        out
    }

    /// For each feature of a scene, the skin's frame with it differs from
    /// the frame without it: nothing in a scene goes unpainted (ADR-0038).
    /// Under the skin of a tileset with terrain tiles, and under the mixed
    /// skin of one without (units as sprites on glyph terrain).
    #[test]
    fn every_scene_feature_is_painted() {
        let c = ctx();
        let own = with_walking_brigand(&c, &with_lord(&skin(&c)));
        for s in [own, with_lord(&sheets(&c))] {
            let features = features(&c);
            assert_eq!(features.len(), 23 + 5 + 6);
            for f in features {
                assert_ne!(f.without, f.with, "{}: no change", f.name);
                let (without, with) = (painted(&c, &s, &f.without), painted(&c, &s, &f.with));
                assert_ne!(without, with, "{} under {}: not painted", f.name, s.name());
            }
        }
    }

    /// The ring of terrain round the view and the map's look change the
    /// frame of a tileset that has layers and another look (and of no
    /// other: a tileset without them has nothing to paint them with).
    #[test]
    fn the_ring_and_the_look_are_painted_by_a_tileset_that_has_layers_and_looks() {
        let c = ctx();
        let water = c.game.content.terrain.display.id_of("water");
        let s = with_indoor(&layered(&c));
        let base = plains(&c);
        let mut ring = base.clone();
        // Left of the view's first tile.
        ring.rim[5] = water;
        let mut indoor = base.clone();
        indoor.look.tiles = "indoor".to_owned();
        for (name, with) in [("rim", &ring), ("look", &indoor)] {
            assert_ne!(base, *with, "{name}: no change");
            let (without, with) = (painted(&c, &s, &base), painted(&c, &s, with));
            assert_ne!(without, with, "{name}: not painted");
        }
        for plain in [skin(&c), sheets(&c)] {
            assert_eq!(painted(&c, &plain, &base), painted(&c, &plain, &ring));
            assert_eq!(painted(&c, &plain, &base), painted(&c, &plain, &indoor));
        }
    }

    /// The map's light changes the frame of every tileset with terrain
    /// tiles (ADR-0058), and of no skin that paints the glyph skin's
    /// ground.
    #[test]
    fn the_light_is_painted_by_a_tileset_that_has_terrain_tiles() {
        let c = ctx();
        let base = plains(&c);
        let mut dusk = base.clone();
        dusk.look.light = "dusk".to_owned();
        assert_ne!(base, dusk);
        for s in [skin(&c), layered(&c)] {
            assert_ne!(painted(&c, &s, &base), painted(&c, &s, &dusk));
        }
        let s = sheets(&c);
        assert_eq!(painted(&c, &s, &base), painted(&c, &s, &dusk));
        let glyphs = |scene: &MapScene| {
            let mut buf = blank();
            MapSkin::glyph().paint(&c, scene, MAP_VIEW, &mut buf);
            buf
        };
        assert_eq!(glyphs(&base), glyphs(&dusk));
    }

    #[test]
    fn a_tileset_without_terrain_paints_units_on_the_glyph_skins_ground() {
        let c = ctx();
        let s = sheets(&c);
        assert_eq!(
            (s.name(), s.tileset_id()),
            ("sprite_units", Some("test_units"))
        );
        assert_eq!(skin(&c).tileset_id(), Some("test"));
        // The glyph skin's tiles: 16 × 16 px, 35 × 30 in the map area.
        assert_eq!(s.tile_size(), (16, 16));
        assert_eq!(
            s.view_tiles(MAP_VIEW),
            MapSkin::glyph().view_tiles(MAP_VIEW)
        );
        let area = Rect::new(2, 1, 6, 2);
        let mut scene = plains(&c);
        scene.tint([p(1, 0)], range::MOVE);
        scene.path = vec![p(0, 0), p(1, 0)];
        scene.cursor = Some(CursorView::new(p(2, 1), 1.0, CursorStyle::Corners));
        assert_eq!(
            s.tile_px(&scene, area, p(1, 1)),
            MapSkin::glyph().tile_px(&scene, area, p(1, 1))
        );
        assert_eq!(
            s.tile_px(&scene, area, p(1, 1)),
            Some(Rect::new(32, 32, 16, 16))
        );
        // With no unit, the frame is the glyph skin's.
        let paint = |skin: &MapSkin, scene: &MapScene| {
            let mut buf = blank();
            skin.paint(&c, scene, area, &mut buf);
            buf
        };
        assert_eq!(paint(&s, &scene), paint(&MapSkin::glyph(), &scene));
        // With one: its tile keeps its background and loses its glyphs,
        // and nothing else of the ground changes.
        let ground = paint(&MapSkin::glyph(), &scene);
        scene.push_unit(brigand(p(1, 0)));
        let buf = paint(&s, &scene);
        for y in 0..32 {
            for x in 0..100 {
                let (ours, theirs) = (buf.get(x, y).unwrap(), ground.get(x, y).unwrap());
                if y == 1 && (4..6).contains(&x) {
                    assert_eq!((ours.glyph, ours.bg), (' ', theirs.bg), "({x}, {y})");
                    assert_ne!(theirs.glyph, ' ');
                } else {
                    assert_eq!(ours, theirs, "({x}, {y})");
                }
            }
        }
        // A unit between two tiles walks over the glyphs: none is cleared,
        // on the tile it leaves or the one it walks to.
        let mut walking = scene.clone();
        walking.units[0].offset = (0.5, 0.0);
        let walked = paint(&s, &walking);
        for x in 4..8 {
            assert_eq!(walked.get(x, 1), ground.get(x, 1), "({x}, 1)");
        }
        // The unit's sprites and bar go between the path's line and its
        // arrowhead; the cursor's marks stay last.
        let path = c.palette.get(UiColor::Path);
        let cursor = c.palette.get(UiColor::Cursor);
        let kinds: Vec<&str> = buf
            .items()
            .iter()
            .map(|i| match i {
                Item::Sprite(s) if s.layer == Layer::Over => "sprite",
                Item::Sprite(_) => "under",
                Item::Rect(o) if o.color == path => "path",
                Item::Rect(o) if o.color == cursor => "cursor",
                Item::Rect(_) => "bar",
            })
            .collect();
        let mut expect = vec!["path"];
        expect.extend(["sprite"; 5]);
        expect.push("bar");
        expect.extend(["path"; 6]);
        expect.extend(["cursor"; 8]);
        assert_eq!(kinds, expect);
        // A unit picked out stands on the terrain's glyph colour; one
        // falling fades out as the glyphs come back.
        let display = &c.game.content.terrain.display;
        let forest = display.get(display.id_of("forest").unwrap()).unwrap();
        let fg = c.palette.lookup(&forest.fg).unwrap();
        scene.units[0].highlight = true;
        assert_eq!(paint(&s, &scene).get(4, 1).map(|c| c.bg), Some(fg));
        scene.units[0] = brigand(p(1, 0)).fading(0.75);
        let falling = paint(&s, &scene);
        assert_eq!(falling.get(4, 1).map(|c| c.glyph), Some(forest.glyphs[0]));
        assert_eq!(falling.sprites().len(), 1);
        assert_eq!(falling.sprites()[0].opacity, 64);
    }

    /// The same small scene under the mixed skin: glyph cells and the
    /// units' sprites.
    #[test]
    fn sprite_units_skin_snapshot() {
        let c = ctx();
        let s = sheets(&c);
        let mut scene = plains(&c);
        scene.tint([p(0, 0)], range::MOVE);
        let mut lord = brigand(p(0, 0));
        lord.faction = Faction::Player;
        lord.class = ClassId("exile".into());
        lord.hp = (5, 30);
        lord.effects.bonus = true;
        scene.push_unit(lord);
        // Below the lord: shaved at its tile's top edge, its mark lowered.
        let mut enemy = brigand(p(0, 1));
        enemy.acted = true;
        enemy.effects.penalty = true;
        scene.push_unit(enemy);
        scene.push_unit(brigand(p(2, 1)).highlighted(true));
        scene.cursor = Some(CursorView::new(p(1, 1), 1.0, CursorStyle::Corners));
        let mut buf = GlyphBuffer::new(10, 4, fill());
        s.paint(&c, &scene, Rect::new(2, 1, 6, 2), &mut buf);
        assert_snapshot!(buf.to_snapshot(&c.palette));
    }

    prop_compose! {
        pub(crate) fn any_area()(area in (-6..104i32, -4..34i32, -2..110i32, -2..40i32)) -> Rect {
            Rect::new(area.0, area.1, area.2, area.3)
        }
    }

    proptest! {
        /// Whatever the scene, the area (even one partly off the console),
        /// the tile size and the zoom, the skin changes no cell and adds no
        /// item
        /// (rectangle or sprite) outside the area's pixels.
        #[test]
        fn sprite_skin_paints_only_the_area(
            scene in any_scene(),
            area in any_area(),
            tile in (8..=64u32, 8..=64u32),
            zoom in prop::sample::select(MapZoom::ALL.to_vec()),
        ) {
            let c = ctx();
            let mut buf = blank();
            let skin = zoomed(sized(&c, tile.0, tile.1), zoom);
            skin.paint(&c, &scene, area, &mut buf);
            for y in 0..32 {
                for x in 0..100 {
                    prop_assert_eq!(buf.get(x, y), Some(&fill()), "cell ({}, {})", x, y);
                }
            }
            let px = px_rect(area);
            for item in buf.items() {
                let seen = item.visible();
                prop_assert_eq!(seen.intersect(&px), Some(seen), "{:?} in {:?}", item, area);
            }
        }

        /// Whatever the scene and the area, the mixed skin (units as
        /// sprites on glyph terrain) changes no cell outside the area and
        /// adds no item outside its pixels.
        #[test]
        fn sprite_units_skin_paints_only_the_area(scene in any_scene(), area in any_area()) {
            let c = ctx();
            let mut buf = blank();
            sheets(&c).paint(&c, &scene, area, &mut buf);
            for y in 0..32 {
                for x in 0..100 {
                    if !area.contains(x, y) {
                        prop_assert_eq!(buf.get(x, y), Some(&fill()), "cell ({}, {})", x, y);
                    }
                }
            }
            let px = px_rect(area);
            for item in buf.items() {
                let seen = item.visible();
                prop_assert_eq!(seen.intersect(&px), Some(seen), "{:?} in {:?}", item, area);
            }
        }

        /// For any tile size at either zoom, every visible tile lies
        /// inside the area, and the tiles sit edge to edge without
        /// overlapping.
        #[test]
        fn tiles_lie_inside_the_area_edge_to_edge(
            area in any_area(),
            tile in (8..=64u32, 8..=64u32),
            origin in (-30..30i32, -30..30i32),
            zoom in prop::sample::select(MapZoom::ALL.to_vec()),
        ) {
            let c = ctx();
            let s = zoomed(sized(&c, tile.0, tile.1), zoom);
            let (w, h) = (side(tile.0) * zoom.factor(), side(tile.1) * zoom.factor());
            let all = px_rect(area);
            prop_assert_eq!(s.view_tiles(area), (all.w.max(0) / w, all.h.max(0) / h));
            let scene = MapScene::new(Pos::new(origin.0, origin.1), s.view_tiles(area));
            let px = px_rect(area);
            let rect = |dx, dy| s.tile_px(&scene, area, Pos::new(origin.0 + dx, origin.1 + dy));
            for dy in 0..scene.size.1 {
                for dx in 0..scene.size.0 {
                    let r = rect(dx, dy);
                    prop_assert!(r.is_some());
                    let r = r.unwrap_or(Rect::new(0, 0, 0, 0));
                    prop_assert_eq!((r.w, r.h), (w, h));
                    prop_assert_eq!(r.intersect(&px), Some(r), "{:?} in {:?}", r, area);
                    if let Some(right) = rect(dx + 1, dy) {
                        prop_assert_eq!((right.x, right.y), (r.x + w, r.y));
                    }
                    if let Some(below) = rect(dx, dy + 1) {
                        prop_assert_eq!((below.x, below.y), (r.x, r.y + h));
                    }
                }
            }
            let (across, down) = scene.size;
            prop_assert!(rect(across, 0).is_none() && rect(0, down).is_none());
            prop_assert!(rect(-1, 0).is_none() && rect(0, -1).is_none());
        }
    }
}
