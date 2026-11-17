//! The glyph look of the attack forecast (`look-and-feel.md` → *Attack
//! forecast*): a double box over the side panel, its title in the top
//! border; the two sides in columns (name in the faction colour, the
//! weapon and its uses, `HP 19 → 6` over a bar blinking what would be
//! lost, ticket 0480); the strikes numbered in the middle,
//! the attacker's on the left and the target's on the right, `!` after an
//! effective one, a skull on the kill and the strikes after it dimmed;
//! then the totals, a row per unit an art strikes (`Hound  Hit 82  11 →
//! 4`, the name in its faction colour) and the notes.

use crate::prelude::*;
use trpg_core::{Side, StatValue};

use super::{TEXT_X, print_clipped};
use crate::color::{Rgb, UiColor};
use crate::console::{ConsoleSizes, ConsoleSpec};
use crate::glyph_buffer::{BoxStyle, Cell, GlyphBuffer, Layer, Overlay, PanelLook, Rect};
use crate::screen::Ctx;
use crate::screens::battle::layout::SIDE_PANEL;
use crate::screens::battle::units::{faction_color, lit_fill, meter_cells};
use crate::screens::battle::view::{
    ForecastSideView, ForecastView, LeftView, SkillLineView, StruckView,
};

/// Left column (the attacker).
pub const LEFT_X: i32 = TEXT_X;

/// Right column (the target).
pub const RIGHT_X: i32 = SIDE_PANEL.x + 16;

/// Widest text in a column.
pub const COLUMN_W: usize = 13;

/// Row of the chosen art's or combat active's name (`Keen Edge`), under
/// the title (0412).
pub const SKILL_ROW: i32 = SIDE_PANEL.y + 1;

/// Widest text of the skill line, inside the panel's border.
const SKILL_W: usize = 26;

/// Row of the names; the weapons and their uses follow.
pub const NAME_ROW: i32 = SIDE_PANEL.y + 2;

/// Row under the weapons: the uses an art spends (`20 → 16`) or
/// `(broken)`, when a side has either. The rows from the HP line down then
/// move down one ([`shift`]).
pub const UNDER_ROW: i32 = NAME_ROW + 2;

/// Row of the HP line (`HP 19 → 6`) without a row under the weapons; the
/// bar, hit and crit follow.
pub const HP_ROW: i32 = NAME_ROW + 2;

/// Row of the affinity marker (`(resist)`, `heals 9`), under the crit.
pub const AFFINITY_ROW: i32 = HP_ROW + 4;

/// Row of the line above the strikes.
pub const RULE_ROW: i32 = HP_ROW + 5;

/// Row of the first strike.
pub const STRIKE_ROW: i32 = RULE_ROW + 1;

/// Length of each side's HP bar, in cells, on its own row (0480).
pub const HP_BAR_CELLS: i32 = 12;

/// How long the HP that would be lost is shown shaded, then full, in
/// seconds (0480, *tunable*).
pub const LOST_BLINK_S: f64 = 0.5;

/// Whether the HP a forecast's bar would lose is shaded at clock time
/// `clock_s` (else it is full, as now): shaded for the first
/// [`LOST_BLINK_S`], full for the next, and so on.
pub fn lost_shaded(clock_s: f64) -> bool {
    if !clock_s.is_finite() {
        return true;
    }
    // Whole halves of the blink so far (no `rem_euclid` without std, for
    // the Playdate's build); the clock only goes forward from 0.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let halves = (clock_s.max(0.0) / LOST_BLINK_S) as u64;
    halves.is_multiple_of(2)
}

/// The two sides, in the view's order.
pub const SIDES: [Side; 2] = [Side::Attacker, Side::Defender];

/// How many rows the forecast's rows from the HP line down move down: one
/// when a side shows a row under its weapon (an art's cost, `(broken)`).
pub fn shift(view: &ForecastView) -> i32 {
    let mut sides = view.sides.iter().zip(SIDES);
    i32::from(sides.any(|(s, by)| under(view, s, by).is_some()))
}

/// What the row under a side's weapon shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Under<'a> {
    /// The uses an art spends, before and after (the attacker's).
    Spends(u32, u32),
    /// The weapon is broken: what that is called.
    Broken(&'a str),
}

/// The row under `side`'s weapon (`by` says which side it is), if any:
/// the attacker's art's cost first, else `(broken)`.
pub fn under<'a>(
    view: &'a ForecastView,
    side: &'a ForecastSideView,
    by: Side,
) -> Option<Under<'a>> {
    let spends = view.skill.as_ref().and_then(|s| s.durability);
    match (by, spends, &side.left) {
        (Side::Attacker, Some((from, to)), _) => Some(Under::Spends(from, to)),
        (_, _, Some(LeftView::Broken(broken))) => Some(Under::Broken(broken)),
        _ => None,
    }
}

