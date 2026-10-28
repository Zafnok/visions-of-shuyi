//! The glyph look of the shop screen (ticket 0409): a double box over the
//! map and the side panel, the shop's name over the top border on the
//! left and the party's gold on the right; the unit and the tabs under it,
//! the shown tab inverted; then a line per item (name, gold right-aligned,
//! what it is), the highlighted one inverted and scrolled into view, the
//! ones that can't be chosen dimmed. A sale being asked about is a small
//! double box over the list: the question, the count between ▲ and ▼ (an
//! arrow dims at its end), and the gold. The help row under it, as on the
//! battle screen.

use super::{clear, paint_help};
use crate::color::UiColor;
use crate::glyph_buffer::{BoxStyle, Cell, GlyphBuffer, Rect};
use crate::prelude::*;
use crate::screen::Ctx;
use crate::screens::battle::layout::{MAP_VIEW, SIDE_PANEL};
use crate::screens::battle::view::{SellConfirmView, ShopScreenView};
use crate::widgets::ListCursor;

/// The box: the map view and the side panel together.
pub const AREA: Rect = Rect::new(
    MAP_VIEW.x,
    MAP_VIEW.y,
    MAP_VIEW.w + SIDE_PANEL.w,
    MAP_VIEW.h,
);

/// First column of the text.
pub const LIST_X: i32 = AREA.x + 3;

/// Row of the unit and the tabs.
pub const TAB_ROW: i32 = AREA.y + 2;

/// Column of the first tab.
pub const TABS_X: i32 = LIST_X + 30;

/// Row of the first line.
pub const TOP_ROW: i32 = TAB_ROW + 2;

/// Widest item name, its count included, in cells.
pub const NAME_W: i32 = 24;

/// Widest gold, in cells (`99 999`).
pub const GOLD_W: i32 = 7;

/// First column of what the item is.
pub const DETAIL_X: i32 = LIST_X + NAME_W + GOLD_W + 3;

/// Rows the list has, above the bottom border and a blank row.
pub const LIST_ROWS: i32 = AREA.y + AREA.h - 2 - TOP_ROW;

/// Paints the whole shop screen `view`: the box and the help row.
pub fn paint(ctx: &Ctx, view: &ShopScreenView, buf: &mut GlyphBuffer) {
    clear(ctx, buf);
    let c = |u| ctx.palette.get(u);
    let bg = c(UiColor::PanelBg);
    let text = c(UiColor::Text);
    let lit = c(UiColor::TextHighlight);
    let focus_bg = c(UiColor::PanelBorderFocus);
    buf.fill_rect(AREA, Cell::new(' ', text, bg));
    buf.draw_box(AREA, BoxStyle::Double, c(UiColor::PanelBorder), bg);
    buf.print(LIST_X, AREA.y, &format!(" {} ", view.shop), lit, bg);
    let gold = format!(" {} ", view.gold);
    let gold_x = AREA.x + AREA.w - 3 - cells(&gold);
    buf.print(gold_x, AREA.y, &gold, lit, bg);
    buf.print(LIST_X, TAB_ROW, &view.unit, lit, bg);
    let mut x = TABS_X;
    for (i, tab) in view.tabs.iter().enumerate() {
        let label = format!(" {tab} ");
        let (fg, tab_bg) = if i == view.tab {
            (bg, focus_bg)
        } else {
            (c(UiColor::TextDim), bg)
        };
        buf.print(x, TAB_ROW, &label, fg, tab_bg);
        x += cells(&label) + 1;
    }
    if view.lines.is_empty() {
        buf.print(LIST_X, TOP_ROW, &view.empty, c(UiColor::TextDim), bg);
    }
    let first = first_shown(view.focus, LIST_ROWS);
    let shown = view.lines.iter().enumerate().skip(first);
    for ((i, line), y) in shown.zip(TOP_ROW..TOP_ROW + LIST_ROWS) {
        let fg = if line.enabled {
            text
        } else {
            c(UiColor::TextDim)
        };
        let (fg, row_bg) = if i == view.focus {
            (bg, focus_bg)
        } else {
            (fg, bg)
        };
        let row = Rect::new(LIST_X - 1, y, AREA.w - 2 * (LIST_X - AREA.x) + 2, 1);
        buf.fill_rect(row, Cell::new(' ', fg, row_bg));
        // check-text: not a data name (the view's own)
        buf.print(LIST_X, y, &line.name, fg, row_bg);
        let gold_x = LIST_X + NAME_W + GOLD_W - cells(&line.gold);
        buf.print(gold_x, y, &line.gold, fg, row_bg);
        buf.print(DETAIL_X, y, &line.detail, fg, row_bg);
    }
    if let Some(sell) = &view.selling {
        paint_sell(ctx, sell, buf);
    }
    paint_help(ctx, &view.help, view.debug_hint.as_ref(), buf);
}

