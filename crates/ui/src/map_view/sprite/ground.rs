//! The ground a sprite skin paints from its tileset's own tiles
//! (ADR-0052): each tile's own picture, then the look's layers over them
//! (pictures between tiles by the [corner rule](crate::map_view::corners),
//! and pictures on tiles), then what tints a tile.
//!
//! A tile off the map gets no picture, and no part of one: a picture
//! between tiles is drawn only inside the tiles of the map it covers.

use trpg_content::tileset::{CornerLayer, Layer, Look, TileLayer};
use trpg_content::{ImageId, ImageRect, Picture, TerrainTiles};
use trpg_core::Pos;

use super::tile_sprite;
use crate::color::{Rgb, UiColor, to_channel};
use crate::glyph_buffer::{GlyphBuffer, Layer as Depth, Overlay, Paint, PxRect, Rect, Sprite};
use crate::map_view::corners::{Shown, mix_of};
use crate::map_view::glyph::cursor::GLOW_MAX;
use crate::map_view::glyph::{OVERLAY_BLEND, named, range_color};
use crate::map_view::grid::Grid;
use crate::map_view::scene::{CursorStyle, CursorView, MapScene, TileView};
use crate::screen::Ctx;

/// Paints `scene`'s ground in `grid` from `terrain`'s tiles, in the look
/// the scene's map asks for (the tileset's own if it lacks that one).
pub(super) fn paint(
    ctx: &Ctx,
    terrain: &TerrainTiles,
    scene: &MapScene,
    grid: &Grid,
    buf: &mut GlyphBuffer,
) {
    let look = terrain.looks.get(&scene.look.tiles);
    let ground = Ground {
        scene,
        grid,
        image: terrain.image,
        look: look.unwrap_or(&terrain.look),
        shown: Shown::of(scene),
    };
    ground.paint_tiles(buf);
    for layer in &ground.look.layers {
        match layer {
            Layer::Corners(layer) => ground.paint_corners(layer, buf),
            Layer::Tiles(layer) => ground.paint_singles(layer, buf),
        }
    }
    ground.paint_tints(ctx, buf);
}

/// One scene's ground being painted.
struct Ground<'a> {
    scene: &'a MapScene,
    grid: &'a Grid,
    /// The image every picture is in.
    image: ImageId,
    look: &'a Look,
    /// The terrain each tile is drawn as.
    shown: Shown,
}