/// The uses shown after `side`'s weapon (`20`): none when the row under it
/// shows them or the weapon is broken.
pub fn uses_after_name(view: &ForecastView, side: &ForecastSideView, by: Side) -> Option<u32> {
    match (&side.left, under(view, side, by)) {
        (Some(LeftView::Count { left, .. }), None) => Some(*left),
        _ => None,
    }
}

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
    buf.panel(SIDE_PANEL, &look.with_border(c(UiColor::PanelBorder)), None);
    let top = Rect::new(SIDE_PANEL.x, SIDE_PANEL.y, SIDE_PANEL.w, 1);
    let title = format!(" {} ", view.title);
    buf.print_centred(top, &title, c(UiColor::TextHighlight));
    for ((side, by), x) in view.sides.iter().zip(SIDES).zip([LEFT_X, RIGHT_X]) {
        paint_side(ctx, view, (side, by), x, buf);
    }
    if let Some(skill) = &view.skill {
        paint_skill(ctx, skill, buf);
    }
    let total_row = paint_strikes(ctx, view, buf);
    let rows = i32::try_from(view.struck.len()).unwrap_or(0);
    for (y, row) in (total_row + 2..).zip(&view.struck) {
        paint_struck(ctx, view, row, y, buf);
    }
    let gap = i32::from(rows > 0);
    paint_notes(ctx, &view.notes, total_row + 2 + rows + gap, buf);
}

/// The weapon's name at `(x, y)`, dimmed, and its uses after it (`Iron
/// Sword 20`). A name too long to leave them room is cut with `…`
/// (`Steel Swo… 25`) until 0498 gives it a short name.
fn paint_weapon(
    ctx: &Ctx,
    buf: &mut GlyphBuffer,
    (x, y): (i32, i32),
    name: &str,
    uses: Option<u32>,
) {
    let c = |u| ctx.palette.get(u);
    let Some(uses) = uses else {
        print_clipped(buf, (x, y), COLUMN_W, name, c(UiColor::TextDim));
        return;
    };
    let uses = uses.to_string();
    let room = COLUMN_W.saturating_sub(uses.chars().count() + 1);
    let shown: String = if name.chars().count() > room {
        let kept: String = name.chars().take(room.saturating_sub(1)).collect();
        format!("{kept}…")
    } else {
        name.to_owned()
    };
    print_clipped(buf, (x, y), room, &shown, c(UiColor::TextDim));
    let at = x + i32::try_from(shown.chars().count() + 1).unwrap_or(0);
    print_clipped(buf, (at, y), uses.len(), &uses, c(UiColor::Text));
}

/// `before → after` from `(x, y)`: `before` in the text colour, the arrow
/// dimmed, `after` in `after_fg`.
fn paint_change(
    ctx: &Ctx,
    buf: &mut GlyphBuffer,
    (x, y): (i32, i32),
    (before, after): (&str, &str),
    after_fg: Rgb,
) {
    let c = |u| ctx.palette.get(u);
    let arrow = x + i32::try_from(before.chars().count()).unwrap_or(0) + 1;
    print_clipped(buf, (x, y), 3, before, c(UiColor::Text));
    print_clipped(buf, (arrow, y), 1, "→", c(UiColor::TextDim));
    print_clipped(buf, (arrow + 2, y), 3, after, after_fg);
}

/// Widest name in a row of the units an art strikes.
const STRUCK_NAME_W: usize = 9;

/// A row of a unit an art strikes, at row `y`: its name, the hit and its
/// HP before and after a hit (`Hound  Hit 82  11 → 4`), `!` after an
/// effective strike.
fn paint_struck(ctx: &Ctx, view: &ForecastView, row: &StruckView, y: i32, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let (text, dim) = (c(UiColor::Text), c(UiColor::TextDim));
    let name_fg = c(faction_color(row.faction));
    // check-text: not a data name (the view's own)
    print_clipped(buf, (LEFT_X, y), STRUCK_NAME_W, &row.name, name_fg);
    print_clipped(buf, (LEFT_X + 10, y), 3, &view.hit_name, dim);
    print_clipped(buf, (LEFT_X + 13, y), 4, &format!("{:>4}", row.hit), text);
    let (before, after) = row.hp;
    let hp = format!("{before:>2} → {after}");
    print_clipped(buf, (LEFT_X + 19, y), 7, &hp, text);
    if row.effective {
        print_clipped(buf, (LEFT_X + 17, y), 1, "!", c(UiColor::TextHighlight));
    }
}

/// The notes, one per row from `row`, stopping above the bottom border.
fn paint_notes(ctx: &Ctx, notes: &[String], row: i32, buf: &mut GlyphBuffer) {
    let bottom = SIDE_PANEL.y + SIDE_PANEL.h - 1;
    let fg = ctx.palette.get(UiColor::Effect);
    for (y, note) in (row..bottom).zip(notes) {
        print_clipped(buf, (LEFT_X, y), SKILL_W, note, fg);
    }
}

/// The art's or combat active's line: `Guard Break` (the uses it spends
/// show under the weapon, 0480).
fn paint_skill(ctx: &Ctx, skill: &SkillLineView, buf: &mut GlyphBuffer) {
    let fg = ctx.palette.get(UiColor::TextHighlight);
    // check-text: not a data name (the view's own)
    print_clipped(buf, (LEFT_X, SKILL_ROW), SKILL_W, &skill.name, fg);
}

