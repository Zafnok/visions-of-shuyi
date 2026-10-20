//! The Zoom map key (ticket 0439, `docs/design/look-and-feel.md`, *Zoom*):
//! under a skin that paints the whole map from pictures it switches the
//! battle map between 1× and 2×, the cursor's tile stays in view, and the
//! game remembers the choice. Played on the generated test tileset (24 px
//! tiles: 23 × 20 of them in the map area at 1×, 11 × 10 at 2×).

use insta::assert_snapshot;
use trpg_core::Pos;
use trpg_ui::harness::{FRAME_DT, Harness};
use trpg_ui::input::{Action, Chord, PadKind};
use trpg_ui::input::{LEFT_HANDED, RIGHT_HANDED};
use trpg_ui::settings::MapZoom;

/// The tiles the test tileset shows at 1× and at 2×.
const VIEW_X1: (i32, i32) = (23, 20);
const VIEW_X2: (i32, i32) = (11, 10);

/// The Quick Battle (a 14 × 8 map) under the skin `skin`, past
/// Preparations and its `PLAYER PHASE` banner, with the game `h`, reached
/// with `keys`.
fn battle_with(mut h: Harness, skin: &str, keys: &str) -> Harness {
    h.with_map_skin(skin);
    h.keys(keys);
    assert_eq!(h.screens(), ["title", "game_flow", "battle"]);
    h
}

/// [`battle_with`] the right-handed layout's keys.
fn battle_in(h: Harness, skin: &str) -> Harness {
    battle_with(h, skin, "Down f Left f f")
}

/// A new game's Quick Battle under the skin `skin`.
fn quick_battle(skin: &str) -> Harness {
    battle_in(Harness::with_layout(RIGHT_HANDED), skin)
}

/// The zoom in the player's settings.
fn saved_zoom(h: &Harness) -> MapZoom {
    h.game().ctx().settings().game.map_zoom
}

/// The zoom the map is painted at, if its skin can zoom.
fn skin_zoom(h: &Harness) -> Option<MapZoom> {
    h.game().ctx().game.map_skin.zoom()
}

/// How many tiles the map scene shows.
fn view(h: &Harness) -> (i32, i32) {
    h.map_scene().map_or((0, 0), |scene| scene.size)
}

/// Whether the cursor's tile is in the view.
fn cursor_in_view(h: &Harness) -> bool {
    let scene = h.map_scene();
    scene
        .zip(h.cursor_tile())
        .is_some_and(|(s, tile)| s.contains(tile))
}

/// The row of the toggles' state, above the help bar.
fn status(h: &Harness) -> String {
    let buf = h.game().buffer();
    let row: String = (0..100)
        .map(|x| buf.get(x, 30).map_or(' ', |c| c.glyph))
        .collect();
    row.trim().to_owned()
}

/// `key` pressed `times` times.
fn presses(key: &str, times: usize) -> String {
    vec![key; times].join(" ")
}

#[test]
fn the_default_key_switches_between_the_two_zooms() {
    let mut h = quick_battle("sprite");
    // A new player starts at 2×.
    assert_eq!(
        (saved_zoom(&h), skin_zoom(&h)),
        (MapZoom::X2, Some(MapZoom::X2))
    );
    assert_eq!(view(&h), VIEW_X2);
    assert!(status(&h).starts_with("q zoom: 2× · "), "{}", status(&h));
    let state = h.battle().map(|b| b.state().clone());
    let cursor = h.cursor_tile();
    h.clear_audio().keys("q");
    assert_eq!(
        (saved_zoom(&h), skin_zoom(&h)),
        (MapZoom::X1, Some(MapZoom::X1))
    );
    assert_eq!(view(&h), VIEW_X1);
    assert!(status(&h).starts_with("q zoom: 1× · "), "{}", status(&h));
    h.keys("q");
    assert_eq!(
        (saved_zoom(&h), skin_zoom(&h)),
        (MapZoom::X2, Some(MapZoom::X2))
    );
    assert_eq!(view(&h), VIEW_X2);
    // Only the view changed: the same battle, the cursor on the same
    // tile, and nothing heard.
    assert_eq!(h.battle().map(|b| b.state().clone()), state);
    assert_eq!(h.cursor_tile(), cursor);
    assert!(h.sounds().is_empty(), "{:?}", h.sounds());
}

