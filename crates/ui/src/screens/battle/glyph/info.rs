//! The glyph look of the unit info screen: a double box over the map and
//! side panel, a 24 × 12 portrait area on the left (a placeholder box), Mov,
//! tags, weapon ranks, affinities and supports under it; name, class, EXP, HP with a
//! bar, stats, timed effects and skills in the middle; weapons, armour,
//! accessory and spells on the right. Lists stop where the box does.

use crate::color::{Palette, Rgb, UiColor};
use crate::glyph_buffer::{BoxStyle, Cell, GlyphBuffer, Rect};
use crate::prelude::*;
use crate::screen::Ctx;
use crate::screens::battle::layout::MAP_VIEW;
use crate::screens::battle::units::{faction_color, paint_hp_bar};
use crate::screens::battle::view::{GearView, InfoView, InfoWeaponView};

/// The whole screen above the help bar.
pub const INFO: Rect = Rect::new(0, 0, 100, MAP_VIEW.h);

/// The portrait area.
pub const PORTRAIT: Rect = Rect::new(2, 1, 24, 12);

/// Left column (under the portrait): movement, tags, weapon ranks.
const LEFT_X: i32 = 3;

/// Middle column: name, level, HP, stats, skills.
const MID_X: i32 = 29;

/// Right column: loadout and spells.
const RIGHT_X: i32 = 56;

/// Widest text in the right column, so nothing reaches the border.
const RIGHT_W: usize = 42;

/// Widest item name or bonus, indented two cells in the right column.
const ITEM_W: usize = 40;

/// Most skills listed, two rows each: name and cost, then the effect (rows
/// 16..28).
const SKILL_MAX: usize = 6;

/// Column of the timed effects, right of the stats.
const EFFECT_X: i32 = 37;

/// Widest text in the effects column.
const EFFECT_W: usize = 18;

/// Most timed effects listed, three rows each (rows 7..13).
const EFFECT_MAX: usize = 2;

/// Most spells listed (rows 18..28).
const SPELL_ROWS: usize = 10;

/// Length of the HP bar, in cells.
const HP_BAR_CELLS: i32 = 10;

/// Shown for an empty list or slot.
const NOTHING: &str = "--";

/// Where the screen is being drawn and in which colours.
struct Pen<'a> {
    buf: &'a mut GlyphBuffer,
    palette: &'a Palette,
    bg: Rgb,
}

impl Pen<'_> {
    fn text(&mut self, x: i32, y: i32, text: &str, color: UiColor) -> i32 {
        let fg = self.palette.get(color);
        let n = self.buf.print(x, y, text, fg, self.bg);
        x + i32::from(n)
    }

    /// `text`, cut to `width` characters.
    fn cut(&mut self, x: i32, y: i32, text: &str, width: usize, color: UiColor) {
        let cut: String = text.chars().take(width).collect();
        self.text(x, y, &cut, color);
    }
}

/// Paints the info screen `view`.
pub fn paint(ctx: &Ctx, view: &InfoView, buf: &mut GlyphBuffer) {
    let palette = &ctx.palette;
    let bg = palette.get(UiColor::PanelBg);
    buf.fill_rect(INFO, Cell::new(' ', palette.get(UiColor::Text), bg));
    let border = palette.get(UiColor::PanelBorder);
    buf.draw_box(INFO, BoxStyle::Double, border, bg);
    let dim = palette.get(UiColor::TextDim);
    buf.draw_box(PORTRAIT, BoxStyle::Single, dim, bg);
    let mut pen = Pen { buf, palette, bg };
    let label = &view.portrait;
    let w = i32::try_from(label.len()).unwrap_or(0);
    pen.text(
        PORTRAIT.x + (PORTRAIT.w - w) / 2,
        PORTRAIT.y + PORTRAIT.h / 2,
        label,
        UiColor::TextDim,
    );
    paint_left(&mut pen, view);
    paint_middle(&mut pen, view);
    paint_right(&mut pen, view);
}

