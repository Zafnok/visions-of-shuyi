//! The glyph look of the Options screen: paints an [`OptionsView`] as a
//! panel of text rows, in the style of the Key bindings screen. Everything
//! about the look lives here (where things go, the box characters, the
//! slider's blocks, the colours); nothing here decides what the screen
//! shows or does. Another look is another `paint` of the same view.

use super::view::{OptionsView, QuestionView, RowView, ValueView};
use crate::color::UiColor;
use crate::glyph_buffer::{BoxStyle, Cell, GlyphBuffer, Rect};
use crate::screen::Ctx;
use crate::screens::print_centred;

/// The panel, in cells.
const PANEL: Rect = Rect::new(20, 3, 60, 20);
/// Column of the row labels.
const LABEL_X: i32 = PANEL.x + 4;
/// Column of the row values.
const VALUE_X: i32 = PANEL.x + 30;
/// Row of the message under the panel.
const MESSAGE_ROW: i32 = PANEL.y + PANEL.h + 1;
/// Height of a question's box: its two lines, a blank row above and below,
/// and the border.
const QUESTION_H: i32 = 6;
/// Cells in a volume's bar.
const VOLUME_BAR: u8 = 10;
/// Cells of the number box, inside its highlight.
const BOX_W: usize = 4;

/// A volume as a bar of [`VOLUME_BAR`] cells (a half-filled cell for half
/// a cell's worth or more) and its number.
fn volume_text(level: u8, max: u8) -> String {
    let level = level.min(max);
    let per_cell = (max / VOLUME_BAR).max(1);
    let full = usize::from(level / per_cell).min(usize::from(VOLUME_BAR));
    let half = usize::from(level % per_cell >= per_cell.div_ceil(2) && per_cell > 1);
    let empty = usize::from(VOLUME_BAR).saturating_sub(full + half);
    let bar = ["█".repeat(full), "▒".repeat(half), "░".repeat(empty)].concat();
    format!("{bar} {level:>3}")
}

/// Paints `view` over the whole of `buf`.
pub fn paint(ctx: &Ctx, view: &OptionsView, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let (black, bg) = (c(UiColor::Black), c(UiColor::PanelBg));
    let (text, dim) = (c(UiColor::Text), c(UiColor::TextDim));
    buf.fill_rect(buf.bounds(), Cell::new(' ', text, black));
    buf.fill_rect(PANEL, Cell::new(' ', text, bg));
    buf.draw_box(PANEL, BoxStyle::Single, c(UiColor::PanelBorder), bg);
    let title = format!(" {} ", view.title);
    buf.print(PANEL.x + 2, PANEL.y, &title, c(UiColor::TextHighlight), bg);

    let mut y = PANEL.y + 2;
    for row in &view.rows {
        if row.starts_group {
            y += 1;
        }
        paint_row(ctx, buf, row, y);
        y += 1;
    }

    if let Some(message) = &view.message {
        print_centred(buf, MESSAGE_ROW, message, c(UiColor::TextHighlight), black);
    }
    if let Some(question) = &view.question {
        paint_question(ctx, buf, question);
    }
    let bottom = i32::from(buf.height()) - 1;
    print_centred(buf, bottom, &view.help, dim, black);
}

/// Paints `row` on console row `y`: the focused row's label is a highlight
/// bar; a value the cursor changes sits between two arrows; the number box
/// is a highlight bar where the value was.
fn paint_row(ctx: &Ctx, buf: &mut GlyphBuffer, row: &RowView, y: i32) {
    let c = |u| ctx.palette.get(u);
    let (bg, text, dim) = (c(UiColor::PanelBg), c(UiColor::Text), c(UiColor::TextDim));
    let bar = c(UiColor::PanelBorderFocus);
    if row.focused {
        buf.print(LABEL_X - 1, y, &format!(" {} ", row.label), bg, bar);
    } else {
        buf.print(LABEL_X, y, &row.label, text, bg);
    }
    let (value, adjustable) = match &row.value {
        ValueView::None => return,
        ValueView::NumberBox { digits, typing } => {
            let caret = if *typing { "_" } else { "" };
            let shown = format!(" {:<BOX_W$} ", format!("{digits}{caret}"));
            buf.print(VALUE_X - 1, y, &shown, bg, bar);
            return;
        }
        ValueView::Text { text, adjustable } => (text.clone(), *adjustable),
        ValueView::Volume { level, max } => (volume_text(*level, *max), true),
    };
    let value_fg = if row.focused {
        c(UiColor::TextHighlight)
    } else {
        text
    };
    buf.print(VALUE_X, y, &value, value_fg, bg);
    if row.focused && adjustable {
        buf.print(VALUE_X - 2, y, "◄", dim, bg);
        let after = VALUE_X + i32::try_from(value.chars().count()).unwrap_or(0) + 1;
        buf.print(after, y, "►", dim, bg);
    }
}

