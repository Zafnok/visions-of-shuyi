use insta::assert_snapshot;

use super::*;
use crate::harness::Harness;
use crate::screen::tests::ctx;
use crate::screens::popped_with;

fn update(s: &mut dyn Screen<Shuyi>, actions: &[Action]) -> String {
    let input = FrameInput::new(actions.to_vec(), 0.0, vec![]);
    format!("{:?}", s.update(&mut ctx(), &input))
}

/// What one update with `actions` pops with: the choice, if it does.
fn chosen(s: &mut GameOverScreen, actions: &[Action]) -> Option<GameOverChoice> {
    let input = FrameInput::new(actions.to_vec(), 0.0, vec![]);
    popped_with(s.update(&mut ctx(), &input))
}

/// A frame of `dt` seconds with nothing pressed.
fn wait(s: &mut GameOverScreen, dt: f32) -> String {
    let input = FrameInput::new(vec![], dt, vec![]);
    format!("{:?}", s.update(&mut ctx(), &input))
}

/// Game Over with its opening animation over.
fn settled(can_rewind: bool) -> GameOverScreen {
    let mut s = GameOverScreen::new(&ctx(), can_rewind);
    s.skip();
    s
}

/// The tests (and the game flow) reach the screen through `as_any`.
#[test]
fn game_over_can_be_found_by_its_type() {
    let s = GameOverScreen::new(&ctx(), false);
    let any = Screen::as_any(&s).expect("opts into as_any");
    assert!(any.downcast_ref::<GameOverScreen>().is_some());
}

/// The tests (and the game flow) reach the screen through `as_any`.
#[test]
fn to_be_continued_can_be_found_by_its_type() {
    let s = ToBeContinuedScreen;
    let any = Screen::as_any(&s).expect("opts into as_any");
    assert!(any.downcast_ref::<ToBeContinuedScreen>().is_some());
}

/// Game Over is an overlay over the lost battle while the battle still
/// shows through its opening animation, and covers it once it is black.
#[test]
fn game_over_shows_the_battle_only_while_it_fades() {
    let s = GameOverScreen::new(&ctx(), false);
    assert!(s.shows_battle());
    assert!(s.is_overlay());
    let s = settled(false);
    assert!(!s.shows_battle());
    assert!(!s.is_overlay());
}

#[test]
fn game_over_offers_retry_and_title() {
    let mut s = settled(false);
    assert_eq!(s.name(), "game_over");
    // Nothing to back out of.
    assert_eq!(update(&mut s, &[Action::Cancel]), "None");
    let retry = chosen(&mut s, &[Action::Confirm]);
    assert_eq!(retry, Some(GameOverChoice::Retry));
    let mut s = settled(false);
    let title = chosen(&mut s, &[Action::CursorDown, Action::Confirm]);
    assert_eq!(title, Some(GameOverChoice::Title));
}

/// The animation runs on the frames' time alone: two seconds from the
/// first frame to the menu.
#[test]
fn the_animation_takes_two_seconds_of_frames() {
    let c = ctx();
    let mut s = GameOverScreen::new(&c, false);
    assert_eq!((s.progress(), s.finished()), (0, false));
    assert!(s.shows_battle());
    assert_eq!(wait(&mut s, 0.5), "None");
    assert_eq!((s.progress(), s.finished()), (64, false));
    assert!(s.shows_battle());
    // The lost battle is gone 35% of the way in.
    wait(&mut s, 0.25);
    assert_eq!(s.progress(), 96);
    assert!(!s.shows_battle());
    wait(&mut s, 1.0);
    assert_eq!((s.progress(), s.finished()), (223, false));
    let view = s.view(&c);
    assert_eq!((view.progress, view.menu, view.help), (223, None, None));
    assert_eq!(view.heading, c.text(GAME_OVER));
    // The last quarter second, and the time past it changes nothing.
    wait(&mut s, 0.25);
    assert_eq!((s.progress(), s.finished()), (FULL, true));
    wait(&mut s, 5.0);
    assert_eq!((s.progress(), s.finished()), (FULL, true));
    assert!(!s.shows_battle());
    let view = s.view(&c);
    assert_eq!(view.progress, FULL);
    assert!(view.menu.is_some());
    assert_eq!(view.help, Some(GameOverScreen::help(&c)));
    // A frame longer than what is left ends it, no further.
    let mut s = GameOverScreen::new(&c, false);
    wait(&mut s, 60.0);
    assert_eq!((s.progress(), s.finished()), (FULL, true));
}

