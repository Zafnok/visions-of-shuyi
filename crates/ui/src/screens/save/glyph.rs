//! The glyph look of the save screens: the save prompt as a centred
//! question over a menu box, and the slot picker as a table in one panel
//! (slot, chapter, mode, army, playtime) with the focused row as a bar and
//! the overwrite question in a double-bordered box over it. Everything
//! about the look lives here; nothing here decides what the screens show.
//! Another look is another `paint` of the same view (ADR-0054).

use super::view::{
    OverwriteView, SavePromptView, SlotColumnsView, SlotEntryView, SlotPickerView, SlotView,
};
use crate::color::UiColor;
use crate::glyph_buffer::{BoxStyle, Cell, GlyphBuffer, Rect};
use crate::screen::Ctx;
use crate::screens::{centre_x, print_centred};
use crate::widgets::menu::glyph as menu_glyph;

/// Row of the question.
const QUESTION_ROW: i32 = 10;
/// Top row of the question's menu.
const MENU_ROW: i32 = 13;

/// The slot picker's panel, in cells.
const PANEL: Rect = Rect::new(2, 1, 96, 26);
/// Row of the column headings.
const HEADING_ROW: i32 = PANEL.y + 2;
/// Row of the first slot shown.
const FIRST_ROW: i32 = PANEL.y + 4;
/// Column of the slot numbers.
const SLOT_X: i32 = PANEL.x + 3;
/// Column of the chapter titles.
const CHAPTER_X: i32 = SLOT_X + 6;
/// Cells a chapter title may take.
const CHAPTER_W: usize = 48;
/// Column of the game modes.
const MODE_X: i32 = CHAPTER_X + 50;
/// Column of the army sizes.
const ARMY_X: i32 = MODE_X + 10;
/// Column where the playtimes end (they are right-aligned).
const TIME_END_X: i32 = PANEL.x + PANEL.w - 3;
/// Row of the message under the panel.
const MESSAGE_ROW: i32 = PANEL.y + PANEL.h + 1;
/// Height of the overwrite question's box.
const OVERWRITE_H: i32 = 6;

/// Cells `text` takes, as a coordinate.
fn cells(text: &str) -> i32 {
    i32::try_from(text.chars().count()).unwrap_or(0)
}

/// Paints the save prompt over the whole of `buf`.
pub fn paint_prompt(ctx: &Ctx, view: &SavePromptView, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let black = c(UiColor::Black);
    buf.fill_rect(buf.bounds(), Cell::new(' ', c(UiColor::Text), black));
    let highlight = c(UiColor::TextHighlight);
    print_centred(buf, QUESTION_ROW, &view.question, highlight, black);
    let (w, _) = menu_glyph::size(&view.menu);
    let x = centre_x(buf, usize::try_from(w).unwrap_or(0));
    menu_glyph::paint(&ctx.palette, &view.menu, buf, x, MENU_ROW);
    let bottom = i32::from(buf.height()) - 1;
    print_centred(buf, bottom, &view.help, c(UiColor::TextDim), black);
}

/// Paints the slot picker over the whole of `buf`.
pub fn paint_slots(ctx: &Ctx, view: &SlotPickerView, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let (black, bg) = (c(UiColor::Black), c(UiColor::PanelBg));
    let (text, dim) = (c(UiColor::Text), c(UiColor::TextDim));
    buf.fill_rect(buf.bounds(), Cell::new(' ', text, black));
    buf.fill_rect(PANEL, Cell::new(' ', text, bg));
    buf.draw_box(PANEL, BoxStyle::Single, c(UiColor::PanelBorder), bg);
    // In the top border, its text over the slot numbers.
    let title = format!(" {} ", view.title);
    buf.print(SLOT_X - 1, PANEL.y, &title, c(UiColor::TextHighlight), bg);
    paint_columns(ctx, &view.columns, buf);
    // Which slots are on show, in the bottom border.
    let range = format!(" {} ", view.range);
    let range_x = PANEL.x + PANEL.w - 2 - cells(&range);
    buf.print(range_x, PANEL.y + PANEL.h - 1, &range, dim, bg);
    for (y, slot) in (FIRST_ROW..).zip(&view.slots) {
        paint_slot(ctx, slot, buf, y);
    }
    if let Some(message) = &view.message {
        print_centred(buf, MESSAGE_ROW, message, c(UiColor::HpLow), black);
    }
    if let Some(question) = &view.question {
        paint_question(ctx, question, buf);
    }
    let bottom = i32::from(buf.height()) - 1;
    print_centred(buf, bottom, &view.help, dim, black);
}

