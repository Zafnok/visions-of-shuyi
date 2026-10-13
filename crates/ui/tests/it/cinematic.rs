//! The "Play test cinematic" debug tool through the Harness (ticket 0817,
//! ADR-0060): `assets-test/cinematics/test.ron` plays on the screen's own
//! clock (a pan over the test battle's map for six seconds, a hard cut to
//! the logo for four), loops, and closes on Cancel.

use insta::assert_snapshot;
use trpg_ui::cinematic::Frame;
use trpg_ui::debug::CinematicScreen;
use trpg_ui::glyph_buffer::Rect;
use trpg_ui::harness::Harness;
use trpg_ui::input::Layout;

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
    // Its middle: half of the 24 pixels there are to move.
    h.wait(3.0 - time(&h));
    assert!((time(&h) - 3.0).abs() < 0.02, "{}", time(&h));
    assert!((pan_x(&h) - 12.0).abs() < 0.2, "{}", pan_x(&h));
    let middle = h.snapshot();
    assert!(middle.contains("--- backdrop: clip 0,0 100x32  origin 12."));
    assert!(middle.contains("zoom 4 ---"));
    assert_snapshot!("cinematic_pan_middle", middle);
    // Just before the cut it is nearly at its end.
    h.wait(2.9);
    assert!(pan_x(&h) > 23.9, "{}", pan_x(&h));
    // The logo: a hard cut, no map behind it.
    h.wait(1.1);
    assert_eq!(frame(&h), Frame::Logo { sparks: None });
    let logo = h.snapshot();
    assert!(!logo.contains("--- backdrop"));
    assert_snapshot!("cinematic_logo", logo);
    // Ten seconds in, it starts again.
    h.wait(3.5);
    assert!(time(&h) < 1.0, "{}", time(&h));
    assert!(pan_x(&h) < 1.0);
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
}
