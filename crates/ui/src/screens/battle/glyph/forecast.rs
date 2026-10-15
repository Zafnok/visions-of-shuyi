//! The glyph look of the attack forecast (`look-and-feel.md` → *Attack
//! forecast*): a double box over the side panel, its title in the top
//! border; the two sides in columns (name in the faction colour, HP with a
//! bar shading what would be lost); the strikes numbered in the middle,
//! the attacker's on the left and the target's on the right, `!` after an
//! effective one, a skull on the kill and the strikes after it dimmed;
//! then the totals and the notes.

use trpg_core::{Side, StatValue};

use super::TEXT_X;
use crate::color::{Rgb, UiColor};
use crate::console::ConsoleSpec;
use crate::glyph_buffer::{BoxStyle, Cell, GlyphBuffer, Layer, Overlay, Rect};
use crate::screen::Ctx;
use crate::screens::battle::layout::SIDE_PANEL;
use crate::screens::battle::units::{faction_color, hp_fill};
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

/// Prints `text` at `(x, y)` on the cells' background, cut to `w` chars.
fn put(buf: &mut GlyphBuffer, x: i32, y: i32, text: &str, fg: Rgb, w: usize) {
    let bg = buf.get(x, y).map_or(fg, |c| c.bg);
    let cut: String = text.chars().take(w).collect();
    buf.print(x, y, &cut, fg, bg);
}

/// Paints the forecast `view` over the side panel.
pub fn paint(ctx: &Ctx, view: &ForecastView, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let bg = c(UiColor::PanelBg);
    buf.fill_rect(SIDE_PANEL, Cell::new(' ', c(UiColor::Text), bg));
    buf.draw_box(SIDE_PANEL, BoxStyle::Double, c(UiColor::PanelBorder), bg);
    let title = format!(" {} ", view.title);
    let title_w = i32::try_from(title.chars().count()).unwrap_or(0);
    let title_x = SIDE_PANEL.x + (SIDE_PANEL.w - title_w) / 2;
    let hi = c(UiColor::TextHighlight);
    put(buf, title_x, SIDE_PANEL.y, &title, hi, title.len());
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
        put(buf, LEFT_X, y, note, fg, SKILL_W);
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
    put(buf, LEFT_X, SKILL_ROW, &text, fg, SKILL_W);
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
    put(buf, x, NAME_ROW, &side.name, name_fg, COLUMN_W);
    put(buf, x, NAME_ROW + 1, &side.weapon, dim, COLUMN_W);
    match &side.left {
        Some(LeftView::Broken(broken)) => {
            put(buf, x, BROKEN_ROW, broken, c(UiColor::HpLow), COLUMN_W);
        }
        Some(LeftView::Count { left, max }) => {
            put(buf, x, BROKEN_ROW, &format!("{left}/{max}"), dim, COLUMN_W);
        }
        None => {}
    }
    if let Some(marker) = &side.affinity {
        let hi = c(UiColor::TextHighlight);
        put(buf, x, AFFINITY_ROW, marker, hi, COLUMN_W);
    }
    let hp = side.hp;
    put(buf, x, HP_ROW, &view.hp_name, dim, 2);
    put(buf, x + 3, HP_ROW, &format!("{:>2}", hp.now), text, 3);
    paint_bar(ctx, buf, x + 6, (hp.now, hp.after, hp.max));
    let shown = |v: Option<u8>| v.map_or_else(|| "--".to_owned(), |v| v.to_string());
    put(buf, x, HP_ROW + 1, &view.hit_name, dim, 3);
    put(buf, x + 5, HP_ROW + 1, &shown(side.hit), text, 3);
    put(buf, x, HP_ROW + 2, &view.crit_name, dim, 4);
    put(buf, x + 5, HP_ROW + 2, &shown(side.crit), text, 3);
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
    for i in 0..HP_BAR_CELLS {
        let (glyph, fg) = if i < kept {
            ('█', c(color))
        } else if i < now {
            ('▒', c(UiColor::HpLow))
        } else {
            ('░', c(UiColor::TextDim))
        };
        buf.set(x + i, HP_ROW, Cell::new(glyph, fg, bg));
    }
}

/// The strike list and the totals under it. Returns the totals' row.
fn paint_strikes(ctx: &Ctx, view: &ForecastView, buf: &mut GlyphBuffer) -> i32 {
    let c = |u| ctx.palette.get(u);
    let (text, dim) = (c(UiColor::Text), c(UiColor::TextDim));
    let rule = c(UiColor::PanelBorder);
    let rule_w = usize::try_from(SIDE_PANEL.w - 4).unwrap_or(0);
    let line = "─".repeat(rule_w);
    put(buf, LEFT_X, RULE_ROW, &line, rule, rule_w);
    for (y, (n, s)) in (STRIKE_ROW..).zip(view.strikes.iter().enumerate()) {
        put(buf, NUMBER_X, y, &(n + 1).to_string(), dim, 1);
        let fg = if s.after_a_fall { dim } else { text };
        let (dmg_x, arrow_x, arrow) = match s.by {
            Side::Attacker => (LEFT_X, LEFT_X + 9, "→"),
            Side::Defender => (RIGHT_X + 2, RIGHT_X, "←"),
        };
        put(buf, dmg_x, y, &s.text, fg, 7);
        put(buf, arrow_x, y, arrow, dim, 1);
        if s.effective {
            put(buf, dmg_x + 6, y, "!", c(UiColor::TextHighlight), 1);
        }
        if s.kills {
            for o in skull(dmg_x + 7, y, c(UiColor::HpLow)) {
                buf.add_overlay(o);
            }
        }
    }
    if let Some(none) = &view.no_counter {
        put(buf, RIGHT_X + 2, STRIKE_ROW, none, dim, 11);
    }
    let rows = i32::try_from(view.strikes.len()).unwrap_or(0);
    let total_row = STRIKE_ROW + rows + 1;
    put(buf, LEFT_X, total_row - 1, &line, rule, rule_w);
    put(buf, NUMBER_X - 2, total_row, &view.total_name, dim, 5);
    for (side, x) in view.sides.iter().zip([LEFT_X, RIGHT_X + 2]) {
        if let Some(total) = side.total {
            let shown = format!("{:>2} ×{}", total.damage, total.strikes);
            put(buf, x, total_row, &shown, text, 7);
        }
    }
    total_row
}

#[cfg(test)]
mod tests;
