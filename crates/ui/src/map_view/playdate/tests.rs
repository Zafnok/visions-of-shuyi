//! The Playdate map skin's look (ADR-0038: tests of a look read cells,
//! items and pixels, and live with the skin), through the one-bit renderer
//! (ADR-0074) where a mark is a pixel pattern.

use proptest::prelude::*;
use trpg_content::Ink;
use trpg_core::{ClassId, Faction, UnitId};

use super::*;
use crate::map_view::glyph::tests::any_scene;
use crate::map_view::place::Place;
use crate::map_view::scene::{
    CursorStyle, CursorView, Facing, STANDING_FRAME, ThreatView, UnitEffects, UnitView,
};
use crate::onebit::{FRAME_BYTES, Font, Frame, ink_is_white, is_white, render};
use crate::prelude::*;
use crate::screen::ShuyiCtx;
use crate::screen::tests::ctx;
use crate::screens::battle::layout::MAP_VIEW;

/// The Playdate's whole screen, in cells.
pub(crate) const SCREEN: Rect = Rect::new(0, 0, 50, 15);

pub(crate) fn p(x: i32, y: i32) -> Pos {
    Pos::new(x, y)
}

/// The colour that is `ink`'s code.
pub(crate) fn c(ink: Ink) -> Rgb {
    Rgb::from(ink.code())
}

/// The test context in the Playdate's palette and skins.
pub(crate) fn pd_ctx() -> Ctx {
    let mut ctx = ctx();
    let def = trpg_content::ink::load_playdate(&ctx.game.content.palette).unwrap();
    ctx.palette = Palette::new::<UiColor>(&def).unwrap();
    ctx.use_playdate_skins();
    ctx
}

/// The Playdate's screen, blank: the white ground.
pub(crate) fn blank() -> GlyphBuffer {
    GlyphBuffer::new(50, 15, Cell::new(' ', c(Ink::Black), c(Ink::White)))
}

/// `scene` painted on the whole screen.
pub(crate) fn painted(ctx: &Ctx, scene: &MapScene) -> GlyphBuffer {
    let mut buf = blank();
    PlaydateSkin.paint(ctx, scene, SCREEN, &mut buf);
    buf
}

/// The game's one-bit font.
pub(crate) fn font() -> Font<'static> {
    let atlas = Box::leak(Box::new(trpg_content::font::load().unwrap()));
    let bits = trpg_content::bundle::bytes(trpg_content::font::ATLAS_1BIT_PATH).unwrap();
    Font::new(atlas, bits).unwrap()
}

/// `buf` as the Playdate shows it.
pub(crate) fn shown(buf: &GlyphBuffer) -> Box<Frame> {
    let mut frame = Box::new([0; FRAME_BYTES]);
    render(buf, &font(), &(), &mut frame);
    frame
}

/// Whether pixel `(x, y)` of a glyph cell is inked by `glyph`.
pub(crate) fn inked(font: &Font<'_>, glyph: char, x: i32, y: i32) -> bool {
    let rows = font.rows(glyph).unwrap();
    (0..8).contains(&x) && usize::try_from(y).is_ok_and(|y| rows[y] >> (7 - x) & 1 == 1)
}

/// The pixels of the tile whose left cell is `(cx, cy)`, each `(x, y)` on
/// the screen with whether one of `glyphs` inks it and whether one is
/// beside it (the eight pixels round it), of the cell it is in.
pub(crate) fn tile_pixels(
    glyphs: [char; 2],
    (cx, cy): (i32, i32),
) -> Vec<((i32, i32), bool, bool)> {
    let font = font();
    let at = |x: i32, y: i32| {
        let (dx, row) = (x - cx * 8, y - cy * 16);
        let i = dx.div_euclid(8);
        (0..2).contains(&i)
            && (0..16).contains(&row)
            && inked(&font, glyphs[usize::try_from(i).unwrap()], dx - 8 * i, row)
    };
    let mut out = Vec::new();
    for y in cy * 16..cy * 16 + 16 {
        for x in cx * 8..cx * 8 + 16 {
            let near =
                (-1..=1).any(|oy| (-1..=1).any(|ox| (ox, oy) != (0, 0) && at(x + ox, y + oy)));
            out.push(((x, y), at(x, y), near));
        }
    }
    out
}

