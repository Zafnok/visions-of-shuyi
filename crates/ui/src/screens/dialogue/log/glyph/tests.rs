//! Tests of the dialogue log's glyph look: where rows, names, faces and the
//! cursor's bar go, the colours, and the scrolling.

use super::*;
use crate::console::ConsoleSpec;
use crate::glyph_buffer::Item;
use crate::portrait::dim_opacity;
use crate::screen::tests::ctx;
use crate::screens::dialogue::log::{BackRow, Face, LogRow};

/// A row of `kind` saying `text`, by Ellery (with the lead's face) unless
/// narration.
fn row(kind: RowKind, text: &str) -> LogRow {
    let speaks = kind != RowKind::Narration;
    LogRow {
        line: None,
        kind,
        name: speaks.then(|| "Ellery".to_owned()),
        face: speaks.then(|| Face {
            portrait: "lead_m".to_owned(),
            expression: "neutral".to_owned(),
        }),
        text: text.to_owned(),
        ahead: false,
        focused: false,
    }
}

fn log(rows: Vec<LogRow>) -> LogView {
    LogView {
        title: "Log".to_owned(),
        rows,
        back_to_latest: None,
        help: "help keys".to_owned(),
    }
}

fn painted(view: &LogView) -> GlyphBuffer {
    let c = ctx();
    let stale = Cell::new(
        'x',
        c.palette.get(UiColor::Enemy),
        c.palette.get(UiColor::Enemy),
    );
    let mut buf = GlyphBuffer::with_spec(ConsoleSpec::PC, stale);
    paint(&c, view, &mut buf);
    buf
}

fn text(buf: &GlyphBuffer, y: i32) -> String {
    (0..i32::from(buf.width()))
        .map(|x| buf.get(x, y).map_or(' ', |c| c.glyph))
        .collect()
}

/// The column `word` starts at in console row `y`, in cells.
fn col(buf: &GlyphBuffer, y: i32, word: &str) -> Option<i32> {
    let row = text(buf, y);
    let at = row.find(word)?;
    i32::try_from(row[..at].chars().count()).ok()
}

/// The first console row whose text has `word`.
fn row_of(buf: &GlyphBuffer, word: &str) -> i32 {
    (0..i32::from(buf.height()))
        .find(|&y| text(buf, y).contains(word))
        .unwrap_or_else(|| panic!("no {word:?}"))
}

/// The sprites painted, as (top-left cell, opacity).
fn faces(buf: &GlyphBuffer) -> Vec<((i32, i32), u8)> {
    buf.items()
        .iter()
        .filter_map(|item| match item {
            Item::Sprite(s) => Some((
                (
                    s.dest.x / i32::from(ConsoleSpec::PC.cell_w),
                    s.dest.y / i32::from(ConsoleSpec::PC.cell_h),
                ),
                s.opacity,
            )),
            Item::Rect(_) => None,
        })
        .collect()
}

#[test]
fn it_fills_the_screen_with_the_title_and_the_help_on_the_border() {
    let buf = painted(&log(vec![row(RowKind::Speech, "Again.")]));
    assert!(text(&buf, 0).starts_with("┌── Log ─"));
    let bottom = i32::from(ConsoleSpec::PC.rows) - 1;
    assert!(text(&buf, bottom).trim_end().ends_with(" help keys ──┘"));
    let c = ctx();
    let stale = c.palette.get(UiColor::Enemy);
    let left = (0..i32::from(ConsoleSpec::PC.rows))
        .flat_map(|y| (0..i32::from(ConsoleSpec::PC.cols)).map(move |x| (x, y)))
        .filter(|&(x, y)| buf.get(x, y).is_some_and(|c| c.bg == stale))
        .count();
    assert_eq!(left, 0, "it covers the whole buffer");
}

