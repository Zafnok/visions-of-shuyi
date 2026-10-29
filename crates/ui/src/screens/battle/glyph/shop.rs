//! The glyph look of the shop screen (ticket 0409): a double box over the
//! map and the side panel, the shop's name over the top border on the
//! left and the party's gold on the right; the unit and the tabs under it,
//! the shown tab inverted; then a line per item (name, gold right-aligned,
//! what it is), the highlighted one inverted and scrolled into view, the
//! ones that can't be chosen dimmed. A sale being asked about is a small
//! double box over the list: the question, the count between ▲ and ▼ (an
//! arrow dims at its end), and the gold. The question before selling a
//! special weapon is the engine's question box in the middle. The help row
//! under it, as on the battle screen.

use super::{clear, paint_help};
use crate::color::UiColor;
use crate::glyph_buffer::{BoxStyle, Cell, GlyphBuffer, Rect};
use crate::prelude::*;
use crate::screen::Ctx;
use crate::screens::battle::layout::{MAP_VIEW, SIDE_PANEL};
use crate::screens::battle::view::{SellConfirmView, ShopScreenView};
use crate::widgets::ListCursor;
use crate::widgets::ask::glyph as ask_glyph;

/// The box: the map view and the side panel together.
pub const AREA: Rect = Rect::new(
    MAP_VIEW.x,
    MAP_VIEW.y,
    MAP_VIEW.w + SIDE_PANEL.w,
    MAP_VIEW.h,
);

/// First column of the text in a box at `area`.
pub const fn list_x(area: Rect) -> i32 {
    area.x + 3
}

/// First column of the text.
pub const LIST_X: i32 = list_x(AREA);

/// Row of the unit and the tabs in a box at `area`.
pub const fn tab_row(area: Rect) -> i32 {
    area.y + 2
}

/// Row of the unit and the tabs.
pub const TAB_ROW: i32 = tab_row(AREA);

/// Column of the first tab, from the text's first column `list_x`.
pub const fn tabs_x(list_x: i32) -> i32 {
    list_x + 30
}

/// Column of the first tab.
pub const TABS_X: i32 = tabs_x(LIST_X);

/// Row of the first line in a box at `area`: a blank row under the tabs.
pub const fn top_row(area: Rect) -> i32 {
    area.y + 4
}

/// Row of the first line.
pub const TOP_ROW: i32 = top_row(AREA);

/// Widest item name, its count included, in cells.
pub const NAME_W: i32 = 24;

/// Widest gold, in cells (`99 999`).
pub const GOLD_W: i32 = 7;

/// First column of what the item is, from the text's first column.
pub const fn detail_x(list_x: i32) -> i32 {
    list_x + NAME_W + GOLD_W + 3
}

/// First column of what the item is.
pub const DETAIL_X: i32 = detail_x(LIST_X);

/// Width of a line's highlight in a box at `area`: from a cell left of
/// the text to two cells inside the right border.
pub const fn row_w(area: Rect) -> i32 {
    area.x + area.w - 1 - list_x(area)
}

/// Width of a line's highlight.
pub const ROW_W: i32 = row_w(AREA);

/// Rows the list has in a box at `area`, above the bottom border and a
/// blank row.
pub const fn list_rows(area: Rect) -> i32 {
    area.y + area.h - 2 - top_row(area)
}

/// Rows the list has.
pub const LIST_ROWS: i32 = list_rows(AREA);

/// Paints the whole shop screen `view`: the box and the help row.
pub fn paint(ctx: &Ctx, view: &ShopScreenView, buf: &mut GlyphBuffer) {
    clear(ctx, buf);
    paint_box(ctx, view, AREA, buf);
    paint_help(ctx, &view.help, view.debug_hint.as_ref(), buf);
}

