//! A conversation's short moving opening, where the script asks for one
//! (ticket 0734, `docs/design/look-and-feel.md`, *Places behind
//! conversations*): Chapter 1's letter opens on the lead and Hollis
//! sparring in the yard, and its last scene on Dace pacing the study.
//! What happens is read from the screen's view and its map scene, not
//! from cells; one snapshot shows the look, in glyphs.

use insta::assert_snapshot;
use trpg_core::{CharacterId, LeadGender, Pos};
use trpg_ui::harness::Harness;
use trpg_ui::input::RIGHT_HANDED;
use trpg_ui::map_view::{Facing, MapScene, UnitView};
use trpg_ui::screens::DialogueScreen;
use trpg_ui::screens::dialogue::view::DialogueView;
use trpg_ui::{Ctx, Game};

fn p(x: i32, y: i32) -> Pos {
    Pos::new(x, y)
}

/// The game opened on `scene` with a lead of `gender`, as `--scene` does.
fn preview(scene: &str, gender: LeadGender) -> Harness {
    let ctx = Ctx::embedded()
        .unwrap_or_else(|e| panic!("{e}"))
        .with_layout(RIGHT_HANDED);
    let game = Game::start_on_scene(ctx, scene, gender).unwrap_or_else(|e| panic!("{e}"));
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

/// `ch01_intro` past its three captions on black, up to the opening.
fn letter(gender: LeadGender) -> Harness {
    let mut h = preview("ch01_intro", gender);
    for _ in 0..3 {
        assert!(!dialogue(&h).is_opening());
        h.wait(3.0);
        h.keys("f");
    }
    h
}

/// The letter opens on the lead and Hollis in the yard at full brightness,
/// with no talk on screen; the first line comes up after the opening's
/// 1.55 s and half a second of darkening, and they stay behind it.
#[test]
fn the_letter_opens_on_the_lead_and_hollis_sparring() {
    let mut h = letter(LeadGender::Female);
    assert!(dialogue(&h).is_opening());
    let shown = view(&h);
    assert_eq!(shown.opening.map(|o| o.darkened), Some(0.0));
    assert_eq!(
        standing(&h),
        [
            at("lead", p(11, 8), Facing::Right),
            at("retainer", p(13, 8), Facing::Left)
        ]
    );
    // The yard's own place, with them on it.
    let mut bare = place(&h);
    bare.units.clear();
    assert!(!place(&h).units.iter().any(|u| u.marks));
    // The lead has stepped in.
    h.wait(0.2);
    assert_eq!(standing(&h)[0], at("lead", p(12, 8), Facing::Right));
    h.wait(1.6);
    assert!(dialogue(&h).is_opening(), "still darkening");
    let darkening = view(&h).opening.map_or(0.0, |o| o.darkened);
    assert!(darkening > 0.0 && darkening < 1.0, "{darkening}");
    h.wait(0.3);
    assert!(!dialogue(&h).is_opening());
    let talk = view(&h);
    assert_eq!(talk.opening, None);
    assert!(talk.text.lines[0].starts_with("Dawn, in the yard"));
    assert!(talk.text.shown > 0);
    // They stand where it left them, darkened with the yard, behind the
    // talk.
    assert_eq!(
        standing(&h),
        [
            at("lead", p(11, 8), Facing::Right),
            at("retainer", p(13, 8), Facing::Left)
        ]
    );
    let mut behind = place(&h);
    behind.units.clear();
    assert_eq!(behind, bare);
}

/// Confirm during the opening shows the first line at once.
#[test]
fn confirm_skips_the_opening_to_the_first_line() {
    let mut h = letter(LeadGender::Male);
    h.wait(0.5);
    assert!(dialogue(&h).is_opening());
    h.keys("f");
    assert!(!dialogue(&h).is_opening());
    let talk = view(&h);
    assert!(talk.opening.is_none());
    assert!(talk.text.lines[0].starts_with("Dawn, in the yard"));
    // Where the opening would have left them.
    assert_eq!(standing(&h)[0], at("lead", p(11, 8), Facing::Right));
}

/// Cancel during the opening asks whether to skip the scene, as on any
/// line; answering no shows the first line.
#[test]
fn the_skip_question_still_works_during_the_opening() {
    let mut h = letter(LeadGender::Male);
    h.wait(0.3);
    h.keys("d");
    assert!(!dialogue(&h).is_opening());
    assert!(dialogue(&h).is_asking_skip());
    h.keys("d");
    assert!(!dialogue(&h).is_asking_skip());
    assert!(view(&h).text.lines[0].starts_with("Dawn, in the yard"));
    // Skipping from the captions passes over the opening: the reply choice
    // comes up at once.
    let mut h = preview("ch01_intro", LeadGender::Male);
    h.keys("d f");
    assert!(!dialogue(&h).is_opening());
    assert!(dialogue(&h).menu().is_some());
}

/// The last scene opens on Dace pacing the study, then the talk.
#[test]
fn the_last_scene_opens_on_dace_pacing_the_study() {
    let mut h = preview("ch01_tbc", LeadGender::Male);
    assert!(dialogue(&h).is_opening());
    assert_eq!(standing(&h), [at("rival", p(9, 8), Facing::Right)]);
    h.wait(0.45);
    let pacing = &standing(&h)[0];
    assert!((10..15).contains(&pacing.1.x), "{pacing:?}");
    h.wait(2.0);
    assert!(!dialogue(&h).is_opening());
    assert_eq!(standing(&h), [at("rival", p(12, 8), Facing::Left)]);
    assert!(view(&h).text.lines[0].starts_with("Night."));
}

/// No other Chapter 1 scene has an opening.
#[test]
fn other_scenes_open_straight_on_their_talk() {
    for scene in ["ch01_prebattle", "ch01_victory"] {
        let h = preview(scene, LeadGender::Male);
        assert!(!dialogue(&h).is_opening(), "{scene}");
        let units = view(&h).place.map(|p| p.units);
        assert_eq!(units, Some(Vec::new()), "{scene}");
    }
}

/// The female lead walks the yard as herself: her map picture is looked
/// up by her look (`lead_f`), like on the battle map.
#[test]
fn the_lead_in_the_opening_is_the_one_the_player_made() {
    let h = letter(LeadGender::Female);
    let lead = place(&h).units.into_iter().next();
    let lead = lead.unwrap_or_else(|| panic!("nobody in the yard"));
    assert_eq!(lead.character, Some(CharacterId("lead".into())));
    assert_eq!(h.game().ctx().lead.portrait_for("lead"), "lead_f");
}

/// The opening in the glyph look, a moment in: the yard bright, the two
/// sparring partners' labels on it, the caption over it and no talk.
#[test]
fn the_opening_in_glyphs() {
    let mut h = letter(LeadGender::Male);
    assert_eq!(h.game().ctx().map_skin.name(), "glyph");
    h.wait(0.2);
    assert_snapshot!(h.snapshot());
}
