//! Tests of the player, on the test cinematic
//! (`assets-test/cinematics/test.ron`): a pan over the test battle's map
//! at 4× for six seconds, two text boxes for four, three faces for six,
//! then the logo for four (the talk and characters shots: `shot_tests`).

use insta::assert_snapshot;
use trpg_content::TilePoint;
use trpg_core::UnitId;

use super::*;
use crate::console::{ConsoleSizes, ConsoleSpec};
use crate::map_view::{Facing, skin_named};
use crate::screen::tests::ctx;
use crate::screens::title::glyph;

const SCREEN: Rect = Rect::new(0, 0, 100, 32);

fn player(ctx: &Ctx) -> CinematicPlayer {
    CinematicPlayer::new(ctx, "test", None).unwrap()
}

/// `player`'s frame at `t` and `brightness`, over a stale buffer.
fn draw(ctx: &Ctx, player: &CinematicPlayer, t: f32, brightness: f32) -> GlyphBuffer {
    let p = &ctx.palette;
    let stale = Cell::new('x', p.get(UiColor::Enemy), p.get(UiColor::PanelBg));
    let mut buf = GlyphBuffer::with_spec(ConsoleSpec::PC, stale);
    player.draw(ctx, t, brightness, &mut buf);
    buf
}

fn near(a: (f32, f32), b: (f32, f32)) -> bool {
    (a.0 - b.0).abs() < 1e-3 && (a.1 - b.1).abs() < 1e-3
}

// The timeline and easing are the engine's (`shuyi_ui::cinematic`), tested
// there (ticket 1113).

#[test]
fn the_timeline_of_a_cinematic_is_its_shots_starts_and_its_length() {
    let ctx = ctx();
    let def = &ctx.game.content.cinematics["test"];
    let timeline = Timeline::new(vec![0, 6000, 10_000, 16_000], 20_000);
    assert_eq!(Timeline::of(def), timeline);
    assert_eq!(player(&ctx).timeline(), &timeline);
}

#[test]
fn a_battles_scene_is_its_whole_map_with_its_units_where_they_start() {
    let ctx = ctx();
    let def = &ctx.game.content.battles["test"];
    let scene = battle_scene(&ctx, def);
    assert_eq!((scene.origin, scene.size), (Pos::new(0, 0), (14, 8)));
    assert_eq!(scene.tiles.len(), 14 * 8);
    assert!(scene.tiles.iter().all(|t| t.terrain.is_some()));
    assert_eq!(scene.look, ctx.game.content.battle_look(def));
    // The lead and the test knight in their slots, and the brigand.
    let places: Vec<(i32, i32)> = scene.units.iter().map(|u| (u.pos.x, u.pos.y)).collect();
    assert_eq!(places, [(3, 5), (3, 6), (8, 3)]);
    assert!(scene.units.iter().all(|u| !u.acted && u.fade == 0.0));
    assert!(scene.cursor.is_none() && scene.path.is_empty() && scene.threat.is_none());
}