#[test]
fn a_line_has_its_face_then_the_name_over_the_text() {
    let c = ctx();
    let buf = painted(&log(vec![
        row(RowKind::Speech, "Again."),
        row(RowKind::Narration, "Three strokes."),
        row(RowKind::Reply, "What does he want?"),
    ]));
    // The first row starts under the border: name, then text.
    assert_eq!(row_of(&buf, "Ellery"), 1);
    assert_eq!(col(&buf, 1, "Ellery"), Some(TEXT_X));
    assert_eq!(row_of(&buf, "Again."), 2);
    let fg = |x, y| buf.get(x, y).unwrap().fg;
    assert_eq!(fg(TEXT_X, 1), c.palette.get(UiColor::TextHighlight));
    assert_eq!(fg(TEXT_X, 2), c.palette.get(UiColor::Text));
    // The face is 4 rows tall: the narration comes after it and a blank row,
    // with no name, in the narration colour.
    let narration = row_of(&buf, "Three strokes.");
    assert_eq!(narration, 1 + FACE.1 + 1);
    assert_eq!(fg(TEXT_X, narration), c.palette.get(UiColor::TextDim));
    // A reply reads as the lead's line, after `> `.
    let reply = row_of(&buf, "> What does he want?");
    assert_eq!(reply, narration + 2 + 1);
    // One face per speaking row, full colour, at the left.
    assert_eq!(
        faces(&buf),
        [((FACE_X, 1), 255), ((FACE_X, reply - 1), 255)]
    );
}

#[test]
fn the_cursors_row_is_a_dark_bar_with_a_marker() {
    let c = ctx();
    let mut view = log(vec![
        row(RowKind::Speech, "Again."),
        row(RowKind::Speech, "Good."),
    ]);
    view.rows[1].focused = true;
    let buf = painted(&view);
    let y = row_of(&buf, "Good.") - 1;
    assert_eq!(buf.get(MARKER_X + 1, y).unwrap().glyph, '▶');
    let black = c.palette.get(UiColor::Black);
    for dy in 0..FACE.1 {
        assert_eq!(buf.get(TEXT_X + 40, y + dy).unwrap().bg, black, "row {dy}");
    }
    assert_eq!(
        buf.get(TEXT_X, y + 1).unwrap().fg,
        c.palette.get(UiColor::TextHighlight)
    );
    // The other row isn't.
    assert_ne!(buf.get(TEXT_X + 40, 1).unwrap().bg, black);
    assert_ne!(buf.get(MARKER_X + 1, 1).unwrap().glyph, '▶');
}

#[test]
fn lines_seen_past_the_one_on_screen_are_dimmer_with_back_to_latest_last() {
    let c = ctx();
    let mut view = log(vec![
        row(RowKind::Speech, "Again."),
        row(RowKind::Speech, "Good."),
    ]);
    view.rows[1].ahead = true;
    view.back_to_latest = Some(BackRow {
        label: "Back to latest".to_owned(),
        focused: true,
    });
    let buf = painted(&view);
    let good = row_of(&buf, "Good.");
    let dim = c.palette.get(UiColor::PanelBorder);
    assert_eq!(buf.get(TEXT_X, good).unwrap().fg, dim);
    assert_eq!(buf.get(TEXT_X, good - 1).unwrap().fg, dim, "its name too");
    let opacities: Vec<u8> = faces(&buf).iter().map(|&(_, o)| o).collect();
    assert_eq!(opacities, [255, dim_opacity(AHEAD_DIM)]);
    let back = row_of(&buf, "Back to latest");
    assert_eq!(back, good - 1 + FACE.1 + 1);
    assert_eq!(buf.get(MARKER_X + 1, back).unwrap().glyph, '▶');
}

#[test]
fn a_long_log_scrolls_to_keep_the_cursor_in_view() {
    let rows: Vec<LogRow> = (0..20)
        .map(|i| row(RowKind::Narration, &format!("line {i:02}")))
        .collect();
    // Two console rows each: 15 fit in the 30 under the border.
    let mut view = log(rows);
    view.rows[19].focused = true;
    let buf = painted(&view);
    assert_eq!(row_of(&buf, "line 19"), 29);
    assert_eq!(row_of(&buf, "line 05"), 1);
    assert!(!(0..32).any(|y| text(&buf, y).contains("line 04")));
    // At the top, the rows after it fill the screen.
    view.rows[19].focused = false;
    view.rows[0].focused = true;
    let buf = painted(&view);
    assert_eq!(row_of(&buf, "line 00"), 1);
    assert_eq!(row_of(&buf, "line 14"), 29);
    // In the middle, two rows after the cursor's show.
    view.rows[0].focused = false;
    view.rows[16].focused = true;
    let buf = painted(&view);
    assert_eq!(row_of(&buf, "line 18"), 29);
    assert_eq!(row_of(&buf, "line 16"), 25);
}