impl Ground<'_> {
    /// The pixels of the tile `dx` right of and `dy` below the view's
    /// first, if it is drawn and on the map.
    fn tile(&self, dx: i32, dy: i32) -> Option<PxRect> {
        let (across, down) = self.grid.tiles;
        let drawn = (0..across).contains(&dx) && (0..down).contains(&dy);
        let on_map = self.scene.tile_at(dx, dy)?.terrain.is_some();
        (drawn && on_map).then(|| self.grid.rect_at(dx, dy))
    }

    /// Every drawn tile that is on the map: where it is in the view, and
    /// its pixels.
    fn tiles(&self) -> impl Iterator<Item = (i32, i32, PxRect)> + '_ {
        let (across, down) = self.grid.tiles;
        let places = (0..down).flat_map(move |dy| (0..across).map(move |dx| (dx, dy)));
        places.filter_map(|(dx, dy)| Some((dx, dy, self.tile(dx, dy)?)))
    }

    /// `rect` of the tileset's image.
    const fn picture(&self, rect: ImageRect) -> Picture {
        Picture {
            image: self.image,
            rect,
        }
    }

    /// The own picture of the terrain shown on the tile `dx` right of and
    /// `dy` below the view's first, if the look has one.
    fn own(&self, dx: i32, dy: i32) -> Option<Picture> {
        let id = self.shown.at(dx, dy)?;
        self.look.tiles.get(&id).map(|&rect| self.picture(rect))
    }

    /// Each tile's own picture.
    fn paint_tiles(&self, buf: &mut GlyphBuffer) {
        for (dx, dy, rect) in self.tiles() {
            if let Some(picture) = self.own(dx, dy) {
                buf.add_sprite(tile_sprite(picture, rect));
            }
        }
    }

    /// The pictures of `layer`: one centred on each point where four
    /// tiles meet, for the mix of those four being in the layer, drawn
    /// inside the ones that are drawn and on the map.
    fn paint_corners(&self, layer: &CornerLayer, buf: &mut GlyphBuffer) {
        let (w, h) = self.grid.tile;
        let (left, top) = self.grid.corner;
        let (across, down) = self.grid.tiles;
        for (i, j) in (0..=down).flat_map(|j| (0..=across).map(move |i| (i, j))) {
            let mix = mix_of(self.shown.corners(i, j), &layer.of);
            let Some(rect) = layer.tiles.get(usize::from(mix)).copied().flatten() else {
                continue;
            };
            let dest = Rect::new(left + i * w - w / 2, top + j * h - h / 2, w, h);
            let sprite = tile_sprite(self.picture(rect), dest);
            let round = [(i - 1, j - 1), (i, j - 1), (i - 1, j), (i, j)];
            let parts: Vec<PxRect> = round
                .iter()
                .filter_map(|&(dx, dy)| self.tile(dx, dy)?.intersect(&dest))
                .collect();
            if parts.len() == round.len() {
                buf.add_sprite(sprite);
            } else {
                for clip in parts {
                    buf.add_sprite(Sprite { clip, ..sprite });
                }
            }
        }
    }

    /// The picture of `layer` on each of its tiles: the one for the mix
    /// of the tile's four neighbours being terrain the layer looks for.
    fn paint_singles(&self, layer: &TileLayer, buf: &mut GlyphBuffer) {
        for (dx, dy, rect) in self.tiles() {
            if self
                .shown
                .at(dx, dy)
                .is_some_and(|id| layer.of.contains(&id))
            {
                let sides = mix_of(self.shown.sides(dx, dy), &layer.beside);
                let picture = self.picture(layer.picture(sides));
                buf.add_sprite(tile_sprite(picture, rect));
            }
        }
    }

    /// What tints each tile, over its pictures.
    fn paint_tints(&self, ctx: &Ctx, buf: &mut GlyphBuffer) {
        let black = ctx.palette.get(UiColor::Black);
        let (across, down) = self.grid.tiles;
        for (dx, dy) in (0..down).flat_map(|dy| (0..across).map(move |dx| (dx, dy))) {
            let Some(tile) = self.scene.tile_at(dx, dy) else {
                continue;
            };
            let pos = Pos::new(self.scene.origin.x + dx, self.scene.origin.y + dy);
            let cursor = self.scene.cursor.filter(|c| c.pos == pos);
            let own = self.tile(dx, dy).and_then(|_| self.own(dx, dy));
            let rect = self.grid.rect_at(dx, dy);
            paint_tinted(buf, rect, own, &tints(ctx, tile, cursor), black);
        }
    }
}

/// What tints `tile`, in order, as colours and strengths: its flashes
/// (towards its terrain's glyph colour, as on the glyph skin), its ranges,
/// then the glow of `cursor` (if it is on the tile and glows). A tile a
/// spell would change is drawn as what it would become, so only the glow
/// tints it.
fn tints(ctx: &Ctx, tile: &TileView, cursor: Option<CursorView>) -> Vec<(Rgb, f32)> {
    let mut out = Vec::new();
    if tile.becomes.is_none() {
        let display = &ctx.content.terrain.display;
        let fg = tile
            .terrain
            .and_then(|id| display.get(id))
            .map(|t| named(&ctx.palette, &t.fg, UiColor::Text));
        if let Some(fg) = fg {
            out.extend(tile.flashes.iter().map(|&s| (fg, OVERLAY_BLEND * s)));
        }
        for &kind in &tile.tints {
            out.push((ctx.palette.get(range_color(kind)), OVERLAY_BLEND));
        }
    }
    if let Some(c) = cursor.filter(|c| c.style == CursorStyle::TileGlow) {
        out.push((ctx.palette.get(UiColor::Cursor), GLOW_MAX * c.brightness));
    }
    out
}

/// Tinting a picture by each of `tints` in turn (as `Rgb::lerp` towards
/// the colour by the strength, the glyph skin's `blend_bg`) leaves
/// `kept × picture + added`: returns `kept` and `added`.
fn blend(tints: &[(Rgb, f32)]) -> (f32, [f32; 3]) {
    let mut kept = 1.0;
    let mut added = [0.0; 3];
    for &(color, t) in tints {
        let t = if t.is_nan() { 0.0 } else { t.clamp(0.0, 1.0) };
        kept *= 1.0 - t;
        for (a, c) in added.iter_mut().zip([color.r, color.g, color.b]) {
            *a = *a * (1.0 - t) + f32::from(c) * t;
        }
    }
    (kept, added)
}