#[test]
fn a_map_picture_is_the_whole_scene_as_the_skin_paints_it() {
    let mut ctx = ctx();
    let scene = battle_scene(&ctx, &ctx.game.content.battles["test"]);
    // Glyphs: 14 × 8 tiles of two cells by one, 16 px each way.
    let picture = MapPicture::paint(&ctx, ctx.game.map_skin.as_ref(), &scene).unwrap();
    assert_eq!((picture.scene.width(), picture.scene.height()), (28, 8));
    assert_eq!(picture.tile_px, (16, 16));
    assert_eq!(picture.size_px(), (224, 128));
    // A whole number is a tile's middle.
    assert_eq!(picture.point_px((0.0, 0.0)), (8.0, 8.0));
    assert_eq!(picture.point_px((5.75, 3.5)), (100.0, 64.0));
    assert_eq!(picture.point_px((-0.5, 7.5)), (0.0, 128.0));
    // The units are on it: the lead's tile isn't bare terrain.
    let bare = MapPicture::paint(
        &ctx,
        ctx.game.map_skin.as_ref(),
        &MapScene {
            units: vec![],
            ..scene.clone()
        },
    )
    .unwrap();
    assert_ne!(picture.scene.get(6, 5), bare.scene.get(6, 5));
    assert_eq!(picture.scene.get(0, 0), bare.scene.get(0, 0));
    assert!(picture.scene.backdrop().is_none());
    // The test tileset's tiles are 24 px: 336 × 192, in whole cells.
    ctx.game.map_skin = skin_named(&ctx.game.content, "sprite").unwrap();
    let picture = MapPicture::paint(&ctx, ctx.game.map_skin.as_ref(), &scene).unwrap();
    assert_eq!((picture.scene.width(), picture.scene.height()), (42, 12));
    assert_eq!(picture.tile_px, (24, 24));
    assert_eq!(picture.point_px((1.0, 2.0)), (36.0, 60.0));
    assert!(!picture.scene.sprites().is_empty());
    // A size that isn't whole cells is rounded up to them.
    let odd = MapScene::new(Pos::new(0, 0), (1, 1));
    let picture = MapPicture::paint(&ctx, ctx.game.map_skin.as_ref(), &odd).unwrap();
    assert_eq!((picture.scene.width(), picture.scene.height()), (3, 2));
    assert_eq!(picture.size_px(), (24, 32));
    // Nothing to paint.
    assert!(
        MapPicture::paint(
            &ctx,
            ctx.game.map_skin.as_ref(),
            &MapScene::new(Pos::new(0, 0), (0, 0))
        )
        .is_none()
    );
}

#[test]
fn the_pan_moves_the_middle_of_the_screen_from_one_point_to_the_other() {
    let ctx = ctx();
    let player = player(&ctx);
    let origin = |t: f32| match player.frame(&ctx, t, SCREEN) {
        Frame::MapPan { origin_px, zoom: 4 } => origin_px,
        other => panic!("{t}: {other:?}"),
    };
    // At 4× the screen shows 200 × 128 of the 224 × 128 picture: the
    // middle goes from x = 100 to x = 124, so the left edge from 0 to 24.
    assert!(near(origin(0.0), (0.0, 0.0)));
    assert!(near(origin(3.0), (12.0, 0.0)));
    // Eased: a quarter of the time is less than a quarter of the way.
    assert!(near(origin(1.5), (24.0 * 0.15625, 0.0)));
    assert!(near(origin(4.5), (24.0 * 0.84375, 0.0)));
    assert!(origin(5.999).0 > 23.99);
    // A hard cut to the next shot; after the logo, round again.
    assert!(matches!(
        player.frame(&ctx, 6.0, SCREEN),
        Frame::Talk { .. }
    ));
    assert_eq!(
        player.frame(&ctx, 16.0, SCREEN),
        Frame::Logo { sparks: None }
    );
    assert_eq!(
        player.frame(&ctx, 19.999, SCREEN),
        Frame::Logo { sparks: None }
    );
    assert!(near(origin(20.0), (0.0, 0.0)));
    assert!(near(origin(23.0), (12.0, 0.0)));
    // On a smaller screen the window is another, and so is its corner:
    // 50 × 4 cells at 4× show 100 × 16 pixels round (112, 64).
    let small = player.frame(&ctx, 3.0, Rect::new(0, 0, 50, 4));
    assert_eq!(
        small,
        Frame::MapPan {
            origin_px: (62.0, 56.0),
            zoom: 4
        }
    );
}

