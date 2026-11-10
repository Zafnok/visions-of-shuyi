//! The Playdate's look of the shop screen (ticket 1066), a town's on the
//! world map and a village's in battle: the PC's shop (0409) in the one-bit
//! look of [`crate::screens::playdate`], on the 50 × 15 screen.
//!
//! A window the screen's width over all but its bottom row: the shop's
//! name in the top of the frame on the left, the party's gold on the
//! right; the town (or the unit) and the tabs under it, the shown tab the
//! bar; between two rules, a line per item, its name and its gold, the
//! focused one the bar, one that can't be chosen in grey dots, scrolling
//! with the focus (`▲`/`▼` on the rules where there are more); under the
//! second rule, **what the focused item is** (its numbers, the repair or
//! why the shop won't take it): the PC's lines have no room for it on the
//! Playdate, as the battle's menus show their focused line's description
//! (1038). The help on the bottom row, as on Options (1045). A sale being
//! asked about is a window in the middle: the question, the count between
//! `▲` and `▼` (in grey dots at its end), the gold; the question before
//! selling a special weapon is the shared question window.

use crate::glyph_buffer::{GlyphBuffer, Rect};
use crate::prelude::*;
use crate::screen::Ctx;
use crate::screens::battle::view::{SellConfirmView, ShopScreenView};
use crate::screens::playdate::{
    self as pd, Item, MORE_ABOVE, MORE_BELOW, cells, cut, inks, to_i32, to_usize,
};

/// Rows of item lines between the rules.
pub const LINES: i32 = 8;
/// Cells of a line's gold, right-aligned (`99 999`).
const GOLD_W: usize = 7;

/// The rows of the shop's window from its top: the town and the tabs, the
/// first rule, the first line, the second rule, what the item is.
const TABS: i32 = 1;
const RULE_1: i32 = 2;
const FIRST: i32 = 3;
const RULE_2: i32 = FIRST + LINES;
const DETAIL: i32 = RULE_2 + 1;

/// Paints `view` over the whole of `buf`.
pub fn paint(ctx: &Ctx, view: &ShopScreenView, buf: &mut GlyphBuffer) {
    pd::clear(ctx, buf);
    let bottom = i32::from(buf.height()) - 1;
    let rect = Rect::new(0, 0, i32::from(buf.width()), DETAIL + 2);
    pd::window(ctx, buf, rect, &view.shop);
    let ink = inks(ctx);
    let gold = format!(" {} ", view.gold);
    buf.print(rect.w - 2 - cells(&gold), 0, &gold, ink.text, ink.inside);
    let inner = rect.w - 4;
    buf.print(2, TABS, &cut(&view.unit, 20), ink.text, ink.inside);
    let tabs: i32 = view.tabs.iter().map(|t| cells(t) + 3).sum();
    let mut x = rect.w - 1 - tabs;
    for (i, tab) in view.tabs.iter().enumerate() {
        if i == view.tab {
            pd::bar(ctx, buf, x, TABS, cells(tab) + 2, tab);
        } else {
            buf.print(x + 1, TABS, tab, ink.text, ink.inside);
        }
        x += cells(tab) + 3;
    }
    let first = pd::first_shown(view.lines.len(), to_usize(LINES), Some(view.focus));
    let more_below = first + to_usize(LINES) < view.lines.len();
    rule(ctx, buf, rect, RULE_1, (first > 0).then_some(MORE_ABOVE));
    rule(ctx, buf, rect, RULE_2, more_below.then_some(MORE_BELOW));
    if view.lines.is_empty() {
        buf.print(
            2,
            FIRST,
            &cut(&view.empty, to_usize(inner)),
            ink.off,
            ink.inside,
        );
    }
    let shown = view.lines.iter().enumerate().skip(first);
    for ((i, line), y) in shown.zip(FIRST..RULE_2) {
        let name_w = to_usize(inner) - GOLD_W - 1;
        // check-text: not a data name (the view's own)
        let text = format!(
            "{:<name_w$} {:>GOLD_W$}",
            cut(&line.name, name_w),
            line.gold
        );
        let state = Item::of(i == view.focus, line.enabled);
        pd::item(ctx, buf, (2, y), inner, &text, state);
    }
    if let Some(line) = view.lines.get(view.focus) {
        let detail = cut(&line.detail, to_usize(inner));
        buf.print(2, DETAIL, &detail, ink.text, ink.inside);
    }
    pd::centred(ctx, buf, bottom, &cut(&view.help, usize::from(buf.width())));
    if let Some(sell) = &view.selling {
        paint_sell(ctx, sell, buf);
    }
    if let Some(ask) = &view.special {
        pd::ask_view(ctx, buf, ask);
    }
}

/// A rule across `rect` on its row `y`, joined to its frame, `mark` (if
/// any) at its right end as in a frame.
fn rule(ctx: &Ctx, buf: &mut GlyphBuffer, rect: Rect, y: i32, mark: Option<&str>) {
    let ink = inks(ctx);
    let line = format!("╟{}╢", "─".repeat(to_usize(rect.w - 2)));
    buf.print(rect.x, rect.y + y, &line, ink.frame, ink.inside);
    if let Some(mark) = mark {
        buf.print(rect.x + rect.w - 3, rect.y + y, mark, ink.text, ink.inside);
    }
}

/// The sale `sell` asked about, in a window in the middle of `buf`.
fn paint_sell(ctx: &Ctx, sell: &SellConfirmView, buf: &mut GlyphBuffer) {
    let ink = inks(ctx);
    let count = format!("{} / {}", sell.count, sell.max);
    let up = if sell.count < sell.max {
        ink.text
    } else {
        ink.off
    };
    let down = if sell.count > 1 { ink.text } else { ink.off };
    let lines = [
        (sell.question.as_str(), ink.text),
        ("▲", up),
        (count.as_str(), ink.text),
        ("▼", down),
        (sell.gold.as_str(), ink.text),
    ];
    let widest = lines.iter().map(|(l, _)| cells(l)).max().unwrap_or(0);
    let w = (widest + 4).min(i32::from(buf.width()));
    let h = to_i32(lines.len()) + 2;
    let rect = Rect::new(
        (i32::from(buf.width()) - w) / 2,
        (i32::from(buf.height()) - h) / 2,
        w,
        h,
    );
    pd::window(ctx, buf, rect, "");
    for (y, (line, fg)) in (rect.y + 1..).zip(lines) {
        let shown = cut(line, to_usize(w - 2));
        let x = rect.x + (w - cells(&shown)) / 2;
        buf.print(x, y, &shown, fg, ink.inside);
    }
}

#[cfg(test)]
mod tests;