/// The column headings.
fn paint_columns(ctx: &Ctx, columns: &SlotColumnsView, buf: &mut GlyphBuffer) {
    let bg = ctx.palette.get(UiColor::PanelBg);
    let dim = ctx.palette.get(UiColor::TextDim);
    buf.print(SLOT_X, HEADING_ROW, &columns.slot, dim, bg);
    buf.print(CHAPTER_X, HEADING_ROW, &columns.chapter, dim, bg);
    buf.print(MODE_X, HEADING_ROW, &columns.mode, dim, bg);
    buf.print(ARMY_X, HEADING_ROW, &columns.army, dim, bg);
    let x = TIME_END_X - cells(&columns.time);
    buf.print(x, HEADING_ROW, &columns.time, dim, bg);
}

/// Slot `slot` on console row `y`.
fn paint_slot(ctx: &Ctx, slot: &SlotView, buf: &mut GlyphBuffer, y: i32) {
    let color = |u| ctx.palette.get(u);
    let panel = color(UiColor::PanelBg);
    let (bg, text, dim, bad) = if slot.focused {
        let bar = color(UiColor::PanelBorderFocus);
        buf.fill_rect(
            Rect::new(PANEL.x + 1, y, PANEL.w - 2, 1),
            Cell::new(' ', panel, bar),
        );
        // On the focus bar every colour is the bar's text colour.
        (bar, panel, panel, panel)
    } else {
        (
            panel,
            color(UiColor::Text),
            color(UiColor::TextDim),
            color(UiColor::HpLow),
        )
    };
    buf.print(SLOT_X, y, &format!("{:02}", slot.number), text, bg);
    match &slot.entry {
        SlotEntryView::Saved {
            chapter,
            mode_name,
            army,
            playtime,
            ..
        } => {
            let title: String = chapter.chars().take(CHAPTER_W).collect();
            buf.print(CHAPTER_X, y, &title, text, bg);
            buf.print(MODE_X, y, mode_name, text, bg);
            buf.print(ARMY_X, y, army, text, bg);
            buf.print(TIME_END_X - cells(playtime), y, playtime, text, bg);
        }
        SlotEntryView::Unreadable(why) => {
            buf.print(CHAPTER_X, y, why, bad, bg);
        }
        SlotEntryView::Empty(label) => {
            buf.print(CHAPTER_X, y, label, dim, bg);
        }
    }
}

/// The overwrite question in a double-bordered box in the middle of the
/// screen, with its answers' keys under it.
fn paint_question(ctx: &Ctx, question: &OverwriteView, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let bg = c(UiColor::PanelBg);
    let widest = question
        .question
        .chars()
        .count()
        .max(question.answers.chars().count());
    let w = i32::try_from(widest).unwrap_or(0) + 4;
    let rect = Rect::new(
        (i32::from(buf.width()) - w) / 2,
        (i32::from(buf.height()) - OVERWRITE_H) / 2,
        w,
        OVERWRITE_H,
    );
    buf.fill_rect(rect, Cell::new(' ', c(UiColor::Text), bg));
    buf.draw_box(rect, BoxStyle::Double, c(UiColor::PanelBorderFocus), bg);
    let (x, y) = (rect.x + 2, rect.y + 2);
    buf.print(x, y, &question.question, c(UiColor::Text), bg);
    buf.print(x, y + 1, &question.answers, c(UiColor::TextDim), bg);
}

#[cfg(test)]
mod tests;