#[test]
fn a_pan_goes_both_ways_at_its_own_zoom() {
    let mut ctx = ctx();
    let def = ctx.game.content.cinematics.get_mut("test").unwrap();
    def.shots[0].shot = Shot::MapPan {
        battle: "test".to_owned(),
        from: TilePoint::new(9.5, 5.0),
        to: TilePoint::new(3.5, 2.0),
        zoom: 2,
        action: None,
    };
    // On a 50 × 4 screen, 2× shows 200 × 32 pixels: room to move down the
    // picture (128 high) but not across it (224 wide: 0 to 24).
    let player = player(&ctx);
    let screen = Rect::new(0, 0, 50, 4);
    let origin = |t: f32| match player.frame(&ctx, t, screen) {
        Frame::MapPan { origin_px, zoom: 2 } => origin_px,
        other => panic!("{t}: {other:?}"),
    };
    // From (160, 88) to (64, 40): the window's corner is 100 left and 16 up.
    assert!(near(origin(0.0), (24.0, 72.0)));
    assert!(near(origin(3.0), (12.0, 48.0)));
    assert!(near(origin(5.999), (0.0, 24.0)));
}

#[test]
fn the_logo_shot_carries_the_sparks_time_when_screens_move() {
    let mut ctx = ctx();
    let player = player(&ctx);
    assert_eq!(
        player.frame(&ctx, 17.25, SCREEN),
        Frame::Logo { sparks: None }
    );
    ctx.idle_motion = IdleMotion::On;
    assert_eq!(
        player.frame(&ctx, 17.25, SCREEN),
        Frame::Logo {
            sparks: Some(17250)
        }
    );
    // The same moment a round later: the same sparks.
    assert_eq!(
        player.frame(&ctx, 37.25, SCREEN),
        Frame::Logo {
            sparks: Some(17250)
        }
    );
    // The pan has none.
    assert!(matches!(
        player.frame(&ctx, 1.0, SCREEN),
        Frame::MapPan { .. }
    ));
}

#[test]
fn the_pan_is_the_battles_picture_behind_the_whole_screen() {
    let ctx = ctx();
    let player = player(&ctx);
    let buf = draw(&ctx, &player, 3.0, 1.0);
    let backdrop = buf.backdrop().unwrap();
    assert_eq!(backdrop.clip(), buf.bounds());
    assert_eq!(backdrop.zoom(), 4);
    assert!(near(backdrop.origin_px(), (12.0, 0.0)));
    // Three seconds in the lead is on its way: the picture is of the
    // battle as the action has it then.
    let start = battle_scene(&ctx, &ctx.game.content.battles["test"]);
    let scene = player.scene(3.0).unwrap();
    assert_ne!(scene, start);
    let picture = MapPicture::paint(&ctx, ctx.game.map_skin.as_ref(), &scene).unwrap();
    assert_eq!(backdrop.scene(), &*picture.scene);
    // Before anyone has moved: the battle as it starts.
    let picture = MapPicture::paint(&ctx, ctx.game.map_skin.as_ref(), &start).unwrap();
    let first = draw(&ctx, &player, 0.0, 1.0);
    assert_eq!(first.backdrop().unwrap().scene(), &*picture.scene);
    assert_ne!(first.backdrop().unwrap().scene(), backdrop.scene());
    // Every cell is a hole, and nothing is left of the frame before.
    let hole = Cell::see_through(' ', ctx.palette.get(UiColor::Text));
    for y in 0..i32::from(ConsoleSpec::PC.rows) {
        for x in 0..i32::from(ConsoleSpec::PC.cols) {
            assert_eq!(buf.get(x, y), Some(&hole), "({x}, {y})");
        }
    }
    assert!(buf.items().is_empty());
}

#[test]
fn the_logo_shot_is_the_titles_own_picture() {
    let mut ctx = ctx();
    let backdrop = ctx.game.content.title.backdrops.get(1).copied();
    assert!(backdrop.is_some());
    for (motion, sparks) in [(IdleMotion::Off, None), (IdleMotion::On, Some(17250))] {
        ctx.idle_motion = motion;
        // With no backdrop given, the logo is on none.
        for given in [None, backdrop] {
            let player = player(&ctx).with_backdrop(given);
            let buf = draw(&ctx, &player, 17.25, 1.0);
            let mut title = draw(&ctx, &player, 0.0, 1.0);
            glyph::paint_logo(&ctx, given, sparks, &mut title);
            assert_eq!(buf, title);
            assert!(buf.backdrop().is_none());
            assert_eq!(buf.sprites().len(), if given.is_some() { 4 } else { 1 });
        }
    }
}