/// Tints the tile on the pixels `rect` by `tints`. A tile with a picture
/// of its `own` gets that picture's shape over it, in the one colour the
/// tints add up to and as strong as they are together, so whatever is
/// painted there reads as tinted; a tile with none (off the map) is the
/// tints over `base`.
fn paint_tinted(
    buf: &mut GlyphBuffer,
    rect: PxRect,
    own: Option<Picture>,
    tints: &[(Rgb, f32)],
    base: Rgb,
) {
    let (kept, added) = blend(tints);
    let rgb = |[r, g, b]: [f32; 3]| Rgb::new(to_channel(r), to_channel(g), to_channel(b));
    match own {
        Some(picture) if kept < 1.0 => {
            let color = rgb(added.map(|a| a / (1.0 - kept)));
            let mut wash = tile_sprite(picture, rect).painted(Paint::Solid(color));
            wash.opacity = to_channel(255.0 * (1.0 - kept));
            buf.add_sprite(wash);
        }
        None if !tints.is_empty() => {
            let base = [base.r, base.g, base.b];
            let mut color = added;
            for (c, b) in color.iter_mut().zip(base) {
                *c += kept * f32::from(b);
            }
            buf.add_overlay(Overlay::new(rect, rgb(color), Depth::Under));
        }
        _ => {}
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::collections::BTreeSet;

    use proptest::prelude::*;
    use trpg_content::tileset::MIXES;
    use trpg_core::TerrainId;

    use super::*;
    use crate::map_view::glyph::tests::any_scene;
    use crate::map_view::scene::RangeKind;
    use crate::map_view::skin::MapSkin;
    use crate::map_view::sprite::tests::{any_area, blank, p, plains, rect_at, skin, sprite_at};
    use crate::map_view::sprite::{SpriteSkin, src_rect};
    use crate::screen::tests::ctx;

    /// 9 × 5 cells: 72 × 80 px, so 3 × 3 tiles of 24 px with 4 px to
    /// spare above and below. The first tile's corner is (0, 4).
    const AREA: Rect = Rect::new(0, 0, 9, 5);

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
    pub(crate) fn layered(c: &Ctx) -> SpriteSkin {
        let id = |name| c.content.terrain.display.id_of(name).unwrap();
        let mut tileset = c.content.tilesets["test"].clone();
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
        terrain.look.layers = vec![Layer::Corners(shore), Layer::Tiles(bridge)];
        SpriteSkin::new(tileset)
    }

    /// `skin` with an indoor look: every terrain's tile another one of
    /// the image's, and no layers.
    pub(crate) fn with_indoor(skin: &SpriteSkin) -> SpriteSkin {
        let mut tileset = skin.tileset().clone();
        let terrain = tileset.terrain.as_mut().unwrap();
        let mut indoor = Look::default();
        for (&id, rect) in &terrain.look.tiles {
            indoor.tiles.insert(id, ImageRect { y: 72, ..*rect });
        }
        terrain.looks.insert("indoor".to_owned(), indoor);
        SpriteSkin::new(tileset)
    }

    /// A 3 × 3 view of `rows` (`.` plain, `~` water, `=` bridge, a space
    /// off the map), with the same terrain running on round it.
    fn view(c: &Ctx, rows: [&str; 3]) -> MapScene {
        let id = |name| c.content.terrain.display.id_of(name);
        let mut scene = MapScene::new(p(0, 0), (3, 3));
        scene.set_terrain(|pos| {
            let row = rows.get(usize::try_from(pos.y.clamp(0, 2)).ok()?)?;
            match row.chars().nth(usize::try_from(pos.x.clamp(0, 2)).ok()?)? {
                '.' => id("plain"),
                '~' => id("water"),
                '=' => id("bridge"),
                _ => None,
            }
        });
        scene
    }

    fn painted(c: &Ctx, skin: &SpriteSkin, scene: &MapScene) -> GlyphBuffer {
        let mut buf = blank();
        skin.paint(c, scene, AREA, &mut buf);
        buf
    }

    /// The sprites of `buf` from the `from`th on: each one's source
    /// rectangle and the pixels it is drawn in.
    fn drawn(buf: &GlyphBuffer, from: usize) -> Vec<(PxRect, PxRect)> {
        let sprites = buf.sprites();
        sprites[from..].iter().map(|s| (s.src, s.clip)).collect()
    }

    #[test]
    fn a_layer_draws_a_picture_round_each_point_where_its_tiles_meet_others() {
        let c = ctx();
        let s = layered(&c);
        let lake = view(&c, ["...", ".~.", "..."]);
        let buf = painted(&c, &s, &lake);
        // Each tile's own picture first: nine, the water in the middle.
        let display = &c.content.terrain.display;
        let own = |name| src_rect(s.tileset().tile(display.id_of(name).unwrap()).unwrap().rect);
        let sprites = buf.sprites();
        for (i, sprite) in sprites[..9].iter().enumerate() {
            let name = if i == 4 { "water" } else { "plain" };
            let (dx, dy) = (i32::try_from(i % 3).unwrap(), i32::try_from(i / 3).unwrap());
            let tile = Rect::new(24 * dx, 4 + 24 * dy, 24, 24);
            assert_eq!(
                (sprite.src, sprite.dest, sprite.clip),
                (own(name), tile, tile)
            );
            assert_eq!(sprite.layer, Depth::Under);
        }
        // Then the shore: a picture centred on each corner of the water
        // tile, for the corner of it that is water.
        let cell = |x, y| Rect::new(x, y, 24, 24);
        assert_eq!(
            drawn(&buf, 9),
            [
                (src_rect(mix_rect(0b0001)), cell(12, 16)),
                (src_rect(mix_rect(0b0010)), cell(36, 16)),
                (src_rect(mix_rect(0b0100)), cell(12, 40)),
                (src_rect(mix_rect(0b1000)), cell(36, 40)),
            ]
        );
        for sprite in &sprites[9..] {
            assert_eq!((sprite.dest, sprite.layer), (sprite.clip, Depth::Under));
            assert_eq!((sprite.opacity, sprite.paint), (255, Paint::Image));
            assert_eq!(sprite.image.path(), "tilesets/test.png");
        }
        assert_eq!(buf.items().len(), 13);
        // No layer, no pictures between tiles.
        assert_eq!(painted(&c, &skin(&c), &lake).items().len(), 9);
    }

    #[test]
    fn a_picture_between_tiles_is_drawn_only_inside_the_tiles_of_the_map() {
        let c = ctx();
        let s = layered(&c);
        // Water down the left column, running on past the view.
        let river = view(&c, ["~..", "~..", "~.."]);
        let buf = painted(&c, &s, &river);
        let full = src_rect(mix_rect(0b1111));
        let bank = src_rect(mix_rect(0b1010));
        let r = Rect::new;
        assert_eq!(
            drawn(&buf, 9),
            [
                // Above the first row: only the halves inside the view.
                (full, r(0, 4, 12, 12)),
                (bank, r(12, 4, 12, 12)),
                (bank, r(24, 4, 12, 12)),
                // Between rows: the left edge cuts the water's in half;
                // the bank's is whole.
                (full, r(0, 16, 12, 12)),
                (full, r(0, 28, 12, 12)),
                (bank, r(12, 16, 24, 24)),
                (full, r(0, 40, 12, 12)),
                (full, r(0, 52, 12, 12)),
                (bank, r(12, 40, 24, 24)),
                // Below the last row.
                (full, r(0, 64, 12, 12)),
                (bank, r(12, 64, 12, 12)),
                (bank, r(24, 64, 12, 12)),
            ]
        );
        // A part keeps its picture's place: the whole picture's pixels.
        let top = buf.sprites()[10];
        assert_eq!(top.dest, r(12, -8, 24, 24));
        // A tile off the map has no picture and no part of one: of a map
        // of one tile, only the quarters inside it.
        let mut gap = MapScene::new(p(0, 0), (3, 3));
        gap.tiles[0].terrain = display_id(&c, "water");
        let buf = painted(&c, &s, &gap);
        assert_eq!(buf.sprites().len(), 1 + 4);
        assert_eq!(drawn(&buf, 0)[0].1, r(0, 4, 24, 24));
        for (src, clip) in drawn(&buf, 1) {
            assert_eq!(src, full);
            assert_eq!(clip.intersect(&r(0, 4, 24, 24)), Some(clip));
            assert_eq!((clip.w, clip.h), (12, 12));
        }
        // A view wider than the area: the tiles that don't fit get none.
        let mut wide = MapScene::new(p(0, 0), (5, 1));
        wide.set_terrain(|_| display_id(&c, "water"));
        let buf = painted(&c, &s, &wide);
        // Three tiles; above and below each, a half of the picture on
        // each of its upper and lower corners.
        assert_eq!(buf.sprites().len(), 3 + 2 * 6);
        let px = crate::map_view::grid::px_rect(AREA);
        for sprite in buf.sprites() {
            assert_eq!(sprite.clip.intersect(&px), Some(sprite.clip));
        }
    }

    fn display_id(c: &Ctx, name: &str) -> Option<TerrainId> {
        c.content.terrain.display.id_of(name)
    }

    #[test]
    fn an_odd_tile_is_cut_where_its_pictures_meet() {
        let c = ctx();
        let mut tileset = layered(&c).tileset().clone();
        tileset.terrain.as_mut().unwrap().tile_px = (9, 13);
        let s = SpriteSkin::new(tileset);
        let mut scene = MapScene::new(p(0, 0), (2, 2));
        scene.set_terrain(|pos| display_id(&c, if pos.x <= 0 { "water" } else { "plain" }));
        let mut buf = blank();
        // 3 × 2 cells: 24 × 32 px, so 2 × 2 tiles of 9 × 13 px from (3, 3).
        s.paint(&c, &scene, Rect::new(0, 0, 3, 2), &mut buf);
        let r = Rect::new;
        let clips: Vec<PxRect> = drawn(&buf, 4).into_iter().map(|(_, clip)| clip).collect();
        // Each picture starts 4 px left of and 6 px above its point.
        assert_eq!(
            clips,
            [
                r(3, 3, 5, 7),
                r(8, 3, 4, 7),
                r(12, 3, 5, 7),
                r(3, 10, 5, 6),
                r(3, 16, 5, 7),
                r(8, 10, 9, 13),
                r(3, 23, 5, 6),
                r(8, 23, 4, 6),
                r(12, 23, 5, 6),
            ]
        );
        assert_eq!(buf.sprites()[9].dest, r(8, 10, 9, 13));
    }

    #[test]
    fn a_tile_layers_picture_goes_by_the_neighbours_it_looks_for() {
        let c = ctx();
        let s = layered(&c);
        let plain = src_rect(mix_rect(15));
        let across = src_rect(mix_rect(0));
        let tile = |dx, dy| Rect::new(24 * dx, 4 + 24 * dy, 24, 24);
        // Water left and right of the bridge: its other picture.
        let buf = painted(&c, &s, &view(&c, ["...", "~=~", "..."]));
        let last = *buf.sprites().last().unwrap();
        assert_eq!(
            (last.src, last.dest, last.clip),
            (across, tile(1, 1), tile(1, 1))
        );
        // Water above and below: its plain one.
        let buf = painted(&c, &s, &view(&c, [".~.", ".=.", ".~."]));
        assert_eq!(buf.sprites().last().unwrap().src, plain);
        // At the view's edge the neighbour is the tile outside it: here
        // the bridge's own terrain, which isn't water.
        let buf = painted(&c, &s, &view(&c, ["...", "=~.", "..."]));
        let last = *buf.sprites().last().unwrap();
        assert_eq!((last.src, last.dest), (plain, tile(0, 1)));
        // One picture for each bridge, after the shore's.
        let buf = painted(&c, &s, &view(&c, ["=.=", "...", ".=."]));
        let bridges: Vec<PxRect> = buf.sprites().iter().rev().take(3).map(|s| s.dest).collect();
        assert_eq!(bridges, [tile(1, 2), tile(2, 0), tile(0, 0)]);
        // A tile off the map gets nothing, whatever it would become.
        let mut scene = view(&c, ["...", "~.~", "..."]);
        scene.tiles[4].terrain = None;
        scene.tiles[4].becomes = display_id(&c, "bridge");
        for (src, clip) in drawn(&painted(&c, &s, &scene), 0) {
            assert_eq!(clip.intersect(&tile(1, 1)), None, "{src:?} at {clip:?}");
            assert!(src != across && src != plain);
        }
    }

    #[test]
    fn a_map_is_painted_in_the_look_it_names_if_the_tileset_has_it() {
        let c = ctx();
        let s = with_indoor(&layered(&c));
        let mut lake = view(&c, ["...", ".~.", "..."]);
        let outdoor = painted(&c, &s, &lake);
        assert_eq!(outdoor, painted(&c, &layered(&c), &lake));
        lake.look.tiles = "indoor".to_owned();
        let buf = painted(&c, &s, &lake);
        assert_eq!(buf.sprites().len(), 9);
        assert!(buf.sprites().iter().all(|s| s.src.y == 72));
        // A look the tileset lacks: its own.
        lake.look.tiles = "cave".to_owned();
        assert_eq!(painted(&c, &s, &lake), outdoor);
    }

    #[test]
    fn a_tile_a_spell_would_change_is_painted_as_what_it_would_become() {
        let c = ctx();
        let s = layered(&c);
        let mut scene = view(&c, ["...", "...", "..."]);
        scene.tiles[4].becomes = display_id(&c, "water");
        // As the lake it would be: its neighbours' pictures join it.
        let lake = view(&c, ["...", ".~.", "..."]);
        assert_eq!(painted(&c, &s, &scene), painted(&c, &s, &lake));
        // Its ranges and flashes don't tint it; the cursor's glow does.
        scene.tint([p(1, 1)], RangeKind::Danger);
        scene.tiles[4].flashes.push(1.0);
        assert_eq!(painted(&c, &s, &scene), painted(&c, &s, &lake));
        scene.cursor = Some(CursorView {
            pos: p(1, 1),
            brightness: 1.0,
            style: CursorStyle::TileGlow,
        });
        let buf = painted(&c, &s, &scene);
        assert_eq!(buf.items().len(), 13 + 1);
        let glow = sprite_at(&buf, 13);
        let cursor = c.palette.get(UiColor::Cursor);
        assert_eq!(glow.paint, Paint::Solid(cursor));
        // What the tileset lacks becomes nothing: the tile has no picture.
        let mut scene = view(&c, ["...", "...", "..."]);
        scene.tiles[4].becomes = Some(TerrainId(999));
        assert_eq!(painted(&c, &s, &scene).sprites().len(), 8);
    }

    #[test]
    fn tints_blend_as_the_glyph_skin_blends() {
        let red = Rgb::new(200, 0, 0);
        let blue = Rgb::new(0, 0, 255);
        assert_eq!(blend(&[]), (1.0, [0.0; 3]));
        let (kept, added) = blend(&[(red, 0.75)]);
        assert!((kept - 0.25).abs() < 1e-6);
        assert!((added[0] - 150.0).abs() < 1e-4 && added[1].abs() < 1e-6);
        // Bad strengths: none (NaN) or clamped.
        assert_eq!(blend(&[(red, f32::NAN)]), (1.0, [0.0; 3]));
        assert_eq!(blend(&[(red, -1.0)]), (1.0, [0.0; 3]));
        assert_eq!(blend(&[(red, 2.0)]), blend(&[(red, 1.0)]));
        // Whatever the tile's colour, `kept × tile + added` is the tile
        // lerped towards each tint in turn.
        let tints = [(red, 0.75), (blue, 0.75), (Rgb::new(9, 200, 9), 0.3)];
        let (kept, added) = blend(&tints);
        for tile in [
            Rgb::new(0, 0, 0),
            Rgb::new(255, 255, 255),
            Rgb::new(30, 140, 60),
        ] {
            let glyph = tints.iter().fold(tile, |bg, &(c, t)| bg.lerp(c, t));
            let channels = [tile.r, tile.g, tile.b].into_iter().zip(added);
            let ours: Vec<f32> = channels.map(|(v, a)| kept * f32::from(v) + a).collect();
            for (o, g) in ours.iter().zip([glyph.r, glyph.g, glyph.b]) {
                assert!(
                    (o - f32::from(g)).abs() <= 1.5,
                    "{tile:?}: {ours:?} vs {glyph:?}"
                );
            }
        }
    }

    #[test]
    fn a_tinted_tile_has_its_own_shape_over_it_in_the_tints_colour() {
        let c = ctx();
        let s = layered(&c);
        let pal = &c.palette;
        let mut scene = view(&c, ["...", ".~.", "..."]);
        scene.tint([p(0, 0)], RangeKind::Move);
        scene.tint([p(1, 1)], RangeKind::Danger);
        scene.tint([p(1, 1)], RangeKind::Attack);
        let buf = painted(&c, &s, &scene);
        // After the tiles and the shore: the plain tile's own shape in
        // the range's colour, three quarters strong.
        let own = |name| {
            src_rect(
                s.tileset()
                    .tile(display_id(&c, name).unwrap())
                    .unwrap()
                    .rect,
            )
        };
        let wash = sprite_at(&buf, 13);
        let move_range = pal.get(UiColor::MoveRange);
        assert_eq!(
            (wash.src, wash.dest),
            (own("plain"), Rect::new(0, 4, 24, 24))
        );
        assert_eq!((wash.paint, wash.opacity), (Paint::Solid(move_range), 191));
        assert_eq!((wash.clip, wash.layer), (wash.dest, Depth::Under));
        // Two ranges: their mix, fifteen sixteenths strong.
        let both = sprite_at(&buf, 14);
        let (danger, attack) = (pal.get(UiColor::DangerZone), pal.get(UiColor::AttackRange));
        assert_eq!(both.paint, Paint::Solid(danger.lerp(attack, 0.8)));
        assert_eq!((both.src, both.opacity), (own("water"), 239));
        assert_eq!(both.dest, Rect::new(24, 28, 24, 24));
        assert_eq!(buf.items().len(), 15);
        // Off the map, a range tints the black under it.
        let mut scene = plains(&c);
        scene.tiles[0].terrain = None;
        scene.tint([p(0, 0)], RangeKind::Heal);
        let buf = painted(&c, &s, &scene);
        // After the five tiles on the map.
        let off = rect_at(&buf, 5);
        let black = pal.get(UiColor::Black);
        let heal = pal.get(UiColor::HealRange);
        assert_eq!(off.color, black.lerp(heal, OVERLAY_BLEND));
        assert_eq!(
            (off.rect, off.layer),
            (Rect::new(0, 4, 24, 24), Depth::Under)
        );
    }

    #[test]
    fn a_tile_is_tinted_only_as_strongly_as_its_tints_are() {
        let rect = Rect::new(8, 16, 24, 24);
        let c = ctx();
        let own = skin(&c).tileset().tile(TerrainId(0));
        assert!(own.is_some());
        let base = Rgb::new(200, 100, 40);
        let red = Rgb::new(250, 0, 0);
        // Whatever is under a tile with no picture: here not black.
        let mut buf = blank();
        paint_tinted(&mut buf, rect, None, &[(red, 0.5)], base);
        let over = rect_at(&buf, 0);
        assert_eq!((over.rect, over.color), (rect, Rgb::new(225, 50, 20)));
        assert_eq!(over.color, base.lerp(red, 0.5));
        // No tints: nothing, with a picture or without.
        paint_tinted(&mut buf, rect, None, &[], base);
        paint_tinted(&mut buf, rect, own, &[], base);
        // Tints of no strength: nothing over a picture.
        paint_tinted(&mut buf, rect, own, &[(red, 0.0)], base);
        assert_eq!(buf.items().len(), 1);
        // The faintest tint is there.
        paint_tinted(&mut buf, rect, own, &[(red, 0.01)], base);
        let faint = sprite_at(&buf, 1);
        assert_eq!((faint.paint, faint.opacity), (Paint::Solid(red), 3));
        // A full one covers the tile.
        paint_tinted(&mut buf, rect, own, &[(red, 1.0)], base);
        assert_eq!(sprite_at(&buf, 2).opacity, 255);
    }

    #[test]
    fn flashes_tint_towards_the_terrain_colour_then_the_ranges_then_the_glow() {
        let c = ctx();
        let pal = &c.palette;
        let display = &c.content.terrain.display;
        let plain = display.get(display.id_of("plain").unwrap()).unwrap();
        let fg = pal.lookup(&plain.fg).unwrap();
        let attack = pal.get(UiColor::AttackRange);
        let cursor = pal.get(UiColor::Cursor);
        let mut tile = TileView {
            terrain: display.id_of("plain"),
            flashes: vec![1.0, 0.5],
            tints: vec![RangeKind::Attack],
            becomes: None,
        };
        let glow = |brightness| CursorView {
            pos: p(0, 0),
            brightness,
            style: CursorStyle::TileGlow,
        };
        assert_eq!(
            tints(&c, &tile, Some(glow(0.5))),
            [
                (fg, OVERLAY_BLEND),
                (fg, OVERLAY_BLEND * 0.5),
                (attack, OVERLAY_BLEND),
                (cursor, GLOW_MAX * 0.5),
            ]
        );
        // A cursor of corner marks doesn't tint.
        let corners = CursorView {
            style: CursorStyle::Corners,
            ..glow(1.0)
        };
        assert_eq!(tints(&c, &tile, Some(corners)).len(), 3);
        assert_eq!(tints(&c, &tile, None).len(), 3);
        // A tile a spell would change: only the glow.
        tile.becomes = display.id_of("forest");
        assert_eq!(tints(&c, &tile, Some(glow(1.0))), [(cursor, GLOW_MAX)]);
        assert!(tints(&c, &tile, None).is_empty());
        // Off the map, a flash has no colour to go towards.
        let off = TileView {
            terrain: None,
            flashes: vec![1.0],
            tints: vec![RangeKind::Attack],
            becomes: None,
        };
        assert_eq!(tints(&c, &off, None), [(attack, OVERLAY_BLEND)]);
    }

    proptest! {
        /// Whatever the scene, the area and the tile size: every tile of
        /// the map that is drawn has exactly one picture of its own, on
        /// its own pixels (none if the tileset has no tile for what it
        /// shows); and every picture, or part of one, lies inside drawn
        /// tiles of the map.
        #[test]
        fn every_tile_of_the_map_has_one_picture_of_its_own_and_none_spills(
            scene in any_scene(),
            area in any_area(),
            tile in (8..=64u32, 8..=64u32),
        ) {
            let c = ctx();
            let mut tileset = with_indoor(&layered(&c)).tileset().clone();
            tileset.terrain.as_mut().unwrap().tile_px = tile;
            let s = SpriteSkin::new(tileset);
            // The ground alone: nothing on it, nothing tinting it.
            let mut scene = scene;
            scene.units.clear();
            scene.cursor = None;
            scene.path.clear();
            for tile in &mut scene.tiles {
                tile.flashes.clear();
                tile.tints.clear();
            }
            let mut buf = blank();
            s.paint(&c, &scene, area, &mut buf);
            prop_assert!(buf.overlays().is_empty());
            let grid = s.grid(&scene, area);
            let shown = Shown::of(&scene);
            let (across, down) = grid.tiles;
            let on_map: Vec<(i32, i32, PxRect)> = (0..down)
                .flat_map(|dy| (0..across).map(move |dx| (dx, dy)))
                .filter(|&(dx, dy)| scene.tile_at(dx, dy).is_some_and(|t| t.terrain.is_some()))
                .map(|(dx, dy)| (dx, dy, grid.rect_at(dx, dy)))
                .collect();
            // A terrain's own tile is in the image's first two rows (the
            // outdoor look) or its fourth (the indoor one); the layers'
            // pictures are below them.
            let sprites = buf.sprites();
            let own = |sprite: &&Sprite| sprite.src.y < 96;
            for &(dx, dy, rect) in &on_map {
                let pictures = sprites.iter().filter(own).filter(|s| s.dest == rect);
                // A tile wholly off the console has nothing to show.
                let seen = rect.intersect(&buf.pixel_bounds()).is_some();
                let has = seen && shown.at(dx, dy).is_some_and(|id| id.0 < 16);
                prop_assert_eq!(pictures.count(), usize::from(has), "tile ({}, {})", dx, dy);
            }
            let area_of = |r: PxRect| i64::from(r.w) * i64::from(r.h);
            for sprite in &sprites {
                let parts = on_map.iter().filter_map(|(_, _, tile)| tile.intersect(&sprite.clip));
                let covered: i64 = parts.map(area_of).sum();
                prop_assert_eq!(covered, area_of(sprite.clip), "{:?}", sprite);
            }
        }
    }
}
