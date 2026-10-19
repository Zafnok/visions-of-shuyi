//! The Playdate's look of the Profile screen (ticket 1045): the profiles
//! in a window, the one in use marked, then the row that adds one; the
//! menu of what can be done to a profile, the questions and the name boxes
//! in windows over it; the message under the window and the help on the
//! bottom row, in the one-bit look of [`crate::screens::playdate`]. The
//! focus is the bar; the add row that does nothing is grey.

use super::RowKind;
use super::view::{ProfileView, RowView};
use crate::glyph_buffer::{GlyphBuffer, Rect};
use crate::prelude::*;
use crate::screen::Ctx;
use crate::screens::playdate::{
    Answers, Item, ask, centred, clear, inks, item, list, to_i32, window,
};
use crate::widgets::name_entry::playdate::{grid_height, paint_grid, paint_typing};

/// The window's left column and width.
const PANEL_X: (i32, i32) = (5, 40);
/// Column of the row labels, from the window's left edge.
const LABEL_DX: i32 = 4;
/// Column of the mark of the profile in use, from the window's left edge.
const MARK_DX: i32 = 24;
/// The rows the window may take from the top: the message's and the
/// help's are under it.
const ROOM: i32 = 13;
/// Left column and width of the letter grid's and the typing box's
/// windows.
const BOX_X: (i32, i32) = (2, 46);
/// The typing box's top row and height.
const TYPE_BOX: (i32, i32) = (4, 7);

/// The window for `rows` rows and the blank one before the add row: a
/// blank row above and below them where they fit, from the second row;
/// from the top without the blank rows otherwise. Returns the window and
/// the row of its first line.
fn panel(rows: usize) -> (Rect, i32) {
    let lines = to_i32(rows) + 1;
    if lines + 4 < ROOM {
        (Rect::new(PANEL_X.0, 1, PANEL_X.1, lines + 4), 3)
    } else {
        (Rect::new(PANEL_X.0, 0, PANEL_X.1, lines + 2), 1)
    }
}

/// Paints `view` over the whole of `buf`.
pub fn paint(ctx: &Ctx, view: &ProfileView, buf: &mut GlyphBuffer) {
    clear(ctx, buf);
    let (rect, first) = panel(view.rows.len());
    window(ctx, buf, rect, &view.title);
    let mut y = first;
    for row in &view.rows {
        if row.kind == RowKind::Add {
            y += 1;
        }
        paint_row(ctx, buf, row, &view.current_mark, (rect.x, y));
        y += 1;
    }
    if let Some(message) = &view.message {
        centred(ctx, buf, rect.y + rect.h, message);
    }
    let height = i32::from(buf.height());
    if let Some(actions) = &view.actions {
        let items: Vec<_> = actions
            .items
            .iter()
            .map(|i| (i.label.as_str(), i.focused))
            .collect();
        list(ctx, buf, &actions.title, &items);
    }
    if let Some(grid) = &view.grid {
        let y = ((height - grid_height(grid)) / 2).max(0);
        paint_grid(ctx, buf, Rect::new(BOX_X.0, y, BOX_X.1, 0), grid);
    }
    if let Some(typing) = &view.typing {
        let rect = Rect::new(BOX_X.0, TYPE_BOX.0, BOX_X.1, TYPE_BOX.1);
        paint_typing(ctx, buf, rect, typing);
    }
    if let Some(question) = &view.question {
        let choices: Vec<_> = question
            .choices
            .iter()
            .map(|c| (c.label.as_str(), c.focused))
            .collect();
        ask(ctx, buf, &question.text, Answers::Choices(&choices));
    }
    centred(ctx, buf, height - 1, &view.help);
}