/// At each corner of the map, the cursor's tile is in the view after
/// every switch (at 2× the 14-tile map is wider than the 11-tile view).
#[test]
fn the_cursors_tile_stays_in_view_at_every_corner_of_the_map() {
    let corners = [
        ("Left", "Up", Pos::new(0, 0)),
        ("Right", "Up", Pos::new(13, 0)),
        ("Right", "Down", Pos::new(13, 7)),
        ("Left", "Down", Pos::new(0, 7)),
    ];
    for (across, down, corner) in corners {
        let mut h = quick_battle("sprite");
        h.keys(&presses(across, 14)).keys(&presses(down, 8));
        assert_eq!(h.cursor_tile(), Some(corner));
        for zoom in [MapZoom::X1, MapZoom::X2, MapZoom::X1, MapZoom::X2] {
            h.keys("q");
            assert_eq!(skin_zoom(&h), Some(zoom), "{corner:?}");
            assert_eq!(h.cursor_tile(), Some(corner));
            assert!(cursor_in_view(&h), "{corner:?} at {zoom:?}");
            // And a frame later, once the camera is fitted to the view.
            h.wait(FRAME_DT);
            assert!(cursor_in_view(&h), "{corner:?} at {zoom:?}");
            // The view never shows past the map's sides at 2×.
            let origin = h.map_scene().map(|s| s.origin.x);
            if zoom == MapZoom::X2 {
                assert!(origin.is_some_and(|x| (0..=3).contains(&x)), "{origin:?}");
            }
        }
        // The cursor still scrolls the map as before.
        h.keys(&presses(
            if across == "Left" { "Right" } else { "Left" },
            13,
        ));
        assert!(cursor_in_view(&h), "{corner:?}");
    }
}

/// It works with a unit selected, while picking a target and in a menu's
/// absence only: a menu or a prompt keeps the zoom as it is.
#[test]
fn it_works_with_a_unit_selected_and_not_in_a_menu() {
    let mut h = quick_battle("sprite");
    // The lord selected: its range shown.
    h.keys("f");
    assert_eq!(h.screens(), ["title", "game_flow", "battle"]);
    h.keys("q");
    assert_eq!(skin_zoom(&h), Some(MapZoom::X1));
    assert_eq!(view(&h), VIEW_X1);
    // Its action menu (Confirm on its own tile), the map menu and the
    // end-turn prompt: the key does what a key bound to nothing does
    // (time passes in both, for whatever pulses).
    for menu in ["f f", "Right Right Right d", "Space"] {
        let mut h = quick_battle("sprite");
        let mut idle = quick_battle("sprite");
        h.keys(menu).keys("q");
        idle.keys(menu).keys("z");
        assert_eq!(skin_zoom(&h), Some(MapZoom::X2), "{menu}");
        assert_eq!(h.snapshot(), idle.snapshot(), "{menu}");
        // It was a menu: the map is back at 1× once the key works.
        assert_ne!(h.snapshot(), quick_battle("sprite").snapshot(), "{menu}");
    }
}

/// The glyph skin's tile is two font cells: the key changes nothing
/// there, on screen or in the settings, and the help doesn't list it.
/// The same under unit pictures on glyph terrain.
#[test]
fn skins_on_glyph_tiles_dont_zoom_and_dont_list_the_key() {
    for skin in ["glyph", "sprite_units"] {
        let mut h = quick_battle(skin);
        assert_eq!(skin_zoom(&h), None, "{skin}");
        assert!(!h.snapshot().contains("zoom"), "{skin}");
        // The same as a key bound to nothing (the cursor pulses, so time
        // has to pass in both).
        let mut idle = quick_battle(skin);
        idle.keys("z");
        h.keys("q");
        assert_eq!(h.snapshot(), idle.snapshot(), "{skin}");
        assert_eq!(saved_zoom(&h), MapZoom::X2, "{skin}");
    }
}

