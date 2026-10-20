//! The glyph look of the attack forecast (`look-and-feel.md` → *Attack
//! forecast*): a double box over the side panel, its title in the top
//! border; the two sides in columns (name in the faction colour, HP with a
//! bar shading what would be lost); the strikes numbered in the middle,
//! the attacker's on the left and the target's on the right, `!` after an
//! effective one, a skull on the kill and the strikes after it dimmed;
//! then the totals and the notes.

use crate::prelude::*;
use trpg_core::{Side, StatValue};

use super::{TEXT_X, print_clipped};
use crate::color::{Rgb, UiColor};
use crate::console::{ConsoleSizes, ConsoleSpec};
use crate::glyph_buffer::{BoxStyle, Cell, GlyphBuffer, Layer, Overlay, PanelLook, Rect};
use crate::screen::Ctx;
use crate::screens::battle::layout::SIDE_PANEL;
use crate::screens::battle::units::{faction_color, hp_fill, meter_cells};
use crate::screens::battle::view::{ForecastSideView, ForecastView, LeftView, SkillLineView};

/// Left column (the attacker).
pub const LEFT_X: i32 = TEXT_X;

/// Right column (the target).
pub const RIGHT_X: i32 = SIDE_PANEL.x + 16;

/// Widest text in a column.
pub const COLUMN_W: usize = 13;

/// Row of the combat active's name and durability change (`Keen Edge (20 →
/// 17)`), under the title, when one is chosen (0412).
pub const SKILL_ROW: i32 = SIDE_PANEL.y + 1;

/// Widest text of the skill line, inside the panel's border.
const SKILL_W: usize = 26;

/// Row of the names; weapons, a broken marker, HP, hit and crit follow.
pub const NAME_ROW: i32 = SIDE_PANEL.y + 2;

/// Row of the HP line.
pub const HP_ROW: i32 = NAME_ROW + 3;

/// Row of the `(broken)` marker: under the weapon, above the HP line.
pub const BROKEN_ROW: i32 = HP_ROW - 1;

/// Row of the affinity marker (`(resist)`, `heals 9`), under the crit.
pub const AFFINITY_ROW: i32 = HP_ROW + 3;

/// Row of the line above the strikes.
pub const RULE_ROW: i32 = HP_ROW + 4;

/// Row of the first strike.
pub const STRIKE_ROW: i32 = RULE_ROW + 1;

/// Length of each side's HP bar, in cells.
pub const HP_BAR_CELLS: i32 = 5;

/// Column of the strike numbers, between the two sides.
pub const NUMBER_X: i32 = SIDE_PANEL.x + 14;

/// The kill mark: a skull, one cell wide, `X` = a pixel of the mark.
// check-text: not player text (a picture)
pub const SKULL: [&str; 9] = [
    "..XXXX..", ".XXXXXX.", "XXXXXXXX", "X..XX..X", "X..XX..X", "XXX..XXX", ".XXXXXX.", ".X.XX.X.",
    "..XXXX..",
];

/// Pixel row of the skull's top inside its cell.
const SKULL_TOP: i32 = 4;

/// The skull drawn in the cell at `(x, y)` in `color`: one overlay per run
/// of pixels in a row.
pub fn skull(x: i32, y: i32, color: Rgb) -> Vec<Overlay> {
    let (cw, ch) = ConsoleSpec::PC.cell_px();
    let mut runs = Vec::new();
    for (row, line) in (0..).zip(SKULL) {
        let mut start = None;
        for (col, px) in (0..).zip(line.chars().chain(['.'])) {
            match (px == 'X', start) {
                (true, None) => start = Some(col),
                (false, Some(s)) => {
                    let rect = Rect::new(x * cw + s, y * ch + SKULL_TOP + row, col - s, 1);
                    runs.push(Overlay::new(rect, color, Layer::Over));
                    start = None;
                }
                _ => {}
            }
        }
    }
    runs
}

/// Paints the forecast `view` over the side panel.
pub fn paint(ctx: &Ctx, view: &ForecastView, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let bg = c(UiColor::PanelBg);
    let look = PanelLook::new(BoxStyle::Double, c(UiColor::Text), bg);
    buf.panel(SIDE_PANEL, &look.border(c(UiColor::PanelBorder)), None);
    let top = Rect::new(SIDE_PANEL.x, SIDE_PANEL.y, SIDE_PANEL.w, 1);
    let title = format!(" {} ", view.title);
    buf.print_centred(top, &title, c(UiColor::TextHighlight));
    for (side, x) in view.sides.iter().zip([LEFT_X, RIGHT_X]) {
        paint_side(ctx, view, side, x, buf);
    }
    if let Some(skill) = &view.skill {
        paint_skill(ctx, skill, buf);
    }
    let total_row = paint_strikes(ctx, view, buf);
    paint_notes(ctx, &view.notes, total_row + 2, buf);
}

/// The notes, one per row from `row`, stopping above the bottom border.
fn paint_notes(ctx: &Ctx, notes: &[String], row: i32, buf: &mut GlyphBuffer) {
    let bottom = SIDE_PANEL.y + SIDE_PANEL.h - 1;
    let fg = ctx.palette.get(UiColor::Effect);
    for (y, note) in (row..bottom).zip(notes) {
        print_clipped(buf, (LEFT_X, y), SKILL_W, note, fg);
    }
}

/// The art's or combat active's line: `Guard Break (20 → 16)`.
fn paint_skill(ctx: &Ctx, skill: &SkillLineView, buf: &mut GlyphBuffer) {
    let cost = skill
        .durability
        .map_or_else(String::new, |(from, to)| format!(" ({from} → {to})"));
    // check-text: not a data name (the view's own)
    let text = format!("{}{cost}", skill.name);
    let fg = ctx.palette.get(UiColor::TextHighlight);
    print_clipped(buf, (LEFT_X, SKILL_ROW), SKILL_W, &text, fg);
}

