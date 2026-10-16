//! Conversations drawn over their place, darkened (ticket 0732,
//! `docs/design/look-and-feel.md`, *Places behind conversations*):
//! Chapter 1's scenes before and after its battle over the battle's map,
//! and a scene during a battle over the battle map as it stands.

use insta::assert_snapshot;
use trpg_core::{LeadGender, Pos};
use trpg_ui::flow::{FlowScreen, Stage};
use trpg_ui::glyph_buffer::Rect;
use trpg_ui::harness::Harness;
use trpg_ui::input::Layout;
use trpg_ui::map_view::{MapScene, Place};
use trpg_ui::screens::DialogueScreen;
use trpg_ui::screens::dialogue::glyph::PLACE_BRIGHTNESS;
use trpg_ui::{Ctx, Game};

/// The dialogue screen playing: on the stack, or the game flow's scene.
fn dialogue(h: &Harness) -> &DialogueScreen {
    let flow = h.flow().map(FlowScreen::stage);
    let in_flow = match flow {
        Some(Stage::Scene(screen)) => Some(&**screen),
        _ => None,
    };
    let Some(screen) = h.game().screen::<DialogueScreen>().or(in_flow) else {
        panic!("no scene is playing: {:?}", h.screens());
    };
    screen
}

/// The place behind the talk now.
fn place(h: &Harness) -> Option<MapScene> {
    dialogue(h).view(h.game().ctx()).place
}

/// Chapter 1's field, with nobody on it, as the whole screen shows it
/// centred on `centre` with `ctx`'s map skin.
fn field(ctx: &Ctx, centre: Pos) -> MapScene {
    let def = &ctx.content.battles["ch01"];
    let view = ctx.map_skin.view_tiles(Rect::new(0, 0, 100, 32));
    Place::of_battle(&ctx.content, def).scene(centre, view)
}

/// The game opened on `scene`, as `--scene` does.
fn preview(scene: &str) -> Harness {
    let ctx = Ctx::embedded()
        .unwrap_or_else(|e| panic!("{e}"))
        .with_layout(Layout::RightHanded);
    let game = Game::start_on_scene(ctx, scene, LeadGender::Male).unwrap_or_else(|e| panic!("{e}"));
    Harness::from_game(game)
}

/// From New Game, the letter (no place yet), then the Red Company on the
/// green: the battle's field at Harrowby, by the chapel, with nobody on it
/// yet. In the glyph look.
#[test]
fn the_scene_before_the_battle_is_drawn_over_its_field_at_harrowby() {
    let mut h = Harness::with_layout(Layout::RightHanded).with_shipped_story();
    h.keys("f f f Up f");
    assert_eq!(dialogue(&h).player().scene_id(), "ch01_intro");
    assert_eq!(place(&h), None);
    // Skip the letter (skipping stops at each reply choice: the first
    // reply is picked).
    for _ in 0..8 {
        if dialogue(&h).player().scene_id() != "ch01_intro" {
            break;
        }
        h.keys("d f");
    }
    assert_eq!(dialogue(&h).player().scene_id(), "ch01_prebattle");
    let ctx = h.game().ctx();
    assert_eq!(ctx.map_skin.name(), "glyph");
    let shown = place(&h);
    assert_eq!(shown, Some(field(ctx, Pos::new(3, 12))));
    assert!(shown.is_some_and(|s| s.units.is_empty()));
    // The first line typed out.
    h.wait(5.0);
    assert_snapshot!(h.snapshot());
}

/// After the battle, at the toll-fort.
#[test]
fn the_scene_after_the_battle_is_drawn_over_its_field_at_the_toll_fort() {
    let h = preview("ch01_victory");
    let ctx = h.game().ctx();
    assert_eq!(place(&h), Some(field(ctx, Pos::new(20, 3))));
    // The scene of the letter has none yet (its place comes with 0733).
    assert_eq!(place(&preview("ch01_intro")), None);
}

/// A scene played over the battle map darkens it: every map cell around
/// the talk is the battle's own, at [`PLACE_BRIGHTNESS`].
#[test]
fn the_battle_map_behind_a_battles_scene_is_darkened() {
    let mut h = Harness::with_layout(Layout::RightHanded);
    h.keys("Down f Left f f");
    assert_eq!(h.screens(), ["title", "battle"]);
    let before = h.game().buffer().clone();
    h.keys("F2 Down Down Down f");
    assert_eq!(h.screens(), ["title", "battle", "dialogue"]);
    assert!(dialogue(&h).view(h.game().ctx()).overlay);
    let during = h.game().buffer();
    let mut terrain = 0;
    // Between the portraits' frames, above the text box.
    for (x, y) in (2..19).flat_map(|y| (36..64).map(move |x| (x, y))) {
        let (Some(was), Some(now)) = (before.get(x, y), during.get(x, y)) else {
            continue;
        };
        if !matches!(was.glyph, '^' | '♣' | '♠' | '▲') || now.glyph != was.glyph {
            continue;
        }
        terrain += 1;
        let darkened = (
            was.fg.scale(PLACE_BRIGHTNESS),
            was.bg.scale(PLACE_BRIGHTNESS),
        );
        assert_eq!((now.fg, now.bg), darkened, "cell {x},{y}");
    }
    assert!(terrain > 10, "only {terrain} terrain cells");
}
