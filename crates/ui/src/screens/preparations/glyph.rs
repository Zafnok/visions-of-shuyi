//! The glyph look of the Preparations screen: the heading and the tabs on
//! one row (the shown tab lit, as a bar while the cursor is on the tabs),
//! the shown tab's lists as titled boxes side by side (the one with the
//! cursor has a lit border and its row as a bar), the message and the help
//! line at the bottom. Everything about the look lives here; nothing here
//! decides what the screen shows. Another look is another `paint` of the
//! same view (ADR-0054).

use super::view::{
    ConsumableRowView, GearView, ItemRowView, ListState, ListView, LoadoutsView, PackView,
    PrepBody, PreparationsView, SlotRowView,
};
use crate::color::{Rgb, UiColor};
use crate::glyph_buffer::{BoxStyle, Cell, GlyphBuffer, Layer, PxRect, Rect, Sprite};
use crate::prelude::*;
use crate::screen::Ctx;
use crate::screens::battle::glyph::shop as shop_glyph;
use crate::screens::print_centred;
use crate::widgets::ListCursor;
use shuyi_ui::screens::print_centred_styled;
use trpg_content::Portrait;

/// Shown for an empty slot.
const EMPTY: &str = "-";
/// Shown at the stock's top right while rows are hidden above.
const MORE_ABOVE: &str = "▲";
/// Shown at the stock's bottom right while rows are hidden below.
const MORE_BELOW: &str = "▼";
/// The cells a holder's face takes on a swap row, and so the row's height.
const FACE: (i32, i32) = (6, 3);
/// A holder's face's side in pixels: three quarters of a bought 64×64
/// face (Nick, ticket 0479).
const FACE_PX: i32 = 48;
/// The expression a holder's face shows.
const FACE_EXPRESSION: &str = "neutral";

/// Row of the heading and the tabs.
const TABS_ROW: i32 = 1;
/// Column of the first tab.
const TABS_X: i32 = 30;
/// The units list.
const UNITS_BOX: Rect = Rect::new(0, 3, 16, 25);
/// The chosen unit's slots.
const SLOTS_BOX: Rect = Rect::new(16, 3, 36, 25);
/// The stock for the chosen slot.
const STOCK_BOX: Rect = Rect::new(52, 3, 48, 25);
/// The stock's consumables (Pack tab).
const SPARE_BOX: Rect = Rect::new(8, 3, 40, 25);
/// The pack (Pack tab).
const PACK_BOX: Rect = Rect::new(52, 3, 40, 25);
/// The shop (Shop tab).
pub const SHOP_BOX: Rect = Rect::new(0, 3, 100, 25);
/// Row of what the battle starts with (Fight tab).
const READY_ROW: i32 = 14;
/// Row of the message.
const MESSAGE_ROW: i32 = 29;

/// The first row shown of a list of `len` in a window of `rows` rows, so
/// that row `focus` is in it.
fn window_start(focus: usize, len: usize, rows: usize) -> usize {
    ListCursor::new(len).at(focus).window(rows).start
}

/// Paints the Preparations screen over the whole of `buf`.
pub fn paint(ctx: &Ctx, view: &PreparationsView, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let black = c(UiColor::Black);
    buf.fill_rect(buf.bounds(), Cell::new(' ', c(UiColor::Text), black));
    paint_tabs(ctx, view, buf);
    match &view.body {
        PrepBody::Loadouts(loadouts) => paint_loadouts(ctx, loadouts, buf),
        PrepBody::Pack(pack) => paint_pack(ctx, pack, buf),
        PrepBody::Shop(shop) => shop_glyph::paint_box(ctx, shop, SHOP_BOX, buf),
        PrepBody::Options => {}
        PrepBody::Fight(ready) => {
            print_centred(buf, READY_ROW, ready, c(UiColor::Text), black);
        }
    }
    // The question takes the message's place.
    let question = view.leave_question.as_ref().map(|ask| &ask.text);
    if let Some(message) = question.or(view.message.as_ref()) {
        print_centred(buf, MESSAGE_ROW, message, c(UiColor::TextHighlight), black);
    }
    let bottom = i32::from(buf.height()) - 1;
    print_centred_styled(
        buf,
        bottom,
        &view.help,
        ctx.palette.text_colors_on(c(UiColor::TextDim), black),
    );
}

