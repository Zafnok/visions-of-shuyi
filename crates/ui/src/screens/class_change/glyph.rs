//! The glyph look of the class-choice screen: the title and the unit's
//! line at the top (the seals in stock on the right, in the warning colour
//! when there are none), the columns as boxes side by side (the focused one
//! double-bordered, a refused one dimmed whole with its reason in the
//! warning colour, each gain and `MAX` mark lit), the question as a box in
//! the middle, and the change's pages where the battle screen puts them.
//! Everything about the look lives here; nothing here decides what the
//! screen shows. Another look is another `paint` of the same view
//! (ADR-0054).

use super::COLUMNS;
use super::view::{
    ClassChangeBody, ClassChangeView, ColumnView, ColumnsView, QuestionView, StatRowView,
};
use crate::color::UiColor;
use crate::glyph_buffer::{BoxStyle, Cell, GlyphBuffer, Rect};
use crate::prelude::*;
use crate::screen::Ctx;
use crate::screens::battle::progress;

/// Width of a column's box, in cells.
const COLUMN_W: i32 = 24;

/// Column of the first column's box.
const COLUMNS_X: i32 = 2;

/// Row of the columns' top border.
const COLUMNS_Y: i32 = 4;

/// Height of a column's box.
const COLUMN_H: i32 = 25;

/// Widest text in a column (one blank cell inside each border).
const TEXT_W: usize = 20;

/// Row of the title.
const TITLE_ROW: i32 = 1;

/// Row of the unit's line.
const UNIT_ROW: i32 = 2;

/// Row of the `← more` / `more →` marks.
const MORE_ROW: i32 = 3;

/// Shown for a class with no weapons or no active skill.
const NOTHING: &str = "--";

/// The right edge of the columns: text aligned right ends here.
fn right_edge() -> i32 {
    COLUMNS_X + COLUMN_W * i32::try_from(COLUMNS).unwrap_or(0)
}

/// How many cells `text` takes.
fn width(text: &str) -> i32 {
    i32::try_from(text.chars().count()).unwrap_or(0)
}

/// The text of a stat row: `Str   9 → 12  +3` if it grows, else
/// `Mag   1`; with the `mark` after a stat that ends at its ceiling.
pub fn stat_line(row: &StatRowView, mark: &str) -> String {
    let (head, lit) = stat_parts(row, mark);
    head + &lit
}

/// [`stat_line`] in two parts: the plain text, then what is lit (the gain
/// and the mark, from the gain's `+` on, or just the mark).
fn stat_parts(row: &StatRowView, mark: &str) -> (String, String) {
    // check-text: not a data name (the view's own)
    let name = &row.name;
    let (mut head, gain) = progress::stat_row_parts(name, row.before, row.after - row.before);
    let mut lit = gain.unwrap_or_default();
    if row.at_max {
        if lit.is_empty() {
            head.push(' ');
        } else {
            lit.push(' ');
        }
        lit.push_str(mark);
    }
    (head, lit)
}

/// Paints the class-choice screen over the whole of `buf`.
pub fn paint(ctx: &Ctx, view: &ClassChangeView, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let black = c(UiColor::Black);
    buf.fill_rect(buf.bounds(), Cell::new(' ', c(UiColor::Text), black));
    match &view.body {
        ClassChangeBody::Columns(columns) => {
            paint_header(ctx, columns, buf);
            paint_columns(ctx, columns, buf);
            if let Some(question) = &columns.question {
                paint_question(ctx, question, buf);
            }
        }
        // The pages name the unit and its classes themselves.
        ClassChangeBody::Result(pages) => {
            let look = (&ctx.palette, ctx.words());
            progress::draw(buf, look, (&ctx.content.portraits, &ctx.lead), pages);
        }
        ClassChangeBody::Done => {}
    }
    let bottom = i32::from(buf.height()) - 1;
    buf.print(1, bottom, &view.help, c(UiColor::TextDim), black);
}

/// The title, the unit's line and the seals in stock for the focused
/// class.
fn paint_header(ctx: &Ctx, view: &ColumnsView, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let black = c(UiColor::Black);
    let hi = c(UiColor::TextHighlight);
    buf.print(COLUMNS_X, TITLE_ROW, &view.title, hi, black);
    let n = buf.print(COLUMNS_X, UNIT_ROW, &view.unit, c(UiColor::Player), black);
    let x = COLUMNS_X + i32::from(n) + 2;
    buf.print(x, UNIT_ROW, &view.class_level, c(UiColor::Text), black);
    let Some(seal) = &view.seal else {
        return;
    };
    // check-text: not a data name (the view's own)
    let text = format!("{} ×{}", seal.name, seal.held);
    let color = if seal.held == 0 {
        UiColor::HpLow
    } else {
        UiColor::Text
    };
    buf.print(
        right_edge() - width(&text),
        UNIT_ROW,
        &text,
        c(color),
        black,
    );
}

