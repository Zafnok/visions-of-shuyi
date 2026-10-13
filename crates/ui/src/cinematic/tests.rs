//! Tests of the timeline and the player, on the test cinematic
//! (`assets-test/cinematics/test.ron`): a pan over the test battle's map
//! at 4× for six seconds, then the logo for four.

use insta::assert_snapshot;
use proptest::prelude::*;
use trpg_content::TilePoint;

use super::*;
use crate::console::{CONSOLE_H, CONSOLE_W};
use crate::map_view::skin_named;
use crate::screen::tests::ctx;
use crate::screens::title::glyph;

const SCREEN: Rect = Rect::new(0, 0, 100, 32);

fn test_timeline() -> Timeline {
    Timeline::new(vec![0, 6000], 10_000)
}

fn at(index: usize, progress: f32) -> ShotAt {
    ShotAt { index, progress }
}

fn player(ctx: &Ctx) -> CinematicPlayer {
    CinematicPlayer::new(ctx, "test").unwrap()
}

/// `player`'s frame at `t` and `brightness`, over a stale buffer.
fn draw(ctx: &Ctx, player: &CinematicPlayer, t: f32, brightness: f32) -> GlyphBuffer {
    let p = &ctx.palette;
    let stale = Cell::new('x', p.get(UiColor::Enemy), p.get(UiColor::PanelBg));
    let mut buf = GlyphBuffer::new(CONSOLE_W, CONSOLE_H, stale);
    player.draw(ctx, t, brightness, &mut buf);
    buf
}

fn near(a: (f32, f32), b: (f32, f32)) -> bool {
    (a.0 - b.0).abs() < 1e-3 && (a.1 - b.1).abs() < 1e-3
}

#[test]
fn a_time_falls_in_the_shot_that_started_last() {
    let t = test_timeline();
    assert_eq!(t.length_ms(), 10_000);
    // A shot's start, middle and end.
    assert_eq!(t.at_ms(0), at(0, 0.0));
    assert_eq!(t.at_ms(3000), at(0, 0.5));
    assert_eq!(t.at_ms(1500), at(0, 0.25));
    let end = t.at_ms(5999);
    assert_eq!(end.index, 0);
    assert!(end.progress > 0.999 && end.progress < 1.0, "{end:?}");
    // The next shot starts on the millisecond; the last ends at the length.
    assert_eq!(t.at_ms(6000), at(1, 0.0));
    assert_eq!(t.at_ms(8000), at(1, 0.5));
    assert_eq!(t.at_ms(9000), at(1, 0.75));
    assert_eq!(t.at_ms(9999).index, 1);
    // The wrap: the length is the start again, and so on round.
    assert_eq!(t.at_ms(10_000), at(0, 0.0));
    assert_eq!(t.at_ms(13_000), at(0, 0.5));
    assert_eq!(t.at_ms(28_000), at(1, 0.5));
    assert_eq!(t.wrapped_ms(28_000), 8000);
    assert_eq!(t.wrapped_ms(9999), 9999);
    assert_eq!(t.wrapped_ms(u64::MAX), 1615);
    // Three shots: the middle one ends where the third starts.
    let three = Timeline::new(vec![0, 1000, 3000], 4000);
    assert_eq!(three.at_ms(999).index, 0);
    assert_eq!(three.at_ms(1000), at(1, 0.0));
    assert_eq!(three.at_ms(2500), at(1, 0.75));
    assert_eq!(three.at_ms(3500), at(2, 0.5));
}

#[test]
fn seconds_are_whole_milliseconds_from_zero() {
    let t = test_timeline();
    assert_eq!(t.at(0.0), at(0, 0.0));
    assert_eq!(t.at(3.0), at(0, 0.5));
    assert_eq!(t.at(6.0), at(1, 0.0));
    assert_eq!(t.at(10.0), at(0, 0.0));
    assert_eq!(t.at(18.0), at(1, 0.5));
    assert_eq!(millis(1.2344), 1234);
    assert_eq!(millis(1.2346), 1235);
    assert_eq!(millis(0.0004), 0);
    assert_eq!(millis(0.0006), 1);
    for bad in [-0.5, -1e9, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert_eq!(millis(bad), 0, "{bad}");
        assert_eq!(t.at(bad), at(0, 0.0), "{bad}");
    }
    assert_eq!(millis(f32::MAX), u64::MAX);
}