#[test]
fn the_same_time_draws_the_same_frame() {
    let mut ctx = ctx();
    ctx.idle_motion = IdleMotion::On;
    let player = player(&ctx);
    for t in [0.0, 1.7, 3.0, 5.999, 6.0, 8.3, 10.0, 12.4, 16.0, 18.3] {
        let first = draw(&ctx, &player, t, 1.0);
        assert_eq!(draw(&ctx, &player, t, 1.0), first, "{t}");
        // Whatever was drawn in between, and a whole round later.
        draw(&ctx, &player, 19.1, 0.3);
        assert_eq!(draw(&ctx, &player, t, 1.0), first, "{t}");
        assert_eq!(draw(&ctx, &player, t + 20.0, 1.0), first, "{t}");
        assert_ne!(draw(&ctx, &player, t + 0.5, 1.0), first, "{t}");
    }
}

#[test]
fn brightness_fades_the_whole_frame_cells_and_items_alike() {
    let ctx = ctx();
    let player = player(&ctx).with_backdrop(ctx.game.content.title.backdrops.first().copied());
    for t in [3.0, 8.0, 12.0, 17.0] {
        let full = draw(&ctx, &player, t, 1.0);
        let mut faded = full.clone();
        faded.fade(0.5);
        let half = draw(&ctx, &player, t, 0.5);
        assert_eq!(half, faded, "{t}");
        assert_ne!(half, full, "{t}");
        // Above full brightness is full brightness.
        assert_eq!(draw(&ctx, &player, t, 2.0), full, "{t}");
    }
    // The pan: the picture behind the screen is half as bright.
    let (full, half) = (draw(&ctx, &player, 3.0, 1.0), draw(&ctx, &player, 3.0, 0.5));
    let cell = |b: &GlyphBuffer| *b.backdrop().unwrap().scene().get(6, 5).unwrap();
    assert_eq!(cell(&half).fg, cell(&full).fg.scale(0.5));
    assert_eq!(cell(&half).bg, cell(&full).bg.scale(0.5));
    assert_eq!(cell(&half).glyph, cell(&full).glyph);
    // The logo: its sprites (the backdrop's three, then the logo) are
    // half as bright.
    let half = draw(&ctx, &player, 17.0, 0.5);
    let brightness: Vec<u8> = half.sprites().iter().map(|s| s.brightness).collect();
    assert_eq!(brightness, [128; 4]);
    // Black is black.
    let black = draw(&ctx, &player, 17.0, 0.0);
    assert!(black.sprites().iter().all(|s| s.brightness == 0));
    assert_snapshot!(
        "pan_middle_half_bright",
        draw(&ctx, &player, 3.0, 0.5).to_snapshot(&ctx.palette)
    );
}

#[test]
fn a_pan_over_a_sprite_map_fades_its_sprites() {
    let mut ctx = ctx();
    ctx.game.map_skin = skin_named(&ctx.game.content, "sprite").unwrap();
    let player = player(&ctx);
    let scene_sprites = |brightness: f32| {
        let buf = draw(&ctx, &player, 3.0, brightness);
        buf.backdrop().unwrap().scene().sprites()
    };
    let (full, half) = (scene_sprites(1.0), scene_sprites(0.5));
    assert!(!full.is_empty());
    assert_eq!(full.len(), half.len());
    for (full, half) in full.iter().zip(&half) {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let expected = (f32::from(full.brightness) * 0.5).round() as u8;
        assert_eq!((half.brightness, half.opacity), (expected, full.opacity));
    }
    // 336 × 192 at 4×: room both ways, and the 24 px tiles put the
    // points elsewhere: (150, 96) to (186, 96).
    let Frame::MapPan { origin_px, .. } = player.frame(&ctx, 0.0, SCREEN) else {
        panic!("not a pan");
    };
    assert!(near(origin_px, (50.0, 32.0)));
}

