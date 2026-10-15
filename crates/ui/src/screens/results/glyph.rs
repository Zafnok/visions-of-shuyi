//! The glyph look of the results screen: the tally as one double-bordered
//! panel (the gold, rewind and bonus lines, a rule, then a row with an EXP
//! bar for each unit), and a level-up page over the middle of the screen.
//! Everything about the look lives here; nothing here decides what the
//! screen shows. Another look is another `paint` of the same view
//! (ADR-0054).

use super::view::{ResultsBody, ResultsView, TallyView, UnitView};
use crate::color::UiColor;
use crate::glyph_buffer::{BoxStyle, Cell, GlyphBuffer, Rect};
use crate::prelude::*;
use crate::screen::Ctx;
use crate::screens::battle::glyph::progress::{self, EXP_BAR_CELLS, bar_cells};
use crate::screens::battle::layout::MAP_VIEW;
use crate::screens::{centre_x, print_centred};

/// The panel.
const PANEL: Rect = Rect::new(14, 3, 72, 24);
/// Column of the labels and the unit names.
const X: i32 = PANEL.x + 4;
/// Column of the numbers on the gold, rewinds and bonus lines.
const VALUE_X: i32 = X + 44;
/// Row of the gold line.
const GOLD_ROW: i32 = PANEL.y + 2;
/// Row of the rewinds line; the bonus line is the next one.
const REWINDS_ROW: i32 = PANEL.y + 4;
/// Row of the rule between the lines and the units.
const RULE_ROW: i32 = PANEL.y + 7;
/// Row of the first unit.
const UNIT_ROW: i32 = PANEL.y + 9;
/// Unit rows the panel has room for.
const UNIT_ROWS: i32 = PANEL.y + PANEL.h - 1 - UNIT_ROW;
/// Columns of a unit row, from [`X`]: class, level, `EXP`, bar, number, mark.
const COLS: [i32; 6] = [14, 23, 29, 33, 54, 58];
/// Widest name and class shown.
const NAME_W: usize = 13;
const CLASS_W: usize = 8;

/// Paints `view` over the whole of `buf`.
pub fn paint(ctx: &Ctx, view: &ResultsView, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let blank = Cell::new(' ', c(UiColor::Text), c(UiColor::Black));
    buf.fill_rect(buf.bounds(), blank);
    match &view.body {
        ResultsBody::LevelUp(None) => {}
        ResultsBody::LevelUp(Some(view)) => {
            // The pages are laid out on the battle's map view: drawn
            // there, then moved to the middle of the screen.
            let (w, h) = (MAP_VIEW.x + MAP_VIEW.w, MAP_VIEW.y + MAP_VIEW.h);
            let size = |v: i32| u16::try_from(v).unwrap_or(0);
            let mut page = GlyphBuffer::new(size(w), size(h), blank);
            progress::paint(ctx, view, &mut page);
            buf.blit(&page, centre_x(buf, usize::from(page.width())), 0);
        }
        ResultsBody::Tally(tally) => paint_tally(ctx, tally, buf),
    }
    let bottom = i32::from(buf.height()) - 1;
    let (dim, bg) = (c(UiColor::TextDim), c(UiColor::Black));
    print_centred(buf, bottom, &view.help, dim, bg);
}

/// The gold, rewinds and bonus lines and the units' bars.
fn paint_tally(ctx: &Ctx, tally: &TallyView, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let bg = c(UiColor::PanelBg);
    let (text, dim, hi) = (
        c(UiColor::Text),
        c(UiColor::TextDim),
        c(UiColor::TextHighlight),
    );
    buf.fill_rect(PANEL, Cell::new(' ', text, bg));
    buf.draw_box(PANEL, BoxStyle::Double, c(UiColor::PanelBorderFocus), bg);
    let heading = format!(" {} ", tally.heading);
    buf.print(PANEL.x + 2, PANEL.y, &heading, hi, bg);
    if let Some(gold) = &tally.gold {
        buf.print(X, GOLD_ROW, &gold.label, text, bg);
        let n = buf.print(VALUE_X, GOLD_ROW, &format!("+{}", gold.gained), hi, bg);
        buf.print(VALUE_X + i32::from(n) + 1, GOLD_ROW, &gold.now, dim, bg);
    }
    buf.print(X, REWINDS_ROW, &tally.rewinds.label, text, bg);
    buf.print(VALUE_X, REWINDS_ROW, &tally.rewinds.left, hi, bg);
    let Some(bonus) = &tally.bonus else {
        return;
    };
    buf.print(X, REWINDS_ROW + 1, &bonus.label, text, bg);
    buf.print(VALUE_X, REWINDS_ROW + 1, &format!("+{}", bonus.exp), hi, bg);
    for x in X..PANEL.x + PANEL.w - 4 {
        buf.set(x, RULE_ROW, Cell::new('─', c(UiColor::PanelBorder), bg));
    }
    // A blank row between units while they all fit that way.
    let spaced = (0..UNIT_ROWS).step_by(2).len();
    let step = if tally.units.len() <= spaced { 2 } else { 1 };
    let rows = (0..UNIT_ROWS).step_by(step).zip(&tally.units);
    for (dy, unit) in rows {
        paint_unit(ctx, unit, buf, UNIT_ROW + dy);
    }
}

/// One unit's name, class, level, EXP bar and number on row `y`.
fn paint_unit(ctx: &Ctx, unit: &UnitView, buf: &mut GlyphBuffer, y: i32) {
    let c = |u| ctx.palette.get(u);
    let bg = c(UiColor::PanelBg);
    let (text, dim, hi) = (
        c(UiColor::Text),
        c(UiColor::TextDim),
        c(UiColor::TextHighlight),
    );
    // check-text: not a data name (the view's own)
    let name: String = unit.name.chars().take(NAME_W).collect();
    let class: String = unit.class.chars().take(CLASS_W).collect();
    buf.print(X, y, &name, c(UiColor::Player), bg);
    buf.print(X + COLS[0], y, &class, dim, bg);
    buf.print(X + COLS[1], y, &unit.level, text, bg);
    buf.print(X + COLS[2], y, &unit.exp_label, dim, bg);
    let filled = bar_cells(unit.exp, unit.exp_max);
    for i in 0..EXP_BAR_CELLS {
        let (glyph, fg) = if i < filled {
            ('█', c(UiColor::ExpBar))
        } else {
            ('░', dim)
        };
        buf.set(X + COLS[3] + i, y, Cell::new(glyph, fg, bg));
    }
    buf.print(X + COLS[4], y, &format!("{:>2}", unit.exp), text, bg);
    if let Some(mark) = &unit.level_up {
        buf.print(X + COLS[5], y, mark, hi, bg);
    }
}

#[cfg(test)]
mod tests;
