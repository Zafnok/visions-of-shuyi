//! The glyph look of the Profile screen: a panel with the profiles, the
//! one in use marked, then the row that adds one (in the style of the
//! Options screen); the menu of what can be done to a profile, the
//! questions and the name boxes in double-bordered boxes over it.
//! Everything about the look lives here; nothing here decides what the
//! screen shows. Another look is another `paint` of the same view
//! (ADR-0054).

use super::view::{ActionsView, ProfileView, QuestionView, RowView};
use crate::color::UiColor;
use crate::glyph_buffer::{BoxStyle, Cell, GlyphBuffer, Rect};
use crate::prelude::*;
use crate::screen::Ctx;
use crate::screens::print_centred;
use crate::widgets::name_entry::glyph::{paint_grid, paint_typing};

/// The panel's top left and width, in cells; it is as tall as its rows
/// ([`panel`]).
const PANEL: Rect = Rect::new(30, 5, 40, 0);
/// Column of the row labels.
const LABEL_X: i32 = PANEL.x + 4;
/// Column of the mark of the profile in use.
const MARK_X: i32 = PANEL.x + 24;
/// The typing box.
const TYPE_BOX: Rect = Rect::new(27, 10, 46, 7);
/// The name grid's box (its height is the grid's).
const GRID_BOX: Rect = Rect::new(27, 5, 46, 0);
/// Height of a question's box: its two lines, a blank row above and below,
/// and the border.
const QUESTION_H: i32 = 6;
/// Cells between two answers of a question.
const CHOICE_GAP: usize = 3;

/// Cells `text` takes, as a coordinate.
fn cells(text: &str) -> i32 {
    i32::try_from(text.chars().count()).unwrap_or(0)
}

/// The panel for `rows` rows: the border, a blank row above and below
/// them, and one between the profiles and the row that adds one.
fn panel(rows: usize) -> Rect {
    let rows = i32::try_from(rows).unwrap_or(0);
    Rect::new(PANEL.x, PANEL.y, PANEL.w, rows + 5)
}

/// Paints `view` over the whole of `buf`.
pub fn paint(ctx: &Ctx, view: &ProfileView, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let (black, bg) = (c(UiColor::Black), c(UiColor::PanelBg));
    let (text, dim) = (c(UiColor::Text), c(UiColor::TextDim));
    let panel = panel(view.rows.len());
    buf.fill_rect(buf.bounds(), Cell::new(' ', text, black));
    buf.fill_rect(panel, Cell::new(' ', text, bg));
    buf.draw_box(panel, BoxStyle::Single, c(UiColor::PanelBorder), bg);
    let title = format!(" {} ", view.title);
    buf.print(panel.x + 2, panel.y, &title, c(UiColor::TextHighlight), bg);

    let mut y = PANEL.y + 2;
    for row in &view.rows {
        if row.kind == super::RowKind::Add {
            // A blank row between the profiles and the row that adds one.
            y += 1;
        }
        paint_row(ctx, buf, row, &view.current_mark, y);
        y += 1;
    }

    if let Some(message) = &view.message {
        // One blank row under the panel.
        let y = panel.y + panel.h + 1;
        print_centred(buf, y, message, c(UiColor::TextHighlight), black);
    }
    if let Some(actions) = &view.actions {
        paint_actions(ctx, buf, actions);
    }
    if let Some(grid) = &view.grid {
        paint_grid(ctx, buf, GRID_BOX, grid);
    }
    if let Some(typing) = &view.typing {
        paint_typing(ctx, buf, TYPE_BOX, typing);
    }
    if let Some(question) = &view.question {
        paint_question(ctx, buf, question);
    }
    let bottom = i32::from(buf.height()) - 1;
    print_centred(buf, bottom, &view.help, dim, black);
}

/// Paints `row` on console row `y`: the focused row's label is a highlight
/// bar, the profile in use has `mark` beside it, a row that does nothing
/// is dim.
fn paint_row(ctx: &Ctx, buf: &mut GlyphBuffer, row: &RowView, mark: &str, y: i32) {
    let c = |u| ctx.palette.get(u);
    let bg = c(UiColor::PanelBg);
    let fg = if row.enabled {
        c(UiColor::Text)
    } else {
        c(UiColor::TextDim)
    };
    if row.focused {
        let bar = c(UiColor::PanelBorderFocus);
        buf.print(LABEL_X - 1, y, &format!(" {} ", row.label), bg, bar);
    } else {
        buf.print(LABEL_X, y, &row.label, fg, bg);
    }
    if row.current {
        buf.print(MARK_X, y, mark, c(UiColor::TextHighlight), bg);
    }
}

