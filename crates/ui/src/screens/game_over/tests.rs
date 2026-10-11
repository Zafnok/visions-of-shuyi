use insta::assert_snapshot;

use super::*;
use crate::harness::Harness;
use crate::screen::tests::ctx;

fn update(s: &mut dyn Screen, actions: &[Action]) -> String {
    let input = FrameInput::new(actions.to_vec(), 0.0, vec![]);
    format!("{:?}", s.update(&mut ctx(), &input))
}

#[test]
fn game_over_offers_retry_and_title() {
    let mut s = GameOverScreen::new(&ctx());
    assert_eq!(s.name(), "game_over");
    // Nothing to back out of.
    assert_eq!(update(&mut s, &[Action::Cancel]), "None");
    assert_eq!(s.result(), None);
    assert_eq!(update(&mut s, &[Action::Confirm]), "Pop");
    assert_eq!(s.result(), Some(GameOverChoice::Retry));
    let mut s = GameOverScreen::new(&ctx());
    assert_eq!(
        update(&mut s, &[Action::CursorDown, Action::Confirm]),
        "Pop"
    );
    assert_eq!(s.result(), Some(GameOverChoice::Title));
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

/// The view on the first frame: the heading, both ways on with `Retry
/// Battle` focused, and the help.
#[test]
fn the_game_over_view_offers_retry_focused_and_title() {
    let c = ctx();
    let view = GameOverScreen::new(&c).view(&c);
    assert_eq!(view.heading, c.text(GAME_OVER));
    let labels: Vec<_> = view.menu.items.iter().map(|i| i.label.as_str()).collect();
    assert_eq!(
        labels,
        [c.text("game_over.retry"), c.text("game_over.title")]
    );
    assert_eq!(
        view.menu.focused().unwrap().label,
        c.text("game_over.retry")
    );
    assert_eq!(view.help, GameOverScreen::help(&c));
}

#[test]
fn the_game_over_focus_moves_in_the_view() {
    let mut c = ctx();
    let mut s = GameOverScreen::new(&c);
    let input = FrameInput::new(vec![Action::CursorDown], 0.0, vec![]);
    s.update(&mut c, &input);
    let view = s.view(&c);
    assert_eq!(
        view.menu.focused().unwrap().label,
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

#[test]
fn game_over_snapshot() {
    let h = Harness::with_screen(Box::new(GameOverScreen::new(&ctx())));
    assert_snapshot!(h.snapshot());
}

#[test]
fn to_be_continued_snapshot() {
    let h = Harness::with_screen(Box::new(ToBeContinuedScreen));
    assert_snapshot!(h.snapshot());
}