#[test]
fn a_timeline_never_divides_by_nothing() {
    // No length: one millisecond, always its start.
    let none = Timeline::new(vec![0], 0);
    assert_eq!(none.length_ms(), 1);
    assert_eq!(none.at_ms(7), at(0, 0.0));
    // No shots: the whole length is "shot 0".
    let empty = Timeline::new(vec![], 1000);
    assert_eq!(empty.at_ms(250), at(0, 0.25));
    // A first shot that starts late: the time before it is its start.
    let late = Timeline::new(vec![400, 800], 1000);
    assert_eq!(late.at_ms(100), at(0, 0.0));
    assert_eq!(late.at_ms(600), at(0, 0.5));
    assert_eq!(late.at_ms(900), at(1, 0.5));
    // Two shots at once: the later one shows.
    let twice = Timeline::new(vec![0, 500, 500], 1000);
    assert_eq!(twice.at_ms(500), at(2, 0.0));
    assert_eq!(twice.at_ms(750), at(2, 0.5));
    // A shot past the end never shows; the one before it runs towards it.
    let past = Timeline::new(vec![0, 2000], 1000);
    assert_eq!(past.at_ms(500), at(0, 0.25));
    assert_eq!(past.at_ms(1500), at(0, 0.25));
}

#[test]
fn the_timeline_of_a_cinematic_is_its_shots_starts_and_its_length() {
    let ctx = ctx();
    let def = &ctx.content.cinematics["test"];
    assert_eq!(Timeline::of(def), test_timeline());
    assert_eq!(player(&ctx).timeline(), &test_timeline());
}

#[test]
fn easing_is_slow_at_both_ends() {
    let eased = |p: f32, expected: f32| {
        let got = ease_in_out(p);
        assert!((got - expected).abs() < 1e-6, "{p}: {got}");
    };
    eased(0.0, 0.0);
    eased(0.5, 0.5);
    eased(1.0, 1.0);
    eased(0.25, 0.15625);
    eased(0.75, 0.84375);
    eased(-3.0, 0.0);
    eased(7.0, 1.0);
    eased(f32::NAN, 0.0);
    eased(f32::INFINITY, 0.0);
}

proptest! {
    #[test]
    fn any_time_is_in_a_shot_and_the_same_one_a_length_later(
        gaps in prop::collection::vec(1u32..5000, 0..6),
        tail in 1u32..5000,
        t_ms in prop_oneof![0u64..100_000, any::<u64>()],
    ) {
        let mut starts = vec![0];
        for gap in gaps {
            starts.push(starts.last().copied().unwrap_or(0) + gap);
        }
        let length = starts.last().copied().unwrap_or(0) + tail;
        let timeline = Timeline::new(starts.clone(), length);
        let shot = timeline.at_ms(t_ms);
        prop_assert!(shot.index < starts.len());
        prop_assert!((0.0..=1.0).contains(&shot.progress), "{shot:?}");
        // The shot is the one whose span holds the wrapped time.
        let t = timeline.wrapped_ms(t_ms);
        prop_assert!(t < length);
        prop_assert!(starts[shot.index] <= t);
        prop_assert!(starts.get(shot.index + 1).is_none_or(|&next| t < next));
        if let Some(later) = t_ms.checked_add(u64::from(length)) {
            prop_assert_eq!(timeline.at_ms(later), shot);
        }
    }

    #[test]
    fn any_seconds_are_in_a_shot(t in any::<f32>()) {
        let shot = test_timeline().at(t);
        prop_assert!(shot.index < 2);
        prop_assert!((0.0..=1.0).contains(&shot.progress), "{shot:?}");
    }

    #[test]
    fn easing_stays_between_its_ends_and_never_goes_back(a in 0f32..=1.0, b in 0f32..=1.0) {
        let (low, high) = (a.min(b), a.max(b));
        prop_assert!((0.0..=1.0).contains(&ease_in_out(a)));
        prop_assert!(ease_in_out(low) <= ease_in_out(high));
        // The same shape from both ends.
        prop_assert!((ease_in_out(a) + ease_in_out(1.0 - a) - 1.0).abs() < 1e-5);
    }
}

