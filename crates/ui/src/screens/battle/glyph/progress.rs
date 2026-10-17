//! The glyph look of the EXP bar, level-up and class pages: the EXP bar
//! and the class box where the fight box was; the level-up panel over the
//! map view, the face in a frame on the left (a placeholder without one),
//! the banner lit, the name, what changed and the stat rows on the right,
//! each gain lit. The class change and the results screen paint the same
//! pages.

use super::fight_box::BOX;
use crate::color::UiColor;
use crate::glyph_buffer::{BoxStyle, Cell, GlyphBuffer, Rect};
use crate::portrait::draw_portrait;
use crate::prelude::*;
use crate::screen::Ctx;
use crate::screens::battle::layout::MAP_VIEW;
use crate::screens::battle::progress::stat_row_parts;
use crate::screens::battle::view::{ClassProgressView, ExpView, GainsView, ProgressView};

/// The EXP box: where the combat playback's box was.
pub const EXP_BOX: Rect = Rect::new(BOX.x, BOX.y, BOX.w, 3);

/// Length of the EXP bar, in cells (5 EXP each).
pub const EXP_BAR_CELLS: i32 = 20;

/// Widest unit name in the EXP and class boxes.
const NAME_W: usize = 12;

/// Where the EXP bar or the class line starts, after the name.
const NAME_COL: i32 = 13;

/// Widest class line (the box's inside, less the name column and padding).
const CLASS_W: usize = 27;

/// Widest line of what was learned or stowed in the class box.
const LINE_W: usize = 38;

/// The level-up panel, over the map view.
pub const LEVEL_PANEL: Rect = Rect::new(MAP_VIEW.x + 5, MAP_VIEW.y + 5, 60, 20);

/// The portrait frame in the level-up panel: a 32×16-cell portrait and its
/// border.
pub const PORTRAIT_FRAME: Rect = Rect::new(LEVEL_PANEL.x + 2, LEVEL_PANEL.y + 1, 34, 18);

/// The text column of the level-up panel.
pub const LEVEL_TEXT_X: i32 = PORTRAIT_FRAME.x + PORTRAIT_FRAME.w + 2;

/// Row of the `LEVEL UP!` banner.
pub const LEVEL_BANNER_ROW: i32 = LEVEL_PANEL.y + 2;

/// Row of the first stat.
pub const STAT_ROW: i32 = LEVEL_PANEL.y + 7;

/// Widest text in the level-up panel's text column.
const LEVEL_TEXT_W: usize = 20;

/// How many of an EXP bar's [`EXP_BAR_CELLS`] `exp` of `max` fills.
pub fn bar_cells(exp: u32, max: u32) -> i32 {
    let cells = u32::try_from(EXP_BAR_CELLS).unwrap_or(0);
    let filled = (exp.min(max) * cells).checked_div(max).unwrap_or(0);
    i32::try_from(filled).unwrap_or(EXP_BAR_CELLS)
}

/// Paints the page `view`.
pub fn paint(ctx: &Ctx, view: &ProgressView, buf: &mut GlyphBuffer) {
    match view {
        ProgressView::Exp(p) => paint_exp(ctx, p, buf),
        ProgressView::Gains(p) => paint_gains(ctx, p, buf),
        ProgressView::Class(p) => paint_class(ctx, p, buf),
    }
}

/// The EXP box: `Test Lord   EXP ████████░░░░ 80`.
fn paint_exp(ctx: &Ctx, page: &ExpView, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let bg = c(UiColor::PanelBg);
    buf.fill_rect(EXP_BOX, Cell::new(' ', c(UiColor::Text), bg));
    buf.draw_box(EXP_BOX, BoxStyle::Double, c(UiColor::PanelBorderFocus), bg);
    let y = EXP_BOX.y + 1;
    let x = EXP_BOX.x + 2;
    // check-text: not a data name (the view's own)
    let name: String = page.name.chars().take(NAME_W).collect();
    buf.print(x, y, &name, c(UiColor::Player), bg);
    let bar_x = x + NAME_COL;
    buf.print(bar_x, y, &page.exp_name, c(UiColor::TextDim), bg);
    let filled = bar_cells(page.exp, page.max);
    for i in 0..EXP_BAR_CELLS {
        let (glyph, fg) = if i < filled {
            ('█', c(UiColor::ExpBar))
        } else {
            ('░', c(UiColor::TextDim))
        };
        buf.set(bar_x + 4 + i, y, Cell::new(glyph, fg, bg));
    }
    let n = format!("{:>2}", page.exp);
    buf.print(bar_x + 5 + EXP_BAR_CELLS, y, &n, c(UiColor::Text), bg);
}