#[test]
fn a_battle_is_painted_once_for_all_its_shots() {
    let mut ctx = ctx();
    let def = ctx.game.content.cinematics.get_mut("test").unwrap();
    // Two pans in which nobody moves.
    still(&mut def.shots[0].shot);
    let pan = def.shots[0].shot.clone();
    def.shots[1].shot = pan;
    let player = player(&ctx);
    let scene = |t: f32| {
        let buf = draw(&ctx, &player, t, 1.0);
        std::ptr::from_ref(buf.backdrop().unwrap().scene())
    };
    assert_eq!(scene(1.0), scene(7.0));
    // The scene is the battle as it starts, the whole shot long.
    let start = battle_scene(&ctx, &ctx.game.content.battles["test"]);
    assert_eq!(player.scene(1.0), Some(start.clone()));
    assert_eq!(player.scene(9.5), Some(start));
}

/// Takes the action out of the map pan `shot`.
fn still(shot: &mut Shot) {
    match shot {
        Shot::MapPan { action, .. } => *action = None,
        other => panic!("not a pan: {other:?}"),
    }
}

const LEAD: UnitId = UnitId(1);
const BRIGAND: UnitId = UnitId(3);

/// The test cinematic's action (`assets-test/cinematics/test.ron`): from
/// half a second in, the lead walks from (3, 5) up to (3, 3) and along to
/// (7, 3) in three seconds; at four seconds it strikes the brigand on
/// (8, 3), leaving it 40% of its HP, and at five it fells it.
#[test]
fn the_test_cinematics_units_walk_and_fight() {
    let ctx = ctx();
    let player = player(&ctx);
    let start = battle_scene(&ctx, &ctx.game.content.battles["test"]);
    let p = Pos::new;
    assert_eq!(start.unit(LEAD).unwrap().pos, p(3, 5));
    assert_eq!(start.unit(BRIGAND).unwrap().pos, p(8, 3));
    let full = start.unit(BRIGAND).unwrap().hp.1;
    let scene = |t: f32| player.scene(t).unwrap();
    let unit = |t: f32, id| scene(t).unit(id).cloned().unwrap();
    // Nobody has moved yet.
    assert_eq!(scene(0.0), start);
    assert_eq!(scene(0.499), start);
    // Mid-walk: three and a half of its six tiles, heading right.
    let lead = unit(2.25, LEAD);
    assert_eq!((lead.pos, lead.facing), (p(4, 3), Facing::Right));
    assert!(near(lead.offset, (0.5, 0.0)), "{:?}", lead.offset);
    assert_eq!(scene(2.25).unit_at(p(3, 5)), None);
    // Only the lead differs.
    let mut others = scene(2.25);
    others.units.retain(|u| u.id != LEAD);
    let mut before = start.clone();
    before.units.retain(|u| u.id != LEAD);
    assert_eq!(others, before);
    // The corner, on the way up.
    assert_eq!(unit(1.25, LEAD).facing, Facing::Up);
    assert_eq!(
        (unit(1.6, LEAD).pos, unit(1.6, LEAD).facing),
        (p(3, 3), Facing::Right)
    );
    // Arrived, next to the brigand.
    let lead = unit(3.5, LEAD);
    assert_eq!(
        (lead.pos, lead.facing, lead.offset),
        (p(7, 3), Facing::Down, (0.0, 0.0))
    );
    // The first blow lands at 4.2: the brigand flashes with 40% of its HP.
    assert_eq!(unit(4.199, BRIGAND).hp.0, full);
    let (lead, brigand) = (unit(4.2, LEAD), unit(4.2, BRIGAND));
    assert!(near(lead.offset, (LUNGE_TILES, 0.0)), "{:?}", lead.offset);
    assert_eq!(lead.facing, Facing::Right);
    assert_eq!(brigand.hp, ((full * 40 + 99) / 100, full));
    assert!(brigand.hp.0 > 0 && brigand.hp.0 < full);
    assert!((brigand.flash - 1.0).abs() < 1e-4 && brigand.fade == 0.0);
    assert_eq!(unit(4.5, BRIGAND).flash.to_bits(), 0.0_f32.to_bits());
    // The second, at 5.2, fells it: it fades out and is gone by 5.7.
    let brigand = unit(5.45, BRIGAND);
    assert_eq!(brigand.hp.0, 0);
    assert!((brigand.fade - 0.5).abs() < 1e-4, "{}", brigand.fade);
    assert!(scene(5.699).unit(BRIGAND).is_some());
    for t in [5.7, 5.999] {
        assert_eq!(scene(t).unit(BRIGAND), None, "{t}");
        assert_eq!(scene(t).units.len(), start.units.len() - 1);
        assert_eq!(scene(t).unit_at(p(8, 3)), None);
    }
    // The shots after it are no map; then it all starts again.
    for t in [6.0, 9.0, 12.0, 17.0, 19.999] {
        assert_eq!(player.scene(t), None, "{t}");
    }
    assert_eq!(scene(20.0), start);
    assert_eq!(scene(22.25), scene(2.25));
    // The terrain never changes.
    assert_eq!(scene(5.9).tiles, start.tiles);
    assert_eq!(scene(5.9).look, start.look);
}