/// Width of the sale's box, in cells.
pub const SELL_W: i32 = 30;

/// Height of the sale's box: the question, ▲, the count, ▼ and the gold,
/// inside its border with a blank row above and below.
pub const SELL_H: i32 = 9;

/// The sale's box, in the middle of the list.
pub const SELL_BOX: Rect = Rect::new(AREA.x + (AREA.w - SELL_W) / 2, TOP_ROW + 2, SELL_W, SELL_H);

/// Paints the confirmation `sell` over the list.
fn paint_sell(ctx: &Ctx, sell: &SellConfirmView, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let bg = c(UiColor::PanelBg);
    let text = c(UiColor::Text);
    let dim = c(UiColor::TextDim);
    buf.fill_rect(SELL_BOX, Cell::new(' ', text, bg));
    buf.draw_box(SELL_BOX, BoxStyle::Double, c(UiColor::PanelBorderFocus), bg);
    let centred = |buf: &mut GlyphBuffer, y: i32, line: &str, fg| {
        let x = SELL_BOX.x + (SELL_BOX.w - cells(line)) / 2;
        buf.print(x, y, line, fg, bg);
    };
    let top = SELL_BOX.y + 2;
    centred(buf, top, &sell.question, c(UiColor::TextHighlight));
    let up = if sell.count < sell.max { text } else { dim };
    let down = if sell.count > 1 { text } else { dim };
    centred(buf, top + 1, "▲", up);
    centred(
        buf,
        top + 2,
        &format!("{} / {}", sell.count, sell.max),
        text,
    );
    centred(buf, top + 3, "▼", down);
    centred(buf, top + 4, &sell.gold, text);
}

/// How many cells `text` takes.
fn cells(text: &str) -> i32 {
    i32::try_from(text.chars().count()).unwrap_or(0)
}

/// The first line shown so that `focus` is within `rows` rows.
fn first_shown(focus: usize, rows: i32) -> usize {
    let rows = usize::try_from(rows).unwrap_or(1);
    ListCursor::new(focus + 1).at(focus).window(rows).start
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sale_box_sits_over_the_list() {
        const { assert!(SELL_BOX.y >= TOP_ROW) };
        const { assert!(SELL_BOX.y + SELL_BOX.h <= TOP_ROW + LIST_ROWS) };
        assert_eq!(SELL_BOX.x + SELL_BOX.w / 2, AREA.x + AREA.w / 2);
    }

    #[test]
    fn the_columns_fit_in_the_box() {
        assert_eq!(AREA, Rect::new(0, 0, 100, 30));
        assert_eq!(TOP_ROW, 4);
        assert_eq!(LIST_ROWS, 24);
        // The widest weapon numbers fit inside the right border.
        let numbers = "Mt 10  Hit 100  Crit 10  Rng 1-2  Wt 10";
        assert!(DETAIL_X + cells(numbers) < AREA.x + AREA.w - 1);
    }
}
