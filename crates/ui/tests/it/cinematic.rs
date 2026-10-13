//! The "Play test cinematic" debug tool through the Harness (tickets 0817
//! and 0827, ADR-0060, ADR-0061): `assets-test/cinematics/test.ron` plays
//! on the screen's own clock (a pan over the test battle's map for six
//! seconds, in which the lead walks up to the brigand and fells it, a
//! hard cut to the logo for four), loops, and closes on Cancel.

use insta::assert_snapshot;
use trpg_core::{Pos, UnitId};
use trpg_ui::cinematic::Frame;
use trpg_ui::debug::CinematicScreen;
use trpg_ui::glyph_buffer::Rect;
use trpg_ui::harness::Harness;
use trpg_ui::input::Layout;
use trpg_ui::map_view::{Facing, MapScene, UnitView};

const SCREEN: Rect = Rect::new(0, 0, 100, 32);

/// The title with the test cinematic opened from the debug menu (its last
/// tool but two: the scene camera and the map skin come after it).
fn opened() -> Harness {
    let mut h = Harness::with_layout(Layout::RightHanded);
    h.keys("F2 Up Up Up f");
    assert_eq!(h.screens(), ["title", "debug_menu", "cinematic"]);
    h
}

/// Seconds into the cinematic (not a number if it isn't on screen).
fn time(h: &Harness) -> f32 {
    let screen = h.game().screen::<CinematicScreen>();
    screen.map_or(f32::NAN, CinematicScreen::time)
}

/// What the cinematic shows now (nothing if it isn't on screen).
fn frame(h: &Harness) -> Frame {
    let screen = h.game().screen::<CinematicScreen>();
    let shown = screen.and_then(|s| Some(s.player()?.frame(h.game().ctx(), s.time(), SCREEN)));
    shown.unwrap_or(Frame::Blank)
}

/// The map scene the cinematic shows now, if it shows a map.
fn scene(h: &Harness) -> Option<MapScene> {
    let screen = h.game().screen::<CinematicScreen>()?;
    screen.player()?.scene(screen.time())
}

/// The unit `id` as the cinematic shows it now, if it is there.
fn unit(h: &Harness, id: UnitId) -> Option<UnitView> {
    scene(h)?.unit(id).cloned()
}

const LEAD: UnitId = UnitId(1);
const BRIGAND: UnitId = UnitId(3);

/// The left edge of the map picture on screen, in its own pixels.
fn pan_x(h: &Harness) -> f32 {
    match frame(h) {
        Frame::MapPan { origin_px, zoom: 4 } => origin_px.0,
        other => panic!("not the pan: {other:?}"),
    }
}