#[test]
fn shots_of_one_action_show_the_same_moment_from_their_own_action_from() {
    let mut ctx = ctx();
    let def = ctx.game.content.cinematics.get_mut("test").unwrap();
    // The same list twice: from its start, then (at six seconds, from
    // another place) from 1.5 seconds into it.
    let mut second = def.shots[0].clone();
    second.at_ms = 6000;
    match &mut second.shot {
        Shot::MapPan {
            action: Some(action),
            from,
            to,
            zoom,
            ..
        } => {
            action.from_ms = 1500;
            (*from, *to, *zoom) = (TilePoint::new(2.0, 2.0), TilePoint::new(9.0, 6.0), 2);
        }
        other => panic!("{other:?}"),
    }
    def.shots[1] = second;
    let player = player(&ctx);
    let units = |t: f32| player.scene(t).unwrap().units;
    // The action clock at 2.25 in the first shot is 2.25; the second
    // shot reaches it 0.75 seconds in.
    assert_eq!(units(2.25), units(6.75));
    assert_eq!(units(1.5), units(6.0));
    assert_eq!(units(5.45), units(9.95));
    assert_ne!(units(2.25), units(6.0));
    // The second shot never shows the walk's start, and the first never
    // shows what comes after its cut.
    let start = battle_scene(&ctx, &ctx.game.content.battles["test"]).units;
    assert_eq!(units(0.0), start);
    let lead = |t: f32| player.scene(t).unwrap().unit(LEAD).cloned().unwrap();
    assert_eq!(lead(6.1).pos, Pos::new(3, 3));
    // Another camera on the same moment.
    assert_ne!(
        player.frame(&ctx, 2.25, SCREEN),
        player.frame(&ctx, 6.75, SCREEN)
    );
    // Past the list's end: how it all ended.
    let def = ctx.game.content.cinematics.get_mut("test").unwrap();
    if let Shot::MapPan {
        action: Some(action),
        ..
    } = &mut def.shots[1].shot
    {
        action.from_ms = 600_000;
    }
    let player = self::player(&ctx);
    let ended = player.scene(6.0).unwrap();
    assert_eq!(ended, player.scene(9.9).unwrap());
    assert_eq!(ended.unit(LEAD).unwrap().pos, Pos::new(7, 3));
    assert_eq!(ended.unit(BRIGAND), None);
}

/// Puts a unit back as it was before something an action did to it.
type Undo = fn(&mut crate::map_view::UnitView);

