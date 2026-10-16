//! The glyph look of the name boxes: the typing box and the letter grid,
//! each a double-bordered box with its title in the top border and the
//! name so far with a cursor. Everything about the look lives here;
//! nothing here decides what a box shows (ADR-0054).

use super::{GridView, TypingView};
use crate::color::UiColor;
use crate::glyph_buffer::{BoxStyle, Cell, GlyphBuffer, Rect};
use crate::prelude::*;
use crate::screen::Ctx;

/// Cells `text` takes, as a coordinate.
fn cells(text: &str) -> i32 {
    i32::try_from(text.chars().count()).unwrap_or(0)
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

/// The typing box at `rect` (seven rows tall): the name so far with a
/// cursor, and the hint to type.
pub fn paint_typing(ctx: &Ctx, buf: &mut GlyphBuffer, rect: Rect, typing: &TypingView) {
    let c = |u| ctx.palette.get(u);
    let bg = c(UiColor::PanelBg);
    paint_box(ctx, buf, rect, &typing.title);
    let shown = format!("{}_", typing.typed);
    let x = centred_in(rect, &shown);
    buf.print(x, rect.y + 2, &shown, c(UiColor::Text), bg);
    let hint = &typing.hint;
    let x = centred_in(rect, hint);
    buf.print(x, rect.y + 4, hint, c(UiColor::TextDim), bg);
}

/// The height of the grid's box for `grid`: the border, the name, and two
/// rows per row of cells.
pub fn grid_height(grid: &GridView) -> i32 {
    4 + 2 * i32::try_from(grid.rows.len()).unwrap_or(0)
}

/// The name grid's box from `rect`'s top left, `rect.w` wide and as tall
/// as [`grid_height`]: the name so far with a cursor, then the grid, the
/// focused cell as a lit bar.
pub fn paint_grid(ctx: &Ctx, buf: &mut GlyphBuffer, rect: Rect, grid: &GridView) {
    let c = |u| ctx.palette.get(u);
    let bg = c(UiColor::PanelBg);
    let rect = Rect::new(rect.x, rect.y, rect.w, grid_height(grid));
    paint_box(ctx, buf, rect, &grid.title);
    let shown = format!("{}_", grid.spelled);
    let x = centred_in(rect, &shown);
    buf.print(x, rect.y + 2, &shown, c(UiColor::Text), bg);
    for (r, labels) in grid.rows.iter().enumerate() {
        let y = rect.y + 4 + i32::try_from(r * 2).unwrap_or(0);
        // Each cell is its label with a space either side.
        let widths: Vec<i32> = labels.iter().map(|l| cells(l) + 2).collect();
        let total: i32 = widths.iter().sum();
        let mut x = rect.x + (rect.w - total) / 2;
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