/// Under the portrait: Mov and movement type, class tags, weapon ranks,
/// and the class's affinities if it has any.
fn paint_left(pen: &mut Pen<'_>, view: &InfoView) {
    let mut y = PORTRAIT.y + PORTRAIT.h + 1;
    let label = format!("{} ", view.mov_name);
    let x = pen.text(LEFT_X, y, &label, UiColor::TextDim);
    let x = pen.text(x, y, &view.mov.to_string(), UiColor::Text);
    if let Some(name) = &view.movement {
        pen.text(x + 2, y, name, UiColor::Text);
    }
    y += 1;
    if !view.tags.is_empty() {
        pen.text(LEFT_X, y, &view.tags.join(" "), UiColor::Text);
    }
    y += 2;
    pen.text(LEFT_X, y, &view.ranks_title, UiColor::TextHighlight);
    for (kind, rank) in &view.ranks {
        y += 1;
        let x = pen.text(LEFT_X, y, kind, UiColor::Text);
        pen.text(x.max(LEFT_X + 10), y, rank, UiColor::Text);
    }
    if !view.affinities.is_empty() {
        y += 2;
        pen.text(LEFT_X, y, &view.affinities_title, UiColor::TextHighlight);
        for (element, affinity) in &view.affinities {
            y += 1;
            let x = pen.text(LEFT_X, y, element, UiColor::Text);
            pen.text(x.max(LEFT_X + 10), y, affinity, UiColor::Text);
        }
    }
    if view.supports.is_empty() {
        return;
    }
    // The list stops where the box does.
    let last = INFO.y + INFO.h - 2;
    y += 2;
    if y > last {
        return;
    }
    pen.text(LEFT_X, y, &view.supports_title, UiColor::TextHighlight);
    for (partner, rank) in &view.supports {
        y += 1;
        if y > last {
            return;
        }
        pen.cut(LEFT_X, y, partner, SUPPORT_NAME_W, UiColor::Text);
        pen.text(LEFT_X + SUPPORT_RANK_X, y, rank, UiColor::Text);
    }
}

/// Widest partner name in the supports list.
const SUPPORT_NAME_W: usize = 18;

/// Column of a partner's rank, from the left column.
const SUPPORT_RANK_X: i32 = 20;

/// Name, class and level, EXP, HP with a bar, stats, effects, skills.
fn paint_middle(pen: &mut Pen<'_>, view: &InfoView) {
    let x = MID_X;
    // check-text: not a data name (the view's own)
    pen.cut(x, 1, &view.name, 25, faction_color(view.faction));
    pen.cut(x, 2, &view.class_level, 25, UiColor::Text);
    let exp = view
        .exp
        .map_or_else(|| NOTHING.to_owned(), |e| e.to_string());
    let label = format!("{} ", view.exp_name);
    let ex = pen.text(x, 3, &label, UiColor::TextDim);
    pen.text(ex, 3, &exp, UiColor::Text);
    let label = format!("{} ", view.hp_name);
    let hx = pen.text(x, 4, &label, UiColor::TextDim);
    let hp = format!("{}/{}", view.hp, view.max);
    pen.text(hx, 4, &hp, UiColor::Text);
    let bar = Rect::new(x + 11, 4, HP_BAR_CELLS, 1);
    paint_hp_bar(pen.buf, pen.palette, bar, (view.hp, view.max), pen.bg);
    pen.text(x, 6, &view.stats_title, UiColor::TextHighlight);
    // A raised or lowered stat shows in the effect colour.
    for (y, stat) in (7..).zip(&view.stats) {
        // check-text: not a data name (the view's own)
        pen.text(x, y, &stat.name, UiColor::Text);
        let value = format!("{:>3}", stat.value);
        let color = if stat.changed {
            UiColor::Effect
        } else {
            UiColor::Text
        };
        pen.text(x + 4, y, &value, color);
    }
    paint_effects(pen, view);
    paint_skills(pen, view);
}

/// The timed effects right of the stats: name, what it changes, and until
/// when it lasts.
fn paint_effects(pen: &mut Pen<'_>, view: &InfoView) {
    if view.effects.is_empty() {
        return;
    }
    pen.text(EFFECT_X, 6, &view.effects_title, UiColor::TextHighlight);
    let mut y = 7;
    for effect in view.effects.iter().take(EFFECT_MAX) {
        // check-text: not a data name (the view's own)
        pen.cut(EFFECT_X, y, &effect.name, EFFECT_W, UiColor::Effect);
        pen.cut(EFFECT_X, y + 1, &effect.mods, EFFECT_W, UiColor::Text);
        pen.cut(EFFECT_X, y + 2, &effect.until, EFFECT_W, UiColor::TextDim);
        y += 3;
    }
}

