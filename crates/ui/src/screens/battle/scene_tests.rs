//! Tests of the battle screen's map scene (ADR-0038, ticket 0432): what each
//! mode puts on the map, as data, and that the view is as big as the map
//! skin says. (The skill and item target modes are in `skill_tests.rs` and
//! `item_tests.rs`, the rewind screen's map in `rewind_tests.rs`.)

use std::rc::Rc;

use proptest::prelude::*;
use trpg_core::{BattleMap, Grid, Pos, TerrainId, UnitId};

use super::ai_phase::{AiAction, PACING, Pacing};
use super::layout::MAP_VIEW;
use super::mode::Mode;
use super::testing::{battle, quick_units, skirmish, vaulted};
use super::walk::{SPRITE_HELD_WALK_TILES_PER_S, SPRITE_WALK_TILES_PER_S};
use super::{BattleScreen, quick_battle};
use crate::color::{Rgb, UiColor};
use crate::console::{CONSOLE_H, CONSOLE_W};
use crate::glyph_buffer::{Cell, GlyphBuffer, PxRect, Rect};
use crate::input::Action;
use crate::map_view::{CursorStyle, Facing, GlyphSkin, MapScene, MapSkin, RangeKind};
use crate::screen::tests::ctx;
use crate::screen::{Ctx, FrameInput, Screen};

fn p(x: i32, y: i32) -> Pos {
    Pos::new(x, y)
}

fn quick() -> BattleScreen {
    BattleScreen::new(quick_battle(&ctx().content).unwrap())
}

/// One frame of no time with `actions`.
fn step(s: &mut BattleScreen, c: &mut Ctx, actions: &[Action]) {
    s.update(c, &FrameInput::new(actions.to_vec(), 0.0, vec![]));
}

/// One frame of `seconds` with no keys.
fn wait(s: &mut BattleScreen, c: &mut Ctx, seconds: f32) {
    s.update(c, &FrameInput::new(vec![], seconds, vec![]));
}

/// `tiles` row by row, as [`MapScene::tinted`] lists them.
fn rows(tiles: impl IntoIterator<Item = Pos>) -> Vec<Pos> {
    let mut tiles: Vec<Pos> = tiles.into_iter().collect();
    tiles.sort_by_key(|p| (p.y, p.x));
    tiles
}

/// Every range on the scene, as `(kind, how many tiles)`.
fn ranges(scene: &MapScene) -> Vec<(RangeKind, usize)> {
    use RangeKind::{Attack, Danger, Heal, Move};
    [Danger, Move, Attack, Heal]
        .into_iter()
        .map(|k| (k, scene.tinted(k).len()))
        .filter(|&(_, n)| n > 0)
        .collect()
}

#[test]
fn browsing_shows_the_terrain_the_units_and_the_cursor() {
    let mut c = ctx();
    let s = quick();
    let scene = s.scene(&c);
    // The game's clock, in milliseconds, for a skin's moving marks.
    assert_eq!(scene.clock_ms, 0);
    c.clock_s = 1.5;
    assert_eq!(s.scene(&c).clock_ms, 1500);
    c.clock_s = -3.0;
    assert_eq!(s.scene(&c).clock_ms, 0);
    c.clock_s = 0.0;
    // `test_small` (14 × 8) centred in the glyph skin's 35 × 30 tiles.
    assert_eq!((scene.origin, scene.size), (p(-10, -11), (35, 30)));
    assert_eq!(scene.tiles.len(), 35 * 30);
    let map = &s.state().map().tiles;
    for pos in (-11..19).flat_map(|y| (-10..25).map(move |x| p(x, y))) {
        let tile = scene.tile(pos).unwrap();
        assert_eq!(tile.terrain, map.get(pos).copied(), "{pos:?}");
        assert!(tile.tints.is_empty(), "{pos:?}");
    }
    let sea = c.content.terrain.display.id_of("sea");
    assert!(sea.is_some());
    assert_eq!(scene.tile(p(0, 0)).unwrap().terrain, sea);
    assert_eq!(scene.tile(p(-1, 0)).unwrap().terrain, None);
    assert_eq!(scene.tile(p(14, 7)).unwrap().terrain, None);
    assert!(scene.tile(p(13, 7)).unwrap().terrain.is_some());
    // Every unit, in the battle's order, where it stands.
    let shown: Vec<_> = scene
        .units
        .iter()
        .map(|u| (u.id, u.pos, u.label.clone(), u.faction, u.hp))
        .collect();
    let units: Vec<_> = s
        .state()
        .units()
        .iter()
        .map(|u| {
            (
                u.id,
                u.pos,
                u.map_label.clone(),
                u.faction,
                (u.hp, u.stats.hp),
            )
        })
        .collect();
    assert_eq!(shown, units);
    assert_eq!(shown.len(), 8);
    for u in &scene.units {
        assert!(!u.acted && !u.has_effect() && u.fade.abs() < f32::EPSILON);
    }
    // The cursor on the lord, at the start of its pulse.
    let cursor = scene.cursor.unwrap();
    assert_eq!((cursor.pos, cursor.style), (p(3, 5), CursorStyle::Corners));
    assert!((cursor.brightness - 1.0).abs() < 1e-6);
    assert!(scene.path.is_empty());
}

