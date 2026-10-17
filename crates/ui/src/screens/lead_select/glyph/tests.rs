//! Tests of where the lead screen puts what its view says, on the screen
//! (`lead_select::tests`) and on a view made by hand.

use trpg_core::LeadGender;

use super::*;
use crate::color::Rgb;
use crate::console::{ConsoleSizes, ConsoleSpec};
use crate::input::Action::{CursorDown, CursorRight};
use crate::screen::tests::ctx;
use crate::screens::lead_select::LeadSelectScreen;
use crate::screens::lead_select::tests::{spelling, typing, update};

/// A buffer full of a colour the screen doesn't use, to see what is left
/// unpainted.
fn stale(c: &Ctx) -> GlyphBuffer {
    let enemy = c.palette.get(UiColor::Enemy);
    GlyphBuffer::with_spec(ConsoleSpec::PC, Cell::new('x', enemy, enemy))
}

/// `view` as painted.
fn painted(c: &Ctx, view: &LeadSelectView) -> GlyphBuffer {
    let mut buf = stale(c);
    paint(c, view, &mut buf);
    buf
}

/// Row `y` of `buf`.
fn row(buf: &GlyphBuffer, y: i32) -> String {
    (0..i32::from(buf.width()))
        .map(|x| buf.get(x, y).map_or(' ', |cell| cell.glyph))
        .collect()
}

/// The glyph at (`x`, `y`).
fn glyph(buf: &GlyphBuffer, x: i32, y: i32) -> char {
    buf.get(x, y).expect("on the console").glyph
}

/// The colours of the cell at (`x`, `y`): text, then background.
fn colours(buf: &GlyphBuffer, x: i32, y: i32) -> (Rgb, Rgb) {
    let cell = buf.get(x, y).expect("on the console");
    (cell.fg, cell.bg)
}

/// The column where `text` first starts on row `y` of `buf`.
fn column(buf: &GlyphBuffer, y: i32, text: &str) -> i32 {
    let line = row(buf, y);
    let at = line.find(text).expect("the text is on the row");
    i32::try_from(line[..at].chars().count()).unwrap_or(0)
}

#[test]
fn paints_every_cell() {
    let c = ctx();
    let buf = painted(&c, &LeadSelectScreen::new().view(&c));
    let enemy = c.palette.get(UiColor::Enemy);
    for y in 0..i32::from(buf.height()) {
        for x in 0..i32::from(buf.width()) {
            assert_ne!(colours(&buf, x, y).1, enemy, "({x}, {y}) unpainted");
        }
    }
}

#[test]
fn places_the_heading_frames_labels_name_and_start() {
    let c = ctx();
    let p = |u| c.palette.get(u);
    let buf = painted(&c, &LeadSelectScreen::new().view(&c));
    assert_eq!(row(&buf, HEADING_ROW).trim(), "Choose your lead");
    // The frames side by side; the chosen one's border is lit, and double
    // while the cursor is on the portraits.
    assert_eq!(glyph(&buf, FRAME_X[0], FRAME_Y), '╔');
    assert_eq!(glyph(&buf, FRAME_X[1], FRAME_Y), '┌');
    let bottom_right = (FRAME_X[1] + FRAME.0 - 1, FRAME_Y + FRAME.1 - 1);
    assert_eq!(glyph(&buf, bottom_right.0, bottom_right.1), '┘');
    let focus = p(UiColor::PanelBorderFocus);
    assert_eq!(colours(&buf, FRAME_X[0], FRAME_Y).0, focus);
    let border = p(UiColor::PanelBorder);
    assert_eq!(colours(&buf, FRAME_X[1], FRAME_Y).0, border);
    // The labels centred under their frames, the chosen one lit.
    let male = column(&buf, LABEL_ROW, "Male");
    let female = column(&buf, LABEL_ROW, "Female");
    assert_eq!(male, FRAME_X[0] + (FRAME.0 - 4) / 2);
    assert_eq!(female, FRAME_X[1] + (FRAME.0 - 6) / 2);
    assert_eq!(colours(&buf, male, LABEL_ROW).0, p(UiColor::TextHighlight));
    assert_eq!(colours(&buf, female, LABEL_ROW).0, p(UiColor::TextDim));
    // The name and Start centred, neither lit; the label and the family
    // name dim.
    assert_eq!(row(&buf, NAME_ROW).trim(), "Name   Ellery Veyne");
    assert_eq!(row(&buf, START_ROW).trim(), "[ Start ]");
    let dim = p(UiColor::TextDim);
    assert_eq!(
        colours(&buf, column(&buf, NAME_ROW, "Name"), NAME_ROW).0,
        dim
    );
    let first = column(&buf, NAME_ROW, "Ellery");
    assert_eq!(colours(&buf, first, NAME_ROW).0, p(UiColor::Text));
    let family = column(&buf, NAME_ROW, "Veyne");
    assert_eq!(colours(&buf, family, NAME_ROW).0, dim);
    let start = column(&buf, START_ROW, "[");
    assert_eq!(colours(&buf, start, START_ROW).0, p(UiColor::Text));
    let bottom = i32::from(buf.height()) - 1;
    assert_eq!(row(&buf, bottom).trim(), "arrows choose · f next · d back");
}

