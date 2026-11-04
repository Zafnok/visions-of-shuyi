use insta::assert_snapshot;

use super::*;
use crate::harness::Harness;
use crate::input::Action;
use crate::screen::tests::ctx;
use crate::screens::popped_with;

fn update(s: &mut VoicesQuestionScreen, actions: &[Action]) -> Transition {
    let input = FrameInput::new(actions.to_vec(), 0.0, vec![]);
    s.update(&mut ctx(), &input)
}

/// The game flow reaches the screen through `as_any`.
#[test]
fn it_can_be_found_by_its_type() {
    let s = VoicesQuestionScreen::new(&ctx());
    assert_eq!(s.name(), "voices_question");
    let any = Screen::as_any(&s).expect("opts into as_any");
    assert!(any.downcast_ref::<VoicesQuestionScreen>().is_some());
}

/// `Yes` is highlighted, so Confirm at once answers yes; `No` is below it.
#[test]
fn confirm_pops_with_the_answer_yes_first() {
    let mut s = VoicesQuestionScreen::new(&ctx());
    assert_eq!(popped_with(update(&mut s, &[Action::Confirm])), Some(true));
    let mut s = VoicesQuestionScreen::new(&ctx());
    let picked = update(&mut s, &[Action::CursorDown, Action::Confirm]);
    assert_eq!(popped_with(picked), Some(false));
}

#[test]
fn cancel_goes_back_without_an_answer() {
    let mut s = VoicesQuestionScreen::new(&ctx());
    assert!(matches!(update(&mut s, &[Action::Cancel]), Transition::Pop));
}

/// The view: the design doc's question, `Yes` focused, then `No`.
#[test]
fn the_view_asks_with_yes_focused() {
    let c = ctx();
    let view = VoicesQuestionScreen::new(&c).view(&c);
    assert_eq!(view.question, "Play with AI-generated voices?");
    let labels: Vec<_> = view
        .menu
        .items
        .iter()
        .map(|i| crate::widgets::menu::text_of(&i.label))
        .collect();
    assert_eq!(labels, ["Yes", "No"]);
    assert_eq!(
        crate::widgets::menu::text_of(&view.menu.focused().unwrap().label),
        "Yes"
    );
}

#[test]
fn voices_question_snapshot() {
    let h = Harness::with_screen(Box::new(VoicesQuestionScreen::new(&ctx())));
    assert_snapshot!(h.snapshot());
}

#[test]
fn voices_question_declares_its_actions() {
    let screen = VoicesQuestionScreen::new(&ctx());
    crate::screens::actions::assert_declares(&screen, crate::screens::actions::VOICES_QUESTION);
}