/// Paints `view`'s box at `area`: everything but the help row (the
/// Preparations screen's Shop tab paints it in its own place, 0443).
pub fn paint_box(ctx: &Ctx, view: &ShopScreenView, area: Rect, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let bg = c(UiColor::PanelBg);
    let text = c(UiColor::Text);
    let lit = c(UiColor::TextHighlight);
    let focus_bg = c(UiColor::PanelBorderFocus);
    let (list_x, top_row, rows) = (list_x(area), top_row(area), list_rows(area));
    buf.fill_rect(area, Cell::new(' ', text, bg));
    buf.draw_box(area, BoxStyle::Double, c(UiColor::PanelBorder), bg);
    buf.print(list_x, area.y, &format!(" {} ", view.shop), lit, bg);
    let gold = format!(" {} ", view.gold);
    let gold_x = area.x + area.w - 3 - cells(&gold);
    buf.print(gold_x, area.y, &gold, lit, bg);
    buf.print(list_x, tab_row(area), &view.unit, lit, bg);
    let mut x = tabs_x(list_x);
    for (i, tab) in view.tabs.iter().enumerate() {
        let label = format!(" {tab} ");
        let (fg, tab_bg) = if i == view.tab {
            (bg, focus_bg)
        } else {
            (c(UiColor::TextDim), bg)
        };
        buf.print(x, tab_row(area), &label, fg, tab_bg);
        x += cells(&label) + 1;
    }
    if view.lines.is_empty() {
        buf.print(list_x, top_row, &view.empty, c(UiColor::TextDim), bg);
    }
    let first = first_shown(view.focus, rows);
    let shown = view.lines.iter().enumerate().skip(first);
    for ((i, line), y) in shown.zip(top_row..top_row + rows) {
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
        let row = Rect::new(list_x - 1, y, row_w(area), 1);
        buf.fill_rect(row, Cell::new(' ', fg, row_bg));
        // check-text: not a data name (the view's own)
        buf.print(list_x, y, &line.name, fg, row_bg);
        let gold_x = list_x + NAME_W + GOLD_W - cells(&line.gold);
        buf.print(gold_x, y, &line.gold, fg, row_bg);
        buf.print(detail_x(list_x), y, &line.detail, fg, row_bg);
    }
    if let Some(sell) = &view.selling {
        paint_sell(ctx, sell, sell_box(area), buf);
    }
    if let Some(ask) = &view.special {
        ask_glyph::paint(&ctx.palette, ask, buf);
    }
}

/// Width of the sale's box, in cells.
pub const SELL_W: i32 = 30;

/// Height of the sale's box: the question, ▲, the count, ▼ and the gold,
/// inside its border with a blank row above and below.
pub const SELL_H: i32 = 9;

/// The sale's box in the middle of the list of a box at `area`.
pub const fn sell_box(area: Rect) -> Rect {
    Rect::new(
        area.x + (area.w - SELL_W) / 2,
        top_row(area) + 2,
        SELL_W,
        SELL_H,
    )
}

/// The sale's box, in the middle of the list.
pub const SELL_BOX: Rect = sell_box(AREA);

