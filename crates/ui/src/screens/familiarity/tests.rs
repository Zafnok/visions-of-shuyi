use insta::assert_snapshot;

use super::*;
use crate::harness::Harness;
use crate::input::Action;
use crate::screen::tests::ctx;

fn update(s: &mut FamiliarityScreen, actions: &[Action]) -> String {
    let input = FrameInput::new(actions.to_vec(), 0.0, vec![]);
    format!("{:?}", s.update(&mut ctx(), &input))
}

/// The tests (and the game flow) reach the screen through `as_any`.
#[test]
fn it_can_be_found_by_its_type() {
    let s = FamiliarityScreen::new(&ctx());
    let any = Screen::as_any(&s).expect("opts into as_any");
    assert!(any.downcast_ref::<FamiliarityScreen>().is_some());
}

#[test]
fn confirm_picks_the_focused_level() {
    let picked = |downs: usize| {
        let mut s = FamiliarityScreen::new(&ctx());
        for _ in 0..downs {
            assert_eq!(update(&mut s, &[Action::CursorDown]), "None");
        }
        assert_eq!(s.result(), None);
        assert_eq!(update(&mut s, &[Action::Confirm]), "Pop");
        s.result()
    };
    assert_eq!(picked(0), Some(TutorialLevel::Full));
    assert_eq!(picked(1), Some(TutorialLevel::Short));
    assert_eq!(picked(2), Some(TutorialLevel::None));
    let mut s = FamiliarityScreen::new(&ctx());
    assert_eq!(s.name(), "familiarity");
    assert_eq!(
        update(&mut s, &[Action::Confirm, Action::CursorDown]),
        "Pop"
    );
    assert_eq!(s.result(), Some(TutorialLevel::Full));
}

#[test]
fn cancel_goes_back_without_a_level() {
    let mut s = FamiliarityScreen::new(&ctx());
    assert_eq!(update(&mut s, &[Action::Cancel]), "Pop");
    assert_eq!(s.result(), None);
}

/// The view on the first frame: the heading, the three answers ("Not
/// familiar" focused), their lines and the help.
#[test]
fn the_view_names_the_three_answers_with_not_familiar_focused() {
    let c = ctx();
    let view = FamiliarityScreen::new(&c).view(&c);
    assert_eq!(view.heading, c.text(HEADING));
    let labels: Vec<_> = view.menu.items.iter().map(|i| i.label.as_str()).collect();
    assert_eq!(
        labels,
        [
            "Not familiar",
            "Familiar with other strategy RPGs",
            "Familiar with this game"
        ]
    );
    assert_eq!(view.menu.focused().unwrap().label, "Not familiar");
    let lines: Vec<_> = view.lines.iter().map(|l| l.text.as_str()).collect();
    assert_eq!(
        lines,
        LEVELS.map(|(_, _, line)| c.text(line)),
        "one line per answer, in the menu's order"
    );
    assert_eq!(
        view.lines.iter().map(|l| l.focused).collect::<Vec<_>>(),
        [true, false, false]
    );
    assert_eq!(view.help, FamiliarityScreen::help(&c));
}

/// Moving the focus moves the line to read with it.
#[test]
fn the_focused_line_follows_the_menu() {
    let mut c = ctx();
    let mut s = FamiliarityScreen::new(&c);
    let input = FrameInput::new(vec![Action::CursorDown], 0.0, vec![]);
    s.update(&mut c, &input);
    let view = s.view(&c);
    assert_eq!(
        view.menu.focused().unwrap().label,
        "Familiar with other strategy RPGs"
    );
    assert_eq!(
        view.lines.iter().map(|l| l.focused).collect::<Vec<_>>(),
        [false, true, false]
    );
}

/// The heading, the three answers (the second focused) and their lines,
/// the focused one brighter.
#[test]
fn familiarity_snapshot() {
    let mut h = Harness::with_screen(Box::new(FamiliarityScreen::new(&ctx())));
    h.keys("Down");
    assert_snapshot!(h.snapshot());
}