/// A walking, a struck and a falling unit each change what both skins
/// paint, and what is drawn is the scene of that moment.
#[test]
fn both_skins_paint_a_moving_a_flashing_and_a_fading_unit() {
    let mut ctx = ctx();
    for skin in ["glyph", "sprite"] {
        ctx.game.map_skin = skin_named(&ctx.game.content, skin).unwrap();
        let player = player(&ctx);
        let painter = shot_skin(&ctx);
        let paint = |scene: &MapScene| MapPicture::paint(&ctx, painter.as_ref(), scene).unwrap();
        let start = battle_scene(&ctx, &ctx.game.content.battles["test"]);
        let moments: [(&str, f32, UnitId, Undo); 4] = [
            ("walking", 2.25, LEAD, |u| u.pos = Pos::new(3, 5)),
            ("lunging", 4.2, LEAD, |u| u.offset = (0.0, 0.0)),
            ("flashing", 4.3, BRIGAND, |u| u.flash = 0.0),
            ("fading", 5.45, BRIGAND, |u| u.fade = 0.0),
        ];
        for (what, t, id, undo) in moments {
            let scene = player.scene(t).unwrap();
            let mut without = scene.clone();
            let unit = without.units.iter_mut().find(|u| u.id == id).unwrap();
            undo(unit);
            assert_ne!(scene, without, "{what}");
            // The glyph skin's units stand in cells: a lean of a few
            // pixels is the sprite skin's to show.
            if (what, skin) != ("lunging", "glyph") {
                assert_ne!(paint(&scene), paint(&without), "{what} under {skin}");
            }
            assert_ne!(paint(&scene), paint(&start), "{what} under {skin}");
            // The frame shows that scene, and the same one each time.
            let buf = draw(&ctx, &player, t, 1.0);
            assert_eq!(buf.backdrop().unwrap().scene(), &*paint(&scene).scene);
            assert_eq!(buf, draw(&ctx, &player, t, 1.0), "{what} under {skin}");
        }
    }
}

#[test]
fn an_acted_shot_is_painted_by_the_skin_at_the_arts_own_size() {
    let mut ctx = ctx();
    let sprite = skin_named(&ctx.game.content, "sprite").unwrap();
    ctx.game.map_skin = sprite.at_zoom(MapZoom::X2).unwrap();
    let player = player(&ctx);
    // Mid-walk, as at the start: 14 × 8 tiles of 24 px, in whole cells.
    for t in [0.0, 2.25] {
        let buf = draw(&ctx, &player, t, 1.0);
        let scene = buf.backdrop().unwrap().scene();
        assert_eq!((scene.width(), scene.height()), (42, 12), "{t}");
    }
}

#[test]
fn the_test_cinematic_mid_walk_and_mid_strike() {
    let ctx = ctx();
    let player = player(&ctx);
    let frame = |t: f32| draw(&ctx, &player, t, 1.0).to_snapshot(&ctx.palette);
    assert_snapshot!("mid_walk", frame(2.25));
    assert_snapshot!("mid_strike", frame(4.3));
    assert_snapshot!("the_brigand_falling", frame(5.45));
    assert_eq!(frame(2.25), frame(2.25));
    assert_eq!(frame(4.3), frame(24.3));
}

#[test]
fn a_cinematic_or_battle_that_is_not_there() {
    let mut ctx = ctx();
    assert!(CinematicPlayer::new(&ctx, "nope", None).is_none());
    // The battle gone after the content was checked: the shot is blank.
    ctx.game.content.battles.remove("test");
    let player = player(&ctx);
    assert_eq!(player.frame(&ctx, 3.0, SCREEN), Frame::Blank);
    let buf = draw(&ctx, &player, 3.0, 1.0);
    assert!(buf.backdrop().is_none());
    let (text, black) = (
        ctx.palette.get(UiColor::Text),
        ctx.palette.get(UiColor::Black),
    );
    assert_eq!(buf.get(0, 0), Some(&Cell::new(' ', text, black)));
    assert_eq!(buf.get(99, 31), Some(&Cell::new(' ', text, black)));
    // The logo still plays.
    assert_eq!(
        player.frame(&ctx, 17.0, SCREEN),
        Frame::Logo { sparks: None }
    );
}