#[test]
fn long_text_wraps_under_the_name() {
    let long = "word ".repeat(40);
    let buf = painted(&log(vec![row(RowKind::Speech, long.trim())]));
    let width = |y| text(&buf, y).trim_end_matches([' ', '│']).chars().count();
    assert!(width(2) <= usize::from(ConsoleSpec::PC.cols) - 3);
    assert!(text(&buf, 3).contains("word"));
    assert_eq!(col(&buf, 3, "word"), Some(TEXT_X));
}

/// A narration row of `lines` text lines saying `tag` first.
fn tall(tag: &str, lines: usize) -> LogRow {
    // Too long to share a line with the tag: one line each.
    let word = "x".repeat(84);
    let rest = vec![word; lines.saturating_sub(1)].join(" ");
    let text = if rest.is_empty() {
        tag.to_owned()
    } else {
        format!("{tag} {rest}")
    };
    row(RowKind::Narration, &text)
}

/// Whether `word` is painted anywhere.
fn shows(buf: &GlyphBuffer, word: &str) -> bool {
    (0..i32::from(buf.height())).any(|y| text(buf, y).contains(word))
}

/// The bottom border is whole: nothing was drawn past the last row.
fn border_whole(buf: &GlyphBuffer) -> bool {
    let bottom = text(buf, i32::from(ConsoleSpec::PC.rows) - 1);
    !bottom.contains('x') && !bottom.contains("tag")
}

#[test]
fn rows_after_the_cursor_only_while_they_fit() {
    // Two 10-row lines after the cursor's: only one fits under it.
    let mut view = log(vec![tall("tagA", 10), tall("tagB", 10), tall("tagC", 10)]);
    view.rows[0].focused = true;
    let buf = painted(&view);
    assert!(shows(&buf, "tagA") && shows(&buf, "tagB"));
    assert!(!shows(&buf, "tagC"));
    assert!(border_whole(&buf));
}

#[test]
fn rows_that_fill_the_panel_exactly_all_show() {
    // 11 + 11 + 9 = 31 rows with their gaps: the last gap is the border.
    let mut rows: Vec<LogRow> = (0..4).map(|i| tall(&format!("tagE{i}"), 1)).collect();
    rows.extend([tall("tagF", 10), tall("tagG", 10), tall("tagH", 8)]);
    let mut view = log(rows);
    view.rows[4].focused = true;
    let buf = painted(&view);
    assert_eq!(row_of(&buf, "tagF"), 1, "no earlier row fits");
    assert!(!shows(&buf, "tagE3"));
    assert_eq!(row_of(&buf, "tagH"), 23);
    assert!(
        text(&buf, 30).contains('x'),
        "its last line is the panel's last"
    );
    assert!(border_whole(&buf));
    // From the top: one 2-row line and 14 one-row lines are 31 too.
    let mut rows = vec![LogRow {
        face: None,
        ..row(RowKind::Speech, "tagS")
    }];
    rows.extend((0..14).map(|i| tall(&format!("tagN{i:02}"), 1)));
    let mut view = log(rows);
    view.rows[0].focused = true;
    let buf = painted(&view);
    assert_eq!(row_of(&buf, "tagN13"), 30);
    assert!(border_whole(&buf));
}

#[test]
fn earlier_rows_only_while_they_fit() {
    // Before the cursor's 10-row line: a 1-row line, then a 19-row one
    // that doesn't fit with them.
    let mut view = log(vec![tall("tagA", 19), tall("tagB", 1), tall("tagC", 10)]);
    view.rows[2].focused = true;
    let buf = painted(&view);
    assert!(!shows(&buf, "tagA"));
    assert_eq!(row_of(&buf, "tagB"), 1);
    assert_eq!(row_of(&buf, "tagC"), 3);
    assert!(border_whole(&buf));
}

#[test]
fn nothing_is_drawn_past_the_last_row_that_fits() {
    let rows: Vec<LogRow> = (0..20).map(|i| tall(&format!("tagL{i:02}"), 1)).collect();
    let mut view = log(rows);
    view.rows[0].focused = true;
    let buf = painted(&view);
    assert_eq!(row_of(&buf, "tagL14"), 29);
    assert!(!shows(&buf, "tagL15"));
    assert!(border_whole(&buf));
}
