//! The Playdate's look of the fight box (ticket 1039), shown when combat
//! animations are off: a window across the top of the screen as on the
//! PC, the art's name in the top of its frame; each combatant in a
//! column, its name the bar while it flashes, HP with a bar, and the
//! result under the one struck.

use crate::prelude::*;

use super::hp_bar;
use crate::glyph_buffer::{GlyphBuffer, Rect};
use crate::screen::Ctx;
use crate::screens::battle::glyph::fight_box::{BOX_BAR_CELLS, BOX_W};
use crate::screens::battle::view::FightBoxView;
use crate::screens::playdate::{self as pd, cells, cut, inks, to_usize};

/// The box over `area`: [`BOX_W`] wide (as on the PC), centred across its
/// top edge, five rows tall. Shared with the EXP and class boxes.
pub fn box_rect(area: Rect, h: i32) -> Rect {
    let w = BOX_W.min(area.w);
    Rect::new(area.x + (area.w - w) / 2, area.y, w, h)
}

/// The columns of the attacker's and the defender's text in `rect`.
pub fn columns(rect: Rect) -> [i32; 2] {
    [rect.x + 2, rect.x + 2 + rect.w / 2]
}

/// Paints the fight box `view` across the top of `area`.
pub fn paint(ctx: &Ctx, view: &FightBoxView, area: Rect, buf: &mut GlyphBuffer) {
    let ink = inks(ctx);
    let rect = box_rect(area, 5);
    pd::window(ctx, buf, rect, view.banner.as_deref().unwrap_or_default());
    let room = to_usize(rect.w / 2 - 3);
    for (f, x) in view.sides.iter().zip(columns(rect)) {
        // check-text: not a data name (the view's own)
        let name = cut(&f.name, room);
        if f.flashing {
            pd::bar(ctx, buf, x - 1, rect.y + 1, cells(&name) + 2, &name);
        } else {
            buf.print(x, rect.y + 1, &name, ink.text, ink.inside);
        }
        let hp = format!("{} {:>2}", view.hp_name, f.hp);
        buf.print(x, rect.y + 2, &hp, ink.text, ink.inside);
        let bar = (x + 6, rect.y + 2);
        hp_bar(ctx, buf, bar, BOX_BAR_CELLS, (f.hp, f.hp, f.max));
        if let Some(result) = &f.result {
            buf.print(x, rect.y + 3, &cut(result, room), ink.text, ink.inside);
        }
    }
}