/// One side's block at column `x`: name, weapon, what is left of it, HP
/// with its bar, hit, crit, affinity marker.
fn paint_side(
    ctx: &Ctx,
    view: &ForecastView,
    side: &ForecastSideView,
    x: i32,
    buf: &mut GlyphBuffer,
) {
    let c = |u| ctx.palette.get(u);
    let (text, dim) = (c(UiColor::Text), c(UiColor::TextDim));
    let name_fg = c(faction_color(side.faction));
    // check-text: not a data name (the view's own)
    print_clipped(buf, (x, NAME_ROW), COLUMN_W, &side.name, name_fg);
    print_clipped(buf, (x, NAME_ROW + 1), COLUMN_W, &side.weapon, dim);
    match &side.left {
        Some(LeftView::Broken(broken)) => {
            print_clipped(buf, (x, BROKEN_ROW), COLUMN_W, broken, c(UiColor::HpLow));
        }
        Some(LeftView::Count { left, max }) => {
            print_clipped(
                buf,
                (x, BROKEN_ROW),
                COLUMN_W,
                &format!("{left}/{max}"),
                dim,
            );
        }
        None => {}
    }
    if let Some(marker) = &side.affinity {
        let hi = c(UiColor::TextHighlight);
        print_clipped(buf, (x, AFFINITY_ROW), COLUMN_W, marker, hi);
    }
    let hp = side.hp;
    print_clipped(buf, (x, HP_ROW), 2, &view.hp_name, dim);
    print_clipped(buf, (x + 3, HP_ROW), 3, &format!("{:>2}", hp.now), text);
    paint_bar(ctx, buf, x + 6, (hp.now, hp.after, hp.max));
    let shown = |v: Option<u8>| v.map_or_else(|| "--".to_owned(), |v| v.to_string());
    print_clipped(buf, (x, HP_ROW + 1), 3, &view.hit_name, dim);
    print_clipped(buf, (x + 5, HP_ROW + 1), 3, &shown(side.hit), text);
    print_clipped(buf, (x, HP_ROW + 2), 4, &view.crit_name, dim);
    print_clipped(buf, (x + 5, HP_ROW + 2), 3, &shown(side.crit), text);
}

/// The HP bar at `(x, HP_ROW)`: `after` HP filled in its HP colour, the HP
/// that would be lost (`hp - after`) shaded in `hp_low`, the rest empty.
fn paint_bar(
    ctx: &Ctx,
    buf: &mut GlyphBuffer,
    x: i32,
    (hp, after, max): (StatValue, StatValue, StatValue),
) {
    let c = |u| ctx.palette.get(u);
    let (kept, color) = hp_fill(after, max, HP_BAR_CELLS);
    let (now, _) = hp_fill(hp.max(after), max, HP_BAR_CELLS);
    let bg = c(UiColor::PanelBg);
    let lost = Cell::new('▒', c(UiColor::HpLow), bg);
    let empty = Cell::new('░', c(UiColor::TextDim), bg);
    meter_cells(buf, Rect::new(x, HP_ROW, HP_BAR_CELLS, 1), now, lost, empty);
    let full = Cell::new('█', c(color), bg);
    meter_cells(buf, Rect::new(x, HP_ROW, now, 1), kept, full, lost);
}

/// The strike list and the totals under it. Returns the totals' row.
fn paint_strikes(ctx: &Ctx, view: &ForecastView, buf: &mut GlyphBuffer) -> i32 {
    let c = |u| ctx.palette.get(u);
    let (text, dim) = (c(UiColor::Text), c(UiColor::TextDim));
    let rule = c(UiColor::PanelBorder);
    let rule_w = usize::try_from(SIDE_PANEL.w - 4).unwrap_or(0);
    let line = "─".repeat(rule_w);
    print_clipped(buf, (LEFT_X, RULE_ROW), rule_w, &line, rule);
    for (y, (n, s)) in (STRIKE_ROW..).zip(view.strikes.iter().enumerate()) {
        print_clipped(buf, (NUMBER_X, y), 1, &(n + 1).to_string(), dim);
        let fg = if s.after_a_fall { dim } else { text };
        let (dmg_x, arrow_x, arrow) = match s.by {
            Side::Attacker => (LEFT_X, LEFT_X + 9, "→"),
            Side::Defender => (RIGHT_X + 2, RIGHT_X, "←"),
        };
        print_clipped(buf, (dmg_x, y), 7, &s.text, fg);
        print_clipped(buf, (arrow_x, y), 1, arrow, dim);
        if s.effective {
            print_clipped(buf, (dmg_x + 6, y), 1, "!", c(UiColor::TextHighlight));
        }
        if s.kills {
            for o in skull(dmg_x + 7, y, c(UiColor::HpLow)) {
                buf.add_overlay(o);
            }
        }
    }
    if let Some(none) = &view.no_counter {
        print_clipped(buf, (RIGHT_X + 2, STRIKE_ROW), 11, none, dim);
    }
    let rows = i32::try_from(view.strikes.len()).unwrap_or(0);
    let total_row = STRIKE_ROW + rows + 1;
    print_clipped(buf, (LEFT_X, total_row - 1), rule_w, &line, rule);
    print_clipped(buf, (NUMBER_X - 2, total_row), 5, &view.total_name, dim);
    for (side, x) in view.sides.iter().zip([LEFT_X, RIGHT_X + 2]) {
        if let Some(total) = side.total {
            let shown = format!("{:>2} ×{}", total.damage, total.strikes);
            print_clipped(buf, (x, total_row), 7, &shown, text);
        }
    }
    total_row
}

#[cfg(test)]
mod tests;