#[test]
fn a_shot_is_painted_at_the_arts_own_size_whatever_the_players_map_zoom() {
    let mut ctx = ctx();
    // The glyph skin can't zoom: it is used as it is.
    assert_eq!(shot_skin(&ctx).name(), "glyph");
    let sprite = skin_named(&ctx.game.content, "sprite").unwrap();
    for zoom in MapZoom::ALL {
        ctx.game.map_skin = sprite.at_zoom(zoom).unwrap();
        assert_eq!(ctx.game.map_skin.zoom(), Some(zoom));
        assert_eq!(shot_skin(&ctx).zoom(), Some(MapZoom::X1));
        assert_eq!(shot_skin(&ctx).tileset_id(), ctx.game.map_skin.tileset_id());
        // 14 × 8 tiles of 24 px, in whole cells.
        let buf = draw(&ctx, &player(&ctx), 0.0, 1.0);
        let scene = buf.backdrop().unwrap().scene();
        assert_eq!((scene.width(), scene.height()), (42, 12));
    }
}

/// Ticket 0833: the title's battle moves as the game plays one. Every walk
/// goes at the speed the game's pictures walk in a battle (the bought art
/// the game ships with), and only one unit walks at a time.
#[test]
fn the_title_walks_one_unit_at_a_time_at_the_games_speed() {
    use crate::screens::battle::walk::SPRITE_WALK_TILES_PER_S;
    let ctx = ctx();
    // The map shots share one action list: each walk once.
    let mut walks = std::collections::BTreeSet::new();
    for shot in &ctx.game.content.cinematics["title"].shots {
        let Shot::MapPan {
            action: Some(action),
            ..
        } = &shot.shot
        else {
            continue;
        };
        for entry in &action.actions {
            if let trpg_content::Act::Walk { unit, path, ms, .. } = &entry.act {
                let tiles: u32 = path.windows(2).map(|w| Pos::manhattan(w[0], w[1])).sum();
                walks.insert((entry.at_ms, unit.0, tiles, *ms));
            }
        }
    }
    assert!(walks.len() >= 10, "{walks:?}");
    let mut free_from = 0;
    for &(at_ms, unit, tiles, ms) in &walks {
        #[allow(clippy::cast_precision_loss)]
        let due = tiles as f32 * 1000.0 / SPRITE_WALK_TILES_PER_S;
        #[allow(clippy::cast_precision_loss)]
        let off = (ms as f32 - due).abs();
        assert!(
            off <= 1.0,
            "unit {unit} at {at_ms} ms: {tiles} tiles in {ms} ms"
        );
        assert!(
            at_ms >= free_from,
            "unit {unit} at {at_ms} ms walks with another"
        );
        free_from = at_ms + ms;
    }
}

/// Ticket 0833: a fight of the title is never cut off: each combat shot
/// lasts at least as long as its fight.
#[test]
fn the_titles_fights_end_before_their_cuts() {
    let ctx = ctx();
    let title = &ctx.game.content.cinematics["title"];
    let lead = LeadProfile::new(
        ctx.words().name(&ctx.game.content.names, LEAD_ID),
        LeadGender::Male,
    );
    let mut fights = 0;
    for (i, shot) in title.shots.iter().enumerate() {
        let Shot::Combat(combat) = &shot.shot else {
            continue;
        };
        let duel = Duel::new(&ctx, combat, &lead).unwrap();
        let end = title.shots.get(i + 1).map_or(title.length_ms, |s| s.at_ms);
        #[allow(clippy::cast_precision_loss)]
        let lasts = (end - shot.at_ms) as f32 / 1000.0;
        assert!(
            duel.seconds() <= lasts,
            "shot {}: {} s",
            i + 1,
            duel.seconds()
        );
        fights += 1;
    }
    assert!(fights >= 1);
}
