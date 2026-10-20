use insta::assert_snapshot;

use super::*;
use crate::harness::Harness;
use crate::input::Action;
use crate::screen::tests::ctx;
use crate::screens::popped_with;

fn update(s: &mut ModeSelectScreen, actions: &[Action]) -> Transition {
    let input = FrameInput::new(actions.to_vec(), 0.0, vec![]);
    s.update(&mut ctx(), &input)
}

/// The tests (and the game flow) reach the screen through `as_any`.
#[test]
fn it_can_be_found_by_its_type() {
    let s = ModeSelectScreen::new(&ctx());
    let any = Screen::as_any(&s).expect("opts into as_any");
    assert!(any.downcast_ref::<ModeSelectScreen>().is_some());
}

/// Confirm pops with the focused mode, for the game flow below.
#[test]
fn confirm_pops_with_the_focused_mode() {
    let mut s = ModeSelectScreen::new(&ctx());
    assert_eq!(s.name(), "mode_select");
    let moved = update(&mut s, &[Action::CursorDown]);
    assert!(matches!(moved, Transition::None));
    let picked = update(&mut s, &[Action::Confirm]);
    assert_eq!(popped_with(picked), Some(GameMode::Casual));
    let mut s = ModeSelectScreen::new(&ctx());
    let picked = update(&mut s, &[Action::Confirm, Action::CursorDown]);
    assert_eq!(popped_with(picked), Some(GameMode::Classic));
}

#[test]
fn cancel_goes_back_without_a_mode() {
    let mut s = ModeSelectScreen::new(&ctx());
    assert!(matches!(update(&mut s, &[Action::Cancel]), Transition::Pop));
}

/// The view on the first frame: the heading, both modes (Classic focused),
/// both lines and the help.
#[test]
fn the_view_names_both_modes_with_classic_focused() {
    let c = ctx();
    let view = ModeSelectScreen::new(&c).view(&c);
    assert_eq!(view.heading, c.text(HEADING));
    let labels: Vec<_> = view.menu.items.iter().map(|i| i.label.as_str()).collect();
    assert_eq!(labels, ["Classic", "Casual"]);
    assert_eq!(view.menu.focused().unwrap().label, "Classic");
    let lines: Vec<_> = view.lines.iter().map(|l| l.text.as_str()).collect();
    assert_eq!(
        lines,
        [
            c.text("mode_select.classic_line"),
            c.text("mode_select.casual_line")
        ]
    );
    assert_eq!(
        view.lines.iter().map(|l| l.focused).collect::<Vec<_>>(),
        [true, false]
    );
    assert_eq!(view.help, ModeSelectScreen::help(&c));
}

/// Moving the focus moves the line to read with it.
#[test]
fn the_focused_line_follows_the_menu() {
    let mut c = ctx();
    let mut s = ModeSelectScreen::new(&c);
    let input = FrameInput::new(vec![Action::CursorDown], 0.0, vec![]);
    s.update(&mut c, &input);
    let view = s.view(&c);
    assert_eq!(view.menu.focused().unwrap().label, "Casual");
    assert_eq!(
        view.lines.iter().map(|l| l.focused).collect::<Vec<_>>(),
        [false, true]
    );
}

/// The heading, the two modes (Casual focused) and both lines, the
/// focused one brighter.
#[test]
fn mode_select_snapshot() {
    let mut h = Harness::with_screen(Box::new(ModeSelectScreen::new(&ctx())));
    h.keys("Down");
    assert_snapshot!(h.snapshot());
}
