//! A conversation's short moving opening, where the script asks for one
//! (ticket 0734, `docs/design/look-and-feel.md`, *Places behind
//! conversations*): Chapter 1's intro opens on two people in its place,
//! and its last scene on one. What happens is read from the screen's view
//! and its map scene, not from cells; one snapshot shows the look, in
//! glyphs.
//!
//! These play Chapter 1's own openings (ADR-0118): they skip without the
//! content, the snapshot is in the private repository's
//! `content/tests/snapshots/`, and a line is read from the content, never
//! quoted. Moving them to a test opening is ticket 0162's.

use trpg_content::bundle::{content_tests, skip_without_the_game};
use trpg_content::dialogue::Present;
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

/// The game opened on `scene` with a lead of `gender`, as `--scene` does.
fn preview(scene: &str, gender: LeadGender) -> Harness {
    let ctx = Ctx::embedded()
        .unwrap_or_else(|e| panic!("{e}"))
        .with_layout(RIGHT_HANDED);
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
/// the game says it with everyone there: up to the line's first token
/// the names don't fill in (the lead's), or the box's first row.
fn shows_line(h: &Harness, scene: &str, n: usize) -> bool {
    let content = &h.game().ctx().game.content;
    let found = content.dialogue.get(scene);
    let scene = found.unwrap_or_else(|| panic!("no scene {scene}"));
    let resolved = scene.resolved(&Present::Everyone);
    let line = content.names.substitute(resolved.lines()[n].text);
    let head: String = line.chars().take_while(|&c| c != '{').collect();
    let shown = view(h).text.lines;
    let first = shown.first().map_or("", |row| row.trim_end());
    !head.is_empty() && !first.is_empty() && (first.starts_with(&head) || head.starts_with(first))
}

/// The first line after `ch01_intro`'s three captions on black.
const INTRO_TALK: usize = 3;

/// `ch01_intro` past its three captions on black, up to the opening.
fn intro_opening(gender: LeadGender) -> Harness {
    let mut h = preview("ch01_intro", gender);
    for _ in 0..3 {
        assert!(!dialogue(&h).is_opening());
        h.wait(3.0);
        h.keys("f");
    }
    h
}

/// The intro opens on the lead and a companion in its place at full
/// brightness, with no talk on screen, on no side (no outline, 0738); the
/// first line comes up after the opening's 2.05 s and half a second of
/// darkening, and they stay behind it.
#[test]
fn chapter_1s_intro_opens_on_two_people_in_its_place() {
    if skip_without_the_game() {
        return;
    }
    let mut h = intro_opening(LeadGender::Female);
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
    // The place itself, with them on it.
    let mut bare = place(&h);
    bare.units.clear();
    assert!(!place(&h).units.iter().any(|u| u.sided));
    // The lead has stepped in.
    h.wait(0.2);
    assert_eq!(standing(&h)[0], at("lead", p(12, 8), Facing::Right));
    h.wait(1.6);
    assert!(
        view(&h).opening.is_some_and(|o| o.darkened == 0.0),
        "still moving"
    );
    h.wait(0.5);
    assert!(dialogue(&h).is_opening(), "still darkening");
    let darkening = view(&h).opening.map_or(0.0, |o| o.darkened);
    assert!(darkening > 0.0 && darkening < 1.0, "{darkening}");
    h.wait(0.3);
    assert!(!dialogue(&h).is_opening());
    let talk = view(&h);
    assert_eq!(talk.opening, None);
    assert!(shows_line(&h, "ch01_intro", INTRO_TALK));
    assert!(talk.text.shown > 0);
    // They stand where it left them, darkened with the place, behind the
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
    if skip_without_the_game() {
        return;
    }
    let mut h = intro_opening(LeadGender::Male);
    h.wait(0.5);
    assert!(dialogue(&h).is_opening());
    h.keys("f");
    assert!(!dialogue(&h).is_opening());
    let talk = view(&h);
    assert!(talk.opening.is_none());
    assert!(shows_line(&h, "ch01_intro", INTRO_TALK));
    // Where the opening would have left them.
    assert_eq!(standing(&h)[0], at("lead", p(11, 8), Facing::Right));
}

/// Cancel during the opening asks whether to skip the scene, as on any
/// line; answering no shows the first line.
#[test]
fn the_skip_question_still_works_during_the_opening() {
    if skip_without_the_game() {
        return;
    }
    let mut h = intro_opening(LeadGender::Male);
    h.wait(0.3);
    h.keys("d");
    assert!(!dialogue(&h).is_opening());
    assert!(dialogue(&h).is_asking_skip());
    h.keys("d");
    assert!(!dialogue(&h).is_asking_skip());
    assert!(shows_line(&h, "ch01_intro", INTRO_TALK));
    // Skipping from the captions passes over the opening: the reply choice
    // comes up at once.
    let mut h = preview("ch01_intro", LeadGender::Male);
    h.keys("d f");
    assert!(!dialogue(&h).is_opening());
    assert!(dialogue(&h).menu().is_some());
}

/// The last scene opens on one person pacing its place, then the talk.
#[test]
fn chapter_1s_last_scene_opens_on_someone_pacing() {
    if skip_without_the_game() {
        return;
    }
    let mut h = preview("ch01_tbc", LeadGender::Male);
    assert!(dialogue(&h).is_opening());
    assert_eq!(standing(&h), [at("rival", p(9, 8), Facing::Right)]);
    h.wait(0.45);
    let pacing = &standing(&h)[0];
    assert!((10..15).contains(&pacing.1.x), "{pacing:?}");
    // Back across the rug (Nick, 0738: "Longer").
    h.wait(1.0);
    let back = &standing(&h)[0];
    assert!(
        back.2 == Facing::Left && (9..15).contains(&back.1.x),
        "{back:?}"
    );
    h.wait(1.6);
    assert!(!dialogue(&h).is_opening());
    assert_eq!(standing(&h), [at("rival", p(12, 8), Facing::Right)]);
    assert!(shows_line(&h, "ch01_tbc", 0));
}

/// No other Chapter 1 scene has an opening.
#[test]
fn other_scenes_open_straight_on_their_talk() {
    if skip_without_the_game() {
        return;
    }
    for scene in ["ch01_prebattle", "ch01_victory"] {
        let h = preview(scene, LeadGender::Male);
        assert!(!dialogue(&h).is_opening(), "{scene}");
        let units = view(&h).place.map(|p| p.units);
        assert_eq!(units, Some(Vec::new()), "{scene}");
    }
}

/// The female lead walks the place as herself: her map picture is looked
/// up by her look (`lead_f`), like on the battle map.
#[test]
fn the_lead_in_the_opening_is_the_one_the_player_made() {
    if skip_without_the_game() {
        return;
    }
    let h = intro_opening(LeadGender::Female);
    let lead = place(&h).units.into_iter().next();
    let lead = lead.unwrap_or_else(|| panic!("nobody in the place"));
    assert_eq!(lead.character, Some(CharacterId("lead".into())));
    assert_eq!(h.game().ctx().game.lead.portrait_for("lead"), "lead_f");
}

/// The opening in the glyph look, a moment in: the place bright, the two
/// people's labels on it, the caption over it and no talk.
#[test]
fn the_opening_in_glyphs() {
    let Some(dir) = content_tests() else {
        return;
    };
    let mut h = intro_opening(LeadGender::Male);
    assert_eq!(h.game().ctx().game.map_skin.name(), "glyph");
    h.wait(0.2);
    assert_private_snapshot!(dir, h.snapshot());
}
