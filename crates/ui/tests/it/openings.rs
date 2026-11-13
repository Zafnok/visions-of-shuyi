//! A conversation's short moving opening, where the script asks for one
//! (ticket 0734, `docs/design/look-and-feel.md`, *Places behind
//! conversations*): the opening's people move on its place at full
//! brightness, then the place darkens and the talk starts. What happens is
//! read from the screen's view and its map scene, not from cells; one
//! snapshot shows the look, in glyphs.
//!
//! On the test files (ADR-0118 §1 (b)): the test opening `test_opening`
//! (the lead steps in and back, the test knight walks round a corner, on
//! `test_buildings`) is given to the test chapter's intro, and to the test
//! scene with a reply choice, as their scripts would ask for it with
//! `@place` and `@opening` ([`with_test_opening`]). A line is read from the
//! content, never quoted.

use trpg_content::dialogue::{PlaceLine, Present, Stage, Step};
use trpg_core::{CharacterId, LeadGender, Pos};
use trpg_ui::Ctx;
use trpg_ui::ShuyiCtx;
use trpg_ui::harness::Harness;
use trpg_ui::input::RIGHT_HANDED;
use trpg_ui::map_view::{Facing, MapScene, UnitView};
use trpg_ui::screens::DialogueScreen;
use trpg_ui::screens::dialogue::view::DialogueView;

fn p(x: i32, y: i32) -> Pos {
    Pos::new(x, y)
}

/// The test chapter's intro: its caption, then the opening, then its talk.
const INTRO: &str = "test_intro";

/// The test scene with a reply choice: a line of narration on black, then
/// the opening, then its talk.
const WITH_A_CHOICE: &str = "test";

/// Puts `@place test_buildings 4,2` and `@opening test_opening` before
/// step `at` of scene `scene`.
fn with_test_opening(ctx: &mut Ctx, scene: &str, at: usize) {
    let scenes = &mut ctx.game.content.dialogue.scenes;
    let found = scenes.get_mut(scene);
    let found = found.unwrap_or_else(|| panic!("no scene {scene}"));
    let place = PlaceLine::Map {
        map: "test_buildings".into(),
        at: p(4, 2),
        bright: None,
    };
    let opening = Stage::Opening("test_opening".into());
    let steps = [
        Step::Directive(Stage::Setting(place)),
        Step::Directive(opening),
    ];
    found.steps.splice(at..at, steps);
}

/// The game opened on `scene` with a lead of `gender`, as `--scene` does,
/// the two test scenes given the test opening.
fn preview(scene: &str, gender: LeadGender) -> Harness {
    let mut ctx = Ctx::embedded()
        .unwrap_or_else(|e| panic!("{e}"))
        .with_layout(RIGHT_HANDED);
    // After the intro's caption; after the other's narration on black.
    with_test_opening(&mut ctx, INTRO, 1);
    with_test_opening(&mut ctx, WITH_A_CHOICE, 2);
    let game = trpg_ui::game::start_on_scene(ctx, scene, gender).unwrap_or_else(|e| panic!("{e}"));
    Harness::from_game(game)
}

fn dialogue(h: &Harness) -> &DialogueScreen {
    let screen = h.game().screen::<DialogueScreen>();
    screen.unwrap_or_else(|| panic!("no scene is playing: {:?}", h.screens()))
}

fn view(h: &Harness) -> DialogueView {
    dialogue(h).view(h.game().ctx())
}

/// The place behind the talk now.
fn place(h: &Harness) -> MapScene {
    let place = view(h).place;
    place.unwrap_or_else(|| panic!("no place: {:?}", h.screens()))
}

/// Who stands on the place now: each character, tile and facing.
fn standing(h: &Harness) -> Vec<(String, Pos, Facing)> {
    let who = |u: &UnitView| {
        u.character
            .as_ref()
            .map_or_else(String::new, |c| c.0.clone())
    };
    place(h)
        .units
        .iter()
        .map(|u| (who(u), u.pos, u.facing))
        .collect()
}

fn at(who: &str, pos: Pos, facing: Facing) -> (String, Pos, Facing) {
    (who.to_owned(), pos, facing)
}

/// Whether the text box on screen starts line `n` (from 0) of `scene`, as
/// the game says it with everyone there and the lead's name filled in: up
/// to the line's first other token, or the box's first row.
fn shows_line(h: &Harness, scene: &str, n: usize) -> bool {
    let ctx = h.game().ctx();
    let content = &ctx.game.content;
    let found = content.dialogue.get(scene);
    let scene = found.unwrap_or_else(|| panic!("no scene {scene}"));
    let resolved = scene.resolved(&Present::Everyone);
    let line = content.names.substitute(resolved.lines()[n].text);
    let line = line.replace("{lead}", &ctx.game.lead.name);
    let head: String = line.chars().take_while(|&c| c != '{').collect();
    let shown = view(h).text.lines;
    let first = shown.first().map_or("", |row| row.trim_end());
    !head.is_empty() && !first.is_empty() && (first.starts_with(&head) || head.starts_with(first))
}

/// Where the test opening's cast starts, and ends.
fn the_cast_at_the_start() -> Vec<(String, Pos, Facing)> {
    vec![
        at("lead", p(1, 0), Facing::Right),
        at("test_knight", p(3, 0), Facing::Left),
    ]
}

