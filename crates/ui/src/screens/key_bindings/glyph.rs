//! The glyph look of the Key bindings screen: paints a [`KeyBindingsView`]
//! as a panel of text rows, a column per slot, with the focused slot as a
//! bar. Everything about the look lives here (where things go, the box
//! characters, the colours); nothing here decides what the screen shows or
//! does. Another look is another `paint` of the same view (ADR-0054).

use super::Side;
use super::view::{ChoicesView, KeyBindingsView, QuestionView, RowView, SlotView, SwitchView};
use crate::color::UiColor;
use crate::glyph_buffer::{BoxStyle, Cell, GlyphBuffer, Rect};
use crate::screen::Ctx;
use crate::screens::print_centred;

/// The panel, in cells.
const PANEL: Rect = Rect::new(2, 1, 96, 27);
/// Column of the action labels.
const LABEL_X: i32 = PANEL.x + 4;
/// Column of the title and the group headings, two cells left of the
/// labels.
const HEADING_X: i32 = LABEL_X - 2;
/// Row of the keyboard / controller switch.
const SWITCH_Y: i32 = PANEL.y + 1;
/// Row of the slot columns' headings; the first group's heading is on the
/// next row.
const COLUMNS_Y: i32 = PANEL.y + 3;
/// Column of the first slot's highlight bar; its text starts one cell in.
const SLOTS_X: i32 = LABEL_X + 23;
/// Cells a slot's key name may take (the longest chord name fits).
const SLOT_W: usize = 15;
/// Columns from one slot to the next.
const SLOT_PITCH: i32 = 18;
/// What an empty slot shows.
const EMPTY_SLOT: &str = "·";
/// Height of the restore question's box: its two lines, a blank row above
/// and below, and the border.
const QUESTION_H: i32 = 6;
/// The box of the choices, in cells: a line each and the border.
const CHOICE_W: i32 = 12;
const CHOICE_H: i32 = 4;
/// Row of the message under the panel.
const MESSAGE_ROW: i32 = PANEL.y + PANEL.h + 1;

/// Left edge of slot `i`'s highlight bar.
fn slot_x(i: usize) -> i32 {
    SLOTS_X + i32::try_from(i).unwrap_or(0) * SLOT_PITCH
}

/// The console rows of what is listed under the columns' headings: each
/// group's heading, each action, and Restore defaults.
#[derive(Debug, PartialEq, Eq)]
struct Rows {
    /// Row of each group's heading.
    headings: Vec<i32>,
    /// Row of each action, in the order of [`KeyBindingsView::rows`].
    actions: Vec<i32>,
    /// Row of Restore defaults.
    restore: i32,
}

/// Places the groups one under the other: a heading, then its actions; a
/// blank row between groups and before Restore defaults.
fn place(view: &KeyBindingsView) -> Rows {
    let mut y = COLUMNS_Y + 1;
    let (mut headings, mut actions) = (Vec::new(), Vec::new());
    for (i, group) in view.groups.iter().enumerate() {
        if i > 0 {
            y += 1;
        }
        headings.push(y);
        y += 1;
        for _ in &group.rows {
            actions.push(y);
            y += 1;
        }
    }
    Rows {
        headings,
        actions,
        restore: y + 1,
    }
}

/// Paints `view` over the whole of `buf`.
pub fn paint(ctx: &Ctx, view: &KeyBindingsView, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let (black, bg) = (c(UiColor::Black), c(UiColor::PanelBg));
    let (text, dim) = (c(UiColor::Text), c(UiColor::TextDim));
    let bar = c(UiColor::PanelBorderFocus);
    buf.fill_rect(buf.bounds(), Cell::new(' ', text, black));
    buf.fill_rect(PANEL, Cell::new(' ', text, bg));
    buf.draw_box(PANEL, BoxStyle::Single, c(UiColor::PanelBorder), bg);
    let title = format!(" {} ", view.title);
    buf.print(HEADING_X, PANEL.y, &title, c(UiColor::TextHighlight), bg);
    paint_switch(ctx, buf, &view.switch);

    for (j, heading) in view.columns.iter().enumerate() {
        buf.print(slot_x(j) + 1, COLUMNS_Y, heading, dim, bg);
    }
    let placed = place(view);
    let mut rows = view.rows().zip(&placed.actions);
    for (group, &heading_y) in view.groups.iter().zip(&placed.headings) {
        buf.print(HEADING_X, heading_y, &group.heading, bar, bg);
        for _ in &group.rows {
            if let Some((row, &y)) = rows.next() {
                paint_row(ctx, buf, view, row, y);
            }
        }
    }
    if view.restore.focused {
        let label = format!(" {} ", view.restore.label);
        buf.print(LABEL_X - 1, placed.restore, &label, bg, bar);
    } else {
        buf.print(LABEL_X, placed.restore, &view.restore.label, text, bg);
    }

    if let Some(message) = &view.message {
        print_centred(buf, MESSAGE_ROW, message, c(UiColor::HpLow), black);
    }
    if let Some(question) = &view.question {
        paint_question(ctx, buf, question);
    }
    if let Some(choices) = &view.choices {
        let focused_y = view
            .rows()
            .zip(&placed.actions)
            .find_map(|(row, &y)| row.focused().then_some(y));
        if let Some(y) = focused_y {
            paint_choices(ctx, buf, choices, y);
        }
    }
    let bottom = i32::from(buf.height()) - 1;
    print_centred(buf, bottom, &view.help, dim, black);
}