/// The heading and the tabs: the shown one lit, as a bar while the cursor
/// is on the tabs.
fn paint_tabs(ctx: &Ctx, view: &PreparationsView, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let black = c(UiColor::Black);
    buf.print(2, TABS_ROW, &view.heading, c(UiColor::TextHighlight), black);
    let mut x = TABS_X;
    for tab in &view.tabs {
        let label = format!(" {} ", tab.label);
        let (fg, bg) = match (tab.shown, view.on_tabs) {
            (true, true) => (c(UiColor::PanelBg), c(UiColor::PanelBorderFocus)),
            (true, false) => (c(UiColor::TextHighlight), black),
            (false, _) => (c(UiColor::Text), black),
        };
        buf.print(x, TABS_ROW, &label, fg, bg);
        x += buf.text_cells(&label) + 4;
    }
}

/// A list in `rect`: its titled box, the border lit while it has the
/// cursor, and its rows as `row` says them (`(text, enabled)`), scrolled so
/// the cursor's row shows. That row is a bar while the cursor is in the
/// list and lit while it is the one chosen; disabled rows are dim.
fn paint_list<Row>(
    ctx: &Ctx,
    buf: &mut GlyphBuffer,
    rect: Rect,
    list: &ListView<Row>,
    row: impl Fn(&Row) -> (String, bool),
) {
    paint_frame(ctx, buf, rect, list);
    let height = usize::try_from(rect.h - 2).unwrap_or(0);
    let start = window_start(list.cursor, list.rows.len(), height);
    let shown = list.rows.iter().enumerate().skip(start).take(height);
    for (y, (i, item)) in (rect.y + 1..).zip(shown) {
        let (text, enabled) = row(item);
        let (fg, row_bg) = row_colours(ctx, list, i, enabled);
        let line = Rect::new(rect.x + 1, y, rect.w - 2, 1);
        buf.fill_rect(line, Cell::new(' ', fg, row_bg));
        buf.print(rect.x + 2, y, &text, fg, row_bg);
    }
}

/// `list`'s box in `rect`, emptied: the border lit while it has the
/// cursor, the title on the top border.
fn paint_frame<Row>(ctx: &Ctx, buf: &mut GlyphBuffer, rect: Rect, list: &ListView<Row>) {
    let c = |u| ctx.palette.get(u);
    let bg = c(UiColor::PanelBg);
    let border = if list.state == ListState::Focused {
        UiColor::PanelBorderFocus
    } else {
        UiColor::PanelBorder
    };
    buf.fill_rect(rect, Cell::new(' ', c(UiColor::Text), bg));
    buf.draw_box(rect, BoxStyle::Single, c(border), bg);
    let title = format!(" {} ", list.title);
    buf.print(rect.x + 2, rect.y, &title, c(UiColor::TextHighlight), bg);
}

/// The text and background colours of row `i` of `list` (`enabled` or
/// not): a bar on the cursor's row while the list has the cursor, lit on
/// the chosen one, dim when disabled.
fn row_colours<Row>(ctx: &Ctx, list: &ListView<Row>, i: usize, enabled: bool) -> (Rgb, Rgb) {
    let c = |u| ctx.palette.get(u);
    let bg = c(UiColor::PanelBg);
    match (i == list.cursor, enabled) {
        (true, _) if list.state == ListState::Focused => (bg, c(UiColor::PanelBorderFocus)),
        (_, false) => (c(UiColor::TextDim), bg),
        (true, true) if list.state == ListState::Chosen => (c(UiColor::TextHighlight), bg),
        (_, true) => (c(UiColor::Text), bg),
    }
}