/// Paints the confirmation `sell` over the list, in `sell_box`.
fn paint_sell(ctx: &Ctx, sell: &SellConfirmView, sell_box: Rect, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let bg = c(UiColor::PanelBg);
    let text = c(UiColor::Text);
    let dim = c(UiColor::TextDim);
    buf.fill_rect(sell_box, Cell::new(' ', text, bg));
    buf.draw_box(sell_box, BoxStyle::Double, c(UiColor::PanelBorderFocus), bg);
    let centred = |buf: &mut GlyphBuffer, y: i32, line: &str, fg| {
        let x = sell_box.x + (sell_box.w - cells(line)) / 2;
        buf.print(x, y, line, fg, bg);
    };
    let top = sell_box.y + 2;
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

    use crate::color::Rgb;
    use crate::console::{ConsoleSizes, ConsoleSpec};
    use crate::screens::battle::view::ShopLineView;

    /// A view of `n` lines named `l0`, `l1`, …, the `focus`ed one
    /// highlighted.
    fn view(n: usize, focus: usize) -> ShopScreenView {
        let line = |i: usize| ShopLineView {
            name: format!("l{i}"),
            gold: "1 250".into(),
            detail: "Mt 5".into(),
            enabled: i != 1,
        };
        ShopScreenView {
            shop: "Armory".into(),
            gold: "Gold 1 250".into(),
            unit: "Lord".into(),
            tabs: vec!["Buy".into(), "Sell".into()],
            tab: 1,
            lines: (0..n).map(line).collect(),
            focus,
            empty: "Nothing here.".into(),
            selling: None,
            special: None,
            help: "help".into(),
            debug_hint: None,
        }
    }

    fn painted(view: &ShopScreenView) -> GlyphBuffer {
        let c = crate::screen::tests::ctx();
        let blank = Cell::new(' ', Rgb::new(0, 0, 0), Rgb::new(0, 0, 0));
        let mut buf = GlyphBuffer::with_spec(ConsoleSpec::PC, blank);
        paint(&c, view, &mut buf);
        buf
    }

    /// The text of row `y` from column `x`, `n` cells.
    fn text(buf: &GlyphBuffer, x: i32, y: i32, n: i32) -> String {
        (x..x + n)
            .map(|x| buf.get(x, y).map_or(' ', |c| c.glyph))
            .collect()
    }

    #[test]
    fn the_tabs_lines_and_gold_go_in_their_columns() {
        let buf = painted(&view(3, 2));
        assert_eq!(text(&buf, TABS_X, TAB_ROW, 12), " Buy   Sell ");
        assert_eq!((TAB_ROW, TOP_ROW, ROW_W), (2, 4, 96));
        assert_eq!(text(&buf, LIST_X, TOP_ROW, 2), "l0");
        // The gold right-aligned, then the detail.
        let gold_end = LIST_X + NAME_W + GOLD_W;
        assert_eq!(text(&buf, gold_end - 5, TOP_ROW, 5), "1 250");
        assert_eq!(text(&buf, DETAIL_X, TOP_ROW, 4), "Mt 5");
        // The gold over the top border, two cells in from its corner.
        let top = text(&buf, AREA.x + AREA.w - 15, AREA.y, 15);
        assert_eq!(top, " Gold 1 250 ══╗");
    }

    #[test]
    fn the_highlight_fills_the_row_inside_the_border() {
        let c = crate::screen::tests::ctx();
        let focus = c.palette.get(UiColor::PanelBorderFocus);
        let buf = painted(&view(3, 2));
        let y = TOP_ROW + 2;
        let bg = |x| buf.get(x, y).map(|c| c.bg);
        assert_eq!(bg(LIST_X - 1), Some(focus));
        assert_eq!(bg(AREA.x + AREA.w - 3), Some(focus));
        assert_ne!(bg(AREA.x + AREA.w - 2), Some(focus));
        assert_ne!(bg(LIST_X - 2), Some(focus));
        // A line that can't be chosen is dimmed.
        let dim = c.palette.get(UiColor::TextDim);
        assert_eq!(buf.get(LIST_X, TOP_ROW + 1).map(|c| c.fg), Some(dim));
        assert_ne!(buf.get(LIST_X, TOP_ROW).map(|c| c.fg), Some(dim));
    }

    #[test]
    fn a_long_list_scrolls_to_its_focus() {
        let rows = usize::try_from(LIST_ROWS).unwrap();
        let buf = painted(&view(40, 39));
        let last = TOP_ROW + LIST_ROWS - 1;
        assert_eq!(last, 27);
        assert_eq!(text(&buf, LIST_X, last, 3), "l39");
        // Nothing under it: a blank row, then the border.
        assert_eq!(text(&buf, LIST_X, last + 1, 3), "   ");
        assert_eq!(text(&buf, LIST_X, last + 2, 3), "═══");
        // At the top of the list, the lines past the box stay unshown.
        let top = painted(&view(40, 0));
        assert_eq!(text(&top, LIST_X, last, 3), format!("l{}", rows - 1));
        assert_eq!(text(&top, LIST_X, last + 1, 3), "   ");
        assert_eq!(text(&top, LIST_X, last + 2, 3), "═══");
        let first = 40 - rows;
        assert_eq!(text(&buf, LIST_X, TOP_ROW, 3), format!("l{first}"));
        assert_eq!(first_shown(0, 5), 0);
        assert_eq!(first_shown(4, 5), 0);
        assert_eq!(first_shown(5, 5), 1);
        assert_eq!(first_shown(9, 5), 5);
    }

    /// An arrow dims at its end: ▼ at 1, ▲ at all of them.
    #[test]
    fn the_sale_arrows_dim_at_the_ends() {
        let c = crate::screen::tests::ctx();
        let dim = c.palette.get(UiColor::TextDim);
        let arrows = |count: usize| {
            let mut v = view(1, 0);
            v.selling = Some(SellConfirmView {
                question: "Sell?".into(),
                count,
                max: 3,
                gold: "For 1 gold".into(),
            });
            let buf = painted(&v);
            let x = SELL_BOX.x + (SELL_BOX.w - 1) / 2;
            let fg = |y| buf.get(x, y).map(|c| (c.glyph, c.fg == dim));
            (fg(SELL_BOX.y + 3), fg(SELL_BOX.y + 5))
        };
        assert_eq!(arrows(1), (Some(('▲', false)), Some(('▼', true))));
        assert_eq!(arrows(2), (Some(('▲', false)), Some(('▼', false))));
        assert_eq!(arrows(3), (Some(('▲', true)), Some(('▼', false))));
    }

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
