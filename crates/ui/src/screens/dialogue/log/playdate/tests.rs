//! Tests of the dialogue log's Playdate look: the window, a row's name and
//! text, the cursor's bar, lines seen ahead and the scrolling.

use super::*;
use crate::screens::dialogue::log::{BackRow, Face, LogRow};
use crate::screens::playdate::tests::{blank, c, cell, find, is_bar, pd_ctx, row};

/// A row of `kind` saying `text`, by Ellery (with the lead's face) unless
/// narration.
fn line(kind: RowKind, text: &str) -> LogRow {
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
        help: "B close".to_owned(),
    }
}

fn painted(view: &LogView) -> GlyphBuffer {
    let mut buf = blank();
    paint(&pd_ctx(), view, &mut buf);
    buf
}

#[test]
fn a_window_of_names_and_their_lines_with_no_faces() {
    let mut speech = line(RowKind::Speech, &"word ".repeat(12));
    speech.focused = true;
    let view = log(vec![
        line(RowKind::Narration, "Dawn."),
        speech,
        line(RowKind::Reply, "Yes."),
    ]);
    let buf = painted(&view);
    assert!(row(&buf, 0).starts_with("╔═ Log ═"), "{}", row(&buf, 0));
    assert!(row(&buf, 14).contains(" B close "), "{}", row(&buf, 14));
    // Narration alone; a line's speaker, then its text a little in,
    // wrapped inside the window; a reply marked; a blank row between.
    assert_eq!(find(&buf, 1, "Dawn."), Some(2));
    assert_eq!(find(&buf, 3, "Ellery"), Some(2));
    assert_eq!(find(&buf, 4, "word"), Some(4));
    assert_eq!(find(&buf, 5, "word"), Some(4));
    assert_eq!(find(&buf, 7, "Ellery"), Some(2));
    assert_eq!(find(&buf, 8, "> Yes."), Some(4));
    assert_eq!(buf.sprites().len(), 0);
    // The cursor's row is the bar across the window, every line of it.
    assert!(is_bar(&buf, 3, "Ellery"));
    assert!(is_bar(&buf, 5, "word"));
    assert_eq!(cell(&buf, 1, 4).bg, c("text"));
    assert_eq!(cell(&buf, 48, 4).bg, c("text"));
    assert_eq!(cell(&buf, 49, 4).glyph, '║');
    assert!(!is_bar(&buf, 1, "Dawn."));
}

#[test]
fn lines_seen_ahead_are_grey_and_back_to_latest_comes_last() {
    let mut ahead = line(RowKind::Narration, "Later.");
    ahead.ahead = true;
    let mut view = log(vec![line(RowKind::Narration, "Now."), ahead]);
    view.rows[0].focused = true;
    view.back_to_latest = Some(BackRow {
        label: "Back to latest".to_owned(),
        focused: false,
    });
    let buf = painted(&view);
    assert_eq!(cell(&buf, 2, 3).fg, c("window_text_off"));
    assert_eq!(cell(&buf, 2, 1).fg, c("panel_bg"));
    assert_eq!(find(&buf, 5, "Back to latest"), Some(2));
    view.rows[0].focused = false;
    view.back_to_latest = Some(BackRow {
        label: "Back to latest".to_owned(),
        focused: true,
    });
    assert!(is_bar(&painted(&view), 5, "Back to latest"));
}

#[test]
fn a_long_log_scrolls_with_the_cursor_and_says_there_is_more() {
    let rows = (0..20).map(|i| line(RowKind::Narration, &format!("Line {i}.")));
    let mut view = log(rows.collect());
    view.rows[19].focused = true;
    let buf = painted(&view);
    // The last lines, ending with the cursor's, and a ▲ above.
    assert!(find(&buf, 13, "Line 19.").is_some());
    assert!(find(&buf, 1, "Line 13.").is_some());
    assert_eq!(cell(&buf, 47, 0).glyph, '▲');
    assert_ne!(cell(&buf, 47, 14).glyph, '▼');
    view.rows[19].focused = false;
    view.rows[0].focused = true;
    let buf = painted(&view);
    assert!(find(&buf, 1, "Line 0.").is_some());
    assert_eq!(cell(&buf, 47, 14).glyph, '▼');
    assert_ne!(cell(&buf, 47, 0).glyph, '▲');
}

#[test]
fn a_speakers_text_wraps_to_end_two_cells_inside_the_frame() {
    // 45 letters after the 2-cell indent: one too many for a line.
    let text = format!("{} yy", "x".repeat(42));
    let buf = painted(&log(vec![line(RowKind::Speech, &text)]));
    assert_eq!(find(&buf, 2, "xxx"), Some(4));
    assert_eq!(find(&buf, 3, "yy"), Some(4));
}