#[test]
fn the_cursor_has_the_players_style_and_its_pulse() {
    let mut c = ctx();
    c.cursor_style = CursorStyle::TileGlow;
    let mut s = quick();
    wait(&mut s, &mut c, 0.5);
    let cursor = s.scene(&c).cursor.unwrap();
    assert_eq!(cursor.style, CursorStyle::TileGlow);
    assert!((cursor.brightness - s.cursor().brightness()).abs() < 1e-6);
    assert!((cursor.brightness - 0.5).abs() < 1e-6);
}

#[test]
fn a_selected_unit_shows_its_ranges_and_its_path() {
    let mut c = ctx();
    let mut s = quick();
    step(&mut s, &mut c, &[Action::Confirm]);
    let Mode::Selected(sel) = s.mode() else {
        panic!("{:?}", s.mode());
    };
    let scene = s.scene(&c);
    assert_eq!(scene.tinted(RangeKind::Move), rows(sel.moves.iter()));
    assert_eq!(scene.tinted(RangeKind::Attack), rows(sel.attack.iter()));
    assert!(!sel.moves.is_empty() && !sel.attack.is_empty());
    assert_eq!(ranges(&scene).len(), 2);
    // On the unit: the cursor, and a path of its own tile only.
    assert_eq!(scene.cursor.map(|c| c.pos), Some(p(3, 5)));
    assert_eq!(scene.path, [p(3, 5)]);
    // Away from it: the path, and no cursor where its arrowhead points.
    step(&mut s, &mut c, &[Action::CursorRight, Action::CursorRight]);
    let scene = s.scene(&c);
    assert_eq!(scene.path, [p(3, 5), p(4, 5), p(5, 5)]);
    assert_eq!(scene.cursor, None);
    assert_eq!(scene.unit(UnitId(1)).map(|u| u.pos), Some(p(3, 5)));
    // Past its reach: the path stops short and the cursor shows.
    step(&mut s, &mut c, &[Action::CursorRight; 8]);
    let scene = s.scene(&c);
    assert_eq!(scene.path.last(), Some(&p(7, 5)));
    assert_eq!(scene.cursor.map(|c| c.pos), Some(p(13, 5)));
}

#[test]
fn the_danger_zone_lies_under_the_modes_ranges() {
    let mut c = ctx();
    let mut s = quick();
    step(&mut s, &mut c, &[Action::DangerZone]);
    let zone = s.danger().unwrap().clone();
    let scene = s.scene(&c);
    assert_eq!(scene.tinted(RangeKind::Danger), rows(zone.iter()));
    assert_eq!(ranges(&scene), [(RangeKind::Danger, zone.iter().count())]);
    step(&mut s, &mut c, &[Action::Confirm]);
    let Mode::Selected(sel) = s.mode() else {
        panic!("{:?}", s.mode());
    };
    let both = sel.moves.iter().find(|&p| zone.contains(p)).unwrap();
    let scene = s.scene(&c);
    assert_eq!(
        scene.tile(both).unwrap().tints,
        [RangeKind::Danger, RangeKind::Move]
    );
    assert_eq!(scene.tinted(RangeKind::Danger), rows(zone.iter()));
}

#[test]
fn an_enemys_threat_area_is_an_attack_range() {
    let mut c = ctx();
    let mut s = quick();
    s.cursor.jump(p(8, 2));
    step(&mut s, &mut c, &[Action::Confirm]);
    let Mode::Idle {
        threat: Some(threat),
    } = s.mode()
    else {
        panic!("{:?}", s.mode());
    };
    let scene = s.scene(&c);
    let area = rows(threat.area.iter());
    assert!(area.contains(&p(8, 2)));
    assert_eq!(scene.tinted(RangeKind::Attack), area);
    assert_eq!(ranges(&scene), [(RangeKind::Attack, area.len())]);
    assert_eq!(scene.cursor.map(|c| c.pos), Some(p(8, 2)));
    // Hidden again: no range.
    step(&mut s, &mut c, &[Action::Confirm]);
    assert!(ranges(&s.scene(&c)).is_empty());
}

