use trpg_content::CreditGroup;

use super::*;
use crate::console::ConsoleSpec;
use crate::input::Action;
use crate::screen::{Screen, tests::ctx};
use crate::screens::credits::tests::{ctx_with, entry, input, long_list};
use crate::screens::credits::{CreditRowView, CreditsScreen, LIST_H, width};

fn draw(s: &CreditsScreen, c: &Ctx) -> GlyphBuffer {
    let stale = Cell::new(
        'x',
        c.palette.get(UiColor::Enemy),
        c.palette.get(UiColor::Enemy),
    );
    let mut buf = GlyphBuffer::with_spec(ConsoleSpec::PC, stale);
    s.draw(c, &mut buf);
    buf
}

fn row(buf: &GlyphBuffer, y: i32) -> String {
    (0..i32::from(buf.width()))
        .map(|x| buf.get(x, y).map_or(' ', |c| c.glyph))
        .collect()
}

/// The screen's wrapping widths are what the panel holds, and the list it
/// shows fills the panel.
#[test]
fn the_panel_fits_what_the_screen_wraps_to() {
    assert_eq!(HEADING_X, PANEL.x + 2);
    for kind in [
        CreditRowKind::Heading,
        CreditRowKind::Entry,
        CreditRowKind::Detail,
    ] {
        let room = usize::try_from(TEXT_END_X - x(kind) + 1).unwrap();
        assert_eq!(width(kind), room, "{kind:?}");
    }
    assert_eq!(usize::try_from(ROWS).unwrap(), LIST_H);
    // A blank row under the last one, then the border.
    assert_eq!(LIST_Y + ROWS + 1, PANEL.y + PANEL.h - 1);
}

#[test]
fn long_lines_stay_inside_the_panel() {
    let mut long = entry(CreditGroup::Music, &"word ".repeat(30));
    long.source = format!("https://example.org/{}", "x".repeat(120));
    let c = ctx_with(vec![long]);
    let s = CreditsScreen::new(&c);
    // Nothing is drawn on or past the panel's border, or in the scroll
    // marks' column.
    let buf = draw(&s, &c);
    for y in LIST_Y..LIST_Y + 7 {
        let line = row(&buf, y);
        assert_eq!(line.chars().nth(95), Some(' '), "{line}");
        assert_eq!(line.chars().nth(96), Some(' '), "{line}");
        assert_eq!(line.chars().nth(97), Some('│'), "{line}");
    }
    assert_eq!(row(&buf, LIST_Y + 5).chars().nth(94), Some('x'));
}

#[test]
fn a_short_list_draws_no_scroll_marks() {
    let c = ctx_with(vec![entry(CreditGroup::Fonts, "Font")]);
    let buf = draw(&CreditsScreen::new(&c), &c);
    let all: String = (0..i32::from(ConsoleSpec::PC.rows))
        .map(|y| row(&buf, y))
        .collect();
    assert!(!all.contains(MORE_ABOVE) && !all.contains(MORE_BELOW));
}

#[test]
fn draws_the_rows_from_the_top_one_with_scroll_marks() {
    let mut c = long_list();
    let mut s = CreditsScreen::new(&c);
    let buf = draw(&s, &c);
    assert!(row(&buf, 1).starts_with("  ┌─ Credits ─"));
    assert!(row(&buf, 3).starts_with("  │ Music "));
    assert!(row(&buf, 5).starts_with("  │   \"Track00\" by Ann "));
    assert!(row(&buf, 6).starts_with("  │     CC0-1.0 · https://example.org/Track00 "));
    // 25 rows: the last is the eighth entry's second line, with the
    // "more below" mark at the right.
    assert!(row(&buf, 27).starts_with("  │     CC0-1.0 · https://example.org/Track07 "));
    assert_eq!(row(&buf, 27).chars().nth(95), Some(MORE_BELOW));
    assert_eq!(row(&buf, 3).chars().nth(95), Some(' '));
    // The row above the bottom border stays blank.
    assert_eq!(row(&buf, 28).replace(' ', ""), "││");
    assert!(row(&buf, 29).starts_with("  └─"));
    assert_eq!(row(&buf, 31).trim(), s.help(&c));

    s.update(&mut c, &input(&[Action::CursorDown; 3]));
    let buf = draw(&s, &c);
    assert!(row(&buf, 3).starts_with("  │     CC0-1.0 · https://example.org/Track00 "));
    assert_eq!(row(&buf, 3).chars().nth(95), Some(MORE_ABOVE));
    assert_eq!(row(&buf, 27).chars().nth(95), Some(MORE_BELOW));

    s.update(&mut c, &input(&[Action::CursorDown; 40]));
    let buf = draw(&s, &c);
    assert!(row(&buf, 27).starts_with("  │     CC0-1.0 · https://example.org/Track19 "));
    assert_eq!(row(&buf, 27).chars().nth(95), Some(' '));
    assert_eq!(row(&buf, 3).chars().nth(95), Some(MORE_ABOVE));
}

#[test]
fn rows_are_coloured_by_kind() {
    let c = long_list();
    let buf = draw(&CreditsScreen::new(&c), &c);
    let fg = |x, y| buf.get(x, y).map(|cell| cell.fg);
    let p = |u| Some(c.palette.get(u));
    assert_eq!(fg(HEADING_X + 1, 1), p(UiColor::TextHighlight));
    assert_eq!(fg(HEADING_X, 3), p(UiColor::PanelBorderFocus));
    assert_eq!(fg(ENTRY_X, 5), p(UiColor::Text));
    assert_eq!(fg(DETAIL_X, 6), p(UiColor::TextDim));
    assert_eq!(fg(TEXT_END_X + 1, 27), p(UiColor::TextDim));
    assert_eq!(fg(50, 31), p(UiColor::TextDim));
}

/// The skin needs nothing but the view: a hand-made one paints.
#[test]
fn a_hand_made_view_paints() {
    let c = ctx();
    let view = CreditsView {
        title: "Credits".to_owned(),
        rows: vec![CreditRowView {
            kind: CreditRowKind::Heading,
            text: "Music".to_owned(),
        }],
        more_above: false,
        more_below: false,
        help: "help".to_owned(),
    };
    let mut buf = draw(&CreditsScreen::new(&c), &c);
    paint(&c, &view, &mut buf);
    assert!(row(&buf, LIST_Y).contains("Music"));
    assert!(!row(&buf, LIST_Y + 1).contains("Track"));
    assert_eq!(row(&buf, 31).trim(), "help");
}
