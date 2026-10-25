//! Tests of the Zoom map key on the battle screen (ticket 0439): it flips
//! the map skin's zoom whenever the map is what the player is looking at
//! (browsing, a unit selected or walking, picking a target, the AI's
//! phase), the camera then shows the cursor's tile, and nothing of the
//! battle changes. Not in a menu, a fight's playback or the rewind screen.

use trpg_core::{Phase, Pos};

use super::layout::MAP_VIEW;
use super::testing::{skirmish_charged, through_ai_phases};
use super::*;
use crate::harness::{FRAME_DT, Harness};
use crate::map_view::skin_named;
use crate::screen::tests::ctx;
use crate::settings::MapZoom;

/// A context painting the map with the test tileset (which can zoom), at
/// the zoom a new player has: 2×.
fn sprite_ctx() -> Ctx {
    let mut c = ctx();
    c.set_map_skin(skin_named(&c.game.content, "sprite").unwrap());
    assert_eq!(c.game.map_skin.zoom(), Some(MapZoom::X2));
    c
}

/// Each action in a frame of its own.
fn press(s: &mut BattleScreen, c: &mut Ctx, actions: &[Action]) {
    for &action in actions {
        s.update(c, &FrameInput::new(vec![action], 0.0, vec![]));
    }
}

/// A frame with nothing pressed.
fn tick(s: &mut BattleScreen, c: &mut Ctx) {
    s.update(c, &FrameInput::new(vec![], 0.0, vec![]));
}

/// The skirmish (with a rewind charge) in the harness under the test
/// tileset.
fn skirmish_in_harness() -> Harness {
    let state = skirmish_charged(&ctx(), 20, 1);
    let mut h = Harness::with_screen(Box::new(BattleScreen::new(state)));
    h.with_map_skin("sprite").wait(FRAME_DT);
    h
}

fn battle(h: &Harness) -> &BattleScreen {
    h.battle().expect("a battle")
}

fn zoom(h: &Harness) -> Option<MapZoom> {
    h.game().ctx().game.map_skin.zoom()
}

/// Whether the cursor's tile is in the scene `s` shows.
fn cursor_in_view(s: &BattleScreen, c: &Ctx) -> bool {
    s.scene(c).contains(s.cursor().pos)
}

#[test]
fn the_key_flips_the_skins_zoom_and_saves_it() {
    let mut c = sprite_ctx();
    let mut s = BattleScreen::new(quick_battle(&c.game.content).unwrap());
    tick(&mut s, &mut c);
    let (state, cursor) = (s.state().clone(), s.cursor().pos);
    assert_eq!(s.scene(&c).size, (11, 10));
    assert_eq!(
        s.status(&c),
        "q zoom: 2× · w danger zone: OFF · Shift+Space auto-end: OFF"
    );
    press(&mut s, &mut c, &[Action::ZoomMap]);
    assert_eq!(c.game.map_skin.zoom(), Some(MapZoom::X1));
    assert_eq!(c.settings().game.map_zoom, MapZoom::X1);
    assert_eq!(s.scene(&c).size, (23, 20));
    assert!(
        s.status(&c).starts_with("q zoom: 1× · "),
        "{}",
        s.status(&c)
    );
    // Twice in one frame: back where it was.
    s.update(
        &mut c,
        &FrameInput::new(vec![Action::ZoomMap, Action::ZoomMap], 0.0, vec![]),
    );
    assert_eq!(c.game.map_skin.zoom(), Some(MapZoom::X1));
    press(&mut s, &mut c, &[Action::ZoomMap]);
    assert_eq!(c.game.map_skin.zoom(), Some(MapZoom::X2));
    assert_eq!(c.settings().game.map_zoom, MapZoom::X2);
    // The battle and the cursor are untouched; nothing was heard or said.
    assert_eq!((s.state(), s.cursor().pos), (&state, cursor));
    assert_eq!(s.toast(), None);
    assert_eq!(
        c.audio.take(),
        [] as [shuyi_ui::audio::AudioRequest<trpg_content::Variant>; 0]
    );
}

/// Under a skin that can't zoom the key does nothing: the saved zoom is
/// kept for the skin that can, and the toggles' row doesn't list it.
#[test]
fn under_the_glyph_skin_the_key_does_nothing_and_isnt_listed() {
    let mut c = ctx();
    let mut s = BattleScreen::new(quick_battle(&c.game.content).unwrap());
    press(&mut s, &mut c, &[Action::ZoomMap]);
    assert_eq!(c.settings().game.map_zoom, MapZoom::X2);
    assert_eq!(c.game.map_skin.zoom(), None);
    assert_eq!(
        s.status(&c),
        "w danger zone: OFF · Shift+Space auto-end: OFF"
    );
    assert_eq!(s.scene(&c).size, (35, 30));
}