#[test]
fn where_a_unit_may_move_after_its_attack_is_a_move_range() {
    let c = ctx();
    let state = vaulted(&c);
    let tiles = state.move_after_tiles();
    let s = BattleScreen::new(state);
    assert!(matches!(s.mode(), Mode::MoveAfter { .. }), "{:?}", s.mode());
    let scene = s.scene(&c);
    assert!(!tiles.is_empty());
    assert_eq!(scene.tinted(RangeKind::Move), rows(tiles.iter().copied()));
    assert_eq!(ranges(&scene), [(RangeKind::Move, tiles.len())]);
    assert_eq!(scene.cursor.map(|c| c.pos), Some(s.cursor().pos));
}

/// The skirmish with the lord (unit 1) walked one step right to (7, 2) and
/// its action menu open.
fn lord_menu(c: &mut Ctx) -> BattleScreen {
    let mut s = BattleScreen::new(skirmish(c, 20));
    step(
        &mut s,
        c,
        &[Action::Confirm, Action::CursorRight, Action::Confirm],
    );
    assert!(matches!(s.mode(), Mode::Moving { .. }), "{:?}", s.mode());
    wait(&mut s, c, 1.0);
    assert!(
        matches!(s.mode(), Mode::ActionMenu { .. }),
        "{:?}",
        s.mode()
    );
    s
}

#[test]
fn the_targets_of_an_attack_are_an_attack_range() {
    let mut c = ctx();
    let mut s = lord_menu(&mut c);
    // Attack, then the weapon.
    step(&mut s, &mut c, &[Action::Confirm]);
    step(&mut s, &mut c, &[Action::Confirm]);
    let Mode::Targeting(t) = s.mode() else {
        panic!("{:?}", s.mode());
    };
    let at = |id| s.state().unit(id).unwrap().pos;
    let targets: Vec<Pos> = t.targets.iter().map(|&id| at(id)).collect();
    // The raider above and the brigand to the right.
    assert_eq!(rows(targets.clone()), [p(7, 1), p(8, 2)]);
    let scene = s.scene(&c);
    assert_eq!(scene.tinted(RangeKind::Attack), rows(targets));
    assert_eq!(ranges(&scene), [(RangeKind::Attack, 2)]);
    // The attacker where it will stand, the cursor on the target.
    assert_eq!(scene.unit(UnitId(1)).map(|u| u.pos), Some(p(7, 2)));
    assert_eq!(s.state().unit(UnitId(1)).unwrap().pos, p(6, 2));
    assert_eq!(scene.cursor.map(|c| c.pos), Some(at(t.target())));
    assert!(scene.path.is_empty());
}

#[test]
fn walks_menus_and_boxes_hide_the_cursor_and_show_no_range() {
    let mut c = ctx();
    // A walk: the unit along its path.
    let mut s = quick();
    step(&mut s, &mut c, &[Action::Confirm, Action::CursorRight]);
    step(&mut s, &mut c, &[Action::CursorRight, Action::Confirm]);
    assert!(matches!(s.mode(), Mode::Moving { .. }), "{:?}", s.mode());
    wait(&mut s, &mut c, 0.01);
    let scene = s.scene(&c);
    assert_eq!(scene.cursor, None);
    assert!(ranges(&scene).is_empty() && scene.path.is_empty());
    let walker = scene.unit(UnitId(1)).map(|u| u.pos);
    assert_eq!(walker, s.mode().drawn_pos(UnitId(1)));
    // Its action menu: the unit at its path's end.
    wait(&mut s, &mut c, 1.0);
    assert!(
        matches!(s.mode(), Mode::ActionMenu { .. }),
        "{:?}",
        s.mode()
    );
    let scene = s.scene(&c);
    assert_eq!(scene.cursor, None);
    assert!(ranges(&scene).is_empty() && scene.path.is_empty());
    assert_eq!(scene.unit(UnitId(1)).map(|u| u.pos), Some(p(5, 5)));
    // The info screen and the end-turn question.
    for action in [Action::Info, Action::EndTurn] {
        let mut s = quick();
        step(&mut s, &mut c, &[action]);
        let hidden = matches!(s.mode(), Mode::Info { .. } | Mode::EndTurnPrompt { .. });
        assert!(hidden, "{:?}", s.mode());
        assert_eq!(s.scene(&c).cursor, None, "{action:?}");
    }
    // The map menu opens beside the cursor, which stays.
    let mut s = quick();
    step(&mut s, &mut c, &[Action::Cancel]);
    assert!(matches!(s.mode(), Mode::MapMenu { .. }), "{:?}", s.mode());
    assert_eq!(s.scene(&c).cursor.map(|c| c.pos), Some(p(3, 5)));
}