/// `row` at `(x, y)`, `x` the window's left edge: its label (the bar if
/// focused, grey if it does nothing) and, on the profile in use, `mark`.
fn paint_row(ctx: &Ctx, buf: &mut GlyphBuffer, row: &RowView, mark: &str, (x, y): (i32, i32)) {
    let ink = inks(ctx);
    let w = MARK_DX - LABEL_DX - 2;
    item(
        ctx,
        buf,
        (x + LABEL_DX, y),
        w,
        &row.label,
        Item::of(row.focused, row.enabled),
    );
    if row.current {
        buf.print(x + MARK_DX, y, mark, ink.text, ink.inside);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profiles::ProfileId;
    use crate::screens::playdate::tests::{blank, cell, is_bar, pd_ctx, rows};
    use crate::screens::profile::{ActionsView, ChoiceView, QuestionView};
    use trpg_content::Ink;

    fn profile(label: &str, current: bool, focused: bool) -> RowView {
        RowView {
            kind: RowKind::Profile(ProfileId::FIRST),
            label: label.to_owned(),
            current,
            focused,
            enabled: true,
        }
    }

    fn add(enabled: bool) -> RowView {
        RowView {
            kind: RowKind::Add,
            label: "Add a profile".to_owned(),
            current: false,
            focused: false,
            enabled,
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

    fn painted(v: &ProfileView) -> GlyphBuffer {
        let mut buf = blank();
        paint(&pd_ctx(), v, &mut buf);
        buf
    }

    #[test]
    fn the_profiles_then_a_blank_row_then_add_the_one_in_use_marked() {
        let mut v = view(vec![
            profile("Player 1", true, false),
            profile("Mei", false, true),
            add(false),
        ]);
        v.message = Some("No more profiles".to_owned());
        let buf = painted(&v);
        let shown = rows(&buf);
        assert!(shown[1].starts_with("╔═ Profile ═"), "{shown:#?}");
        assert_eq!(shown[2], format!("║{:38}║", ""));
        assert_eq!(shown[3], format!("║   {:<20}in use{:>10}", "Player 1", "║"));
        assert!(shown[4].starts_with("║   Mei"));
        assert!(is_bar(&buf, 4, " Mei"));
        assert_eq!(shown[5], shown[2]);
        assert!(shown[6].starts_with("║   Add a profile"));
        // The add row that does nothing is grey.
        assert_eq!(
            cell(&buf, PANEL_X.0 + LABEL_DX, 6).fg,
            crate::screens::playdate::tests::c(Ink::Grey)
        );
        assert!(shown[8].starts_with('╚'));
        assert_eq!(shown[9], "No more profiles");
        assert_eq!(shown[14], "help");
    }

    #[test]
    fn the_most_profiles_fit_the_window_without_its_blank_rows() {
        let mut list: Vec<RowView> = (1..=8)
            .map(|n| profile(&format!("P{n}"), n == 1, n == 8))
            .collect();
        list.push(add(false));
        let buf = painted(&view(list));
        let shown = rows(&buf);
        assert!(shown[0].starts_with("╔═ Profile ═"));
        assert!(shown[1].starts_with("║   P1"));
        assert!(shown[8].starts_with("║   P8"));
        assert!(shown[10].starts_with("║   Add a profile"));
        assert!(shown[11].starts_with('╚'));
        assert_eq!(shown[14], "help");
    }

    #[test]
    fn the_actions_and_the_question_are_windows_over_the_list() {
        let mut v = view(vec![profile("Player 1", true, true), add(true)]);
        let choice = |label: &str, focused| ChoiceView {
            label: label.to_owned(),
            focused,
        };
        v.actions = Some(ActionsView {
            title: "Player 1".to_owned(),
            items: vec![choice("Rename", true), choice("Delete", false)],
        });
        let buf = painted(&v);
        let shown = rows(&buf);
        let top = shown
            .iter()
            .position(|r| r.contains("═ Player 1 ═"))
            .unwrap();
        assert!(shown[top + 1].contains("Rename"));
        assert!(is_bar(&buf, to_i32(top) + 1, "Rename"));
        assert!(shown[top + 2].contains("Delete"));
        v.actions = None;
        v.question = Some(QuestionView {
            text: "Delete Player 1? Its saves and options go with it.".to_owned(),
            choices: vec![choice("Yes", false), choice("No", true)],
        });
        let buf = painted(&v);
        let shown = rows(&buf);
        let line = shown.iter().position(|r| r.contains("Yes")).unwrap();
        assert!(is_bar(&buf, to_i32(line), " No "));
        assert!(!is_bar(&buf, to_i32(line), "Yes"));
        // The text wrapped over two lines, a blank row before the answers.
        assert!(shown[line - 3].contains("Delete Player 1?"));
        assert_eq!(shown[line - 1].trim_matches('║').trim(), "");
    }
}