/// The columns on screen, and the marks for those scrolled away.
fn paint_columns(ctx: &Ctx, view: &ColumnsView, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let (dim, black) = (c(UiColor::TextDim), c(UiColor::Black));
    if let Some(none) = &view.none {
        buf.print(COLUMNS_X, COLUMNS_Y + 1, none, dim, black);
    }
    for (slot, column) in (0..).zip(&view.columns) {
        paint_column(ctx, view, column, buf, COLUMNS_X + slot * COLUMN_W);
    }
    if let Some(more) = &view.more_left {
        buf.print(COLUMNS_X, MORE_ROW, more, dim, black);
    }
    if let Some(more) = &view.more_right {
        buf.print(right_edge() - width(more), MORE_ROW, more, dim, black);
    }
}

/// One class's column, its box's left edge at column `x`. A refused class
/// is dimmed whole.
fn paint_column(ctx: &Ctx, view: &ColumnsView, column: &ColumnView, buf: &mut GlyphBuffer, x: i32) {
    let c = |u| ctx.palette.get(u);
    let bg = c(UiColor::PanelBg);
    let rect = Rect::new(x, COLUMNS_Y, COLUMN_W, COLUMN_H);
    buf.fill_rect(rect, Cell::new(' ', c(UiColor::Text), bg));
    let (style, border) = if column.focused {
        (BoxStyle::Double, UiColor::PanelBorderFocus)
    } else {
        (BoxStyle::Single, UiColor::PanelBorder)
    };
    buf.draw_box(rect, style, c(border), bg);
    let shade = |color| {
        if column.offered {
            color
        } else {
            UiColor::TextDim
        }
    };
    let pen = |buf: &mut GlyphBuffer, row: i32, text: &str, color: UiColor| {
        let cut: String = text.chars().take(TEXT_W).collect();
        buf.print(x + 2, COLUMNS_Y + row, &cut, c(shade(color)), bg);
    };
    let title = if column.focused {
        UiColor::TextHighlight
    } else {
        UiColor::Text
    };
    // check-text: not a data name (the view's own)
    pen(buf, 1, &column.name, title);
    let record = column.record.as_deref().unwrap_or_default();
    pen(
        buf,
        2,
        &format!("{:<9}{record}", column.tier),
        UiColor::TextDim,
    );
    let (mov, movement) = (&column.mov, &column.movement);
    pen(buf, 4, &format!("{mov:<12}{movement}"), UiColor::Text);
    pen(buf, 5, &column.tags.join(" "), UiColor::Text);
    pen(buf, 7, &view.weapons_heading, UiColor::TextHighlight);
    if column.weapons.is_empty() {
        pen(buf, 8, NOTHING, UiColor::TextDim);
    }
    for (row, line) in (8..11).zip(&column.weapons) {
        pen(buf, row, line, UiColor::Text);
    }
    pen(buf, 12, &view.active_heading, UiColor::TextHighlight);
    match &column.active {
        Some(name) => pen(buf, 13, name, UiColor::Text),
        None => pen(buf, 13, NOTHING, UiColor::TextDim),
    }
    for (row, stat) in (15..).zip(&column.stats) {
        let color = if stat.after > stat.before {
            UiColor::Text
        } else {
            UiColor::TextDim
        };
        let (head, lit) = stat_parts(stat, &view.max_mark);
        let (tx, ty) = (x + 2, COLUMNS_Y + row);
        let n = buf.print(tx, ty, &head, c(shade(color)), bg);
        // The gain and the mark in the highlight colour.
        let hi = c(shade(UiColor::TextHighlight));
        buf.print(tx + i32::from(n), ty, &lit, hi, bg);
    }
    if let Some(reason) = &column.refused {
        let cut: String = reason.chars().take(TEXT_W).collect();
        buf.print(x + 2, COLUMNS_Y + 23, &cut, c(UiColor::HpLow), bg);
    }
}

/// The question box, centred.
fn paint_question(ctx: &Ctx, question: &QuestionView, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let bg = c(UiColor::PanelBg);
    let lines = [
        (&question.ask, UiColor::Text),
        (&question.cost, UiColor::Text),
        (&question.keys, UiColor::TextDim),
    ];
    let widest = lines.iter().map(|(l, _)| l.chars().count()).max();
    let w = i32::try_from(widest.unwrap_or(0)).unwrap_or(0) + 4;
    let h = i32::try_from(lines.len()).unwrap_or(0) + 4;
    let (bw, bh) = (i32::from(buf.width()), i32::from(buf.height()));
    let rect = Rect::new((bw - w) / 2, (bh - h) / 2, w, h);
    buf.fill_rect(rect, Cell::new(' ', c(UiColor::Text), bg));
    buf.draw_box(rect, BoxStyle::Double, c(UiColor::PanelBorderFocus), bg);
    for (y, (line, color)) in (rect.y + 2..).zip(lines) {
        buf.print(rect.x + 2, y, line, c(color), bg);
    }
}

#[cfg(test)]
mod tests;
