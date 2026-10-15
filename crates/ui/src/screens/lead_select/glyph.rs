//! The glyph look of the lead screen: the two portraits side by side in
//! frames (the chosen one in full colour, its frame lit, double while the
//! cursor is on them; the other faded), the name and `[ Start ]` under
//! them, and the typing box or the letter grid in a double-bordered box
//! over the screen. Everything about the look lives here; nothing here
//! decides what the screen shows. Another look is another `paint` of the
//! same view (ADR-0054).

use super::Row;
use super::view::{GenderView, GridView, LeadSelectView, NameRowView, TypingView};
use crate::color::UiColor;
use crate::glyph_buffer::{BoxStyle, Cell, GlyphBuffer, Rect};
use crate::portrait::draw_portrait;
use crate::prelude::*;
use crate::screen::Ctx;
use crate::screens::print_centred;

/// Row of the heading.
const HEADING_ROW: i32 = 1;
/// Portrait frame size: a 32×16-cell portrait plus its border.
const FRAME: (i32, i32) = (34, 18);
/// Top row of the portrait frames.
const FRAME_Y: i32 = 3;
/// Left column of the male and female frames.
const FRAME_X: [i32; 2] = [14, 52];
/// Row of the gender labels under the frames.
const LABEL_ROW: i32 = 21;
/// Row of the name.
const NAME_ROW: i32 = 24;
/// Row of the Start button.
const START_ROW: i32 = 27;
/// The name grid's box.
const GRID_BOX: Rect = Rect::new(27, 8, 46, 14);
/// The typing box.
const TYPE_BOX: Rect = Rect::new(27, 10, 46, 7);
/// How much the portrait not chosen fades (0 = full colour, 1 = gone).
const DIM_NOT_CHOSEN: f32 = 0.45;

/// Cells `text` takes, as a coordinate.
fn cells(text: &str) -> i32 {
    i32::try_from(text.chars().count()).unwrap_or(0)
}

/// Paints the lead screen over the whole of `buf`.
pub fn paint(ctx: &Ctx, view: &LeadSelectView, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let black = c(UiColor::Black);
    buf.fill_rect(buf.bounds(), Cell::new(' ', c(UiColor::Text), black));
    let highlight = c(UiColor::TextHighlight);
    print_centred(buf, HEADING_ROW, &view.heading, highlight, black);
    let on_genders = view.cursor == Some(Row::Gender);
    for (gender, x) in view.genders.iter().zip(FRAME_X) {
        paint_gender(ctx, buf, gender, on_genders, x);
    }
    paint_rows(ctx, view, buf);
    if let Some(grid) = &view.grid {
        paint_grid(ctx, buf, grid);
    }
    if let Some(typing) = &view.typing {
        paint_typing(ctx, buf, typing);
    }
    let bottom = i32::from(buf.height()) - 1;
    print_centred(buf, bottom, &view.help, c(UiColor::TextDim), black);
}

/// One gender's frame, portrait and label at column `x`; the chosen one is
/// lit, with a double border while the cursor is on the portraits
/// (`focused`).
fn paint_gender(ctx: &Ctx, buf: &mut GlyphBuffer, gender: &GenderView, focused: bool, x: i32) {
    let c = |u| ctx.palette.get(u);
    let bg = c(UiColor::PanelBg);
    let (style, border) = match (gender.chosen, focused) {
        (true, true) => (BoxStyle::Double, UiColor::PanelBorderFocus),
        (true, false) => (BoxStyle::Single, UiColor::PanelBorderFocus),
        (false, _) => (BoxStyle::Single, UiColor::PanelBorder),
    };
    let frame = Rect::new(x, FRAME_Y, FRAME.0, FRAME.1);
    buf.fill_rect(frame, Cell::new(' ', c(UiColor::Text), bg));
    buf.draw_box(frame, style, c(border), bg);
    if let Some(art) = ctx.content.portraits.get(gender.portrait) {
        let dim = if gender.chosen { 0.0 } else { DIM_NOT_CHOSEN };
        let at = (x + 1, FRAME_Y + 1);
        draw_portrait(buf, at, art, gender.expression, dim, false);
    }
    let fg = if gender.chosen {
        UiColor::TextHighlight
    } else {
        UiColor::TextDim
    };
    let label_x = x + (FRAME.0 - cells(&gender.label)) / 2;
    let black = c(UiColor::Black);
    buf.print(label_x, LABEL_ROW, &gender.label, c(fg), black);
}

