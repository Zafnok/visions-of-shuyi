use insta::assert_snapshot;

use super::*;
use crate::harness::Harness;
use crate::input::Action;
use crate::screen::tests::ctx;

fn update(s: &mut ModeSelectScreen, actions: &[Action]) -> String {
    let input = FrameInput::new(actions.to_vec(), 0.0, vec![]);
    format!("{:?}", s.update(&mut ctx(), &input))
}

#[test]
fn confirm_picks_the_focused_mode() {
    let mut s = ModeSelectScreen::new(&ctx());
    assert_eq!(s.name(), "mode_select");
    assert_eq!(update(&mut s, &[Action::CursorDown]), "None");
    assert_eq!(s.result(), None);
    assert_eq!(update(&mut s, &[Action::Confirm]), "Pop");
    assert_eq!(s.result(), Some(GameMode::Casual));
    let mut s = ModeSelectScreen::new(&ctx());
    assert_eq!(
        update(&mut s, &[Action::Confirm, Action::CursorDown]),
        "Pop"
    );
    assert_eq!(s.result(), Some(GameMode::Classic));
}

#[test]
fn cancel_goes_back_without_a_mode() {
    let mut s = ModeSelectScreen::new(&ctx());
    assert_eq!(update(&mut s, &[Action::Cancel]), "Pop");
    assert_eq!(s.result(), None);
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