/// The opening plays on the place at full brightness, with no talk on
/// screen and nobody on a side (no outline, 0738): the lead steps in, and
/// back still facing the way it looked; the test knight walks round the
/// corner. The first line comes up after the opening's second and half a
/// second of darkening, and they stay behind it where it left them.
#[test]
fn an_opening_plays_on_its_place_then_the_talk_starts() {
    let mut h = preview(INTRO, LeadGender::Female);
    assert!(dialogue(&h).is_opening());
    let shown = view(&h);
    assert_eq!(shown.opening.map(|o| o.darkened), Some(0.0));
    assert_eq!(standing(&h), the_cast_at_the_start());
    // The place itself, with them on it.
    let mut bare = place(&h);
    bare.units.clear();
    assert!(!place(&h).units.iter().any(|u| u.sided));
    // The lead has stepped in, then back: facing right all the while.
    h.wait(0.2);
    assert_eq!(standing(&h)[0], at("lead", p(2, 0), Facing::Right));
    h.wait(0.3);
    assert_eq!(standing(&h)[0], at("lead", p(1, 0), Facing::Right));
    h.wait(0.3);
    assert!(
        view(&h).opening.is_some_and(|o| o.darkened == 0.0),
        "still moving"
    );
    h.wait(0.4);
    assert!(dialogue(&h).is_opening(), "still darkening");
    let darkening = view(&h).opening.map_or(0.0, |o| o.darkened);
    assert!(darkening > 0.0 && darkening < 1.0, "{darkening}");
    h.wait(0.4);
    assert!(!dialogue(&h).is_opening());
    let talk = view(&h);
    assert_eq!(talk.opening, None);
    assert!(shows_line(&h, INTRO, 0));
    assert!(talk.text.shown > 0);
    // They stand where it left them, darkened with the place, behind the
    // talk.
    let left = standing(&h);
    assert_eq!(left[0], at("lead", p(1, 0), Facing::Right));
    assert_eq!((left[1].0.as_str(), left[1].1), ("test_knight", p(4, 2)));
    let mut behind = place(&h);
    behind.units.clear();
    assert_eq!(behind, bare);
}

/// Confirm during the opening shows the first line at once.
#[test]
fn confirm_skips_the_opening_to_the_first_line() {
    let mut h = preview(INTRO, LeadGender::Male);
    h.wait(0.5);
    assert!(dialogue(&h).is_opening());
    h.keys("f");
    assert!(!dialogue(&h).is_opening());
    let talk = view(&h);
    assert!(talk.opening.is_none());
    assert!(shows_line(&h, INTRO, 0));
    // Where the opening would have left them.
    assert_eq!(standing(&h)[0], at("lead", p(1, 0), Facing::Right));
    assert_eq!(standing(&h)[1].1, p(4, 2));
}

/// Cancel during the opening asks whether to skip the scene, as on any
/// line; answering no shows the first line.
#[test]
fn the_skip_question_still_works_during_the_opening() {
    let mut h = preview(INTRO, LeadGender::Male);
    h.wait(0.3);
    h.keys("d");
    assert!(!dialogue(&h).is_opening());
    assert!(dialogue(&h).is_asking_skip());
    h.keys("d");
    assert!(!dialogue(&h).is_asking_skip());
    assert!(shows_line(&h, INTRO, 0));
    // Skipping from the line on black passes over the opening: the reply
    // choice comes up at once.
    let mut h = preview(WITH_A_CHOICE, LeadGender::Male);
    assert!(!dialogue(&h).is_opening());
    h.keys("d f");
    assert!(!dialogue(&h).is_opening());
    assert!(dialogue(&h).menu().is_some());
}

/// A line on black before the opening: the opening waits for it.
#[test]
fn an_opening_after_a_line_waits_for_it() {
    let mut h = preview(WITH_A_CHOICE, LeadGender::Male);
    assert!(!dialogue(&h).is_opening());
    assert!(shows_line(&h, WITH_A_CHOICE, 0));
    h.wait(3.0);
    h.keys("f");
    assert!(dialogue(&h).is_opening());
    assert_eq!(standing(&h), the_cast_at_the_start());
}

/// A scene whose script asks for no opening opens straight on its talk.
#[test]
fn other_scenes_open_straight_on_their_talk() {
    let h = preview("test_victory", LeadGender::Male);
    assert!(!dialogue(&h).is_opening());
    assert_eq!(view(&h).opening, None);
    let units = view(&h).place.map(|p| p.units);
    assert_eq!(units.unwrap_or_default(), Vec::new());
}

/// The female lead walks the place as herself: her map picture is looked
/// up by her look (`lead_f`), like on the battle map.
#[test]
fn the_lead_in_the_opening_is_the_one_the_player_made() {
    let h = preview(INTRO, LeadGender::Female);
    let lead = place(&h).units.into_iter().next();
    let lead = lead.unwrap_or_else(|| panic!("nobody in the place"));
    assert_eq!(lead.character, Some(CharacterId("lead".into())));
    assert_eq!(h.game().ctx().game.lead.portrait_for("lead"), "lead_f");
}

/// The opening in the glyph look, a moment in: the place bright, the two
/// people's labels on it, the caption over it and no talk.
#[test]
fn the_opening_in_glyphs() {
    let mut h = preview(INTRO, LeadGender::Male);
    assert_eq!(h.game().ctx().game.map_skin.name(), "glyph");
    h.wait(0.2);
    insta::assert_snapshot!(h.snapshot());
}
