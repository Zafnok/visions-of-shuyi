//! Where the rewind screen is painted. What it says is tested on its view
//! (`battle/rewind/tests.rs`, `battle/rewind_tests.rs`).

use super::*;
use crate::console::{ConsoleSizes, ConsoleSpec};
use crate::screen::tests::ctx;

#[test]
fn wrap_keeps_lines_within_the_panel_and_indents_the_rest() {
    let lines = wrap("Turn 12 · Test Lord attacked Brigand (2 hits, 14 dmg, defeated)");
    assert_eq!(
        lines,
        [
            "Turn 12 · Test Lord",
            "  attacked Brigand (2",
            "  hits, 14 dmg, defeated)"
        ]
    );
    assert!(lines.iter().all(|l| l.chars().count() <= TEXT_W));
    assert_eq!(wrap("short"), ["short"]);
    // Exactly the panel's width fits on one line.
    let full = format!("{} {}", "a".repeat(12), "b".repeat(TEXT_W - 13));
    assert_eq!(wrap(&full), std::slice::from_ref(&full));
    // A word longer than the panel isn't preceded by an empty line.
    let long = "c".repeat(TEXT_W + 4);
    assert_eq!(wrap(&long), std::slice::from_ref(&long));
}

#[test]
fn first_shown_scrolls_just_enough_to_show_the_focus() {
    let b = |n| vec![String::new(); n];
    let blocks = [b(2), b(2), b(3), b(1)];
    assert_eq!(first_shown(&blocks, 0, 4), 0);
    assert_eq!(first_shown(&blocks, 1, 4), 0);
    assert_eq!(first_shown(&blocks, 2, 4), 2);
    assert_eq!(first_shown(&blocks, 2, 5), 1);
    assert_eq!(first_shown(&blocks, 3, 6), 1);
    assert_eq!(first_shown(&blocks, 3, 4), 2);
    // A block taller than the rows: shown from itself.
    assert_eq!(first_shown(&[b(5)], 0, 4), 0);
    assert_eq!(first_shown(&blocks, 2, 2), 2);
}

#[test]
fn panel_rows() {
    assert_eq!(
        (TITLE_ROW, CHARGES_ROW, LIST_ROW, PROMPT_ROW),
        (1, 2, 4, 27)
    );
}

/// A list of `lines` entries `e0`, `e1`… focused on `focus`.
fn view(lines: usize, focus: usize) -> RewindView {
    RewindView {
        title: "Rewind".into(),
        charges: "3 of 3 charges left".into(),
        spent: false,
        entries: (0..lines).map(|i| format!("e{i}")).collect(),
        focus,
        nothing: None,
        prompt: None,
    }
}

/// `view` painted.
fn painted(view: &RewindView) -> GlyphBuffer {
    let c = ctx();
    let black = c.palette.get(UiColor::Black);
    let mut buf = GlyphBuffer::with_spec(ConsoleSpec::PC, Cell::new(' ', black, black));
    paint_list(&c, view, &mut buf);
    buf
}

/// Row `y` of the panel of `buf`, inside its border, trimmed.
fn row(buf: &GlyphBuffer, y: i32) -> String {
    (TEXT_X..SIDE_PANEL.x + SIDE_PANEL.w - 1)
        .map(|x| buf.get(x, y).map_or(' ', |c| c.glyph))
        .collect::<String>()
        .trim()
        .to_owned()
}

#[test]
fn a_long_list_fills_the_rows_above_the_prompt_and_scrolls_to_the_focus() {
    // 22 rows from LIST_ROW to the blank row above the prompt.
    let buf = painted(&view(30, 0));
    assert_eq!(row(&buf, LIST_ROW), "e0");
    assert_eq!(row(&buf, PROMPT_ROW - 2), "e21");
    assert_eq!(row(&buf, PROMPT_ROW - 1), "");
    assert_eq!(row(&buf, PROMPT_ROW), "");
    let buf = painted(&view(30, 29));
    assert_eq!(row(&buf, LIST_ROW), "e8");
    assert_eq!(row(&buf, PROMPT_ROW - 2), "e29");
    assert_eq!(row(&buf, PROMPT_ROW - 1), "");
}

#[test]
fn the_title_charges_focus_and_prompt_go_in_their_rows() {
    let c = ctx();
    let p = &c.palette;
    let mut v = view(3, 1);
    v.prompt = Some("Use 1 of 3 charges?".into());
    let buf = painted(&v);
    assert_eq!(row(&buf, TITLE_ROW), "Rewind");
    assert_eq!(row(&buf, CHARGES_ROW), "3 of 3 charges left");
    assert_eq!(row(&buf, PROMPT_ROW), "Use 1 of 3 charges?");
    let fg = |y| buf.get(TEXT_X, y).unwrap().fg;
    assert_eq!(fg(CHARGES_ROW), p.get(UiColor::Text));
    assert_eq!(fg(PROMPT_ROW), p.get(UiColor::TextHighlight));
    // The highlighted entry inverted, across the panel.
    let focused = buf.get(TEXT_X + 10, LIST_ROW + 1).unwrap();
    assert_eq!(focused.bg, p.get(UiColor::PanelBorderFocus));
    let other = buf.get(TEXT_X, LIST_ROW).unwrap();
    assert_eq!(other.bg, p.get(UiColor::PanelBg));
    // No charge left: in the warning colour. Nothing to go back to: said
    // where the list goes, and no list.
    v.spent = true;
    v.entries.clear();
    v.nothing = Some("Nothing to rewind yet".into());
    let buf = painted(&v);
    assert_eq!(
        buf.get(TEXT_X, CHARGES_ROW).unwrap().fg,
        p.get(UiColor::HpLow)
    );
    assert_eq!(row(&buf, LIST_ROW), "Nothing to rewind yet");
    assert_eq!(row(&buf, LIST_ROW + 1), "");
}
