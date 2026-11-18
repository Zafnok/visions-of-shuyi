use insta::assert_snapshot;

use super::*;
use crate::harness::Harness;
use crate::screen::tests::ctx;

fn update(s: &mut ChapterCardScreen, actions: &[Action], dt: f32) -> Transition {
    let input = FrameInput::new(actions.to_vec(), dt, vec![]);
    s.update(&mut ctx(), &input)
}

fn card(title: &str) -> ChapterCardScreen {
    ChapterCardScreen {
        view: ChapterCardView::of_title(title),
        shown_s: 0.0,
    }
}

#[test]
fn a_title_splits_into_its_number_and_its_name() {
    assert_eq!(
        ChapterCardView::of_title("Chapter 1: The Letter"),
        ChapterCardView {
            line: "Chapter 1".to_owned(),
            heading: "The Letter".to_owned(),
        }
    );
    // At the first split only.
    assert_eq!(ChapterCardView::of_title("A: B: C").heading, "B: C");
    // A title without one is the name alone.
    let view = ChapterCardView::of_title("Quick Battle");
    assert_eq!(
        (view.line.as_str(), view.heading.as_str()),
        ("", "Quick Battle")
    );
}

#[test]
fn it_shows_the_chapters_title_in_the_players_language() {
    let c = ctx();
    let chapter = c.game.content.chapters.get("test").unwrap();
    let s = ChapterCardScreen::new(&c, chapter);
    assert_eq!(s.view(), &ChapterCardView::of_title(&chapter.title));
    assert_eq!(s.name(), "chapter_card");
    let any = Screen::as_any(&s).expect("opts into as_any");
    assert!(any.downcast_ref::<ChapterCardScreen>().is_some());
}

#[test]
fn it_closes_by_itself_after_its_time() {
    let mut s = card("Chapter 1: The Letter");
    assert!(matches!(
        update(&mut s, &[], CARD_S * 0.5),
        Transition::None
    ));
    assert!(matches!(
        update(&mut s, &[], CARD_S * 0.49),
        Transition::None
    ));
    assert!(matches!(
        update(&mut s, &[], CARD_S * 0.01),
        Transition::Pop
    ));
}

#[test]
fn confirm_or_cancel_closes_it_and_nothing_else_does() {
    for action in [Action::Confirm, Action::Cancel] {
        let mut s = card("Chapter 1: The Letter");
        assert!(matches!(update(&mut s, &[action], 0.0), Transition::Pop));
    }
    let mut s = card("Chapter 1: The Letter");
    let other = [Action::CursorDown, Action::Options];
    assert!(matches!(update(&mut s, &other, 0.0), Transition::None));
}

#[test]
fn chapter_card_declares_its_actions() {
    let s = card("Chapter 1: The Letter");
    crate::screens::actions::assert_declares(&s, crate::screens::actions::CHAPTER_CARD);
}

#[test]
fn chapter_card_snapshot() {
    let h = Harness::with_screen(Box::new(card("Chapter 1: The Letter")));
    assert_snapshot!(h.snapshot());
}