/// A double-bordered box `w` wide and `h` tall in the middle of the
/// screen.
fn middle_box(ctx: &Ctx, buf: &mut GlyphBuffer, w: i32, h: i32) -> Rect {
    let c = |u| ctx.palette.get(u);
    let bg = c(UiColor::PanelBg);
    let rect = Rect::new(
        (i32::from(buf.width()) - w) / 2,
        (i32::from(buf.height()) - h) / 2,
        w,
        h,
    );
    buf.fill_rect(rect, Cell::new(' ', c(UiColor::Text), bg));
    buf.draw_box(rect, BoxStyle::Double, c(UiColor::PanelBorderFocus), bg);
    rect
}

/// The menu of what can be done to a profile: a box in the middle with
/// the profile's name in its top border and the items under each other,
/// the focused one as a highlight bar.
fn paint_actions(ctx: &Ctx, buf: &mut GlyphBuffer, actions: &ActionsView) {
    let c = |u| ctx.palette.get(u);
    let (bg, text) = (c(UiColor::PanelBg), c(UiColor::Text));
    let widest = actions.items.iter().map(|i| cells(&i.label)).max();
    let w = widest.unwrap_or(0).max(cells(&actions.title) + 2) + 6;
    let h = i32::try_from(actions.items.len()).unwrap_or(0) + 4;
    let rect = middle_box(ctx, buf, w, h);
    let title = format!(" {} ", actions.title);
    buf.print(rect.x + 2, rect.y, &title, c(UiColor::TextHighlight), bg);
    for (i, item) in actions.items.iter().enumerate() {
        let y = rect.y + 2 + i32::try_from(i).unwrap_or(0);
        if item.focused {
            let bar = c(UiColor::PanelBorderFocus);
            buf.print(rect.x + 2, y, &format!(" {} ", item.label), bg, bar);
        } else {
            buf.print(rect.x + 3, y, &item.label, text, bg);
        }
    }
}

/// The answers of `question` on one line: each name with a space either
/// side, [`CHOICE_GAP`] cells apart. Returns the line and where each
/// answer starts in it.
fn choices_line(question: &QuestionView) -> (String, Vec<usize>) {
    let mut line = String::new();
    let mut starts = Vec::new();
    for (i, choice) in question.choices.iter().enumerate() {
        if i > 0 {
            line.push_str(&" ".repeat(CHOICE_GAP));
        }
        starts.push(line.chars().count());
        line.push(' ');
        line.push_str(&choice.label);
        line.push(' ');
    }
    (line, starts)
}

