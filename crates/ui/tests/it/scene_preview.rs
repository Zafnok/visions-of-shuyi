//! Scripted tests of the scene preview (ticket 0723): the game opened
//! straight on one scene, as `app` does for `--scene <id>`.

use trpg_core::LeadGender;
use trpg_ui::harness::Harness;
use trpg_ui::input::Layout;
use trpg_ui::screens::DialogueScreen;
use trpg_ui::{Ctx, Game};

/// The embedded content, with a layout saved or (a first launch) none.
fn ctx(layout: Option<Layout>) -> Ctx {
    let ctx = Ctx::embedded().unwrap_or_else(|e| panic!("{e}"));
    match layout {
        Some(layout) => ctx.with_layout(layout),
        None => ctx,
    }
}

/// The game opened on `scene` with the right-handed layout.
fn preview(scene: &str, gender: LeadGender) -> Harness {
    let ctx = ctx(Some(Layout::RightHanded));
    let game = Game::start_on_scene(ctx, scene, gender).unwrap_or_else(|e| panic!("{e}"));
    Harness::from_game(game)
}

/// The dialogue screen on the stack.
fn dialogue(h: &Harness) -> &DialogueScreen {
    let Some(screen) = h.game().screen::<DialogueScreen>() else {
        panic!("no scene is playing: {:?}", h.screens());
    };
    screen
}

/// Presses `f` until the scene ends; returns how many presses it took.
fn play_to_the_end(h: &mut Harness) -> usize {
    for presses in 1..=200 {
        h.keys("f");
        if h.top_screen() != "dialogue" {
            return presses;
        }
    }
    panic!("the scene never ended");
}

#[test]
fn the_game_opens_on_the_scene_with_the_default_lead() {
    let h = preview("ch01_intro", LeadGender::Male);
    assert_eq!(h.screens(), ["scene_preview", "dialogue"]);
    let player = dialogue(&h).player();
    assert_eq!(player.scene_id(), "ch01_intro");
    assert_eq!(player.lead().name, "Ellery");
    assert_eq!(player.lead().gender, LeadGender::Male);
    assert!(!h.quit_requested());
}

#[test]
fn the_other_lead_can_be_asked_for() {
    let h = preview("ch01_intro", LeadGender::Female);
    let player = dialogue(&h).player();
    assert_eq!(player.lead().name, "Ellery");
    assert_eq!(player.lead().gender, LeadGender::Female);
    assert_eq!(h.game().ctx().lead.gender, LeadGender::Female);
}

#[test]
fn when_the_scene_ends_confirm_plays_it_again_and_cancel_quits() {
    let mut h = preview("ch01_intro", LeadGender::Male);
    let first_page = dialogue(&h).page_lines().to_vec();
    let presses = play_to_the_end(&mut h);
    assert_eq!(h.screens(), ["scene_preview"]);
    assert!(h.snapshot().contains("End of scene ch01_intro"));
    assert!(!h.quit_requested());
    // Other keys do nothing; Confirm starts the scene over.
    h.keys("Down");
    assert_eq!(h.screens(), ["scene_preview"]);
    h.keys("f");
    assert_eq!(h.screens(), ["scene_preview", "dialogue"]);
    assert_eq!(dialogue(&h).page_lines(), first_page);
    assert_eq!(play_to_the_end(&mut h), presses);
    h.keys("d");
    assert!(h.quit_requested());
}

#[test]
fn a_first_launch_picks_a_layout_before_the_scene() {
    let game = Game::start_on_scene(ctx(None), "ch01_intro", LeadGender::Male)
        .unwrap_or_else(|e| panic!("{e}"));
    let mut h = Harness::from_game(game);
    assert_eq!(h.screens(), ["scene_preview", "dialogue", "layout_picker"]);
    h.keys("f");
    assert_eq!(h.screens(), ["scene_preview", "dialogue"]);
    assert!(h.game().ctx().layout().is_some());
}

#[test]
fn an_unknown_scene_is_an_error_that_lists_the_scenes() {
    let Err(error) = Game::start_on_scene(ctx(None), "ch01_intr", LeadGender::Male) else {
        panic!("there is no such scene");
    };
    assert!(
        error.starts_with("no scene \"ch01_intr\" in assets/dialogue/; the scenes are: "),
        "{error}"
    );
    assert!(error.contains(", ch01_intro, "), "{error}");
}