#[test]
fn a_combat_shows_its_units_as_the_playback_has_them() {
    let mut c = ctx();
    // The brigand on 1 HP: the lord's hit fells it.
    let mut s = BattleScreen::new(skirmish(&c, 1));
    step(
        &mut s,
        &mut c,
        &[Action::Confirm, Action::CursorRight, Action::Confirm],
    );
    wait(&mut s, &mut c, 1.0);
    step(&mut s, &mut c, &[Action::Confirm]);
    step(&mut s, &mut c, &[Action::Confirm]);
    let Mode::Targeting(t) = s.mode() else {
        panic!("{:?}", s.mode());
    };
    // Onto the brigand (unit 4), then attack.
    if t.target() != UnitId(4) {
        step(&mut s, &mut c, &[Action::CursorRight]);
    }
    step(&mut s, &mut c, &[Action::Confirm]);
    assert!(matches!(s.mode(), Mode::Combat(_)), "{:?}", s.mode());
    let mut fades = Vec::new();
    for _ in 0..600 {
        if !matches!(s.mode(), Mode::Combat(_)) {
            break;
        }
        let scene = s.scene(&c);
        assert_eq!(scene.cursor, None);
        assert!(ranges(&scene).is_empty() && scene.path.is_empty());
        let shown: Vec<_> = s
            .shown_units()
            .iter()
            .map(|(u, fade)| (u.id, u.pos, u.hp, u.acted, *fade))
            .collect();
        let drawn: Vec<_> = scene
            .units
            .iter()
            .map(|u| (u.id, u.pos, u.hp.0, u.acted, u.fade))
            .collect();
        assert_eq!(drawn, shown);
        fades.extend(scene.unit(UnitId(4)).map(|u| u.fade));
        wait(&mut s, &mut c, 0.05);
    }
    assert!(
        !matches!(s.mode(), Mode::Combat(_)),
        "the combat never ended"
    );
    // The brigand stood, then faded out.
    assert!(fades.first().is_some_and(|f| f.abs() < f32::EPSILON));
    assert!(fades.iter().any(|&f| f > 0.0 && f < 1.0), "{fades:?}");
    assert_eq!(s.scene(&c).unit(UnitId(4)), None);
}

/// A skin that shows 10 × 8 tiles whatever the area (as bigger tiles would
/// show fewer), drawn as the glyph skin draws them.
#[derive(Debug)]
struct Narrow;

impl MapSkin for Narrow {
    fn name(&self) -> &'static str {
        "narrow"
    }

    fn view_tiles(&self, _: Rect) -> (i32, i32) {
        (10, 8)
    }

    fn paint(&self, ctx: &Ctx, scene: &MapScene, area: Rect, buf: &mut GlyphBuffer) {
        GlyphSkin.paint(ctx, scene, area, buf);
    }

    fn tile_px(&self, scene: &MapScene, area: Rect, tile: Pos) -> Option<PxRect> {
        GlyphSkin.tile_px(scene, area, tile)
    }
}

/// The Quick Battle's units on a 64 × 40 plain, the lord at (30, 20), the
/// knight at (34, 20) and the archer at (35, 20).
fn big_battle(c: &Ctx) -> BattleScreen {
    let (_, mut units) = quick_units(c);
    units[0].pos = p(30, 20);
    units[1].pos = p(34, 20);
    units[2].pos = p(35, 20);
    let map = BattleMap::new("Big", Grid::filled(64, 40, TerrainId(0)));
    BattleScreen::new(battle(c, map, units))
}

