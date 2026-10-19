//! The shared one-bit look, and what the screens' Playdate skins' tests
//! use: the Playdate's context and screen, and a row's text.

use trpg_content::Ink;

use super::*;
pub(crate) use crate::map_view::playdate::tests::{blank, c, pd_ctx};
use crate::widgets::MenuItemView;

/// Row `y` of `buf`, its glyphs trimmed.
pub(crate) fn row(buf: &GlyphBuffer, y: i32) -> String {
    (0..i32::from(buf.width()))
        .map(|x| buf.get(x, y).map_or(' ', |c| c.glyph))
        .collect::<String>()
        .trim()
        .to_owned()
}

/// Every row of `buf`, trimmed, top to bottom.
pub(crate) fn rows(buf: &GlyphBuffer) -> Vec<String> {
    (0..i32::from(buf.height())).map(|y| row(buf, y)).collect()
}

/// The first cell of row `y` holding `text`'s first letter where `text`
/// starts, if it is on that row.
pub(crate) fn find(buf: &GlyphBuffer, y: i32, text: &str) -> Option<i32> {
    let line: String = (0..i32::from(buf.width()))
        .map(|x| buf.get(x, y).map_or(' ', |c| c.glyph))
        .collect();
    let at = line.find(text)?;
    Some(to_i32(line[..at].chars().count()))
}

/// The cell at `(x, y)`.
pub(crate) fn cell(buf: &GlyphBuffer, x: i32, y: i32) -> Cell {
    *buf.get(x, y)
        .unwrap_or_else(|| panic!("no cell at {x}, {y}"))
}

/// Whether the cell where `text` starts on row `y` is the focus bar:
/// white letters on black.
pub(crate) fn is_bar(buf: &GlyphBuffer, y: i32, text: &str) -> bool {
    let Some(x) = find(buf, y, text) else {
        return false;
    };
    let at = cell(buf, x, y);
    (at.fg, at.bg) == (c(Ink::White), c(Ink::Black))
}

fn item(label: &str, focused: bool, enabled: bool) -> MenuItemView {
    MenuItemView {
        label: label.to_owned(),
        suffix: None,
        note: None,
        enabled,
        focused,
    }
}

#[test]
fn the_playdate_palettes_inks_are_black_on_white_and_a_white_on_black_bar() {
    let ink = inks(&pd_ctx());
    assert_eq!((ink.ground, ink.inside), (c(Ink::White), c(Ink::White)));
    assert_eq!((ink.text, ink.frame), (c(Ink::Black), c(Ink::Black)));
    assert_eq!(ink.bar, (c(Ink::Black), c(Ink::White)));
    assert_eq!(ink.off, c(Ink::Grey));
}

#[test]
fn a_window_is_a_double_frame_with_its_title_and_footer_in_it() {
    let ctx = pd_ctx();
    let mut buf = blank();
    let rect = Rect::new(2, 1, 20, 5);
    window(&ctx, &mut buf, rect, "Title");
    footer(&ctx, &mut buf, rect, "more");
    assert_eq!(row(&buf, 1), "╔═ Title ══════════╗");
    assert_eq!(row(&buf, 2), "║                  ║");
    assert_eq!(row(&buf, 5), "╚══════ more ══════╝");
    assert_eq!(cell(&buf, 3, 3).bg, c(Ink::White));
    // A title or footer too long for the frame is cut inside the corners.
    window(&ctx, &mut buf, rect, "A title far too long to fit");
    footer(&ctx, &mut buf, rect, "a footer far too long to fit");
    assert_eq!(row(&buf, 1), "╔═ A title far to…═╗");
    assert_eq!(row(&buf, 5), "╚═ a footer far t…═╝");
}

#[test]
fn a_menu_is_a_window_an_item_a_row_the_focus_a_bar_across_it() {
    let ctx = pd_ctx();
    let mut buf = blank();
    let view = MenuView {
        items: vec![
            item("One", false, true),
            item("Second", true, true),
            item("Off", false, false),
        ],
        first_shown: 0,
        shown: 3,
    };
    assert_eq!(menu_size(&view), (10, 5));
    super::menu(&ctx, &mut buf, (1, 0), &view);
    assert_eq!(
        rows(&buf)[..5],
        [
            "╔════════╗",
            "║ One    ║",
            "║ Second ║",
            "║ Off    ║",
            "╚════════╝"
        ]
    );
    // The focus is a bar across the inside; what can't be chosen is grey.
    assert!(is_bar(&buf, 2, "Second"));
    for x in 2..=9 {
        assert_eq!(cell(&buf, x, 2).bg, c(Ink::Black), "{x}");
    }
    assert!(!is_bar(&buf, 1, "One"));
    assert_eq!(cell(&buf, 3, 3).fg, c(Ink::Grey));
}

#[test]
fn wrap_breaks_by_words_and_keeps_the_texts_own_line_breaks() {
    assert_eq!(
        wrap("one two three\nfour five", 9),
        ["one two", "three", "four five"]
    );
    assert!(wrap("", 9).is_empty());
}