/// One side's block at column `x`: name, weapon and its uses, the row
/// under the weapon (an art's cost, `(broken)`), `HP 19 → 6` over its
/// bar, hit, crit, affinity marker.
fn paint_side(
    ctx: &Ctx,
    view: &ForecastView,
    (side, by): (&ForecastSideView, Side),
    x: i32,
    buf: &mut GlyphBuffer,
) {
    let c = |u| ctx.palette.get(u);
    let (text, dim) = (c(UiColor::Text), c(UiColor::TextDim));
    let name_fg = c(faction_color(side.faction));
    // check-text: not a data name (the view's own)
    print_clipped(buf, (x, NAME_ROW), COLUMN_W, &side.name, name_fg);
    let uses = uses_after_name(view, side, by);
    paint_weapon(ctx, buf, (x, NAME_ROW + 1), &side.weapon, uses);
    match under(view, side, by) {
        Some(Under::Spends(from, to)) => {
            let (from, to) = (from.to_string(), to.to_string());
            let hi = c(UiColor::TextHighlight);
            paint_change(ctx, buf, (x, UNDER_ROW), (&from, &to), hi);
        }
        Some(Under::Broken(broken)) => {
            print_clipped(buf, (x, UNDER_ROW), COLUMN_W, broken, c(UiColor::HpLow));
        }
        None => {}
    }
    let hp_row = HP_ROW + shift(view);
    if let Some(marker) = &side.affinity {
        let hi = c(UiColor::TextHighlight);
        print_clipped(buf, (x, hp_row + 4), COLUMN_W, marker, hi);
    }
    let hp = side.hp;
    print_clipped(buf, (x, hp_row), 2, &view.hp_name, dim);
    let (now, after) = (format!("{:>2}", hp.now), hp.after.to_string());
    let after_fg = c(lit_fill(hp.after, hp.max, HP_BAR_CELLS).1);
    paint_change(ctx, buf, (x + 3, hp_row), (&now, &after), after_fg);
    let shaded = lost_shaded(ctx.clock_s);
    paint_bar(
        ctx,
        buf,
        (x, hp_row + 1),
        (hp.now, hp.after, hp.max),
        shaded,
    );
    let shown = |v: Option<u8>| v.map_or_else(|| "--".to_owned(), |v| v.to_string());
    print_clipped(buf, (x, hp_row + 2), 3, &view.hit_name, dim);
    print_clipped(buf, (x + 5, hp_row + 2), 3, &shown(side.hit), text);
    print_clipped(buf, (x, hp_row + 3), 4, &view.crit_name, dim);
    print_clipped(buf, (x + 5, hp_row + 3), 3, &shown(side.crit), text);
}

/// The HP bar at `(x, y)`, [`HP_BAR_CELLS`] long: `after` HP filled in its
/// HP colour, the HP that would be lost (`hp - after`) shaded in `hp_low`
/// when `shaded` (else full, in the colour of `hp`), the rest empty. A
/// unit with HP keeps a cell ([`lit_fill`]).
fn paint_bar(
    ctx: &Ctx,
    buf: &mut GlyphBuffer,
    (x, y): (i32, i32),
    (hp, after, max): (StatValue, StatValue, StatValue),
    shaded: bool,
) {
    let c = |u| ctx.palette.get(u);
    let (kept, color) = lit_fill(after, max, HP_BAR_CELLS);
    let (now, now_color) = lit_fill(hp.max(after), max, HP_BAR_CELLS);
    let bg = c(UiColor::PanelBg);
    let lost = if shaded {
        Cell::new('▒', c(UiColor::HpLow), bg)
    } else {
        Cell::new('█', c(now_color), bg)
    };
    let empty = Cell::new('░', c(UiColor::TextDim), bg);
    meter_cells(buf, Rect::new(x, y, HP_BAR_CELLS, 1), now, lost, empty);
    let full = Cell::new('█', c(if shaded { color } else { now_color }), bg);
    meter_cells(buf, Rect::new(x, y, now, 1), kept, full, lost);
}

/// The strike list and the totals under it. Returns the totals' row.
fn paint_strikes(ctx: &Ctx, view: &ForecastView, buf: &mut GlyphBuffer) -> i32 {
    let c = |u| ctx.palette.get(u);
    let (text, dim) = (c(UiColor::Text), c(UiColor::TextDim));
    let rule = c(UiColor::PanelBorder);
    let rule_w = usize::try_from(SIDE_PANEL.w - 4).unwrap_or(0);
    let line = "─".repeat(rule_w);
    let (rule_row, strike_row) = (RULE_ROW + shift(view), STRIKE_ROW + shift(view));
    print_clipped(buf, (LEFT_X, rule_row), rule_w, &line, rule);
    for (y, (n, s)) in (strike_row..).zip(view.strikes.iter().enumerate()) {
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
        print_clipped(buf, (RIGHT_X + 2, strike_row), 11, none, dim);
    }
    let rows = i32::try_from(view.strikes.len()).unwrap_or(0);
    let total_row = strike_row + rows + 1;
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