#[test]
fn the_view_is_as_big_as_the_skin_says() {
    let mut c = ctx();
    c.map_skin = Rc::new(Narrow);
    let mut s = big_battle(&c);
    // The screen starts with the default skin's 35 × 30 tiles in mind…
    assert_eq!(s.camera().origin, p(13, 5));
    // …but its first frame is already this skin's: centred on the cursor.
    let scene = s.scene(&c);
    assert_eq!((scene.origin, scene.size), (p(25, 16), (10, 8)));
    assert_eq!(scene.tiles.len(), 80);
    let ids: Vec<u32> = scene.units.iter().map(|u| u.id.0).collect();
    assert_eq!(ids, [1, 2], "the archer is one tile off the view");
    assert_eq!(scene.cursor.map(|c| c.pos), Some(p(30, 20)));
    // Only those tiles are drawn: the top-left 20 × 8 cells of the map view.
    let stale = Cell::new('x', Rgb::new(1, 2, 3), Rgb::new(1, 2, 3));
    let mut buf = GlyphBuffer::new(CONSOLE_W, CONSOLE_H, stale);
    s.draw(&c, &mut buf);
    let (text, bg) = (c.palette.get(UiColor::Text), c.palette.get(UiColor::Black));
    let blank = Cell::new(' ', text, bg);
    assert_ne!(buf.get(19, 7), Some(&blank));
    assert_eq!(buf.get(20, 7), Some(&blank));
    assert_eq!(buf.get(19, 8), Some(&blank));
    assert_eq!(buf.get(MAP_VIEW.w - 1, MAP_VIEW.h - 1), Some(&blank));
    // The lord at (30, 20): tile (5, 4) of the view, cells 10 and 11.
    assert_eq!(buf.get(10, 4).map(|c| c.glyph), Some('L'));
    // An update keeps the camera for that view…
    step(&mut s, &mut c, &[]);
    assert_eq!(s.camera().origin, p(25, 16));
    assert_eq!(s.scene(&c).origin, p(25, 16));
    // …and it follows the cursor 3 tiles from the edges of 10 × 8.
    step(&mut s, &mut c, &[Action::CursorRight; 4]);
    assert_eq!(s.cursor().pos, p(34, 20));
    assert_eq!(s.camera().origin, p(28, 16));
    let scene = s.scene(&c);
    assert_eq!(scene.origin, p(28, 16));
    assert_eq!(scene.units.len(), 3);
    // Back to the default skin: fitted again, around the cursor.
    c.map_skin = Rc::new(GlyphSkin);
    let scene = s.scene(&c);
    assert_eq!((scene.origin, scene.size), (p(17, 5), (35, 30)));
    assert_eq!(s.camera().origin, p(28, 16));
    step(&mut s, &mut c, &[]);
    assert_eq!(s.camera().origin, p(17, 5));
}

#[test]
fn a_skin_switch_refits_the_kept_camera_and_an_ai_pan() {
    let mut c = ctx();
    let mut s = big_battle(&c);
    step(&mut s, &mut c, &[]);
    assert_eq!(s.camera().origin, p(13, 5));
    // As if the player phase had ended with the cursor on the lord, and
    // an enemy's action were panning over to the archer's tile.
    s.player_view = Some((p(30, 20), s.camera));
    let before = s.state.units().to_vec();
    let archer = before[2].id;
    s.cursor.jump(p(60, 35));
    let pan = (s.camera.origin, p(29, 10));
    let action = AiAction::new(archer, before, pan, vec![], Mode::default(), PACING);
    let walk_start = action.walk_start();
    s.mode = Mode::AiAction(Box::new(action));
    // The debug menu switches to a skin showing 10 × 8 tiles.
    c.map_skin = Rc::new(Narrow);
    s.begin_frame(&c, 0.0);
    // The camera centres on the cursor; the kept one on the kept cursor.
    assert_eq!(s.camera().origin, p(54, 31));
    assert_eq!(s.player_view.map(|(_, c)| c.origin), Some(p(25, 16)));
    // The pan goes from there to show the archer at (35, 20) 3 tiles from
    // the edges, in the same time.
    let Mode::AiAction(a) = &s.mode else {
        panic!("{:?}", s.mode);
    };
    let mut a = a.as_ref().clone();
    assert_eq!(a.camera(), p(54, 31));
    assert!((a.walk_start() - walk_start).abs() < 1e-6);
    a.tick(PACING.pan, false);
    assert_eq!(a.camera(), p(32, 17));
    // The same size again changes nothing.
    let kept = (s.camera, s.player_view);
    s.begin_frame(&c, 0.0);
    assert_eq!((s.camera, s.player_view), kept);
}

