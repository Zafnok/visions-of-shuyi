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
use crate::color::UiColor;
use crate::glyph_buffer::{BoxStyle, Cell, GlyphBuffer, Rect};
use crate::screen::Ctx;
use crate::screens::print_centred;

/// Shown for an empty slot.
const EMPTY: &str = "-";

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
/// Row of what the battle starts with (Fight tab).
const READY_ROW: i32 = 14;
/// Row of the message.
const MESSAGE_ROW: i32 = 29;

/// The first row shown of a list of `len` in a window of `rows` rows, so
/// that row `focus` is in it.
fn window_start(focus: usize, len: usize, rows: usize) -> usize {
    (focus + 1)
        .saturating_sub(rows)
        .min(len.saturating_sub(rows))
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
        PrepBody::Options => {}
        PrepBody::Fight(ready) => {
            print_centred(buf, READY_ROW, ready, c(UiColor::Text), black);
        }
    }
    // The question takes the message's place.
    if let Some(message) = view.leave_question.as_ref().or(view.message.as_ref()) {
        print_centred(buf, MESSAGE_ROW, message, c(UiColor::TextHighlight), black);
    }
    let bottom = i32::from(buf.height()) - 1;
    print_centred(buf, bottom, &view.help, c(UiColor::TextDim), black);
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
        x += i32::try_from(label.chars().count()).unwrap_or(0) + 4;
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
    let c = |u| ctx.palette.get(u);
    let bg = c(UiColor::PanelBg);
    let focused = list.state == ListState::Focused;
    let border = if focused {
        UiColor::PanelBorderFocus
    } else {
        UiColor::PanelBorder
    };
    buf.fill_rect(rect, Cell::new(' ', c(UiColor::Text), bg));
    buf.draw_box(rect, BoxStyle::Single, c(border), bg);
    let title = format!(" {} ", list.title);
    buf.print(rect.x + 2, rect.y, &title, c(UiColor::TextHighlight), bg);

    let height = usize::try_from(rect.h - 2).unwrap_or(0);
    let start = window_start(list.cursor, list.rows.len(), height);
    let shown = list.rows.iter().enumerate().skip(start).take(height);
    for (y, (i, item)) in (rect.y + 1..).zip(shown) {
        let (text, enabled) = row(item);
        let (fg, row_bg) = match (i == list.cursor, enabled) {
            (true, _) if focused => (bg, c(UiColor::PanelBorderFocus)),
            (_, false) => (c(UiColor::TextDim), bg),
            (true, true) if list.state == ListState::Chosen => (c(UiColor::TextHighlight), bg),
            (_, true) => (c(UiColor::Text), bg),
        };
        let line = Rect::new(rect.x + 1, y, rect.w - 2, 1);
        buf.fill_rect(line, Cell::new(' ', fg, row_bg));
        buf.print(rect.x + 2, y, &text, fg, row_bg);
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

/// The chosen unit's slots, with its attack speed under them, and the
/// stock.
fn paint_gear(ctx: &Ctx, gear: &GearView, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let bg = c(UiColor::PanelBg);
    paint_list(ctx, buf, SLOTS_BOX, &gear.slots, |slot: &SlotRowView| {
        let (label, wear) = (&slot.label, &slot.wear);
        let name = slot.item.as_deref().unwrap_or(EMPTY);
        (format!("{label:<9}  {name:<15} {wear}"), true)
    });
    if let Some(speed) = &gear.speed {
        let y = SLOTS_BOX.y + 2 + i32::try_from(gear.slots.rows.len()).unwrap_or(0);
        buf.print(SLOTS_BOX.x + 2, y, speed, c(UiColor::Text), bg);
        if let Some(note) = &gear.benched_note {
            buf.print(SLOTS_BOX.x + 2, y + 2, note, c(UiColor::TextDim), bg);
        }
    }
    paint_list(ctx, buf, STOCK_BOX, &gear.stock, |item: &ItemRowView| {
        // check-text: not a data name (the view's own)
        (format!("{:<15} {}", item.name, item.detail), item.usable)
    });
}

/// One consumable row: `Potion ×4  Restore 10 HP`.
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