/// The Loadouts tab: units, the chosen unit's slots and attack speed, and
/// the stock for the chosen slot. Units left out of the battle are dim.
fn paint_loadouts(ctx: &Ctx, view: &LoadoutsView, buf: &mut GlyphBuffer) {
    paint_list(ctx, buf, UNITS_BOX, &view.units, |u| {
        // check-text: not a data name (the view's own)
        (u.name.clone(), u.deployed)
    });
    if let Some(gear) = &view.gear {
        paint_gear(ctx, gear, buf);
    }
}

/// The chosen unit's slots, with the highlighted item's numbers and its
/// attack speed under them, and the stock.
fn paint_gear(ctx: &Ctx, gear: &GearView, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let bg = c(UiColor::PanelBg);
    paint_list(ctx, buf, SLOTS_BOX, &gear.slots, |slot: &SlotRowView| {
        let (label, wear) = (&slot.label, &slot.wear);
        let name = slot.item.as_deref().unwrap_or(EMPTY);
        (format!("{label:<9}  {name:<15} {wear}"), true)
    });
    let mut y = SLOTS_BOX.y + 2 + i32::try_from(gear.slots.rows.len()).unwrap_or(0);
    if let Some(detail) = &gear.item_detail {
        let width = usize::try_from(SLOTS_BOX.w - 4).unwrap_or(0);
        for line in split_at_gaps(detail, width) {
            buf.print(SLOTS_BOX.x + 2, y, &line, c(UiColor::TextHighlight), bg);
            y += 1;
        }
    }
    if let Some(speed) = &gear.speed {
        buf.print(SLOTS_BOX.x + 2, y, speed, c(UiColor::Text), bg);
        y += 1;
    }
    if let Some(note) = &gear.benched_note {
        buf.print(SLOTS_BOX.x + 2, y + 1, note, c(UiColor::TextDim), bg);
    }
    paint_stock(ctx, buf, &gear.stock);
}

/// The console rows `item` takes: [`FACE`]'s when its holder has a face,
/// else one.
fn stock_row_height(item: &ItemRowView) -> i32 {
    let face = item.holder.as_ref().is_some_and(|h| h.portrait.is_some());
    if face { FACE.1 } else { 1 }
}

/// The first of the rows `heights` shown in `inner` console rows so that
/// row `focus` shows: the top row while it fits, else the first start
/// from which it does (so the window moves only once the cursor leaves
/// it, scrolling down).
fn stock_window_start(heights: &[i32], focus: usize, inner: i32) -> usize {
    let focus = focus.min(heights.len().saturating_sub(1));
    let taken = |start: usize| {
        heights
            .get(start..=focus)
            .map_or(0, |h| h.iter().sum::<i32>())
    };
    (0..focus)
        .find(|&start| taken(start) <= inner)
        .unwrap_or(focus)
}