/// Paints `question` in a double-bordered box in the middle of the screen,
/// with its answers under it (the look of the Key bindings screen's
/// question).
fn paint_question(ctx: &Ctx, buf: &mut GlyphBuffer, question: &QuestionView) {
    let c = |u| ctx.palette.get(u);
    let bg = c(UiColor::PanelBg);
    let widest = question
        .text
        .chars()
        .count()
        .max(question.answers.chars().count());
    let w = i32::try_from(widest).unwrap_or(0) + 4;
    let rect = Rect::new(
        (i32::from(buf.width()) - w) / 2,
        (i32::from(buf.height()) - QUESTION_H) / 2,
        w,
        QUESTION_H,
    );
    buf.fill_rect(rect, Cell::new(' ', c(UiColor::Text), bg));
    buf.draw_box(rect, BoxStyle::Double, c(UiColor::PanelBorderFocus), bg);
    buf.print(rect.x + 2, rect.y + 2, &question.text, c(UiColor::Text), bg);
    let dim = c(UiColor::TextDim);
    buf.print(rect.x + 2, rect.y + 3, &question.answers, dim, bg);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::{CONSOLE_H, CONSOLE_W};
    use crate::screen::tests::ctx;
    use crate::screens::options::Row;

    /// Row `y` of `view` as painted, trimmed.
    fn painted_row(c: &Ctx, view: &OptionsView, y: i32) -> String {
        let black = c.palette.get(UiColor::Black);
        let mut buf = GlyphBuffer::new(CONSOLE_W, CONSOLE_H, Cell::new(' ', black, black));
        paint(c, view, &mut buf);
        let glyphs = (0..i32::from(CONSOLE_W)).map(|x| buf.get(x, y).map_or(' ', |c| c.glyph));
        glyphs.collect::<String>().trim().to_owned()
    }

    fn row(label: &str, value: ValueView, focused: bool) -> RowView {
        RowView {
            row: Row::TextSpeed,
            label: label.to_owned(),
            value,
            focused,
            starts_group: false,
        }
    }

    fn view(rows: Vec<RowView>) -> OptionsView {
        OptionsView {
            title: "Options".to_owned(),
            rows,
            message: None,
            question: None,
            help: "help".to_owned(),
        }
    }

    fn words(text: &str, adjustable: bool) -> ValueView {
        ValueView::Text {
            text: text.to_owned(),
            adjustable,
        }
    }

    #[test]
    fn a_volume_is_a_bar_of_ten_cells_and_its_number() {
        assert_eq!(volume_text(80, 100), "████████░░  80");
        assert_eq!(volume_text(0, 100), "░░░░░░░░░░   0");
        assert_eq!(volume_text(100, 100), "██████████ 100");
        // A half cell for the odd five; under it rounds down.
        assert_eq!(volume_text(85, 100), "████████▒░  85");
        assert_eq!(volume_text(84, 100), "████████░░  84");
        assert_eq!(volume_text(5, 100), "▒░░░░░░░░░   5");
        assert_eq!(volume_text(99, 100), "█████████▒  99");
        // Past the loudest is the loudest; a small scale has no halves.
        assert_eq!(volume_text(200, 100), "██████████ 100");
        assert_eq!(volume_text(7, 10), "███████░░░   7");
        assert_eq!(volume_text(3, 5), "███░░░░░░░   3");
    }

    #[test]
    fn rows_are_painted_from_the_second_row_of_the_panel_with_gaps_before_groups() {
        let c = ctx();
        let mut grouped = row("Layout", words("Right-handed", false), false);
        grouped.starts_group = true;
        let rows = vec![
            row("Text speed", words("Normal", true), true),
            row(
                "Music volume",
                ValueView::Volume {
                    level: 65,
                    max: 100,
                },
                false,
            ),
            grouped,
            row("Key bindings", ValueView::None, false),
        ];
        let v = view(rows);
        assert!(painted_row(&c, &v, PANEL.y).starts_with("┌─ Options ─"));
        assert_eq!(
            painted_row(&c, &v, PANEL.y + 1),
            "│                                                          │"
        );
        // The focused row: arrows round a value the cursor changes.
        let first = painted_row(&c, &v, PANEL.y + 2);
        assert_eq!(
            first,
            format!("│   {:<24}◄ Normal ►{:>22}", "Text speed", "│")
        );
        let second = painted_row(&c, &v, PANEL.y + 3);
        assert!(
            second.contains("Music volume              ██████▒░░░  65"),
            "{second}"
        );
        assert!(!second.contains('◄'));
        // A blank row before the group.
        assert_eq!(
            painted_row(&c, &v, PANEL.y + 4),
            painted_row(&c, &v, PANEL.y + 1)
        );
        assert!(
            painted_row(&c, &v, PANEL.y + 5).contains("Layout                    Right-handed")
        );
        assert!(painted_row(&c, &v, PANEL.y + 6).contains("Key bindings   "));
        assert_eq!(painted_row(&c, &v, 31), "help");
    }

    #[test]
    fn a_focused_value_the_cursor_cant_change_has_no_arrows() {
        let c = ctx();
        let v = view(vec![row("Layout", words("Right-handed", false), true)]);
        let painted = painted_row(&c, &v, PANEL.y + 2);
        assert!(
            painted.contains("Right-handed") && !painted.contains('◄') && !painted.contains('►')
        );
        let v = view(vec![row(
            "Music volume",
            ValueView::Volume {
                level: 80,
                max: 100,
            },
            true,
        )]);
        assert!(painted_row(&c, &v, PANEL.y + 2).contains("◄ ████████░░  80 ►"));
    }

    #[test]
    fn the_number_box_shows_a_caret_only_while_typing() {
        let c = ctx();
        let boxed = |digits: &str, typing| {
            let value = ValueView::NumberBox {
                digits: digits.to_owned(),
                typing,
            };
            painted_row(
                &c,
                &view(vec![row("Sound volume", value, true)]),
                PANEL.y + 2,
            )
        };
        assert!(boxed("37", true).contains(" 37_ "), "{}", boxed("37", true));
        assert!(boxed("", true).contains("  _ "));
        assert!(boxed("80", false).contains(" 80 ") && !boxed("80", false).contains('_'));
        // The box's highlight is as wide whatever is in it.
        let bar = c.palette.get(UiColor::PanelBorderFocus);
        let width = |digits: &str| {
            let value = ValueView::NumberBox {
                digits: digits.to_owned(),
                typing: true,
            };
            let black = c.palette.get(UiColor::Black);
            let mut buf = GlyphBuffer::new(CONSOLE_W, CONSOLE_H, Cell::new(' ', black, black));
            paint(&c, &view(vec![row("Sound volume", value, true)]), &mut buf);
            (VALUE_X - 1..i32::from(CONSOLE_W))
                .take_while(|&x| buf.get(x, PANEL.y + 2).is_some_and(|cell| cell.bg == bar))
                .count()
        };
        assert_eq!((width(""), width("100")), (BOX_W + 2, BOX_W + 2));
    }

    /// The message sits one blank row under the panel's bottom border.
    #[test]
    fn the_message_is_painted_under_the_panel() {
        let c = ctx();
        let mut v = view(vec![]);
        v.message = Some("Tips will show again".to_owned());
        let bottom = PANEL.y + PANEL.h - 1;
        assert!(painted_row(&c, &v, bottom).starts_with('└'));
        assert_eq!(painted_row(&c, &v, bottom + 1), "");
        assert_eq!(painted_row(&c, &v, bottom + 2), "Tips will show again");
        assert_eq!(painted_row(&c, &v, bottom + 3), "");
    }

    #[test]
    fn a_question_is_a_box_in_the_middle_with_its_answers_under_it() {
        let c = ctx();
        let mut v = view(vec![row("Reset tips", ValueView::None, true)]);
        v.question = Some(QuestionView {
            text: "Show every tip again?".to_owned(),
            answers: "f yes / d no".to_owned(),
        });
        let top = (i32::from(CONSOLE_H) - QUESTION_H) / 2;
        assert!(painted_row(&c, &v, top).contains("╔═══════════════════════╗"));
        assert!(painted_row(&c, &v, top + 2).contains("║ Show every tip again? ║"));
        assert!(painted_row(&c, &v, top + 3).contains("║ f yes / d no          ║"));
        assert!(painted_row(&c, &v, top + 5).contains("╚═══════════════════════╝"));
    }
}