/// The zoom is kept for the next battle and the next launch.
#[test]
fn the_zoom_is_remembered_by_the_next_battle_and_the_next_launch() {
    let mut h = quick_battle("sprite");
    h.keys("q");
    assert_eq!(view(&h), VIEW_X1);
    // Restart the battle from the map menu: a new battle screen.
    h.keys("Right Right Right d Up Up f f");
    h.keys("Left f f");
    assert_eq!(h.screens(), ["title", "game_flow", "battle"]);
    assert_eq!(h.battle().map(|b| b.state().turn()), Some(1));
    assert_eq!((skin_zoom(&h), view(&h)), (Some(MapZoom::X1), VIEW_X1));
    // The next launch, with what this one saved.
    let h = battle_in(Harness::with_storage(h.into_storage()), "sprite");
    assert_eq!(saved_zoom(&h), MapZoom::X1);
    assert_eq!((skin_zoom(&h), view(&h)), (Some(MapZoom::X1), VIEW_X1));
    assert!(status(&h).starts_with("q zoom: 1× · "), "{}", status(&h));
}

/// The player's own key works, the default one stops, and with no key at
/// all the hint says so.
#[test]
fn the_key_can_be_rebound_and_left_unbound() {
    let mut h = quick_battle("sprite");
    let layout = RIGHT_HANDED;
    let mut keys = h.game().ctx().layout_bindings(layout);
    let z = Chord::parse("z").unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(keys.bind(Action::ZoomMap, 0, z), Ok(None));
    assert_eq!(
        h.ctx_mut().set_layout_bindings(layout, keys.clone()),
        Ok(())
    );
    h.keys("q");
    assert_eq!(skin_zoom(&h), Some(MapZoom::X2), "the old key does nothing");
    h.keys("z");
    assert_eq!(skin_zoom(&h), Some(MapZoom::X1));
    assert!(status(&h).starts_with("z zoom: 1× · "), "{}", status(&h));
    // Optional: it may be left with no key.
    assert!(!Action::ZoomMap.is_required());
    keys.clear(Action::ZoomMap, 0);
    assert_eq!(h.ctx_mut().set_layout_bindings(layout, keys), Ok(()));
    h.keys("z q").wait(FRAME_DT);
    assert_eq!(skin_zoom(&h), Some(MapZoom::X1));
    let status = status(&h);
    assert!(status.starts_with("! not mapped zoom: 1× · "), "{status}");
}

/// The left-handed layout's key and the controller's button.
#[test]
fn the_left_handed_key_and_the_right_trigger_zoom_too() {
    let left = Harness::with_layout(LEFT_HANDED);
    let mut h = battle_with(left, "sprite", "s j a j j");
    h.keys("p");
    assert_eq!(skin_zoom(&h), Some(MapZoom::X1));
    assert!(status(&h).starts_with("p zoom: 1× · "), "{}", status(&h));
    for (kind, name) in [
        (PadKind::Xbox, "RT"),
        (PadKind::PlayStation, "R2"),
        (PadKind::Nintendo, "ZR"),
    ] {
        let mut h = quick_battle("sprite");
        h.use_pad(kind).pad("RightTrigger");
        assert_eq!(skin_zoom(&h), Some(MapZoom::X1), "{kind:?}");
        let status = status(&h);
        assert!(
            status.starts_with(&format!("{name} zoom: 1× · ")),
            "{status}"
        );
        assert!(status.chars().count() < 100, "{status}");
    }
}

/// The Quick Battle on the test tileset at 2×, the lord selected: tiles,
/// units, HP bars, the cursor and the range all twice the size.
#[test]
fn quick_battle_at_twice_the_size() {
    let mut h = quick_battle("sprite");
    h.keys("f");
    assert_eq!(view(&h), VIEW_X2);
    assert_snapshot!(h.snapshot());
}

/// The same at 2× with the lord's action menu open: beside its tile.
#[test]
fn a_menu_opens_beside_its_unit_at_twice_the_size() {
    let mut h = quick_battle("sprite");
    h.keys("f f");
    assert_snapshot!(h.snapshot());
}
