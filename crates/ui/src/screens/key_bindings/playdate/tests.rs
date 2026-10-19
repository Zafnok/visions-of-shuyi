//! The Playdate's look of the Key bindings screen, on the game's rows and
//! the Playdate's buttons.

use shuyi_ui::Screen;

use super::*;
use crate::input::{Action, Device, PadKind};
use crate::screen::{FrameInput, Keyboard, ShuyiCtx};
use crate::screens::key_bindings::{KeyBindingsScreen, screen};
use crate::screens::playdate::tests::{blank, cell, find, is_bar, pd_ctx, rows};
use crate::screens::playdate::{MORE_ABOVE, MORE_BELOW};

/// The context as the Playdate starts it: its skins and palette, no
/// keyboard, its own buttons.
fn playdate() -> Ctx {
    let mut c = pd_ctx();
    c.keyboard = Keyboard::Absent;
    c.device = Device::Pad(PadKind::Playdate);
    c.use_playdate_buttons();
    c
}

fn press(s: &mut KeyBindingsScreen, c: &mut Ctx, actions: &[Action]) {
    s.update(c, &FrameInput::new(actions.to_vec(), 0.0, vec![]));
}

/// What `s` draws.
fn drawn(s: &KeyBindingsScreen, c: &Ctx) -> GlyphBuffer {
    let mut buf = blank();
    s.draw(c, &mut buf);
    buf
}

/// The row of `buf` naming `label`.
fn line_of(buf: &GlyphBuffer, label: &str) -> i32 {
    let shown = rows(buf);
    let at = shown.iter().position(|r| r.contains(label));
    to_i32(at.unwrap_or_else(|| panic!("no {label}: {shown:#?}")))
}

#[test]
fn the_buttons_of_each_action_under_its_groups_heading() {
    let c = playdate();
    let buf = drawn(&screen(&c), &c);
    let shown = rows(&buf);
    assert!(shown[0].starts_with("╔═ Key bindings ═"), "{shown:#?}");
    assert!(shown[13].starts_with('╚'));
    // No switch: the first group at once.
    assert!(shown[1].contains("Must have a button"));
    assert!(!shown.iter().any(|r| r.contains("Keyboard")));
    // Cursor up: ↑, its first slot the bar.
    assert!(shown[2].contains("Cursor up"));
    assert!(is_bar(&buf, 2, " ↑"));
    assert_eq!(find(&buf, 2, "·"), Some(SLOTS[1].0 + 1));
    // End turn: the chord, A+B.
    let end_turn = line_of(&buf, "End turn");
    assert_eq!(find(&buf, end_turn, "A+B"), Some(SLOTS[0].0 + 1));
    assert!(shown[14].contains('A'), "{}", shown[14]);
    // More below.
    assert!(shown[13].contains(MORE_BELOW) && !shown[0].contains(MORE_ABOVE));
}

#[test]
fn the_crank_and_the_not_mapped_actions_further_down() {
    let mut c = playdate();
    let mut s = screen(&c);
    // Up from the first row: Restore defaults, the window at its end.
    press(&mut s, &mut c, &[Action::CursorUp]);
    let buf = drawn(&s, &c);
    let shown = rows(&buf);
    assert!(shown[0].contains(MORE_ABOVE) && !shown[13].contains(MORE_BELOW));
    assert!(is_bar(&buf, 12, " Restore defaults"), "{shown:#?}");
    assert!(shown[9].contains("In the dialogue log"));
    assert!(shown[10].contains("Open the log"));
    // Up again, past the log and the map menu: the crank's actions, named
    // in full in the first slot.
    for _ in 0..3 {
        press(&mut s, &mut c, &[Action::CursorUp]);
    }
    let buf = drawn(&s, &c);
    let back = line_of(&buf, "Rewind back");
    assert_eq!(find(&buf, back, "Crank back"), Some(SLOTS[0].0 + 1));
    let forward = line_of(&buf, "Rewind forward");
    assert!(is_bar(&buf, forward, " Crank forward"));
    // In the middle of the list: more above and below.
    let shown = rows(&buf);
    assert!(shown[0].contains(MORE_ABOVE) && shown[13].contains(MORE_BELOW));
    // An action with no button: the note over its slots.
    let menu = line_of(&buf, "Map menu");
    let note = c.help_keys().not_mapped().to_owned();
    assert_eq!(find(&buf, menu, &note), Some(SLOTS[0].0 + 1));
    // A long name is cut.
    for _ in 0..7 {
        press(&mut s, &mut c, &[Action::CursorUp]);
    }
    let buf = drawn(&s, &c);
    let prev = line_of(&buf, "Previous ready");
    assert!(is_bar(&buf, prev, " B+←"));
    assert!(rows(&buf)[usize::try_from(prev).unwrap()].contains("Previous ready…"));
}