/// A press during the animation ends it and is used up: it picks nothing
/// and moves nothing, whatever it was.
#[test]
fn a_press_ends_the_animation_and_does_nothing_else() {
    let mut c = ctx();
    for actions in [
        vec![Action::Confirm],
        vec![Action::CursorDown, Action::Confirm],
        vec![Action::Cancel],
    ] {
        let mut s = GameOverScreen::new(&c, true);
        wait(&mut s, 0.5);
        assert_eq!(update(&mut s, &actions), "None");
        assert!(s.finished());
        let menu = s.view(&c).menu.unwrap();
        assert_eq!(
            crate::widgets::menu::text_of(&menu.focused().unwrap().label),
            c.text("game_over.rewind")
        );
        // The next one is the menu's.
        let rewind = chosen(&mut s, &[Action::Confirm]);
        assert_eq!(rewind, Some(GameOverChoice::Rewind));
    }
    // A key bound to nothing, and a controller button, end it too.
    let chord = crate::input::Chord::parse("F9").unwrap();
    let key = FrameInput::new(vec![], 0.0, vec![]).with_typing(vec![chord], vec![]);
    let pad = FrameInput::new(vec![], 0.0, vec![]).with_pad(true);
    for input in [key, pad] {
        let mut s = GameOverScreen::new(&c, false);
        assert_eq!(format!("{:?}", s.update(&mut c, &input)), "None");
        assert!(s.finished());
    }
    // The press's own frame adds no time of its own: it is the end.
    let mut s = GameOverScreen::new(&c, false);
    s.update(&mut c, &FrameInput::new(vec![Action::Confirm], 0.1, vec![]));
    assert_eq!(s.progress(), FULL);
    // No sound for the skip.
    assert_eq!(
        c.audio.take(),
        [] as [shuyi_ui::audio::AudioRequest<trpg_content::Variant>; 0]
    );
}

/// The flow's way to the finished screen (back from the rewind screen).
#[test]
fn skip_jumps_to_the_end() {
    let c = ctx();
    let mut s = GameOverScreen::new(&c, false);
    assert!(s.view(&c).menu.is_none());
    s.skip();
    assert!(s.finished());
    assert_eq!(s.progress(), FULL);
    assert!(s.view(&c).menu.is_some());
}

#[test]
fn to_be_continued_waits_for_confirm() {
    let mut s = ToBeContinuedScreen;
    assert_eq!(s.name(), "to_be_continued");
    assert_eq!(
        update(&mut s, &[Action::Cancel, Action::CursorDown]),
        "None"
    );
    assert_eq!(update(&mut s, &[Action::Confirm]), "Pop");
}

/// The finished view: the heading, both ways on with `Retry Battle`
/// focused, and the help.
#[test]
fn the_game_over_view_offers_retry_focused_and_title() {
    let c = ctx();
    let view = settled(false).view(&c);
    assert_eq!(view.heading, c.text(GAME_OVER));
    let menu = view.menu.unwrap();
    let labels: Vec<_> = menu
        .items
        .iter()
        .map(|i| crate::widgets::menu::text_of(&i.label))
        .collect();
    assert_eq!(
        labels,
        [c.text("game_over.retry"), c.text("game_over.title")]
    );
    assert_eq!(
        crate::widgets::menu::text_of(&menu.focused().unwrap().label),
        c.text("game_over.retry")
    );
    assert_eq!(view.help, Some(GameOverScreen::help(&c)));
}

#[test]
fn the_game_over_focus_moves_in_the_view() {
    let mut c = ctx();
    let mut s = settled(false);
    let input = FrameInput::new(vec![Action::CursorDown], 0.0, vec![]);
    s.update(&mut c, &input);
    let menu = s.view(&c).menu.unwrap();
    assert_eq!(
        crate::widgets::menu::text_of(&menu.focused().unwrap().label),
        c.text("game_over.title")
    );
}

#[test]
fn the_to_be_continued_view_is_the_closing_words_and_the_help() {
    let c = ctx();
    let view = ToBeContinuedScreen.view(&c);
    assert_eq!(view.heading, c.text(TO_BE_CONTINUED));
    assert_eq!(view.help, ToBeContinuedScreen::help(&c));
}

/// The finished screen.
#[test]
fn game_over_snapshot() {
    let h = Harness::with_screen(Box::new(settled(false)));
    assert_snapshot!(h.snapshot());
}

#[test]
fn to_be_continued_snapshot() {
    let h = Harness::with_screen(Box::new(ToBeContinuedScreen));
    assert_snapshot!(h.snapshot());
}

#[test]
fn game_over_offers_rewind_first_while_charges_are_left() {
    let c = ctx();
    let s = settled(true);
    let menu = s.view(&c).menu.unwrap();
    let labels: Vec<_> = menu
        .items
        .iter()
        .map(|i| crate::widgets::menu::text_of(&i.label))
        .collect();
    assert_eq!(
        labels,
        [
            c.text("game_over.rewind"),
            c.text("game_over.retry"),
            c.text("game_over.title")
        ]
    );
    assert_eq!(
        crate::widgets::menu::text_of(&menu.focused().unwrap().label),
        c.text("game_over.rewind")
    );
    let mut s = settled(true);
    let rewind = chosen(&mut s, &[Action::Confirm]);
    assert_eq!(rewind, Some(GameOverChoice::Rewind));
    let mut s = settled(true);
    let retry = chosen(&mut s, &[Action::CursorDown, Action::Confirm]);
    assert_eq!(retry, Some(GameOverChoice::Retry));
    let mut s = settled(true);
    let down_twice = [Action::CursorDown, Action::CursorDown, Action::Confirm];
    assert_eq!(chosen(&mut s, &down_twice), Some(GameOverChoice::Title));
}

#[test]
fn game_over_with_rewind_snapshot() {
    let h = Harness::with_screen(Box::new(settled(true)));
    assert_snapshot!(h.snapshot());
}