#[test]
fn a_long_list_shows_its_focus_in_the_middle_where_it_can() {
    // All fit: from the top.
    assert_eq!(first_shown(5, 8, Some(4)), 0);
    // 20 lines in 8 rows.
    assert_eq!(first_shown(20, 8, None), 0);
    assert_eq!(first_shown(20, 8, Some(3)), 0);
    assert_eq!(first_shown(20, 8, Some(10)), 6);
    assert_eq!(first_shown(20, 8, Some(19)), 12);
    assert_eq!(first_shown(20, 8, Some(99)), 12);
}

#[test]
fn the_more_marks_sit_at_the_frames_right_end() {
    let ctx = pd_ctx();
    let mut buf = blank();
    let rect = Rect::new(0, 0, 10, 4);
    window(&ctx, &mut buf, rect, "");
    more_marks(&ctx, &mut buf, rect, true, false);
    assert_eq!(row(&buf, 0), "╔══════▲═╗");
    assert_eq!(row(&buf, 3), "╚════════╝");
    more_marks(&ctx, &mut buf, rect, false, true);
    assert_eq!(row(&buf, 3), "╚══════▼═╝");
}

/// Whether the cell at `(x, y)` is the bar's.
fn bar_at(buf: &GlyphBuffer, x: i32, y: i32) -> bool {
    cell(buf, x, y).bg == c(Ink::Black)
}

#[test]
fn a_question_is_a_window_in_the_middle_its_answers_under_a_blank_row() {
    let ctx = pd_ctx();
    let mut buf = blank();
    let answers = [("Yes", false), ("No", true)];
    let rect = ask(&ctx, &mut buf, "Delete it?", Answers::Choices(&answers));
    assert_eq!(rect, Rect::new(17, 5, 15, 5));
    let shown = rows(&buf);
    assert_eq!(
        shown[5..10],
        [
            "╔═════════════╗",
            "║ Delete it?  ║",
            "║             ║",
            "║  Yes    No  ║",
            "╚═════════════╝"
        ]
    );
    // The answer in focus is the bar, a cell either side of it.
    assert!(!bar_at(&buf, 25, 8) && bar_at(&buf, 26, 8) && bar_at(&buf, 29, 8));
    assert!(!bar_at(&buf, 30, 8) && !bar_at(&buf, 20, 8));
    // Focus on the first: its bar is five cells.
    let mut buf = blank();
    let answers = [("Yes", true), ("No", false)];
    ask(&ctx, &mut buf, "Delete it?", Answers::Choices(&answers));
    assert!(bar_at(&buf, 19, 8) && bar_at(&buf, 23, 8) && !bar_at(&buf, 24, 8));
}

#[test]
fn a_questions_hint_too_long_for_the_screen_is_cut_inside_the_frame() {
    let ctx = pd_ctx();
    let mut buf = blank();
    let hint = "a hint far longer than the fifty cells the Playdate has";
    let rect = ask(&ctx, &mut buf, "Sure?", Answers::Hint(hint));
    assert_eq!((rect.x, rect.w), (0, 50));
    let y = rect.y + rect.h - 2;
    assert_eq!(cell(&buf, 0, y).glyph, '║');
    assert_eq!(cell(&buf, 1, y).glyph, 'a');
    assert_eq!(cell(&buf, 48, y).glyph, '…');
    assert_eq!(cell(&buf, 49, y).glyph, '║');
    // A short one sits in the middle.
    let mut buf = blank();
    let rect = ask(
        &ctx,
        &mut buf,
        "Restore every option?",
        Answers::Hint("A yes / B no"),
    );
    assert_eq!(rect, Rect::new(12, 5, 25, 5));
    assert_eq!(rows(&buf)[8], "║     A yes / B no      ║");
}

#[test]
fn a_list_is_a_window_in_the_middle_its_title_in_the_frame() {
    let ctx = pd_ctx();
    let mut buf = blank();
    let rect = list(
        &ctx,
        &mut buf,
        "Mei",
        &[("Rename", true), ("Delete", false)],
    );
    assert_eq!(rect, Rect::new(19, 5, 12, 4));
    let shown = rows(&buf);
    assert_eq!(
        shown[5..9],
        [
            "╔═ Mei ════╗",
            "║  Rename  ║",
            "║  Delete  ║",
            "╚══════════╝"
        ]
    );
    assert_eq!(find(&buf, 6, "Rename"), Some(22));
    assert!(!bar_at(&buf, 20, 6) && bar_at(&buf, 21, 6) && bar_at(&buf, 28, 6));
    assert!(!bar_at(&buf, 29, 6) && !bar_at(&buf, 22, 7));
    // A title wider than the items, and a third item: the window grows.
    let mut buf = blank();
    let items = [("Rename", false), ("Delete", false), ("Use", true)];
    let rect = list(&ctx, &mut buf, "Player 12", &items);
    assert_eq!(rect, Rect::new(16, 5, 17, 5));
    assert_eq!(rows(&buf)[5], "╔═ Player 12 ═══╗");
    assert!(rows(&buf)[9].starts_with('╚'));
}