/// Paints the keyboard / controller switch: the shown side highlighted
/// (as a bar while the focus is on the switch), the other dim.
fn paint_switch(ctx: &Ctx, buf: &mut GlyphBuffer, switch: &SwitchView) {
    let c = |u| ctx.palette.get(u);
    let (bg, bar) = (c(UiColor::PanelBg), c(UiColor::PanelBorderFocus));
    let mut x = LABEL_X;
    for (side, name) in [
        (Side::Keyboard, switch.keyboard.as_str()),
        (Side::Controller, switch.controller.as_str()),
    ] {
        if side != switch.shown {
            buf.print(x, SWITCH_Y, name, c(UiColor::TextDim), bg);
        } else if switch.focused {
            buf.print(x - 1, SWITCH_Y, &format!(" {name} "), bg, bar);
        } else {
            buf.print(x, SWITCH_Y, name, c(UiColor::TextHighlight), bg);
        }
        x += i32::try_from(name.chars().count()).unwrap_or(0) + 4;
    }
}

/// Paints `row` on console row `y`: its label (a highlight colour if the
/// cursor is on it, white if it just lost its key), the keys it has that
/// can't change, its slots (the one the cursor is on a bar) and, if
/// nothing is in any, the note.
fn paint_row(ctx: &Ctx, buf: &mut GlyphBuffer, view: &KeyBindingsView, row: &RowView, y: i32) {
    let c = |u| ctx.palette.get(u);
    let (bg, text, dim) = (c(UiColor::PanelBg), c(UiColor::Text), c(UiColor::TextDim));
    let label_fg = if row.focused() {
        c(UiColor::TextHighlight)
    } else if row.lost_key {
        c(UiColor::White)
    } else {
        text
    };
    let label_w = buf.print(LABEL_X, y, &row.label, label_fg, bg);
    if !row.fixed.is_empty() {
        let x = LABEL_X + i32::from(label_w) + 2;
        buf.print(x, y, &format!("+ {}", row.fixed.join("/")), dim, bg);
    }
    for (j, slot) in row.slots.iter().enumerate() {
        let x = slot_x(j);
        if row.focus == Some(j) {
            let shown = match slot {
                SlotView::Capturing(prompt) => prompt.as_str(),
                SlotView::Bound(name) => name.as_str(),
                SlotView::Empty => "",
            };
            let bar = format!(" {shown:<SLOT_W$}");
            buf.print(x, y, &bar, bg, c(UiColor::PanelBorderFocus));
        } else if let SlotView::Bound(name) = slot {
            buf.print(x + 1, y, name, text, bg);
        } else {
            buf.print(x + 1, y, EMPTY_SLOT, dim, bg);
        }
    }
    if row.unmapped {
        let fg = if row.blocks_leaving {
            c(UiColor::HpLow)
        } else {
            dim
        };
        buf.print(slot_x(row.slots.len()) + 1, y, &view.not_mapped, fg, bg);
    }
}

/// Paints `question` in a double-bordered box in the middle of the screen,
/// with its answers under it (the look of the battle's end-turn question).
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
    buf.print(
        rect.x + 2,
        rect.y + 3,
        &question.answers,
        c(UiColor::TextDim),
        bg,
    );
}

/// Paints `choices` in a double-bordered box under the slot on console row
/// `slot_y` (over it on the last rows, where it wouldn't fit), the focused
/// line as a bar.
fn paint_choices(ctx: &Ctx, buf: &mut GlyphBuffer, choices: &ChoicesView, slot_y: i32) {
    let c = |u| ctx.palette.get(u);
    let (bg, bar) = (c(UiColor::PanelBg), c(UiColor::PanelBorderFocus));
    let below = slot_y + 1;
    let y = if below + CHOICE_H < PANEL.y + PANEL.h {
        below
    } else {
        slot_y - CHOICE_H
    };
    let rect = Rect::new(slot_x(choices.slot), y, CHOICE_W, CHOICE_H);
    buf.fill_rect(rect, Cell::new(' ', c(UiColor::Text), bg));
    buf.draw_box(rect, BoxStyle::Double, bar, bg);
    let inner = usize::try_from(CHOICE_W - 3).unwrap_or(0);
    for (i, choice) in (0..).zip(&choices.lines) {
        let (x, y) = (rect.x + 1, rect.y + 1 + i);
        if usize::try_from(i).is_ok_and(|i| i == choices.focused) {
            buf.print(x, y, &format!(" {choice:<inner$}"), bg, bar);
        } else {
            buf.print(x + 1, y, choice, c(UiColor::Text), bg);
        }
    }
}

#[cfg(test)]
mod tests;