/// The skills block: the mark, the name, an active's cost at the right,
/// and each effect in a line under it.
fn paint_skills(pen: &mut Pen<'_>, view: &InfoView) {
    let x = MID_X;
    pen.text(x, 15, &view.skills_title, UiColor::TextHighlight);
    if view.skills.is_empty() {
        pen.text(x, 16, NOTHING, UiColor::TextDim);
    }
    for (y, skill) in (16..).step_by(2).zip(view.skills.iter().take(SKILL_MAX)) {
        pen.text(x, y, &skill.mark, UiColor::TextHighlight);
        let name_w = if skill.cost.is_some() { 16 } else { 23 };
        // check-text: not a data name (the view's own)
        pen.cut(x + 2, y, &skill.name, name_w, UiColor::Text);
        if let Some(cost) = &skill.cost {
            let w = i32::try_from(cost.len()).unwrap_or(0);
            pen.text(x + 25 - w, y, cost, UiColor::TextDim);
        }
        pen.cut(x + 2, y + 1, &skill.effect, 23, UiColor::TextDim);
    }
}

/// The weapon slots (the equipped one marked, durability `20/20` or
/// `broken`), armour, accessory, then the spells.
fn paint_right(pen: &mut Pen<'_>, view: &InfoView) {
    let x = RIGHT_X;
    pen.text(x, 1, &view.weapons_title, UiColor::TextHighlight);
    let mut y = 2;
    for weapon in &view.weapons {
        match weapon {
            Some(w) => paint_weapon(pen, view, w, y),
            None => {
                pen.text(x + 2, y, NOTHING, UiColor::TextDim);
            }
        }
        y += 2;
    }
    y += 1;
    for gear in [&view.armour, &view.accessory] {
        paint_gear(pen, gear, y);
        y += 4;
    }
    paint_spells(pen, view, y);
}

/// One weapon on rows `y` and `y + 1`.
fn paint_weapon(pen: &mut Pen<'_>, view: &InfoView, w: &InfoWeaponView, y: i32) {
    let x = RIGHT_X;
    let mark = if w.equipped {
        view.equipped_mark.as_str()
    } else {
        " "
    };
    let mark = &format!("{mark} ");
    let color = if w.broken.is_some() {
        UiColor::HpLow
    } else {
        UiColor::Text
    };
    pen.text(x, y, mark, UiColor::TextHighlight);
    // check-text: not a data name (the view's own)
    pen.cut(x + 2, y, &w.name, 28, color);
    let dur = w
        .broken
        .clone()
        .unwrap_or_else(|| format!("{:>2}/{}", w.left, w.max));
    let dw = i32::try_from(dur.len()).unwrap_or(0);
    let right = x + i32::try_from(RIGHT_W).unwrap_or(0);
    pen.text(right - dw, y, &dur, color);
    pen.cut(x + 2, y + 1, &w.numbers, ITEM_W, UiColor::TextDim);
}

/// A piece of gear from row `y`: its heading, name and bonus.
fn paint_gear(pen: &mut Pen<'_>, gear: &GearView, y: i32) {
    let x = RIGHT_X;
    pen.text(x, y, &gear.title, UiColor::TextHighlight);
    match &gear.item {
        Some((name, bonus)) => {
            pen.cut(x + 2, y + 1, name, ITEM_W, UiColor::Text);
            pen.cut(x + 2, y + 2, bonus, ITEM_W, UiColor::TextDim);
        }
        None => {
            pen.text(x + 2, y + 1, NOTHING, UiColor::TextDim);
        }
    }
}

/// The spells block from row `y`: each learned spell with its uses left,
/// the equipped one marked.
fn paint_spells(pen: &mut Pen<'_>, view: &InfoView, y: i32) {
    let x = RIGHT_X;
    pen.text(x, y, &view.spells_title, UiColor::TextHighlight);
    if view.spells.is_empty() {
        pen.text(x + 2, y + 1, NOTHING, UiColor::TextDim);
    }
    for (row, spell) in (y + 1..).zip(view.spells.iter().take(SPELL_ROWS)) {
        if spell.equipped {
            pen.text(x, row, &view.equipped_mark, UiColor::TextHighlight);
        }
        // check-text: not a data name (the view's own)
        pen.cut(x + 2, row, &spell.name, 28, UiColor::Text);
        let text = format!("{}/{}", spell.left, spell.uses);
        let w = i32::try_from(text.len()).unwrap_or(0);
        let right = x + i32::try_from(RIGHT_W).unwrap_or(0);
        pen.text(right - w, row, &text, UiColor::Text);
    }
}

#[cfg(test)]
mod tests;