#[test]
fn menus_open_beside_the_tile_where_the_skin_has_it() {
    let mut c = ctx();
    c.map_skin = Rc::new(Narrow);
    let mut s = big_battle(&c);
    step(&mut s, &mut c, &[Action::Cancel]);
    assert!(matches!(s.mode(), Mode::MapMenu { .. }), "{:?}", s.mode());
    let stale = Cell::new('x', Rgb::new(1, 2, 3), Rgb::new(1, 2, 3));
    let mut buf = GlyphBuffer::new(CONSOLE_W, CONSOLE_H, stale);
    s.draw(&c, &mut buf);
    // The cursor's tile is at cells (10, 4) and (11, 4) in this view (it
    // would be at (34, 15) in the default one): the menu's box starts one
    // cell right of it, one row up.
    assert_eq!(buf.get(13, 3).map(|c| c.glyph), Some('┌'));
    assert_eq!(buf.get(37, 14).map(|c| c.glyph), Some(' '));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    /// Wherever the units stand, whatever the player presses and however
    /// many tiles the skin shows, the scene has a tile for every tile of
    /// its view, and every unit in it (and the cursor) is on one of them;
    /// while no combat or AI action plays, it has every unit drawn on a
    /// visible tile.
    #[test]
    fn scene_units_are_visible(
        spots in prop::collection::vec((0..64i32, 0..6i32), 6),
        keys in prop::collection::vec(0usize..7, 0..40),
        narrow in any::<bool>(),
    ) {
        const KEYS: [Action; 7] = [
            Action::CursorLeft,
            Action::CursorRight,
            Action::CursorUp,
            Action::CursorDown,
            Action::Confirm,
            Action::Cancel,
            Action::NextUnit,
        ];
        let mut c = ctx();
        if narrow {
            c.map_skin = Rc::new(Narrow);
        }
        let (_, mut units) = quick_units(&c);
        // Each unit in its own band of six rows: no two on one tile.
        for ((unit, (x, dy)), band) in units.iter_mut().zip(spots).zip(0..) {
            unit.pos = p(x, band * 6 + dy);
        }
        let map = BattleMap::new("Big", Grid::filled(64, 40, TerrainId(0)));
        let mut s = BattleScreen::new(battle(&c, map, units));
        for key in std::iter::once(None).chain(keys.into_iter().map(Some)) {
            if let Some(key) = key {
                s.update(&mut c, &FrameInput::new(vec![KEYS[key]], 0.1, vec![]));
            }
            let scene = s.scene(&c);
            let (w, h) = scene.size;
            prop_assert_eq!((w, h), if narrow { (10, 8) } else { (35, 30) });
            prop_assert_eq!(scene.tiles.len(), usize::try_from(w * h).unwrap());
            for u in &scene.units {
                prop_assert!(scene.contains(u.pos), "{:?} in {:?}", u, s.mode());
            }
            if let Some(cursor) = scene.cursor {
                prop_assert!(scene.contains(cursor.pos), "{:?}", s.mode());
            }
            if !s.playing() && s.rewind().is_none() {
                let drawn = s.state().units().iter().map(|u| s.drawn_pos(u));
                let visible = drawn.filter(|&pos| scene.contains(pos)).count();
                prop_assert_eq!(scene.units.len(), visible, "{:?}", s.mode());
            }
        }
    }
}

/// What the scene shows of unit `id`: its tile, its facing, its offset and
/// its walking frame.
fn shown(s: &BattleScreen, c: &Ctx, id: UnitId) -> (Pos, Facing, (f32, f32), u8) {
    let scene = s.scene(c);
    let u = scene.unit(id).unwrap();
    (u.pos, u.facing, u.offset, u.frame)
}

/// A context whose map skin is the test unit sheets': sprites walk at
/// their own pace.
fn sprite_ctx() -> Ctx {
    let mut c = ctx();
    c.map_skin = crate::map_view::skin_named(&c.content, "sprite_units").unwrap();
    c
}

/// Under the glyph skin a walk is as quick as it always was: 12 tiles a
/// second, tile by tile in the scene as on screen.
#[test]
fn a_glyph_units_walk_keeps_its_pace() {
    let mut c = ctx();
    let mut s = quick();
    let lord = s.state().units()[0].id;
    step(&mut s, &mut c, &[Action::Confirm]);
    step(&mut s, &mut c, &[Action::CursorRight; 3]);
    step(&mut s, &mut c, &[Action::Confirm]);
    let Mode::Moving { pace, .. } = s.mode() else {
        panic!("{:?}", s.mode());
    };
    assert!((pace - 12.0).abs() < 1e-6);
    // Three tiles: over within 0.26 s.
    let mut frames = 0;
    while matches!(s.mode(), Mode::Moving { .. }) {
        wait(&mut s, &mut c, 0.02);
        frames += 1;
        assert!(frames < 100, "the walk never ended");
    }
    assert_eq!(frames, 13);
    assert_eq!(shown(&s, &c, lord).2, (0.0, 0.0));
    // The same walk under a sprite skin: 6 tiles a second, 0.5 s.
    let mut c = sprite_ctx();
    let mut s = quick();
    step(&mut s, &mut c, &[Action::Confirm]);
    step(&mut s, &mut c, &[Action::CursorRight; 3]);
    step(&mut s, &mut c, &[Action::Confirm]);
    let Mode::Moving { pace, .. } = s.mode() else {
        panic!("{:?}", s.mode());
    };
    assert!((pace - 6.0).abs() < 1e-6);
    let mut frames = 0;
    while matches!(s.mode(), Mode::Moving { .. }) {
        wait(&mut s, &mut c, 0.02);
        frames += 1;
        assert!(frames < 100, "the walk never ended");
    }
    assert!((25..=26).contains(&frames), "{frames}");
}