/// The level-up panel: the face (or a placeholder), the banner, the name,
/// what changed, and the stat rows shown so far.
fn paint_gains(ctx: &Ctx, page: &GainsView, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let bg = c(UiColor::PanelBg);
    buf.fill_rect(LEVEL_PANEL, Cell::new(' ', c(UiColor::Text), bg));
    let focus = c(UiColor::PanelBorderFocus);
    buf.draw_box(LEVEL_PANEL, BoxStyle::Double, focus, bg);
    let border = c(UiColor::PanelBorder);
    buf.draw_box(PORTRAIT_FRAME, BoxStyle::Single, border, bg);
    let drawn = page.face.as_ref().is_some_and(|face| {
        ctx.game
            .content
            .portraits
            .get(&face.portrait)
            .is_some_and(|art| {
                let at = (PORTRAIT_FRAME.x + 1, PORTRAIT_FRAME.y + 1);
                draw_portrait(buf, at, art, &face.expression, 0.0, false)
            })
    });
    if !drawn {
        let label = &page.placeholder;
        let w = i32::try_from(label.len()).unwrap_or(0);
        let x = PORTRAIT_FRAME.x + (PORTRAIT_FRAME.w - w) / 2;
        let y = PORTRAIT_FRAME.y + PORTRAIT_FRAME.h / 2;
        buf.print(x, y, label, c(UiColor::TextDim), bg);
    }
    let x = LEVEL_TEXT_X;
    let hi = c(UiColor::TextHighlight);
    buf.print(x, LEVEL_BANNER_ROW, &page.banner, hi, bg);
    let cut = |s: &str| s.chars().take(LEVEL_TEXT_W).collect::<String>();
    // check-text: not a data name (the view's own)
    let name = cut(&page.name);
    buf.print(x, LEVEL_BANNER_ROW + 2, &name, c(UiColor::Player), bg);
    let line = cut(&page.line);
    buf.print(x, LEVEL_BANNER_ROW + 3, &line, c(UiColor::Text), bg);
    for (y, stat) in (STAT_ROW..).zip(&page.stats) {
        // check-text: not a data name (the view's own)
        let (row, gain) = stat_row_parts(&stat.name, stat.before, stat.gain);
        match gain {
            Some(gain) => {
                let n = buf.print(x, y, &row, c(UiColor::Text), bg);
                // The `+1` in the highlight colour.
                buf.print(x + i32::from(n), y, &gain, hi, bg);
            }
            None => {
                buf.print(x, y, &row, c(UiColor::TextDim), bg);
            }
        }
    }
}

/// The class box: the name and the class line, then the mastery, and a
/// line each for what was learned or stowed.
fn paint_class(ctx: &Ctx, page: &ClassProgressView, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let bg = c(UiColor::PanelBg);
    let mastery = i32::from(page.mastered.is_some());
    let h = 3 + mastery + i32::try_from(page.lines.len()).unwrap_or(0);
    let rect = Rect::new(BOX.x, BOX.y, BOX.w, h);
    buf.fill_rect(rect, Cell::new(' ', c(UiColor::Text), bg));
    buf.draw_box(rect, BoxStyle::Double, c(UiColor::PanelBorderFocus), bg);
    let x = rect.x + 2;
    let mut y = rect.y + 1;
    // check-text: not a data name (the view's own)
    let name: String = page.name.chars().take(NAME_W).collect();
    buf.print(x, y, &name, c(UiColor::Player), bg);
    let line: String = page.line.chars().take(CLASS_W).collect();
    buf.print(x + NAME_COL, y, &line, c(UiColor::Text), bg);
    if let Some(mastered) = &page.mastered {
        y += 1;
        buf.print(x, y, mastered, c(UiColor::TextHighlight), bg);
    }
    for line in &page.lines {
        y += 1;
        let line: String = line.chars().take(LINE_W).collect();
        buf.print(x, y, &line, c(UiColor::Text), bg);
    }
}

/// Whether `buf` draws a picture of the portrait of `id` (any of its
/// expressions): for tests of whose face a page shows.
#[cfg(test)]
pub(crate) fn shows_face_of(buf: &GlyphBuffer, ctx: &Ctx, id: &str) -> bool {
    let portrait = &ctx.game.content.portraits[id];
    buf.sprites()
        .iter()
        .any(|s| portrait.expressions.iter().any(|e| e.image == s.image))
}

#[cfg(test)]
mod tests;