#[test]
fn a_battles_scene_is_its_whole_map_with_its_units_where_they_start() {
    let ctx = ctx();
    let def = &ctx.content.battles["test"];
    let scene = battle_scene(&ctx, def);
    assert_eq!((scene.origin, scene.size), (Pos::new(0, 0), (14, 8)));
    assert_eq!(scene.tiles.len(), 14 * 8);
    assert!(scene.tiles.iter().all(|t| t.terrain.is_some()));
    assert_eq!(scene.look, ctx.content.battle_look(def));
    // The lead and the test knight in their slots, and the brigand.
    let places: Vec<(i32, i32)> = scene.units.iter().map(|u| (u.pos.x, u.pos.y)).collect();
    assert_eq!(places, [(3, 5), (3, 6), (8, 3)]);
    assert!(scene.units.iter().all(|u| !u.acted && u.fade == 0.0));
    assert!(scene.cursor.is_none() && scene.path.is_empty() && scene.threat.is_none());
}

#[test]
fn a_map_picture_is_the_whole_scene_as_the_skin_paints_it() {
    let mut ctx = ctx();
    let scene = battle_scene(&ctx, &ctx.content.battles["test"]);
    // Glyphs: 14 × 8 tiles of two cells by one, 16 px each way.
    let picture = MapPicture::paint(&ctx, ctx.map_skin.as_ref(), &scene).unwrap();
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
        ctx.map_skin.as_ref(),
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
    ctx.map_skin = skin_named(&ctx.content, "sprite").unwrap();
    let picture = MapPicture::paint(&ctx, ctx.map_skin.as_ref(), &scene).unwrap();
    assert_eq!((picture.scene.width(), picture.scene.height()), (42, 12));
    assert_eq!(picture.tile_px, (24, 24));
    assert_eq!(picture.point_px((1.0, 2.0)), (36.0, 60.0));
    assert!(!picture.scene.sprites().is_empty());
    // A size that isn't whole cells is rounded up to them.
    let odd = MapScene::new(Pos::new(0, 0), (1, 1));
    let picture = MapPicture::paint(&ctx, ctx.map_skin.as_ref(), &odd).unwrap();
    assert_eq!((picture.scene.width(), picture.scene.height()), (3, 2));
    assert_eq!(picture.size_px(), (24, 32));
    // Nothing to paint.
    assert!(
        MapPicture::paint(
            &ctx,
            ctx.map_skin.as_ref(),
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
    // A hard cut to the logo, then round again.
    assert_eq!(
        player.frame(&ctx, 6.0, SCREEN),
        Frame::Logo { sparks: None }
    );
    assert_eq!(
        player.frame(&ctx, 9.999, SCREEN),
        Frame::Logo { sparks: None }
    );
    assert!(near(origin(10.0), (0.0, 0.0)));
    assert!(near(origin(13.0), (12.0, 0.0)));
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
    let def = ctx.content.cinematics.get_mut("test").unwrap();
    def.shots[0].shot = Shot::MapPan {
        battle: "test".to_owned(),
        from: TilePoint::new(9.5, 5.0),
        to: TilePoint::new(3.5, 2.0),
        zoom: 2,
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
        player.frame(&ctx, 7.25, SCREEN),
        Frame::Logo { sparks: None }
    );
    ctx.idle_motion = IdleMotion::On;
    assert_eq!(
        player.frame(&ctx, 7.25, SCREEN),
        Frame::Logo { sparks: Some(7250) }
    );
    // The same moment a round later: the same sparks.
    assert_eq!(
        player.frame(&ctx, 17.25, SCREEN),
        Frame::Logo { sparks: Some(7250) }
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
    let scene = battle_scene(&ctx, &ctx.content.battles["test"]);
    let picture = MapPicture::paint(&ctx, ctx.map_skin.as_ref(), &scene).unwrap();
    assert_eq!(backdrop.scene(), &*picture.scene);
    // Every cell is a hole, and nothing is left of the frame before.
    let hole = Cell::see_through(' ', ctx.palette.get(UiColor::Text));
    for y in 0..i32::from(CONSOLE_H) {
        for x in 0..i32::from(CONSOLE_W) {
            assert_eq!(buf.get(x, y), Some(&hole), "({x}, {y})");
        }
    }
    assert!(buf.items().is_empty());
}

#[test]
fn the_logo_shot_is_the_titles_own_picture() {
    let mut ctx = ctx();
    let backdrop = ctx.content.title.backdrops.get(1).copied();
    assert!(backdrop.is_some());
    for (motion, sparks) in [(IdleMotion::Off, None), (IdleMotion::On, Some(7250))] {
        ctx.idle_motion = motion;
        // With no backdrop given, the logo is on none.
        for given in [None, backdrop] {
            let player = player(&ctx).with_backdrop(given);
            let buf = draw(&ctx, &player, 7.25, 1.0);
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
    for t in [0.0, 1.7, 3.0, 5.999, 6.0, 8.3] {
        let first = draw(&ctx, &player, t, 1.0);
        assert_eq!(draw(&ctx, &player, t, 1.0), first, "{t}");
        // Whatever was drawn in between, and a whole round later.
        draw(&ctx, &player, 9.1, 0.3);
        assert_eq!(draw(&ctx, &player, t, 1.0), first, "{t}");
        assert_eq!(draw(&ctx, &player, t + 10.0, 1.0), first, "{t}");
        assert_ne!(draw(&ctx, &player, t + 0.5, 1.0), first, "{t}");
    }
}

#[test]
fn brightness_fades_the_whole_frame_cells_and_items_alike() {
    let ctx = ctx();
    let player = player(&ctx).with_backdrop(ctx.content.title.backdrops.first().copied());
    for t in [3.0, 7.0] {
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
    // half as solid.
    let half = draw(&ctx, &player, 7.0, 0.5);
    let opacities: Vec<u8> = half.sprites().iter().map(|s| s.opacity).collect();
    assert_eq!(opacities, [128; 4]);
    // Black is black.
    let black = draw(&ctx, &player, 7.0, 0.0);
    assert!(black.sprites().iter().all(|s| s.opacity == 0));
    assert_snapshot!(
        "pan_middle_half_bright",
        draw(&ctx, &player, 3.0, 0.5).to_snapshot(&ctx.palette)
    );
}

#[test]
fn a_pan_over_a_sprite_map_fades_its_sprites() {
    let mut ctx = ctx();
    ctx.map_skin = skin_named(&ctx.content, "sprite").unwrap();
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
        let expected = (f32::from(full.opacity) * 0.5).round() as u8;
        assert_eq!(half.opacity, expected);
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
    let def = ctx.content.cinematics.get_mut("test").unwrap();
    let pan = def.shots[0].shot.clone();
    def.shots[1].shot = pan;
    let player = player(&ctx);
    let scene = |t: f32| {
        let buf = draw(&ctx, &player, t, 1.0);
        std::ptr::from_ref(buf.backdrop().unwrap().scene())
    };
    assert_eq!(scene(1.0), scene(7.0));
}

#[test]
fn a_cinematic_or_battle_that_is_not_there() {
    let mut ctx = ctx();
    assert!(CinematicPlayer::new(&ctx, "nope").is_none());
    // The battle gone after the content was checked: the shot is blank.
    ctx.content.battles.remove("test");
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
        player.frame(&ctx, 7.0, SCREEN),
        Frame::Logo { sparks: None }
    );
}

#[test]
fn a_shot_is_painted_at_the_arts_own_size_whatever_the_players_map_zoom() {
    let mut ctx = ctx();
    // The glyph skin can't zoom: it is used as it is.
    assert_eq!(shot_skin(&ctx).name(), "glyph");
    let sprite = skin_named(&ctx.content, "sprite").unwrap();
    for zoom in MapZoom::ALL {
        ctx.map_skin = sprite.at_zoom(zoom).unwrap();
        assert_eq!(ctx.map_skin.zoom(), Some(zoom));
        assert_eq!(shot_skin(&ctx).zoom(), Some(MapZoom::X1));
        assert_eq!(shot_skin(&ctx).tileset_id(), ctx.map_skin.tileset_id());
        // 14 × 8 tiles of 24 px, in whole cells.
        let buf = draw(&ctx, &player(&ctx), 0.0, 1.0);
        let scene = buf.backdrop().unwrap().scene();
        assert_eq!((scene.width(), scene.height()), (42, 12));
    }
}