/// Ticket 0440: a move that goes right and then up.
#[test]
fn a_walking_unit_turns_the_way_it_goes_and_glides_from_tile_to_tile() {
    let mut c = sprite_ctx();
    let mut s = quick();
    let lord = s.state().units()[0].id;
    let start = s.state().units()[0].pos;
    step(&mut s, &mut c, &[Action::Confirm]);
    for action in [Action::CursorRight, Action::CursorRight, Action::CursorUp] {
        step(&mut s, &mut c, &[action]);
    }
    let path = s.scene(&c).path.clone();
    let turn = p(start.x + 2, start.y);
    let end = p(start.x + 2, start.y - 1);
    assert_eq!(path, [start, p(start.x + 1, start.y), turn, end]);
    // Selected, not yet walking: it faces the camera on its own tile.
    assert_eq!(shown(&s, &c, lord), (start, Facing::Down, (0.0, 0.0), 1));
    step(&mut s, &mut c, &[Action::Confirm]);
    assert!(matches!(s.mode(), Mode::Moving { .. }), "{:?}", s.mode());
    // The walk, in frames of 25 ms: a tile every 167 ms.
    let mut seen = Vec::new();
    while matches!(s.mode(), Mode::Moving { .. }) {
        seen.push(shown(&s, &c, lord));
        // Only the walker turns or leaves its tile.
        for u in s.scene(&c).units.iter().filter(|u| u.id != lord) {
            assert_eq!((u.facing, u.offset), (Facing::Down, (0.0, 0.0)), "{u:?}");
        }
        wait(&mut s, &mut c, 0.025);
        assert!(seen.len() < 100, "the walk never ended");
    }
    // It was on each tile of the path but the last in turn, facing right
    // and then up.
    let mut tiles: Vec<(Pos, Facing)> = seen.iter().map(|&(pos, f, ..)| (pos, f)).collect();
    tiles.dedup();
    assert_eq!(
        tiles,
        [
            (start, Facing::Right),
            (p(start.x + 1, start.y), Facing::Right),
            (turn, Facing::Up),
        ]
    );
    // On each step its offset went from 0 towards 1: right (+x), then up
    // (−y), never across both.
    for step in seen.chunk_by(|a, b| a.0 == b.0) {
        assert!(step.len() >= 6, "{step:?}");
        let facing = step[0].1;
        let along = |&(_, _, (dx, dy), _): &(Pos, Facing, (f32, f32), u8)| {
            if facing == Facing::Right {
                assert!(dy == 0.0, "{step:?}");
                dx
            } else {
                assert!(dx == 0.0, "{step:?}");
                -dy
            }
        };
        assert!(along(&step[0]) < 0.16, "{step:?}");
        for pair in step.windows(2) {
            assert!(along(&pair[0]) < along(&pair[1]), "{step:?}");
        }
        let last = along(&step[step.len() - 1]);
        assert!((0.8..1.0).contains(&last), "{step:?}");
    }
    // Its legs went: a frame every 100 ms of the half second.
    let mut frames: Vec<u8> = seen.iter().map(|&(.., frame)| frame).collect();
    frames.dedup();
    assert_eq!(frames, [0, 1, 2, 1, 0]);
    // Arrived: on the path's last tile, facing the camera, no offset.
    assert!(
        matches!(s.mode(), Mode::ActionMenu { .. }),
        "{:?}",
        s.mode()
    );
    assert_eq!(shown(&s, &c, lord), (end, Facing::Down, (0.0, 0.0), 1));
    // Nobody else turned or left its tile.
    let scene = s.scene(&c);
    for u in scene.units.iter().filter(|u| u.id != lord) {
        assert_eq!((u.facing, u.offset), (Facing::Down, (0.0, 0.0)), "{u:?}");
    }
}