pub(crate) fn unit(faction: Faction, pos: Pos) -> UnitView {
    UnitView {
        id: UnitId(4),
        pos,
        faction,
        label: "Br".into(),
        class: ClassId("brigand".into()),
        character: None,
        acted: false,
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

/// A 24 × 15 view from the map's corner, every tile plain.
pub(crate) fn plains(ctx: &Ctx) -> MapScene {
    let mut scene = MapScene::new(p(0, 0), (24, 15));
    let plain = ctx.game.content.terrain.display.id_of("plain");
    for tile in &mut scene.tiles {
        tile.terrain = plain;
    }
    scene
}

#[test]
fn the_playdate_screen_shows_24_by_15_tiles() {
    let s = PlaydateSkin;
    assert_eq!(s.name(), "playdate");
    assert_eq!(s.view_tiles(SCREEN), (24, 15));
    // A cell kept at each side: 3 cells hold no tile, 4 hold one.
    assert_eq!(s.view_tiles(Rect::new(0, 0, 3, 1)), (0, 1));
    assert_eq!(s.view_tiles(Rect::new(0, 0, 4, 2)), (1, 2));
    assert_eq!(s.view_tiles(Rect::new(0, 0, 51, 15)), (24, 15));
    assert_eq!(s.view_tiles(Rect::new(0, 0, 52, 15)), (25, 15));
    assert_eq!(s.view_tiles(Rect::new(0, 0, -4, -1)), (0, 0));
    // The PC's battle map area, until 1038 lays the battle out.
    assert_eq!(s.view_tiles(MAP_VIEW), (34, 30));
    // It can't zoom.
    assert_eq!(s.zoom(), None);
    assert!(s.at_zoom(crate::settings::MapZoom::X2).is_none());
}

#[test]
fn the_tiles_are_16_px_squares_in_the_middle_of_the_area() {
    let scene = MapScene::new(p(0, 0), (24, 15));
    let px = |area, tile| PlaydateSkin.tile_px(&scene, area, tile);
    // One cell (8 px) clear at each side of the screen.
    assert_eq!(px(SCREEN, p(0, 0)), Some(Rect::new(8, 0, 16, 16)));
    assert_eq!(px(SCREEN, p(23, 14)), Some(Rect::new(376, 224, 16, 16)));
    assert_eq!(px(SCREEN, p(24, 0)), None);
    assert_eq!(px(SCREEN, p(0, 15)), None);
    assert_eq!(px(SCREEN, p(-1, 0)), None);
    // An odd cell over goes to the right; another area starts at its corner.
    assert_eq!(
        px(Rect::new(0, 0, 51, 15), p(0, 0)),
        Some(Rect::new(8, 0, 16, 16))
    );
    let area = Rect::new(4, 3, 9, 2);
    assert_eq!(px(area, p(0, 0)), Some(Rect::new(40, 48, 16, 16)));
    assert_eq!(px(area, p(2, 1)), Some(Rect::new(72, 64, 16, 16)));
    assert_eq!(px(area, p(3, 0)), None);
    assert_eq!(
        PlaydateSkin.tile_cells(&scene, area, p(1, 1)),
        Some(Rect::new(7, 4, 2, 1))
    );
}

#[test]
fn the_inks_are_the_palettes_names_or_the_designs() {
    let design = Inks {
        player: (c(Ink::Black), c(Ink::White)),
        enemy: (c(Ink::Grey), c(Ink::BlackEdged)),
        acted: (c(Ink::White), c(Ink::Black), c(Ink::Black)),
        other: c(Ink::BlackEdged),
        marked: c(Ink::BlackEdged),
        highlight: c(Ink::Invert),
        arrows: (c(Ink::Black), c(Ink::White)),
    };
    // The Playdate's palette names them; the PC's doesn't.
    assert_eq!(Inks::of(&pd_ctx().palette), design);
    assert_eq!(Inks::of(&ctx().palette), design);
    // A palette that says otherwise wins.
    let base = ctx().game.content.palette;
    let mut def = trpg_content::ink::load_playdate(&base).unwrap();
    let names = [
        "map_player_tile",
        "map_player_letters",
        "map_enemy_tile",
        "map_enemy_letters",
        "map_acted_tile",
        "map_acted_letters",
        "map_acted_outline",
        "map_other_letters",
        "map_marked_glyph",
        "map_highlight",
        "map_threat",
        "map_threat_edge",
    ];
    for name in names {
        def.colors.insert(name.to_owned(), Ink::Dots.code());
    }
    let dots = c(Ink::Dots);
    let all = Inks::of(&Palette::new::<UiColor>(&def).unwrap());
    assert_eq!(
        all,
        Inks {
            player: (dots, dots),
            enemy: (dots, dots),
            acted: (dots, dots, dots),
            other: dots,
            marked: dots,
            highlight: dots,
            arrows: (dots, dots),
        }
    );
    let threat = design.threat();
    assert_eq!(
        (threat.arrow, threat.mark, threat.edge),
        (c(Ink::Black), c(Ink::Black), c(Ink::White))
    );
}

/// Terrain is its PC glyphs, black on the white ground; off the map is
/// left as it was.
#[test]
fn terrain_is_its_glyphs_black_on_white() {
    let ctx = pd_ctx();
    let display = &ctx.game.content.terrain.display;
    let mut scene = MapScene::new(p(0, 0), (24, 15));
    scene.tile_mut(p(0, 0)).unwrap().terrain = display.id_of("plain");
    scene.tile_mut(p(1, 0)).unwrap().terrain = display.id_of("forest");
    // A terrain the content lacks: left blank, as off the map.
    scene.tile_mut(p(2, 0)).unwrap().terrain = Some(TerrainId(999));
    let buf = painted(&ctx, &scene);
    for (name, cx) in [("plain", 1), ("forest", 3)] {
        let glyphs = display.get(display.id_of(name).unwrap()).unwrap().glyphs;
        assert_eq!(
            buf.get(cx, 0),
            Some(&Cell::new(glyphs[0], c(Ink::Black), c(Ink::White)))
        );
        assert_eq!(buf.get(cx + 1, 0).unwrap().glyph, glyphs[1]);
        let frame = shown(&buf);
        for ((x, y), ink, _) in tile_pixels(glyphs, (cx, 0)) {
            assert_eq!(is_white(&frame, x, y), !ink, "{name} ({x}, {y})");
        }
    }
    for x in [0, 5, 6, 49] {
        assert_eq!(buf.get(x, 0), blank().get(x, 0), "cell {x}");
    }
    assert!(buf.items().is_empty());
}

/// The move range is dots, the attack range diagonal lines (and the heal
/// range too, until the design says), the one laid on last showing; the
/// danger zone fills nothing. A filled tile's glyphs keep a white edge.
#[test]
fn ranges_fill_their_tiles_with_dots_and_lines_and_edge_the_glyphs() {
    use RangeKind::{Attack, Danger, Heal, Move};
    let ctx = pd_ctx();
    let mut scene = plains(&ctx);
    let tiles: [(&[RangeKind], Option<Ink>); 6] = [
        (&[Move], Some(Ink::Dots)),
        (&[Attack], Some(Ink::Diagonals)),
        (&[Heal], Some(Ink::Diagonals)),
        (&[Move, Attack], Some(Ink::Diagonals)),
        (&[Attack, Move, Danger], Some(Ink::Dots)),
        (&[Danger], None),
    ];
    for (x, (kinds, _)) in (0..).zip(&tiles) {
        scene.tile_mut(p(x, 0)).unwrap().tints = kinds.to_vec();
    }
    let buf = painted(&ctx, &scene);
    let frame = shown(&buf);
    let display = &ctx.game.content.terrain.display;
    let glyphs = display.get(display.id_of("plain").unwrap()).unwrap().glyphs;
    for (x, (kinds, fill)) in (0..).zip(&tiles) {
        let cx = 1 + 2 * x;
        let (fg, bg) = match fill {
            Some(ink) => (c(Ink::BlackEdged), c(*ink)),
            None => (c(Ink::Black), c(Ink::White)),
        };
        for (i, glyph) in (0..).zip(glyphs) {
            assert_eq!(
                buf.get(cx + i, 0),
                Some(&Cell::new(glyph, fg, bg)),
                "{kinds:?}"
            );
        }
        for ((px, py), ink, near) in tile_pixels(glyphs, (cx, 0)) {
            let white = match fill {
                _ if ink => false,
                Some(_) if near => true,
                Some(ink) => ink_is_white(*ink, px, py),
                None => true,
            };
            // The danger zone's edge (dashes) is checked with the marks.
            let tile_x = px - (8 + 16 * x);
            let edge =
                kinds.contains(&Danger) && !((2..14).contains(&py) && (2..14).contains(&tile_x));
            if fill.is_some() && !edge {
                assert_eq!(is_white(&frame, px, py), white, "{kinds:?} ({px}, {py})");
            }
        }
    }
    // A range off the map fills its tile too.
    let mut off = MapScene::new(p(0, 0), (24, 15));
    off.tint([p(3, 2)], Move);
    let buf = painted(&ctx, &off);
    assert_eq!(buf.get(7, 2).unwrap().bg, c(Ink::Dots));
    assert_eq!(buf.get(8, 2).unwrap().bg, c(Ink::Dots));
    assert_eq!(buf.get(9, 2).unwrap().bg, c(Ink::White));
}

/// A tile a spell would change shows what it would become, range and all
/// (as on the glyph skin); a tile whose terrain just changed is flipped
/// for the first half of its flash.
#[test]
fn a_tile_shows_what_it_becomes_and_flips_while_it_flashes() {
    let ctx = pd_ctx();
    let display = &ctx.game.content.terrain.display;
    let forest = display.get(display.id_of("forest").unwrap()).unwrap();
    let mut scene = plains(&ctx);
    scene.tint([p(0, 0), p(1, 0)], RangeKind::Attack);
    scene.tile_mut(p(0, 0)).unwrap().becomes = display.id_of("forest");
    // Unknown: the tile as it is.
    scene.tile_mut(p(1, 0)).unwrap().becomes = Some(TerrainId(999));
    let flashes: [&[f32]; 5] = [&[0.5], &[1.0], &[0.49], &[f32::NAN], &[0.25, 0.75]];
    for (x, strengths) in (2..).zip(flashes) {
        scene.tile_mut(p(x, 0)).unwrap().flashes = strengths.to_vec();
    }
    let buf = painted(&ctx, &scene);
    let black = c(Ink::Black);
    assert_eq!(
        buf.get(1, 0),
        Some(&Cell::new(forest.glyphs[0], black, c(Ink::White)))
    );
    assert_eq!(buf.get(2, 0).unwrap().glyph, forest.glyphs[1]);
    assert_eq!(buf.get(3, 0).unwrap().bg, c(Ink::Diagonals));
    let flipped: Vec<Rect> = buf.overlays().iter().map(|o| o.rect).collect();
    let tile = |x: i32| Rect::new(8 + 16 * x, 0, 16, 16);
    assert_eq!(flipped, [tile(2), tile(3), tile(6)]);
    assert!(buf.overlays().iter().all(|o| o.color == c(Ink::Invert)));
    // On the screen, the flipped tile is black where a plain tile is white.
    let frame = shown(&buf);
    assert!(!is_white(&frame, 8 + 32 + 15, 15));
    assert!(is_white(&frame, 8 + 64 + 15, 15));
}

/// The threat arrows and their marks are the shared ones, black with a
/// white edge, last over everything.
#[test]
fn threat_arrows_are_black_edged_in_white_over_everything() {
    let ctx = pd_ctx();
    let mut scene = plains(&ctx);
    scene.push_unit(unit(Faction::Enemy, p(5, 3)));
    scene.cursor = Some(CursorView {
        pos: p(2, 2),
        brightness: 1.0,
        style: CursorStyle::Corners,
    });
    let before = painted(&ctx, &scene).overlays();
    let threat = ThreatView {
        tile: p(2, 2),
        enemies: vec![p(5, 3)],
    };
    scene.threat = Some(threat.clone());
    let after = painted(&ctx, &scene).overlays();
    assert_eq!(&after[..before.len()], before);
    let grid = layout(&scene, SCREEN).grid();
    let inks = Inks::of(&ctx.palette);
    let expect = threat::threat_overlays(&threat, &grid, threat::Start::Edge, inks.threat());
    assert_eq!(&after[before.len()..], expect);
    assert!(!expect.is_empty());
    let colors: Vec<Rgb> = expect.iter().map(|o| o.color).collect();
    assert!(colors.contains(&c(Ink::Black)) && colors.contains(&c(Ink::White)));
}

/// The path is the shared arrow, in the palette's path ink (black).
#[test]
fn the_path_is_the_shared_arrow_in_black() {
    let ctx = pd_ctx();
    let mut scene = plains(&ctx);
    scene.path = vec![p(1, 1), p(2, 1), p(2, 2)];
    let buf = painted(&ctx, &scene);
    let grid = layout(&scene, SCREEN).grid();
    let expect = path::path_overlays(&scene.path, &grid, c(Ink::Black));
    assert_eq!(buf.overlays(), expect);
    assert!(!expect.is_empty());
}

/// Units, the cursor and marks off the view are not drawn.
#[test]
fn nothing_off_the_view_is_drawn() {
    let ctx = pd_ctx();
    let mut scene = MapScene::new(p(0, 0), (2, 1));
    scene.units.push(unit(Faction::Player, p(2, 0)));
    scene.cursor = Some(CursorView {
        pos: p(0, 1),
        brightness: 1.0,
        style: CursorStyle::TileGlow,
    });
    assert_eq!(painted(&ctx, &scene), blank());
}

/// Chapter 1's first battle, Harrowby Ford, as it opens on the Playdate:
/// the view round the lead, the party and the enemies, the cursor on the
/// lead. The frame as braille (each character 2 × 4 pixels), so the
/// snapshot is the picture.
#[test]
fn harrowby_fords_opening_view() {
    let ctx = pd_ctx();
    let content = &ctx.game.content;
    let lead = trpg_core::LeadProfile::new("Ellery", trpg_core::LeadGender::Male);
    let campaign = trpg_content::new_campaign(content, trpg_core::GameMode::Classic, lead);
    let setup = campaign.battle_setup(&content.battles["ch01"], &content.tables());
    let (state, _) = trpg_core::BattleState::new(setup);
    let mut place = Place::of_battle(content, &content.battles["ch01"]);
    place.units = state.units().iter().map(UnitView::of).collect();
    let lead = state.units().iter().find(|u| u.is_lord).unwrap().pos;
    let mut scene = place.scene(lead, PlaydateSkin.view_tiles(SCREEN));
    scene.cursor = Some(CursorView {
        pos: lead,
        brightness: 1.0,
        style: CursorStyle::Corners,
    });
    let frame = shown(&painted(&ctx, &scene));
    insta::assert_snapshot!(braille(&frame));
}

/// `frame` as braille characters, each 2 pixels across and 4 down: a dot
/// is a black pixel.
pub(crate) fn braille(frame: &Frame) -> String {
    // The bit of each pixel of a 2 × 4 braille cell, by row then column.
    const DOTS: [[u32; 2]; 4] = [[0x01, 0x08], [0x02, 0x10], [0x04, 0x20], [0x40, 0x80]];
    let mut out = String::new();
    for row in 0..60 {
        for col in 0..200 {
            let mut bits = 0;
            for (dy, line) in (0..).zip(DOTS) {
                for (dx, bit) in (0..).zip(line) {
                    if !is_white(frame, 2 * col + dx, 4 * row + dy) {
                        bits |= bit;
                    }
                }
            }
            out.push(char::from_u32(0x2800 + bits).unwrap_or(' '));
        }
        out.push('\n');
    }
    out
}

proptest! {
    /// Whatever the scene and the area (even one partly off the
    /// screen), the skin changes no cell outside the area and adds no
    /// item outside its pixels.
    #[test]
    fn the_playdate_skin_paints_only_the_area(
        scene in any_scene(),
        area in (-6..54i32, -4..17i32, -2..60i32, -2..20i32),
    ) {
        let ctx = pd_ctx();
        let area = Rect::new(area.0, area.1, area.2, area.3);
        let (mut buf, empty) = (blank(), blank());
        PlaydateSkin.paint(&ctx, &scene, area, &mut buf);
        for y in 0..15 {
            for x in 0..50 {
                if !area.contains(x, y) {
                    prop_assert_eq!(buf.get(x, y), empty.get(x, y), "cell ({}, {})", x, y);
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