/// The stock for the chosen slot, as [`paint_list`] paints a list, but a
/// row of another unit's item starts with its holder: their face, the row
/// [`FACE`] tall, or without one their map letters in the player's colour.
/// `▲` and `▼` at the right edge show rows hidden above and below.
fn paint_stock(ctx: &Ctx, buf: &mut GlyphBuffer, list: &ListView<ItemRowView>) {
    let rect = STOCK_BOX;
    paint_frame(ctx, buf, rect, list);
    let c = |u| ctx.palette.get(u);
    let bottom = rect.y + rect.h - 1;
    let heights: Vec<i32> = list.rows.iter().map(stock_row_height).collect();
    let start = stock_window_start(&heights, list.cursor, rect.h - 2);
    let (mut y, mut end) = (rect.y + 1, start);
    for ((i, item), h) in list.rows.iter().enumerate().zip(&heights).skip(start) {
        if y + h > bottom {
            break;
        }
        let (fg, row_bg) = row_colours(ctx, list, i, item.usable);
        let bar = Rect::new(rect.x + 1, y, rect.w - 2, *h);
        buf.fill_rect(bar, Cell::new(' ', fg, row_bg));
        // check-text: not a data name (the view's own)
        let text = format!("{:<15} {}", item.name, item.detail);
        let holder = item.holder.as_ref();
        let portraits = &ctx.game.content.portraits;
        let face = holder.and_then(|h| portraits.get(h.portrait.as_deref()?));
        match (face, holder) {
            (Some(art), _) => {
                paint_face(buf, (rect.x + 2, y), art);
                buf.print(rect.x + 3 + FACE.0, y + 1, &text, fg, row_bg);
            }
            (None, Some(holder)) => {
                let on_bar = i == list.cursor && list.state == ListState::Focused;
                let label_fg = if on_bar { fg } else { c(UiColor::Player) };
                buf.print(rect.x + 2, y, &holder.label, label_fg, row_bg);
                buf.print(rect.x + 5, y, &text, fg, row_bg);
            }
            (None, None) => {
                buf.print(rect.x + 2, y, &text, fg, row_bg);
            }
        }
        y += h;
        end = i + 1;
    }
    let (dim, bg) = (c(UiColor::TextDim), c(UiColor::PanelBg));
    let mark_x = rect.x + rect.w - 2;
    if start > 0 {
        buf.print(mark_x, rect.y + 1, MORE_ABOVE, dim, bg);
    }
    if end < list.rows.len() {
        buf.print(mark_x, bottom - 1, MORE_BELOW, dim, bg);
    }
}

/// The size a face of `(w, h)` pixels is drawn at: as big as fits
/// [`FACE_PX`] square, keeping its shape (a bought 64×64 face at three
/// quarters).
fn face_size((w, h): (i32, i32)) -> (i32, i32) {
    let side = w.max(h).max(1);
    (w * FACE_PX / side, h * FACE_PX / side)
}

/// `art`'s face at [`face_size`], one sprite under the glyphs whose
/// top-left cell is `at`. Nothing without the expression.
fn paint_face(buf: &mut GlyphBuffer, at: (i32, i32), art: &Portrait) {
    let Some(expression) = art.expression(FACE_EXPRESSION) else {
        return;
    };
    let size = expression.size;
    let (Ok(w), Ok(h)) = (i32::try_from(size.width), i32::try_from(size.height)) else {
        return;
    };
    let (cw, ch) = buf.spec().cell_px();
    let src = PxRect::new(0, 0, w, h);
    let (dw, dh) = face_size((w, h));
    let dest = PxRect::new(at.0 * cw, at.1 * ch, dw, dh);
    buf.add_sprite(Sprite::new(expression.image, src, dest, Layer::Under));
}

/// `line` cut into lines of at most `width` characters at its two-space
/// gaps (`Mt 5  Hit 90  …`), so no number is split from its label.
fn split_at_gaps(line: &str, width: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for part in line.split("  ") {
        match lines.last_mut() {
            Some(last) if last.chars().count() + 2 + part.chars().count() <= width => {
                last.push_str("  ");
                last.push_str(part);
            }
            _ => lines.push(part.to_owned()),
        }
    }
    lines
}

/// One consumable row:`Potion ×4  Restore 10 HP`.
fn consumable_row(row: &ConsumableRowView) -> (String, bool) {
    // check-text: not a data name (the view's own)
    let (name, count, effect) = (&row.name, row.count, &row.effect);
    (format!("{name:<15} ×{count:<2}  {effect}"), true)
}

/// The Pack tab: the stock's consumables and the pack.
fn paint_pack(ctx: &Ctx, view: &PackView, buf: &mut GlyphBuffer) {
    paint_list(ctx, buf, SPARE_BOX, &view.spare, consumable_row);
    paint_list(ctx, buf, PACK_BOX, &view.pack, consumable_row);
}

#[cfg(test)]
mod tests;
