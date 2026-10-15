//! The glyph look of the fight box (combat animations off): a double box
//! centred at the top of the map view, the art's name in its top border;
//! each combatant in a column, its name in the faction colour (inverted
//! while it flashes), HP with a bar, and the result under the target.

use crate::color::UiColor;
use crate::glyph_buffer::{BoxStyle, Cell, GlyphBuffer, Rect};
use crate::prelude::*;
use crate::screen::Ctx;
use crate::screens::battle::layout::MAP_VIEW;
use crate::screens::battle::units::{faction_color, hp_fill};
use crate::screens::battle::view::FightBoxView;

/// The playback box's size, in cells.
pub const BOX_W: i32 = 44;

/// The playback box: centred at the top of the map view.
pub const BOX: Rect = Rect::new(
    MAP_VIEW.x + (MAP_VIEW.w - BOX_W) / 2,
    MAP_VIEW.y + 1,
    BOX_W,
    5,
);

/// Length of the box's HP bars, in cells.
pub const BOX_BAR_CELLS: i32 = 10;

/// Columns of the attacker's and the defender's text in the box.
pub const BOX_COLUMNS: [i32; 2] = [BOX.x + 2, BOX.x + 2 + BOX_W / 2];

/// Paints the fight box `view`.
pub fn paint(ctx: &Ctx, view: &FightBoxView, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let bg = c(UiColor::PanelBg);
    buf.fill_rect(BOX, Cell::new(' ', c(UiColor::Text), bg));
    buf.draw_box(BOX, BoxStyle::Double, c(UiColor::PanelBorderFocus), bg);
    if let Some(name) = &view.banner {
        let text = format!(" {name} ");
        let w = i32::try_from(text.chars().count()).unwrap_or(0);
        let x = BOX.x + ((BOX.w - w) / 2).max(1);
        buf.print(x, BOX.y, &text, c(UiColor::TextHighlight), bg);
    }
    for (f, x) in view.sides.iter().zip(BOX_COLUMNS) {
        let color = c(faction_color(f.faction));
        let (fg, name_bg) = if f.flashing { (bg, color) } else { (color, bg) };
        // check-text: not a data name (the view's own)
        buf.print(x, BOX.y + 1, &f.name, fg, name_bg);
        let hp_text = format!("{} {:>2}", view.hp_name, f.hp);
        buf.print(x, BOX.y + 2, &hp_text, c(UiColor::Text), bg);
        let (filled, bar) = hp_fill(f.hp, f.max, BOX_BAR_CELLS);
        for i in 0..BOX_BAR_CELLS {
            let (glyph, fg) = if i < filled {
                ('█', c(bar))
            } else {
                ('░', c(UiColor::TextDim))
            };
            buf.set(x + 6 + i, BOX.y + 2, Cell::new(glyph, fg, bg));
        }
        if let Some(result) = &f.result {
            let hi = c(UiColor::TextHighlight);
            buf.print(x, BOX.y + 3, result, hi, bg);
        }
    }
}

#[cfg(test)]
mod tests {
    use trpg_core::Faction;

    use super::*;
    use crate::screen::tests::ctx;
    use crate::screens::battle::view::BoxFighterView;

    fn fighter(name: &str, faction: Faction) -> BoxFighterView {
        BoxFighterView {
            name: name.into(),
            faction,
            flashing: false,
            hp: 20,
            max: 20,
            result: None,
        }
    }

    fn painted(change: impl FnOnce(&mut FightBoxView)) -> GlyphBuffer {
        let c = ctx();
        let mut view = FightBoxView {
            banner: None,
            hp_name: "HP".into(),
            sides: [
                fighter("Test Lord", Faction::Player),
                fighter("Brigand", Faction::Enemy),
            ],
        };
        change(&mut view);
        let p = &c.palette;
        let blank = Cell::new(' ', p.get(UiColor::Text), p.get(UiColor::Black));
        let mut buf = GlyphBuffer::new(100, 32, blank);
        paint(&c, &view, &mut buf);
        buf
    }

    fn text(buf: &GlyphBuffer, x: i32, y: i32, n: i32) -> String {
        (x..x + n).map(|x| buf.get(x, y).unwrap().glyph).collect()
    }

    #[test]
    fn a_flashing_name_is_inverted_and_the_result_goes_under_its_fighter() {
        let p = ctx().palette;
        let (lx, rx) = (BOX_COLUMNS[0], BOX_COLUMNS[1]);
        let buf = painted(|_| {});
        assert_eq!(text(&buf, lx, BOX.y + 1, 9), "Test Lord");
        assert_eq!(text(&buf, rx, BOX.y + 1, 7), "Brigand");
        assert_eq!(buf.get(lx, BOX.y + 1).unwrap().fg, p.get(UiColor::Player));
        assert_eq!(buf.get(lx, BOX.y + 1).unwrap().bg, p.get(UiColor::PanelBg));
        assert_eq!(text(&buf, lx, BOX.y + 3, 6), "      ");
        assert_eq!(buf.get(BOX.x, BOX.y).unwrap().glyph, '╔');
        let buf = painted(|v| {
            v.sides[0].flashing = true;
            v.sides[1].result = Some("HIT -8".into());
            v.sides[1].hp = 12;
        });
        assert_eq!(buf.get(lx, BOX.y + 1).unwrap().bg, p.get(UiColor::Player));
        assert_eq!(buf.get(lx, BOX.y + 1).unwrap().fg, p.get(UiColor::PanelBg));
        assert_eq!(text(&buf, rx, BOX.y + 3, 6), "HIT -8");
        assert_eq!(text(&buf, rx, BOX.y + 2, 5), "HP 12");
        // 12 of 20 HP: 6 of the bar's 10 cells.
        assert_eq!(text(&buf, rx + 6, BOX.y + 2, 10), "██████░░░░");
        assert_eq!(text(&buf, lx, BOX.y + 3, 6), "      ");
    }

    #[test]
    fn the_box_and_its_columns_sit_at_the_top_of_the_map_view() {
        // Centred: (70 - 44) / 2 = 13 in; the columns two cells in from
        // each half's left edge.
        assert_eq!(BOX, Rect::new(13, 1, 44, 5));
        assert_eq!(BOX_COLUMNS, [15, 37]);
        let buf = painted(|_| {});
        assert_eq!(text(&buf, 15, 2, 9), "Test Lord");
        assert_eq!(text(&buf, 37, 2, 7), "Brigand");
        assert_eq!(text(&buf, 14, 2, 1), " ");
        assert_eq!(text(&buf, 36, 2, 1), " ");
    }

    #[test]
    fn the_arts_name_is_centred_in_the_top_border() {
        let buf = painted(|v| v.banner = Some("Guard Break".into()));
        let top = text(&buf, BOX.x, BOX.y, BOX.w);
        assert!(top.contains(" Guard Break "), "{top}");
        let x = BOX.x + 16;
        assert_eq!(text(&buf, x, BOX.y, 11), "Guard Break");
        let fg = buf.get(x, BOX.y).unwrap().fg;
        assert_eq!(fg, ctx().palette.get(UiColor::TextHighlight));
    }
}