/// The name row and the Start button, the one with the cursor lit.
fn paint_rows(ctx: &Ctx, view: &LeadSelectView, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let black = c(UiColor::Black);
    let lit = |row| {
        if view.cursor == Some(row) {
            UiColor::TextHighlight
        } else {
            UiColor::Text
        }
    };
    let NameRowView {
        label,
        first,
        family,
    } = &view.name_row;
    let full = format!("{label}   {first} {family}");
    let x = (i32::from(buf.width()) - cells(&full)) / 2;
    buf.print(x, NAME_ROW, label, c(UiColor::TextDim), black);
    let name_x = x + cells(label) + 3;
    buf.print(name_x, NAME_ROW, first, c(lit(Row::Name)), black);
    let family_x = name_x + cells(first) + 1;
    buf.print(family_x, NAME_ROW, family, c(UiColor::TextDim), black);
    let start = format!("[ {} ]", view.start);
    print_centred(buf, START_ROW, &start, c(lit(Row::Start)), black);
}

/// The x that centres `text` in `rect`.
fn centred_in(rect: Rect, text: &str) -> i32 {
    rect.x + (rect.w - cells(text)) / 2
}

/// A double-bordered box at `rect` with `title` in its top border.
fn paint_box(ctx: &Ctx, buf: &mut GlyphBuffer, rect: Rect, title: &str) {
    let c = |u| ctx.palette.get(u);
    let bg = c(UiColor::PanelBg);
    buf.fill_rect(rect, Cell::new(' ', c(UiColor::Text), bg));
    buf.draw_box(rect, BoxStyle::Double, c(UiColor::PanelBorder), bg);
    let title = format!(" {title} ");
    buf.print(rect.x + 2, rect.y, &title, c(UiColor::TextHighlight), bg);
}

/// The typing box: the name so far with a cursor, and the hint to type.
fn paint_typing(ctx: &Ctx, buf: &mut GlyphBuffer, typing: &TypingView) {
    let c = |u| ctx.palette.get(u);
    let bg = c(UiColor::PanelBg);
    paint_box(ctx, buf, TYPE_BOX, &typing.title);
    let shown = format!("{}_", typing.typed);
    let x = centred_in(TYPE_BOX, &shown);
    buf.print(x, TYPE_BOX.y + 2, &shown, c(UiColor::Text), bg);
    let hint = &typing.hint;
    let x = centred_in(TYPE_BOX, hint);
    buf.print(x, TYPE_BOX.y + 4, hint, c(UiColor::TextDim), bg);
}

/// The name grid's box: the name so far with a cursor, then the grid, the
/// focused cell as a lit bar.
fn paint_grid(ctx: &Ctx, buf: &mut GlyphBuffer, grid: &GridView) {
    let c = |u| ctx.palette.get(u);
    let bg = c(UiColor::PanelBg);
    paint_box(ctx, buf, GRID_BOX, &grid.title);
    let shown = format!("{}_", grid.spelled);
    let x = centred_in(GRID_BOX, &shown);
    buf.print(x, GRID_BOX.y + 2, &shown, c(UiColor::Text), bg);
    for (r, labels) in grid.rows.iter().enumerate() {
        let y = GRID_BOX.y + 4 + i32::try_from(r * 2).unwrap_or(0);
        // Each cell is its label with a space either side.
        let widths: Vec<i32> = labels.iter().map(|l| cells(l) + 2).collect();
        let total: i32 = widths.iter().sum();
        let mut x = GRID_BOX.x + (GRID_BOX.w - total) / 2;
        for (col, (label, w)) in labels.iter().zip(&widths).enumerate() {
            let (fg, cell_bg) = if grid.focus == (r, col) {
                (c(UiColor::PanelBg), c(UiColor::PanelBorderFocus))
            } else {
                (c(UiColor::Text), bg)
            };
            buf.print(x, y, &format!(" {label} "), fg, cell_bg);
            x += w;
        }
    }
}

#[cfg(test)]
mod tests;