/// At 2× the map is wider than the view: wherever the cursor is, its
/// tile is shown right after each switch and once the camera is fitted.
#[test]
fn the_camera_shows_the_cursors_tile_after_each_switch() {
    let mut c = sprite_ctx();
    let mut s = BattleScreen::new(quick_battle(&c.game.content).unwrap());
    tick(&mut s, &mut c);
    // To the map's right edge, the far side from where the camera starts.
    press(&mut s, &mut c, &[Action::CursorRight; 12]);
    assert_eq!(s.cursor().pos.x, 13);
    for _ in 0..4 {
        press(&mut s, &mut c, &[Action::ZoomMap]);
        assert!(cursor_in_view(&s, &c), "{:?}", s.scene(&c).origin);
        tick(&mut s, &mut c);
        assert!(cursor_in_view(&s, &c), "{:?}", s.scene(&c).origin);
        // The camera the screen keeps is the one the scene is drawn with.
        assert_eq!(s.camera().origin, s.scene(&c).origin);
        let view = c.game.map_skin.view_tiles(MAP_VIEW);
        assert_eq!(s.scene(&c).size, view);
    }
    // At 2× (11 tiles across a 14-tile map) the view ends at the map's edge.
    assert_eq!(c.game.map_skin.zoom(), Some(MapZoom::X2));
    assert_eq!(s.camera().origin, Pos::new(3, -1));
}

#[test]
fn it_works_while_a_unit_walks_and_while_picking_a_target() {
    let mut h = skirmish_in_harness();
    // The lord selected, one step right: walking.
    h.keys("f Right f");
    assert!(matches!(battle(&h).mode(), Mode::Moving { .. }));
    h.keys("q");
    assert_eq!(zoom(&h), Some(MapZoom::X1));
    // Its menu: nothing. Attack (the iron sword): picking the target.
    h.wait(0.5);
    assert!(matches!(battle(&h).mode(), Mode::ActionMenu { .. }));
    h.keys("q");
    assert_eq!(zoom(&h), Some(MapZoom::X1));
    h.keys("f");
    assert!(matches!(battle(&h).mode(), Mode::Targeting(_)));
    let state = battle(&h).state().clone();
    h.keys("q");
    assert_eq!(zoom(&h), Some(MapZoom::X2));
    assert!(matches!(battle(&h).mode(), Mode::Targeting(_)));
    assert_eq!(battle(&h).state(), &state);
    // The fight's playback: nothing.
    h.keys("Down f");
    assert!(matches!(battle(&h).mode(), Mode::Combat(_)));
    h.keys("q");
    assert_eq!(zoom(&h), Some(MapZoom::X2));
}

#[test]
fn the_rewind_screen_keeps_the_zoom() {
    let mut h = skirmish_in_harness();
    // A move (one step right, Wait), so there is something to rewind.
    h.keys("f Right f").wait(0.5);
    h.keys("Up f").wait(0.5);
    h.keys("r");
    let open = |h: &Harness| h.screen::<RewindScreen>().is_some();
    assert!(open(&h), "{:?}", battle(&h).mode());
    h.keys("q");
    assert_eq!(zoom(&h), Some(MapZoom::X2));
    assert_eq!(h.game().ctx().settings().game.map_zoom, MapZoom::X2);
    // Closed again, it works.
    h.keys("d q");
    assert!(!open(&h));
    assert_eq!(zoom(&h), Some(MapZoom::X1));
}

/// The zoom is the player's view, not a move: it works in the enemy's
/// phase, and the phase plays out to the same battle.
#[test]
fn it_works_in_the_enemy_phase_and_the_phase_ends_the_same() {
    let play = |zooms: bool| {
        let mut c = sprite_ctx();
        let mut s = BattleScreen::new(quick_battle(&c.game.content).unwrap());
        press(&mut s, &mut c, &[Action::EndTurn, Action::EndTurn]);
        assert_eq!(s.state().phase(), Phase::Enemy);
        // The banner, then the first enemy's action.
        press(&mut s, &mut c, &[Action::Confirm]);
        assert!(matches!(s.mode(), Mode::AiAction(_)), "{:?}", s.mode());
        if zooms {
            press(&mut s, &mut c, &[Action::ZoomMap]);
            assert_eq!(c.game.map_skin.zoom(), Some(MapZoom::X1));
            assert!(matches!(s.mode(), Mode::AiAction(_)), "{:?}", s.mode());
            s.update(&mut c, &FrameInput::new(vec![], 0.1, vec![]));
            press(&mut s, &mut c, &[Action::ZoomMap]);
            assert_eq!(c.game.map_skin.zoom(), Some(MapZoom::X2));
        }
        through_ai_phases(&mut s, &mut c, 0.05);
        s.state().clone()
    };
    let (plain, zoomed) = (play(false), play(true));
    assert_eq!(plain.phase(), Phase::Player);
    assert_eq!(plain, zoomed);
}