#[test]
fn a_slots_choices_open_under_it_and_capturing_shows_the_prompt() {
    let mut c = playdate();
    let mut s = screen(&c);
    press(&mut s, &mut c, &[Action::Confirm]);
    let view = s.view(&c);
    let choices = view.choices.unwrap();
    let buf = drawn(&s, &c);
    // Under Cursor up's first slot: Change (the bar), Clear.
    assert!(rows(&buf)[3].contains('╔'));
    assert!(is_bar(&buf, 4, &format!(" {}", choices.lines[0])));
    assert_eq!(find(&buf, 5, &choices.lines[1]), Some(SLOTS[0].0 + 2));
    press(&mut s, &mut c, &[Action::Confirm]);
    assert!(s.is_capturing());
    let buf = drawn(&s, &c);
    let prompt = c.text(crate::input::CAPTURE_BUTTON_PROMPT);
    assert!(is_bar(&buf, 2, &format!(" {}", &prompt[..5])));
}

#[test]
fn with_a_keyboard_the_switch_is_the_first_line() {
    let c = pd_ctx();
    let buf = drawn(&screen(&c), &c);
    let shown = rows(&buf);
    assert!(shown[1].contains("Keyboard") && shown[1].contains("Controller"));
    assert!(shown[3].contains("Must have a key"), "{shown:#?}");
}

#[test]
fn the_restore_question_opens_in_a_window() {
    let mut c = playdate();
    let mut s = screen(&c);
    press(&mut s, &mut c, &[Action::CursorUp]);
    press(&mut s, &mut c, &[Action::Confirm]);
    let question = s.view(&c).question.unwrap();
    let buf = drawn(&s, &c);
    let shown = rows(&buf);
    let at = shown
        .iter()
        .position(|r| r.contains(&question.answers))
        .unwrap();
    assert!(shown[at - 2].contains(&question.text), "{shown:#?}");
}

/// The view of the screen as it opens, to change by hand.
fn opened() -> (Ctx, KeyBindingsView) {
    let c = playdate();
    let view = screen(&c).view(&c);
    (c, view)
}

/// `view` painted.
fn painted(c: &Ctx, view: &KeyBindingsView) -> GlyphBuffer {
    let mut buf = blank();
    paint(c, view, &mut buf);
    buf
}

/// The row of `view` for `action`, to change.
fn row_mut(view: &mut KeyBindingsView, action: Action) -> &mut RowView {
    let mut rows = view.groups.iter_mut().flat_map(|g| g.rows.iter_mut());
    rows.find(|r| r.action == action).unwrap()
}

#[test]
fn an_action_that_lost_its_button_is_marked() {
    let (c, mut view) = opened();
    row_mut(&mut view, Action::Cancel).lost_key = true;
    let buf = painted(&c, &view);
    let cancel = line_of(&buf, "Cancel");
    assert_eq!(find(&buf, cancel, LOST), Some(HEADING_X));
    assert_eq!(find(&buf, line_of(&buf, "Confirm"), LOST), None);
}

#[test]
fn the_not_mapped_note_follows_the_bar_on_an_empty_slot() {
    let (c, mut view) = opened();
    row_mut(&mut view, Action::CursorUp).focus = None;
    row_mut(&mut view, Action::Select).focus = Some(0);
    let buf = painted(&c, &view);
    let select = line_of(&buf, "Select");
    let bar = |x| cell(&buf, x, select);
    let ink = |i| crate::screens::playdate::tests::c(i);
    assert_eq!(bar(SLOTS[0].0).bg, ink(trpg_content::Ink::Black));
    assert_eq!(
        bar(SLOTS[0].0 + SLOTS[0].1 - 1).bg,
        ink(trpg_content::Ink::Black)
    );
    assert_eq!(
        bar(SLOTS[0].0 + SLOTS[0].1).bg,
        ink(trpg_content::Ink::White)
    );
    let note = c.help_keys().not_mapped().to_owned();
    assert_eq!(find(&buf, select, &note), Some(SLOTS[0].0 + SLOTS[0].1 + 1));
}

