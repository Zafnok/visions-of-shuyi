//! The Playdate's look of the Options screen, on the game's rows.

use super::*;
use crate::input::Action;
use crate::screen::{FrameInput, Keyboard};
use crate::screens::options::{Row, screen};
use crate::screens::playdate::tests::{blank, cell, find, is_bar, pd_ctx, rows};
use crate::screens::playdate::{MORE_ABOVE, MORE_BELOW};
use shuyi_ui::Screen;

/// The Playdate's context: its skins and palette, no keyboard.
fn playdate() -> Ctx {
    let mut c = pd_ctx();
    c.platform.keyboard = Keyboard::Absent;
    c
}

fn press(s: &mut super::super::OptionsScreen, c: &mut Ctx, actions: &[Action]) {
    let input = FrameInput::new(actions.to_vec(), 0.0, vec![]);
    s.update(c, &input);
}

/// What `s` draws.
fn drawn(s: &super::super::OptionsScreen, c: &Ctx) -> GlyphBuffer {
    let mut buf = blank();
    s.draw(c, &mut buf);
    buf
}

#[test]
fn the_rows_fill_a_window_the_focused_name_a_bar_its_value_between_arrows() {
    let c = playdate();
    let buf = drawn(&screen(), &c);
    let shown = rows(&buf);
    assert!(shown[0].starts_with("╔═ Options ═"), "{shown:#?}");
    assert!(shown[13].starts_with('╚'));
    // Text speed, focused: the bar, and its value between arrows.
    assert!(is_bar(&buf, 1, " Text speed"));
    assert_eq!(find(&buf, 1, "◄ Normal ►"), Some(VALUE_X - 2));
    assert!(shown[2].starts_with("║  Animation speed"));
    assert!(!is_bar(&buf, 2, "Animation speed"));
    assert_eq!(find(&buf, 2, "Normal"), Some(VALUE_X));
    assert!(!shown[2].contains('◄'));
    // The volumes as bars; a blank row before Reset tips.
    let music = shown
        .iter()
        .position(|r| r.contains("Music volume"))
        .unwrap();
    assert!(shown[music].contains("█"), "{}", shown[music]);
    let reset = shown.iter().position(|r| r.contains("Reset tips")).unwrap();
    assert_eq!(shown[reset - 1], format!("║{:48}║", ""));
    assert!(shown[reset - 2].contains("Key bindings"));
    // All the Playdate's rows fit: no marks.
    assert!(!shown[0].contains(MORE_ABOVE) && !shown[13].contains(MORE_BELOW));
    assert_eq!(shown[14], screen().view(&c).help);
}

#[test]
fn rows_that_dont_fit_scroll_with_the_focus() {
    // The Playdate's skins with a keyboard: Layout, and its blank row, too.
    let mut c = pd_ctx();
    let mut s = screen();
    let buf = drawn(&s, &c);
    let shown = rows(&buf);
    assert!(shown[0].contains("Options") && !shown[0].contains(MORE_ABOVE));
    assert!(shown[13].contains(MORE_BELOW), "{shown:#?}");
    assert!(is_bar(&buf, 1, " Text speed"));
    // Up from the first row: the last, at the bottom of the window.
    press(&mut s, &mut c, &[Action::CursorUp]);
    let buf = drawn(&s, &c);
    let shown = rows(&buf);
    assert!(shown[0].contains(MORE_ABOVE) && !shown[13].contains(MORE_BELOW));
    assert!(is_bar(&buf, 12, " Restore defaults"), "{shown:#?}");
    assert!(!shown.iter().any(|r| r.contains("Text speed")));
}

#[test]
fn a_question_and_the_leaving_question_open_in_a_window() {
    let mut c = playdate();
    let mut s = screen().focused_on(Row::RestoreDefaults.key());
    press(&mut s, &mut c, &[Action::Confirm]);
    let view = s.view(&c);
    let question = view.question.unwrap();
    let buf = drawn(&s, &c);
    let shown = rows(&buf);
    let at = shown
        .iter()
        .position(|r| r.contains(question.keys.as_deref().unwrap_or_default()))
        .unwrap();
    assert!(shown[at + 1].starts_with('╚') || shown[at + 1].contains('╚'));
    assert!(shown[..at].iter().any(|r| r.contains("Restore")));
    // Leaving with a change asks: its answers on a line, the focus a bar.
    press(&mut s, &mut c, &[Action::Cancel]);
    let mut s = screen();
    press(&mut s, &mut c, &[Action::CursorRight, Action::Cancel]);
    let leaving = s.view(&c).leaving.unwrap();
    let focused = leaving.answers.iter().find(|c| c.focused).unwrap();
    let buf = drawn(&s, &c);
    let shown = rows(&buf);
    let line = shown
        .iter()
        .position(|r| r.contains(&focused.label))
        .unwrap();
    assert!(is_bar(&buf, to_i32(line), &format!(" {} ", focused.label)));
    for choice in &leaving.answers {
        assert!(shown[line].contains(&choice.label));
    }
}

#[test]
fn a_message_goes_in_the_bottom_of_the_frame() {
    let mut c = playdate();
    let mut s = screen().focused_on(Row::RestoreDefaults.key());
    press(&mut s, &mut c, &[Action::Confirm]);
    press(&mut s, &mut c, &[Action::Confirm]);
    let message = s.view(&c).message.unwrap();
    let shown = rows(&drawn(&s, &c));
    assert!(
        shown[13].starts_with('╚') && shown[13].contains(&message),
        "{shown:#?}"
    );
}

#[test]
fn a_long_value_is_cut_and_the_number_box_is_a_bar() {
    let c = playdate();
    let mut view = screen().view(&c);
    let long = "a value far too long for its column".to_owned();
    view.rows[1].value = ValueView::Text {
        text: long,
        adjustable: false,
    };
    view.rows[0].focused = false;
    let volume = view
        .rows
        .iter()
        .position(|r| matches!(r.value, ValueView::Volume { .. }));
    let volume = volume.unwrap();
    view.rows[volume].focused = true;
    view.rows[volume].value = ValueView::NumberBox {
        digits: "75".to_owned(),
        typing: true,
    };
    let mut buf = blank();
    paint(&c, &view, &mut buf);
    let shown = rows(&buf);
    // Cut a cell before the arrow's place and the frame.
    assert!(
        shown[2].ends_with("a value far too lon…  ║"),
        "{}",
        shown[2]
    );
    let y = to_i32(volume) + 1;
    assert!(is_bar(&buf, y, " 75_"));
    assert_eq!(find(&buf, y, " 75_"), Some(VALUE_X - 1));
    // The box: four cells and one either side.
    let black = crate::screens::playdate::tests::c(trpg_content::Ink::Black);
    let bar = |x| cell(&buf, x, y).bg == black;
    assert!(bar(VALUE_X + 4) && !bar(VALUE_X + 5));
}