#[test]
fn the_cursor_lights_its_row() {
    let mut c = ctx();
    let mut s = LeadSelectScreen::new();
    update(&mut s, &mut c, &[CursorRight, CursorDown]);
    let buf = painted(&c, &s.view(&c));
    let highlight = c.palette.get(UiColor::TextHighlight);
    // Off the portraits, the chosen frame (now the female one) is single
    // but still lit.
    assert_eq!(glyph(&buf, FRAME_X[0], FRAME_Y), '┌');
    assert_eq!(glyph(&buf, FRAME_X[1], FRAME_Y), '┌');
    let focus = c.palette.get(UiColor::PanelBorderFocus);
    assert_eq!(colours(&buf, FRAME_X[1], FRAME_Y).0, focus);
    let first = column(&buf, NAME_ROW, "Ellery");
    assert_eq!(colours(&buf, first, NAME_ROW).0, highlight);
    update(&mut s, &mut c, &[CursorDown]);
    let buf = painted(&c, &s.view(&c));
    let start = column(&buf, START_ROW, "[");
    assert_eq!(colours(&buf, start, START_ROW).0, highlight);
    assert_ne!(colours(&buf, first, NAME_ROW).0, highlight);
}

#[test]
fn the_typing_box_shows_the_name_with_a_cursor_and_the_hint() {
    let mut c = ctx();
    let s = typing(&mut c);
    let buf = painted(&c, &s.view(&c));
    let p = |u| c.palette.get(u);
    assert_eq!(glyph(&buf, TYPE_BOX.x, TYPE_BOX.y), '╔');
    let title = column(&buf, TYPE_BOX.y, "First name");
    assert_eq!(title, TYPE_BOX.x + 3);
    assert_eq!(
        colours(&buf, title, TYPE_BOX.y).0,
        p(UiColor::TextHighlight)
    );
    let shown = column(&buf, TYPE_BOX.y + 2, "Ellery_");
    assert_eq!(shown, TYPE_BOX.x + (TYPE_BOX.w - 7) / 2);
    let hint = "Type a name on your keyboard.";
    let hint_x = column(&buf, TYPE_BOX.y + 4, hint);
    assert_eq!(hint_x, TYPE_BOX.x + (TYPE_BOX.w - 29) / 2);
    assert_eq!(colours(&buf, hint_x, TYPE_BOX.y + 4).0, p(UiColor::TextDim));
    // Nothing behind the box is lit.
    let first = column(&buf, NAME_ROW, "Ellery");
    assert_eq!(colours(&buf, first, NAME_ROW).0, p(UiColor::Text));
}

#[test]
fn the_letter_grid_lights_its_focused_cell() {
    let mut c = ctx();
    let mut s = spelling(&mut c);
    update(&mut s, &mut c, &[CursorRight]);
    let buf = painted(&c, &s.view(&c));
    let p = |u| c.palette.get(u);
    assert_eq!(glyph(&buf, GRID_BOX.x, GRID_BOX.y), '╔');
    assert_eq!(column(&buf, GRID_BOX.y, "First name"), GRID_BOX.x + 3);
    let shown = column(&buf, GRID_BOX.y + 2, "Ellery_");
    assert_eq!(shown, GRID_BOX.x + (GRID_BOX.w - 7) / 2);
    // 13 letters, each with a space either side, centred in the box; the
    // rows two apart.
    let y = GRID_BOX.y + 4;
    assert_eq!(
        column(&buf, y, " A  B "),
        GRID_BOX.x + (GRID_BOX.w - 39) / 2
    );
    let b_x = column(&buf, y, "B");
    let (bar, panel) = (p(UiColor::PanelBorderFocus), p(UiColor::PanelBg));
    assert_eq!(colours(&buf, b_x - 1, y), (panel, bar));
    assert_eq!(colours(&buf, b_x, y), (panel, bar));
    assert_eq!(colours(&buf, b_x + 1, y), (panel, bar));
    let a_x = column(&buf, y, "A");
    assert_eq!(colours(&buf, a_x, y), (p(UiColor::Text), panel));
    assert!(row(&buf, y + 2).contains(" N  O  P "));
    assert!(row(&buf, y + 8).contains(" -  '  Blank  Delete  Done "));
}

#[test]
fn a_view_made_by_hand() {
    let c = ctx();
    let gender = |gender, label: &str, chosen| GenderView {
        gender,
        label: label.to_owned(),
        portrait: "none",
        expression: "neutral",
        chosen,
    };
    let view = LeadSelectView {
        heading: "Who?".to_owned(),
        genders: vec![
            gender(LeadGender::Male, "He", false),
            gender(LeadGender::Female, "She", true),
        ],
        cursor: Some(Row::Gender),
        name_row: NameRowView {
            label: "N".to_owned(),
            first: "Jo".to_owned(),
            family: "Doe".to_owned(),
        },
        start: "Go".to_owned(),
        typing: None,
        grid: None,
        help: "keys".to_owned(),
    };
    let buf = painted(&c, &view);
    assert_eq!(row(&buf, HEADING_ROW).trim(), "Who?");
    assert_eq!(glyph(&buf, FRAME_X[0], FRAME_Y), '┌');
    assert_eq!(glyph(&buf, FRAME_X[1], FRAME_Y), '╔');
    assert_eq!(
        column(&buf, LABEL_ROW, "He"),
        FRAME_X[0] + (FRAME.0 - 2) / 2
    );
    assert_eq!(
        column(&buf, LABEL_ROW, "She"),
        FRAME_X[1] + (FRAME.0 - 3) / 2
    );
    assert_eq!(row(&buf, NAME_ROW).trim(), "N   Jo Doe");
    assert_eq!(row(&buf, START_ROW).trim(), "[ Go ]");
    let bottom = i32::from(buf.height()) - 1;
    assert_eq!(row(&buf, bottom).trim(), "keys");
}
