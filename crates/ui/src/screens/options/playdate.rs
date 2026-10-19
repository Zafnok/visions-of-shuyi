//! The Playdate's look of the Options screen (ticket 1045): the engine's
//! [`OptionsView`] in the one-bit look of [`crate::screens::playdate`].
//! The rows fill a window the screen's width, its title in the top of its
//! frame: each name, and its value from the middle; the focused row's name
//! is the bar and a value the D-pad changes sits between two arrows, as on
//! the PC. Rows that don't fit scroll with the focus, a mark in the frame
//! saying there are more. A message goes in the bottom of the frame, the
//! help on the bottom row; a question opens in a window over the rows.

use shuyi_ui::screens::options::glyph::volume_text;

use super::view::{OptionsView, RowView, ValueView};
use crate::glyph_buffer::{GlyphBuffer, Rect};
use crate::prelude::*;
use crate::screen::Ctx;
use crate::screens::playdate::{
    Answers, Item, ask, bar, cells, centred, clear, cut, first_shown, footer, inks, item,
    more_marks, to_i32, to_usize, window,
};

/// Column of the rows' names.
const LABEL_X: i32 = 3;
/// Cells a name takes at most.
const LABEL_W: i32 = 20;
/// Column of the rows' values.
const VALUE_X: i32 = 27;
/// Cells of the number box, inside its bar.
const BOX_W: usize = 4;

/// A line of the window: a row, or the blank line before a group.
enum Line<'a> {
    Gap,
    Row(&'a RowView),
}

/// Paints `view` over the whole of `buf`.
pub fn paint(ctx: &Ctx, view: &OptionsView, buf: &mut GlyphBuffer) {
    clear(ctx, buf);
    let bottom = i32::from(buf.height()) - 1;
    let rect = Rect::new(0, 0, i32::from(buf.width()), bottom);
    window(ctx, buf, rect, &view.title);
    let mut lines = Vec::new();
    for row in &view.rows {
        if row.starts_group && !lines.is_empty() {
            lines.push(Line::Gap);
        }
        lines.push(Line::Row(row));
    }
    let focus = lines
        .iter()
        .position(|l| matches!(l, Line::Row(r) if r.focused));
    let room = to_usize(rect.h - 2);
    let first = first_shown(lines.len(), room, focus);
    for (y, line) in (rect.y + 1..).zip(lines.iter().skip(first).take(room)) {
        if let Line::Row(row) = line {
            paint_row(ctx, buf, row, y);
        }
    }
    more_marks(ctx, buf, rect, first > 0, first + room < lines.len());
    if let Some(message) = &view.message {
        footer(ctx, buf, rect, message);
    }
    centred(ctx, buf, bottom, &view.help);
    if let Some(question) = &view.question {
        ask(ctx, buf, &question.text, Answers::Hint(&question.answers));
    }
    if let Some(leaving) = &view.leaving {
        let choices: Vec<_> = leaving
            .choices
            .iter()
            .map(|c| (c.label.as_str(), c.focused))
            .collect();
        ask(ctx, buf, &leaving.text, Answers::Choices(&choices));
    }
}

/// Paints `row` on row `y`: its name (the bar if focused), then its
/// value; a value the D-pad changes between two arrows while focused; the
/// number box a bar where the value was.
fn paint_row(ctx: &Ctx, buf: &mut GlyphBuffer, row: &RowView, y: i32) {
    let ink = inks(ctx);
    let state = Item::of(row.focused, true);
    item(ctx, buf, (LABEL_X, y), LABEL_W, &row.label, state);
    let room = to_usize(i32::from(buf.width()) - VALUE_X - 3);
    let (value, adjustable) = match &row.value {
        ValueView::None => return,
        ValueView::NumberBox { digits, typing } => {
            let caret = if *typing { "_" } else { "" };
            let shown = format!("{digits}{caret}");
            bar(ctx, buf, VALUE_X - 1, y, to_i32(BOX_W) + 2, &shown);
            return;
        }
        ValueView::Text { text, adjustable } => (cut(text, room), *adjustable),
        ValueView::Volume { level, max } => (volume_text(*level, *max), true),
    };
    buf.print(VALUE_X, y, &value, ink.text, ink.inside);
    if row.focused && adjustable {
        buf.print(VALUE_X - 2, y, "◄", ink.text, ink.inside);
        let after = VALUE_X + cells(&value) + 1;
        buf.print(after, y, "►", ink.text, ink.inside);
    }
}

#[cfg(test)]
mod tests;