#[test]
fn choices_near_the_bottom_open_over_the_slot_and_stay_in_the_window() {
    let (c, mut view) = opened();
    row_mut(&mut view, Action::CursorUp).focus = None;
    row_mut(&mut view, Action::DialogueLog).focus = Some(2);
    view.choices = Some(ChoicesView {
        lines: vec!["Change".to_owned(), "Clear".to_owned(), "Swap".to_owned()],
        focused: 1,
        slot: 2,
    });
    let buf = painted(&c, &view);
    let log = line_of(&buf, "Open the log");
    // A row a choice and the frame, over the slot; at the window's right
    // edge.
    let shown = rows(&buf);
    let top = usize::try_from(log - 5).unwrap();
    let x = 50 - CHOICES_W;
    assert_eq!(find(&buf, log - 5, "╔"), Some(x), "{shown:#?}");
    assert!(shown[top + 4].contains('╚'));
    assert!(is_bar(&buf, log - 3, " Clear"));
    // Restore defaults, under it, in full.
    assert!(shown[12].contains("Restore defaults"), "{shown:#?}");
    assert_eq!(find(&buf, log - 4, "Change"), Some(x + 2));
    assert_eq!(find(&buf, log - 2, "Swap"), Some(x + 2));
}

#[test]
fn long_words_stay_inside_their_columns_and_the_frame() {
    let (c, mut view) = opened();
    view.restore.label = "Put every button back as it was".to_owned();
    view.not_mapped = "! no button for this one yet, give it one".to_owned();
    row_mut(&mut view, Action::CursorUp).focus = None;
    row_mut(&mut view, Action::DialogueLog).focus = Some(1);
    let buf = painted(&c, &view);
    // Restore defaults across the names' and the first slot's columns.
    let shown = rows(&buf);
    assert!(
        shown[12].contains("Put every button back as it w…"),
        "{shown:#?}"
    );
    // The note cut a cell before the frame's right side.
    let log = line_of(&buf, "Open the log");
    // Over the slots from the first's column: the cursor is on the second.
    assert_eq!(cell(&buf, SLOTS[0].0 + 1, log).glyph, '!');
    assert_eq!(cell(&buf, 47, log).glyph, '…');
    assert_eq!(cell(&buf, 48, log).glyph, ' ');
    assert_eq!(cell(&buf, 49, log).glyph, '║');
    // A long name in a narrow slot ends a cell before the next one.
    let (c, mut view) = opened();
    row_mut(&mut view, Action::Confirm).slots[1] = SlotView::Bound("Crank forward".to_owned());
    let buf = painted(&c, &view);
    let confirm = line_of(&buf, "Confirm");
    assert_eq!(find(&buf, confirm, "Crank …"), Some(SLOTS[1].0 + 1));
    assert_eq!(cell(&buf, SLOTS[2].0, confirm).glyph, ' ');
}

#[test]
fn the_switch_names_the_shown_side_plainly_and_the_other_in_grey() {
    let c = pd_ctx();
    let buf = drawn(&screen(&c), &c);
    let ink = |i| crate::screens::playdate::tests::c(i);
    let keyboard = find(&buf, 1, "Keyboard").unwrap();
    let controller = find(&buf, 1, "Controller").unwrap();
    assert_eq!(cell(&buf, keyboard, 1).fg, ink(trpg_content::Ink::Black));
    assert_eq!(cell(&buf, controller, 1).fg, ink(trpg_content::Ink::Grey));
    // Three cells apart.
    let switch = screen(&c).view(&c).switch.unwrap();
    assert_eq!(controller, keyboard + cells(&switch.keyboard) + 3);
}

#[test]
fn the_choices_open_under_the_slot_where_they_fit() {
    let mut c = playdate();
    let mut s = screen(&c);
    press(&mut s, &mut c, &[Action::CursorDown, Action::CursorDown]);
    press(&mut s, &mut c, &[Action::Confirm]);
    let buf = drawn(&s, &c);
    // Cursor left, on row 4: the window from row 5, its last at row 8.
    assert!(rows(&buf)[4].contains("Cursor left"));
    assert_eq!(cell(&buf, SLOTS[0].0, 5).glyph, '╔');
    assert_eq!(cell(&buf, SLOTS[0].0, 8).glyph, '╚');
}

#[test]
fn the_choices_window_keeps_its_frame_round_the_bar() {
    let mut c = playdate();
    let mut s = screen(&c);
    press(&mut s, &mut c, &[Action::Confirm]);
    let buf = drawn(&s, &c);
    // Under Cursor up's first slot: from its column, twelve cells wide.
    let (x, y) = (SLOTS[0].0, 4);
    assert_eq!(cell(&buf, x, y).glyph, '║');
    assert_eq!(cell(&buf, x + CHOICES_W - 1, y).glyph, '║');
    assert_eq!(cell(&buf, x + CHOICES_W - 2, y).glyph, ' ');
    assert!(is_bar(&buf, y, " Change"));
}