/// Ticket 0440: a unit that can still act steps on the spot; one that has
/// acted, or is falling, stands still.
#[test]
fn a_unit_that_can_act_steps_on_the_spot_and_an_acted_one_stands_still() {
    let mut c = ctx();
    let s = quick();
    let frames = |c: &Ctx| -> Vec<u8> { s.scene(c).units.iter().map(|u| u.frame).collect() };
    let count = s.scene(&c).units.len();
    assert!(count > 4);
    // Everyone steps together: standing, a foot, standing, the other foot,
    // a frame every 250 ms.
    for (ms, frame) in [(0, 1), (249, 1), (250, 2), (500, 1), (750, 0), (1000, 1)] {
        c.clock_s = f64::from(ms) / 1000.0;
        assert_eq!(frames(&c), vec![frame; count], "at {ms} ms");
    }
    // One that has acted, and one falling, keep the standing frame.
    let mut scene = s.scene(&c);
    scene.clock_ms = 750;
    for u in &mut scene.units {
        u.frame = 1;
    }
    scene.units[0].acted = true;
    scene.units[1].fade = 0.25;
    super::animate(&mut scene, None);
    let frames: Vec<u8> = scene.units.iter().map(|u| u.frame).collect();
    let mut expect = vec![0; count];
    expect[0] = 1;
    expect[1] = 1;
    assert_eq!(frames, expect);
    assert!(scene.units.iter().all(|u| !u.between_tiles()));
    // The rewind screen's map steps too.
    let mut s = quick();
    c.clock_s = 0.25;
    step(&mut s, &mut c, &[Action::Rewind]);
    assert!(s.rewind().is_some());
    assert!(s.scene(&c).units.iter().all(|u| u.frame == 2));
}

/// Ticket 0440: an AI unit's walk looks the same, after its pan and its
/// highlight.
#[test]
fn an_ai_units_walk_turns_and_glides_too() {
    let c = ctx();
    let mut s = quick();
    let before = s.state.units().to_vec();
    let (brigand, from) = (before[4].id, before[4].pos);
    let path = vec![from, p(from.x - 1, from.y), p(from.x - 1, from.y + 1)];
    let pan = (s.camera.origin, s.camera.origin);
    let pacing = Pacing {
        walk_tiles_per_s: SPRITE_WALK_TILES_PER_S,
        held_walk_tiles_per_s: SPRITE_HELD_WALK_TILES_PER_S,
        ..PACING
    };
    let action = AiAction::new(brigand, before, pan, path, Mode::default(), pacing);
    let walk_start = action.walk_start();
    s.mode = Mode::AiAction(Box::new(action));
    let tick = |s: &mut BattleScreen, dt: f32, held: bool| {
        if let Mode::AiAction(a) = &mut s.mode {
            a.tick(dt, held);
        }
    };
    // Marked by the cursor: still on its tile, facing the camera.
    assert_eq!(shown(&s, &c, brigand), (from, Facing::Down, (0.0, 0.0), 1));
    // Half a tile left: a twelfth of a second at 6 tiles a second.
    tick(&mut s, walk_start + 1.0 / 12.0, false);
    let (pos, facing, offset, frame) = shown(&s, &c, brigand);
    assert_eq!((pos, facing, frame), (from, Facing::Left, 0));
    assert!(
        (offset.0 + 0.5).abs() < 1e-3 && offset.1 == 0.0,
        "{offset:?}"
    );
    // A quarter of a tile down, on the path's second tile.
    tick(&mut s, 0.125, false);
    let (pos, facing, offset, frame) = shown(&s, &c, brigand);
    let second = p(from.x - 1, from.y);
    assert_eq!((pos, facing, frame), (second, Facing::Down, 2));
    assert!(
        offset.0 == 0.0 && (offset.1 - 0.25).abs() < 1e-3,
        "{offset:?}"
    );
    // Confirm held: 12 tiles a second, twice as fast, legs and all. Half
    // a tile in a 24th of a second.
    tick(&mut s, 1.0 / 24.0, true);
    let (_, _, offset, frame) = shown(&s, &c, brigand);
    assert!((offset.1 - 0.75).abs() < 1e-3, "{offset:?}");
    assert_eq!(frame, 2);
    // Arrived.
    tick(&mut s, 5.0, false);
    let end = p(from.x - 1, from.y + 1);
    assert_eq!(shown(&s, &c, brigand), (end, Facing::Down, (0.0, 0.0), 1));
}