#[test]
fn the_test_cinematic_plays_cuts_to_the_logo_and_loops() {
    let mut h = opened();
    // The start of the pan: the picture's left edge.
    assert!(time(&h) < 0.05, "{}", time(&h));
    assert!(pan_x(&h) < 0.01);
    assert_snapshot!("cinematic_pan_start", h.snapshot());
    // Everyone stands where the battle starts.
    let (lead, brigand) = (unit(&h, LEAD).unwrap(), unit(&h, BRIGAND).unwrap());
    assert_eq!((lead.pos, brigand.pos), (Pos::new(3, 5), Pos::new(8, 3)));
    let full = brigand.hp;
    assert_eq!(full.0, full.1);
    // Mid-walk: the lead has turned the corner and heads right, between
    // two tiles of row 3.
    h.wait(2.25 - time(&h));
    assert!((time(&h) - 2.25).abs() < 0.02, "{}", time(&h));
    let lead = unit(&h, LEAD).unwrap();
    assert_eq!((lead.pos, lead.facing), (Pos::new(4, 3), Facing::Right));
    assert!(
        lead.offset.0 > 0.3 && lead.offset.0 < 0.7,
        "{:?}",
        lead.offset
    );
    assert_eq!(unit(&h, BRIGAND).unwrap(), brigand);
    assert_snapshot!("cinematic_mid_walk", h.snapshot());
    // The pan's middle: half of the 24 pixels there are to move.
    h.wait(3.0 - time(&h));
    assert!((time(&h) - 3.0).abs() < 0.02, "{}", time(&h));
    assert!((pan_x(&h) - 12.0).abs() < 0.2, "{}", pan_x(&h));
    let middle = h.snapshot();
    assert!(middle.contains("--- backdrop: clip 0,0 100x32  origin 12."));
    assert!(middle.contains("zoom 4 ---"));
    // Mid-strike: the lead, arrived beside the brigand, leans at it, and
    // the brigand flashes with less HP.
    h.wait(4.3 - time(&h));
    let (lead, struck) = (unit(&h, LEAD).unwrap(), unit(&h, BRIGAND).unwrap());
    assert_eq!((lead.pos, lead.facing), (Pos::new(7, 3), Facing::Right));
    assert!(
        lead.offset.0 > 0.0 && lead.offset.1 == 0.0,
        "{:?}",
        lead.offset
    );
    assert!(struck.hp.0 > 0 && struck.hp.0 < full.0, "{:?}", struck.hp);
    assert!(struck.flash > 0.2 && struck.flash < 0.8, "{}", struck.flash);
    assert_eq!((struck.pos, struck.hp.1), (brigand.pos, full.1));
    assert_snapshot!("cinematic_mid_strike", h.snapshot());
    // Just before the cut the pan is nearly at its end, and the brigand
    // has fallen: the others are still there.
    h.wait(5.9 - time(&h));
    assert!(pan_x(&h) > 23.9, "{}", pan_x(&h));
    assert_eq!(unit(&h, BRIGAND), None);
    assert_eq!(scene(&h).unwrap().units.len(), 2);
    assert_eq!(unit(&h, LEAD).unwrap().pos, Pos::new(7, 3));
    // The logo: a hard cut, no map behind it.
    h.wait(1.1);
    assert_eq!(frame(&h), Frame::Logo { sparks: None });
    assert_eq!(scene(&h), None);
    let logo = h.snapshot();
    assert!(!logo.contains("--- backdrop"));
    assert_snapshot!("cinematic_logo", logo);
    // Ten seconds in, it starts again.
    h.wait(3.5);
    assert!(time(&h) < 1.0, "{}", time(&h));
    assert!(pan_x(&h) < 1.0);
    // The brigand is back where it started, unhurt.
    assert_eq!(unit(&h, BRIGAND), Some(brigand));
    // Nothing but Cancel does anything, and nothing makes a sound.
    h.clear_audio();
    h.keys("f Up Down Left Right");
    assert_eq!(h.top_screen(), "cinematic");
    assert!(h.sounds().is_empty(), "{:?}", h.sounds());
}

#[test]
fn cancel_closes_it() {
    let mut h = opened();
    h.wait(7.0);
    h.clear_audio();
    h.keys("d");
    assert_eq!(h.screens(), ["title", "debug_menu"]);
    assert_eq!(h.sounds(), ["menu_cancel"]);
    // Opened again, it starts from the beginning.
    h.keys("f");
    assert_eq!(h.top_screen(), "cinematic");
    assert!(time(&h) < 0.05);
    assert!(matches!(frame(&h), Frame::MapPan { .. }));
}

#[test]
fn the_debug_key_does_nothing_on_it() {
    let mut h = opened();
    h.keys("F2");
    assert_eq!(h.screens(), ["title", "debug_menu", "cinematic"]);
}

#[test]
fn it_follows_the_map_skin_in_use() {
    let mut h = Harness::with_layout(Layout::RightHanded);
    h.with_map_skin("sprite");
    h.keys("F2 Up Up Up f");
    let scene = h.game().buffer().backdrop().unwrap().scene();
    // The test tileset: 14 × 8 tiles of 24 px, in whole cells.
    assert_eq!((scene.width(), scene.height()), (42, 12));
    assert!(!scene.sprites().is_empty());
    // Its units are sprites: a walking one is drawn somewhere else each
    // moment, the ground the same.
    let sprites = |h: &Harness| h.game().buffer().backdrop().unwrap().scene().sprites();
    let standing = sprites(&h);
    h.wait(2.25);
    let walking = sprites(&h);
    assert_eq!(standing.len(), walking.len());
    assert_ne!(standing, walking);
    // Felled, the brigand's sprites are gone.
    h.wait(3.6);
    assert!(sprites(&h).len() < standing.len());
}
