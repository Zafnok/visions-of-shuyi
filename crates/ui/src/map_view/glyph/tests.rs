use proptest::prelude::*;
use trpg_core::{ClassId, Faction, TerrainId, UnitId};

use super::*;
use crate::glyph_buffer::Cell;
use crate::map_view::grid::px_rect;
use shuyi_ui::grid::skin::GridSkin;
use shuyi_ui::grid::{Ends, MapEdges};
use trpg_core::Pos;

use crate::color::Rgb;
use crate::glyph_buffer::Rect;
use crate::map_view::scene::{
    CursorStyle, CursorView, Facing, RangeLayer, RangeName, STANDING_FRAME, ThreatView, TileView,
    UnitEffects, range,
};
use crate::map_view::skin::{MapSkin, with_looks};
use crate::map_view::threat;
use crate::screen::tests::ctx;
use crate::screens::battle::layout::MAP_VIEW;

fn p(x: i32, y: i32) -> Pos {
    Pos::new(x, y)
}

#[test]
fn the_battle_map_area_shows_35_by_30_tiles() {
    assert_eq!(MapSkin::glyph().name(), "glyph");
    assert_eq!(MapSkin::glyph().view_tiles(MAP_VIEW), (35, 30));
    // Where a tile is: the kit's glyph skin's.
    let scene = MapScene::new(Pos::new(-10, -11), (35, 30));
    let tile = MapSkin::glyph().tile_px(&scene, MAP_VIEW, Pos::new(0, 0));
    assert_eq!(tile, Some(Rect::new(160, 176, 16, 16)));
    assert_eq!(
        MapSkin::glyph().tile_px(&scene, MAP_VIEW, Pos::new(25, 18)),
        None
    );
    assert_eq!(px_rect(MAP_VIEW), Rect::new(0, 0, 560, 480));
    // An odd cell is left over; an empty area shows nothing.
    assert_eq!(MapSkin::glyph().view_tiles(Rect::new(3, 4, 7, 5)), (3, 5));
    assert_eq!(MapSkin::glyph().view_tiles(Rect::new(0, 0, -4, -1)), (0, 0));
}

#[test]
fn ranges_have_their_palette_colours() {
    // The kit's skin walks at this game's glyph pace, and its cursor stays
    // above the HP bar.
    assert_eq!(GridSkin::<UnitView>::walk(&GLYPH), GLYPH_WALK);
    assert_eq!(GLYPH.foot_px, units::HP_BAR_H);
    let p = &ctx().palette;
    let colour = |kind: RangeName| p.lookup(kind.0);
    assert_eq!(colour(range::DANGER), Some(p.get(UiColor::DangerZone)));
    assert_eq!(colour(range::MOVE), Some(p.get(UiColor::MoveRange)));
    assert_eq!(colour(range::ATTACK), Some(p.get(UiColor::AttackRange)));
    assert_eq!(colour(range::HEAL), Some(p.get(UiColor::HealRange)));
}

/// A buffer the size of the console, every cell `fill()`.
fn blank() -> GlyphBuffer {
    GlyphBuffer::new(100, 32, fill())
}

fn fill() -> Cell {
    Cell::new('x', Rgb::new(1, 2, 3), Rgb::new(4, 5, 6))
}