/// A question in a box in the middle of the screen (the look of the
/// Options screen's question on leaving), its answers on the line under
/// it, the one in focus as a highlight bar.
fn paint_question(ctx: &Ctx, buf: &mut GlyphBuffer, question: &QuestionView) {
    let c = |u| ctx.palette.get(u);
    let (bg, text) = (c(UiColor::PanelBg), c(UiColor::Text));
    let (line, starts) = choices_line(question);
    let widest = cells(&question.text).max(cells(&line));
    let rect = middle_box(ctx, buf, widest + 4, QUESTION_H);
    buf.print(rect.x + 2, rect.y + 2, &question.text, text, bg);
    let bar = c(UiColor::PanelBorderFocus);
    for (choice, start) in question.choices.iter().zip(starts) {
        let x = rect.x + 2 + i32::try_from(start).unwrap_or(0);
        let shown = format!(" {} ", choice.label);
        if choice.focused {
            buf.print(x, rect.y + 3, &shown, bg, bar);
        } else {
            buf.print(x, rect.y + 3, &shown, text, bg);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::{ConsoleSizes, ConsoleSpec};
    use crate::profiles::ProfileId;
    use crate::screen::tests::ctx;
    use crate::screens::profile::{ChoiceView, RowKind};

    /// `view` as painted.
    fn painted(c: &Ctx, view: &ProfileView) -> GlyphBuffer {
        let black = c.palette.get(UiColor::Black);
        let mut buf = GlyphBuffer::with_spec(ConsoleSpec::PC, Cell::new(' ', black, black));
        paint(c, view, &mut buf);
        buf
    }

    /// Row `y` of `buf`, trimmed.
    fn row(buf: &GlyphBuffer, y: i32) -> String {
        let glyphs = (0..i32::from(buf.width())).map(|x| buf.get(x, y).map_or(' ', |c| c.glyph));
        glyphs.collect::<String>().trim().to_owned()
    }

    fn profile(label: &str, current: bool, focused: bool) -> RowView {
        RowView {
            kind: RowKind::Profile(ProfileId::FIRST),
            label: label.to_owned(),
            current,
            focused,
            enabled: true,
        }
    }

    fn view(rows: Vec<RowView>) -> ProfileView {
        ProfileView {
            title: "Profile".to_owned(),
            rows,
            current_mark: "in use".to_owned(),
            actions: None,
            typing: None,
            grid: None,
            question: None,
            message: None,
            help: "help".to_owned(),
        }
    }

    #[test]
    fn the_profiles_then_a_blank_row_then_the_add_row_the_current_one_marked() {
        let c = ctx();
        let add = RowView {
            kind: RowKind::Add,
            label: "Add a profile".to_owned(),
            current: false,
            focused: false,
            enabled: false,
        };
        let v = view(vec![
            profile("Player 1", true, false),
            profile("Mei", false, true),
            add,
        ]);
        let buf = painted(&c, &v);
        assert!(row(&buf, PANEL.y).starts_with("┌─ Profile ─"));
        let first = row(&buf, PANEL.y + 2);
        assert_eq!(first, format!("│   {:<20}in use{:>10}", "Player 1", "│"));
        assert!(row(&buf, PANEL.y + 3).starts_with("│   Mei "));
        assert_eq!(row(&buf, PANEL.y + 4), row(&buf, PANEL.y + 1));
        assert!(row(&buf, PANEL.y + 5).starts_with("│   Add a profile"));
        // The add row that does nothing is dim; the focused row is lit.
        let dim = c.palette.get(UiColor::TextDim);
        assert_eq!(buf.get(LABEL_X, PANEL.y + 5).map(|c| c.fg), Some(dim));
        let bar = c.palette.get(UiColor::PanelBorderFocus);
        assert_eq!(buf.get(LABEL_X, PANEL.y + 3).map(|c| c.bg), Some(bar));
        assert_eq!(row(&buf, PANEL.y + 6), row(&buf, PANEL.y + 1));
        assert!(row(&buf, PANEL.y + 7).starts_with('└'));
        assert_eq!(row(&buf, 31), "help");
    }

    #[test]
    fn the_message_is_painted_one_blank_row_under_the_panel() {
        let c = ctx();
        let mut v = view(vec![profile("Player 1", true, true)]);
        v.message = Some("There can be at most 8 profiles.".to_owned());
        let buf = painted(&c, &v);
        let bottom = PANEL.y + 5;
        assert!(row(&buf, bottom).starts_with('└'));
        assert_eq!(row(&buf, bottom + 1), "");
        assert_eq!(row(&buf, bottom + 2), "There can be at most 8 profiles.");
    }

    #[test]
    fn a_question_has_its_answers_on_a_line_the_focused_one_lit() {
        let c = ctx();
        let mut v = view(vec![profile("Player 1", true, true)]);
        let choice = |label: &str, focused| ChoiceView {
            label: label.to_owned(),
            focused,
        };
        v.question = Some(QuestionView {
            text: "Delete Mei?".to_owned(),
            choices: vec![choice("Yes", false), choice("No", true)],
        });
        let buf = painted(&c, &v);
        let top = (i32::from(ConsoleSpec::PC.rows) - QUESTION_H) / 2;
        assert!(row(&buf, top + 2).contains("║ Delete Mei?  ║"));
        assert!(row(&buf, top + 3).contains("║  Yes     No  ║"));
        let bar = c.palette.get(UiColor::PanelBorderFocus);
        let lit: String = (0..i32::from(ConsoleSpec::PC.cols))
            .filter_map(|x| buf.get(x, top + 3))
            .filter(|cell| cell.bg == bar)
            .map(|cell| cell.glyph)
            .collect();
        assert_eq!(lit, " No ");
    }

    #[test]
    fn the_actions_are_a_box_titled_with_the_profile_the_focused_item_lit() {
        let c = ctx();
        let mut v = view(vec![profile("Player 1", true, true)]);
        v.actions = Some(ActionsView {
            title: "Mei".to_owned(),
            items: vec![
                ChoiceView {
                    label: "Rename".to_owned(),
                    focused: true,
                },
                ChoiceView {
                    label: "Delete".to_owned(),
                    focused: false,
                },
            ],
        });
        let buf = painted(&c, &v);
        let top = (i32::from(ConsoleSpec::PC.rows) - 6) / 2;
        assert!(row(&buf, top).contains("╔═ Mei ═"), "{}", row(&buf, top));
        assert!(row(&buf, top + 2).contains("║  Rename  ║"));
        assert!(row(&buf, top + 3).contains("║  Delete  ║"));
        // A name wider than the items widens the box.
        if let Some(actions) = &mut v.actions {
            actions.title = "Abcdefghijkl".to_owned();
        }
        let buf = painted(&c, &v);
        assert!(
            row(&buf, top).contains("╔═ Abcdefghijkl ═══╗"),
            "{}",
            row(&buf, top)
        );
    }
}