fn brigand(pos: Pos) -> UnitView {
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

/// The glyph skin's look doesn't turn, step or glide (ticket 0440): a
/// unit's facing, walking frame and offset change nothing it paints.
#[test]
fn a_units_facing_frame_and_offset_change_nothing() {
    let c = ctx();
    let mut scene = MapScene::new(Pos::new(0, 0), (3, 2));
    scene.push_unit(brigand(Pos::new(1, 1)));
    let paint = |scene: &MapScene| {
        let mut buf = blank();
        MapSkin::glyph().paint(&c, scene, Rect::new(2, 1, 6, 4), &mut buf);
        buf
    };
    let still = paint(&scene);
    assert_ne!(still, blank());
    for facing in [Facing::Left, Facing::Right, Facing::Up] {
        for frame in [0, 2] {
            let mut walking = scene.clone();
            walking.units[0].facing = facing;
            walking.units[0].frame = frame;
            walking.units[0].offset = (0.5, -0.5);
            assert_eq!(paint(&walking), still, "{facing:?} {frame}");
        }
    }
}

/// The threat arrows and marks are the last things painted, over the
/// units, the path and the cursor, on the skin's 16 px tiles.
#[test]
fn threat_arrows_and_marks_are_painted_last_over_everything() {
    let c = ctx();
    let pal = &c.palette;
    let area = Rect::new(10, 5, 8, 3);
    let mut scene = MapScene::new(p(2, 2), (4, 3));
    scene.push_unit(brigand(p(5, 3)));
    scene.path = vec![p(2, 2), p(3, 2)];
    scene.cursor = Some(CursorView {
        pos: p(3, 2),
        brightness: 1.0,
        style: CursorStyle::Corners,
    });
    let mut buf = blank();
    MapSkin::glyph().paint(&c, &scene, area, &mut buf);
    let before = buf.overlays();
    let threat = ThreatView {
        tile: p(3, 2),
        enemies: vec![p(5, 3)],
    };
    scene.threat = Some(threat.clone());
    let mut with = blank();
    MapSkin::glyph().paint(&c, &scene, area, &mut with);
    // No cell changes: the arrows are drawn over the glyphs.
    for y in 0..32 {
        for x in 0..100 {
            assert_eq!(with.get(x, y), buf.get(x, y), "({x}, {y})");
        }
    }
    let colors = with_looks(&c, |looks| looks.threat(&MAP_COLORS));
    assert_eq!(colors.arrow, pal.get(UiColor::Enemy));
    assert_eq!(colors.mark, pal.get(UiColor::TextHighlight));
    assert_eq!(colors.edge, pal.get(UiColor::Black));
    let grid = Layout::new(&scene, area).grid();
    let expect = threat::threat_overlays(&threat, &grid, threat::Start::Edge, colors);
    let after = with.overlays();
    assert_eq!(&after[..before.len()], before);
    assert_eq!(&after[before.len()..], expect);
    // The arrow's line starts just off the brigand's tile (cells
    // (16, 6) and (17, 6): pixels 128..144 × 96..112), on its way up
    // and left from the tile's middle, and no red is on the tile; the
    // mark is above that tile, 3 px clear of it.
    let red: Vec<_> = expect.iter().filter(|o| o.color == colors.arrow).collect();
    assert_eq!(red[0].rect, Rect::new(127, 100, 1, 1));
    let tile = Rect::new(128, 96, 16, 16);
    assert!(red.iter().all(|o| o.rect.intersect(&tile).is_none()));
    assert!(
        red.iter()
            .all(|o| o.layer == crate::glyph_buffer::Layer::Over)
    );
    let yellow: Vec<_> = expect.iter().filter(|o| o.color == colors.mark).collect();
    assert_eq!(yellow[0].rect, Rect::new(134, 84, 3, 6));
    assert_eq!(yellow[1].rect, Rect::new(134, 91, 3, 2));
    assert_eq!(yellow.len(), 2);
}

#[test]
fn paints_terrain_then_tints_then_units_path_and_cursor() {
    let c = ctx();
    let pal = &c.palette;
    let display = &c.game.content.terrain.display;
    let plain_id = display.id_of("plain").unwrap();
    let plain = display.get(plain_id).unwrap();
    let (fg, bg) = (
        pal.lookup(&plain.fg).unwrap(),
        pal.lookup(&plain.bg).unwrap(),
    );
    let area = Rect::new(10, 5, 8, 3);
    let mut scene = MapScene::new(p(2, 2), (4, 3));
    for pos in [p(2, 2), p(3, 2), p(4, 2), p(5, 2), p(2, 3)] {
        scene.tile_mut(pos).unwrap().terrain = Some(plain_id);
    }
    // A terrain the content lacks is left blank, as off the map.
    scene.tile_mut(p(3, 3)).unwrap().terrain = Some(TerrainId(999));
    scene.tint([p(3, 2)], range::DANGER);
    scene.tint([p(3, 2), p(4, 2)], range::MOVE);
    // A range off the map still tints.
    scene.tint([p(4, 4)], range::HEAL);
    scene.push_unit(brigand(p(4, 2)));
    // A highlighted unit: its tile's colours swapped.
    scene.push_unit(brigand(p(2, 3)).highlighted(true));
    scene.path = vec![p(2, 2), p(3, 2)];
    scene.cursor = Some(CursorView {
        pos: p(5, 2),
        brightness: 1.0,
        style: CursorStyle::Corners,
    });
    let mut buf = blank();
    MapSkin::glyph().paint(&c, &scene, area, &mut buf);
    let cell = |x, y| *buf.get(x, y).unwrap();
    // Tile (2, 2) at cells (10, 5) and (11, 5): plain.
    assert_eq!(cell(10, 5), Cell::new(plain.glyphs[0], fg, bg));
    assert_eq!(cell(11, 5), Cell::new(plain.glyphs[1], fg, bg));
    // (3, 2): danger under move.
    let blend = |bg: Rgb, kind: RangeName| bg.lerp(pal.lookup(kind.0).unwrap(), OVERLAY_BLEND);
    let both = blend(blend(bg, range::DANGER), range::MOVE);
    assert_eq!(cell(12, 5), Cell::new(plain.glyphs[0], fg, both));
    assert_eq!(cell(13, 5).bg, both);
    // (4, 2): the unit's letters on the tinted tile.
    let moved = blend(bg, range::MOVE);
    assert_eq!(cell(14, 5), Cell::new('B', pal.get(UiColor::Enemy), moved));
    assert_eq!(cell(15, 5), Cell::new('r', pal.get(UiColor::Enemy), moved));
    // (2, 3) plain; (3, 3) unknown and (4, 3) off the map: untouched;
    // (4, 4) off the map but tinted.
    assert_eq!(cell(10, 6), Cell::new('B', bg, pal.get(UiColor::Enemy)));
    assert_eq!(cell(11, 6), Cell::new('r', bg, pal.get(UiColor::Enemy)));
    assert_eq!(cell(12, 6), fill());
    assert_eq!(cell(14, 6), fill());
    let healed = Cell {
        bg: blend(fill().bg, range::HEAL),
        ..fill()
    };
    assert_eq!((cell(14, 7), cell(15, 7)), (healed, healed));
    assert_eq!(cell(16, 7), fill());
    // Overlays in order: the unit's HP bar, the path, the cursor.
    let colors: Vec<Rgb> = buf.overlays().iter().map(|o| o.color).collect();
    let path = pal.get(UiColor::Path);
    let hp = pal.get(UiColor::HpHigh);
    let mut expect = vec![hp, hp, path];
    expect.extend([path; 6]);
    expect.extend([pal.get(UiColor::Cursor); 8]);
    assert_eq!(colors, expect);
    // The HP bar under the unit: cells 14..16 of row 5, a pixel in
    // from each side.
    assert_eq!(buf.overlays()[0].rect, Rect::new(113, 94, 14, 2));
}

#[test]
fn an_arrow_in_the_cursors_colour_on_each_edge_the_map_goes_on_past() {
    let c = ctx();
    let pal = &c.palette;
    let plain_id = c.game.content.terrain.display.id_of("plain").unwrap();
    let plain = c.game.content.terrain.display.get(plain_id).unwrap();
    let bg = pal.lookup(&plain.bg).unwrap();
    let area = Rect::new(10, 5, 8, 3);
    let mut scene = MapScene::new(p(2, 2), (4, 3));
    scene.set_terrain(|_| Some(plain_id));
    // On the right edge's tile: the second letter of its label gives way.
    scene.push_unit(brigand(p(5, 3)));
    let paint = |scene: &MapScene| {
        let mut buf = blank();
        MapSkin::glyph().paint(&c, scene, area, &mut buf);
        buf
    };
    let none = paint(&scene);
    let all = Ends {
        before: true,
        after: true,
    };
    scene.more = MapEdges { x: all, y: all };
    let arrows = [
        ((17, 6), '→'),
        ((10, 6), '←'),
        ((12, 7), '↓'),
        ((12, 5), '↑'),
    ];
    for (clock_ms, k) in [(0, 1.0), (500, edges::FAINTEST)] {
        scene.clock_ms = clock_ms;
        let buf = paint(&scene);
        let fg = bg.lerp(pal.get(UiColor::Cursor), k);
        for ((x, y), glyph) in arrows {
            assert_eq!(buf.get(x, y), Some(&Cell::new(glyph, fg, bg)), "{glyph}");
        }
        // Every other cell is as without the arrows; the overlays too.
        for y in 5..8 {
            for x in 10..18 {
                if !arrows.iter().any(|&(at, _)| at == (x, y)) {
                    assert_eq!(buf.get(x, y), none.get(x, y), "({x}, {y})");
                }
            }
        }
        assert_eq!(buf.overlays(), none.overlays());
    }
    assert_eq!(none.get(16, 6).map(|c| c.glyph), Some('B'));
}

#[test]
fn a_changed_tile_flashes_and_a_tile_a_spell_would_change_shows_what_it_becomes() {
    let c = ctx();
    let pal = &c.palette;
    let display = &c.game.content.terrain.display;
    let look = |name: &str| {
        let t = display.get(display.id_of(name).unwrap()).unwrap();
        let (fg, bg) = (pal.lookup(&t.fg).unwrap(), pal.lookup(&t.bg).unwrap());
        (t.glyphs, fg, bg)
    };
    let (plain, forest) = (look("plain"), look("forest"));
    let mut scene = MapScene::new(p(0, 0), (5, 1));
    for tile in &mut scene.tiles {
        tile.terrain = display.id_of("plain");
    }
    // Half a flash; a full flash under a range; two flashes at once.
    scene.tile_mut(p(0, 0)).unwrap().flashes = vec![0.5];
    scene.tile_mut(p(1, 0)).unwrap().flashes = vec![1.0];
    scene.tile_mut(p(4, 0)).unwrap().flashes = vec![1.0, 0.5];
    scene.tint([p(1, 0), p(2, 0), p(3, 0)], range::ATTACK);
    // A forest-to-be, and a terrain the content lacks.
    scene.tile_mut(p(2, 0)).unwrap().becomes = display.id_of("forest");
    scene.tile_mut(p(3, 0)).unwrap().becomes = Some(TerrainId(999));
    let mut buf = blank();
    MapSkin::glyph().paint(&c, &scene, Rect::new(0, 0, 10, 1), &mut buf);
    let cell = |x| *buf.get(x, 0).unwrap();
    let (glyphs, fg, bg) = plain;
    let attack = pal.get(UiColor::AttackRange);
    // Towards the tile's own glyph colour, by the flash's strength.
    let half = bg.lerp(fg, OVERLAY_BLEND * 0.5);
    assert_eq!(cell(0), Cell::new(glyphs[0], fg, half));
    assert_eq!(cell(1), Cell::new(glyphs[1], fg, half));
    // The range goes over the flash.
    let full = bg.lerp(fg, OVERLAY_BLEND);
    assert_eq!(cell(2).bg, full.lerp(attack, OVERLAY_BLEND));
    // What it would become replaces the tile, range and all…
    assert_eq!(cell(4), Cell::new(forest.0[0], forest.1, forest.2));
    assert_eq!(cell(5), Cell::new(forest.0[1], forest.1, forest.2));
    // …unless the content lacks it.
    assert_eq!(
        cell(6),
        Cell::new(glyphs[0], fg, bg.lerp(attack, OVERLAY_BLEND))
    );
    // Each flash tints in turn.
    assert_eq!(cell(8).bg, full.lerp(fg, OVERLAY_BLEND * 0.5));
    assert_ne!(forest, plain);
}

#[test]
fn units_and_the_cursor_off_the_view_are_not_drawn() {
    let c = ctx();
    let area = Rect::new(0, 0, 4, 1);
    let mut scene = MapScene::new(p(0, 0), (2, 1));
    // Not through `push_unit`: a scene built by hand.
    scene.units.push(brigand(p(2, 0)));
    scene.cursor = Some(CursorView {
        pos: p(0, 1),
        brightness: 1.0,
        style: CursorStyle::TileGlow,
    });
    let mut buf = blank();
    MapSkin::glyph().paint(&c, &scene, area, &mut buf);
    assert_eq!(buf, blank());
}

prop_compose! {
    pub(crate) fn any_pos()(x in -25..60i32, y in -25..60i32) -> Pos {
        Pos::new(x, y)
    }
}

prop_compose! {
    pub(crate) fn any_tile()(
        // Past the last terrain too: an id no terrain has.
        terrain in prop::option::of(0u16..36),
        flashes in prop::collection::vec(0.0f32..1.0, 0..2),
        becomes in prop::option::of(0u16..36),
    ) -> TileView {
        TileView {
            terrain: terrain.map(TerrainId),
            flashes,
            becomes: becomes.map(TerrainId),
        }
    }
}

prop_compose! {
    pub(crate) fn any_range()(
        kind in prop::sample::select(range::ALL.to_vec()),
        tiles in prop::collection::vec(any_pos(), 0..30),
    ) -> RangeLayer {
        RangeLayer { kind, tiles }
    }
}

prop_compose! {
    pub(crate) fn any_unit()(
        pos in any_pos(),
        label in "[A-Za-z]{0,4}",
        hp in -5..40i32,
        flags in 0u8..16,
        fade in -0.5f32..1.5,
        facing in prop::sample::select(vec![
            Facing::Down,
            Facing::Left,
            Facing::Right,
            Facing::Up,
        ]),
        frame in 0u8..5,
        offset in prop::option::of((-1.5f32..1.5, -1.5f32..1.5)),
    ) -> UnitView {
        UnitView {
            label,
            acted: flags & 1 != 0,
            fight: crate::map_view::scene::Fight::In,
            effects: UnitEffects {
                bonus: flags & 2 != 0,
                penalty: flags & 8 != 0,
            },
            highlight: flags & 4 != 0,
            hp: (hp, 30),
            fade,
            facing,
            frame,
            offset: offset.unwrap_or_default(),
            ..brigand(pos)
        }
    }
}

prop_compose! {
    pub(crate) fn any_cursor()(
        pos in any_pos(),
        brightness in 0.5f32..1.0,
        style in prop::sample::select(vec![
            CursorStyle::Corners,
            CursorStyle::LargeCorners,
            CursorStyle::TileGlow,
        ]),
    ) -> CursorView {
        CursorView { pos, brightness, style }
    }
}

prop_compose! {
    pub(crate) fn any_scene()(
        origin in any_pos(),
        size in (-2..45i32, -2..40i32),
        tiles in prop::collection::vec(any_tile(), 0..200),
        ranges in prop::collection::vec(any_range(), 0..4),
        rim in prop::collection::vec(prop::option::of(0u16..36), 0..180),
        look in prop::sample::select(vec!["outdoor", "indoor", "cave"]),
        light in prop::sample::select(vec!["noon", "dusk", "night", "midnight"]),
        units in prop::collection::vec(any_unit(), 0..8),
        cursor in prop::option::of(any_cursor()),
        path in prop::collection::vec(any_pos(), 0..6),
        threat in prop::option::of((any_pos(), prop::collection::vec(any_pos(), 1..4))),
        clock_ms in 0u64..5000,
        more in any::<[bool; 4]>(),
    ) -> MapScene {
        let rim = rim.into_iter().map(|id| id.map(TerrainId)).collect();
        let look = trpg_content::MapLook { tiles: look.to_owned(), light: light.to_owned() };
        let threat = threat.map(|(tile, enemies)| ThreatView { tile, enemies });
        let [left, right, up, down] = more;
        let more = MapEdges {
            x: Ends { before: left, after: right },
            y: Ends { before: up, after: down },
        };
        MapScene { origin, size, more, tiles, ranges, rim, look, units, cursor, path, threat, clock_ms }
    }
}

proptest! {
    /// Whatever the scene and the area (even one partly off the
    /// console), the skin changes no cell outside the area and adds no
    /// item (rectangle or sprite) outside its pixels.
    #[test]
    fn glyph_skin_paints_only_the_area(
        scene in any_scene(),
        area in (-6..104i32, -4..34i32, -2..110i32, -2..40i32),
    ) {
        let c = ctx();
        let area = Rect::new(area.0, area.1, area.2, area.3);
        let mut buf = blank();
        MapSkin::glyph().paint(&c, &scene, area, &mut buf);
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
}
