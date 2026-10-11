//! The glyph look of a [`Menu`](super::Menu): paints a [`MenuView`] as a
//! single-line box of text rows with the focused one as a bar. Everything
//! about the look lives here; nothing here decides what the menu shows.
//! Another look is another `paint` of the same view (ADR-0054).

use super::view::{MenuItemView, MenuView};
use crate::color::{Palette, UiColor};
use crate::glyph_buffer::{BoxStyle, Cell, GlyphBuffer, Rect};

/// Cells the label and suffix of `item` take.
fn width(item: &MenuItemView) -> usize {
    let suffix = item
        .suffix
        .as_ref()
        .map_or(0, |(t, _)| t.chars().count() + 1);
    item.label.chars().count() + suffix
}

/// Box size in cells: the widest label plus a space and a border on each
/// side, by one row per item plus the border.
pub fn size(view: &MenuView) -> (i32, i32) {
    let widest = view.items.iter().map(width).max().unwrap_or(0);
    let w = i32::try_from(widest).unwrap_or(i32::MAX).saturating_add(4);
    let h = i32::try_from(view.items.len())
        .unwrap_or(i32::MAX)
        .saturating_add(2);
    (w, h)
}

/// Paints `view` with its top-left corner at `(x, y)`: a `panel_bg` box
/// with a `panel_border` single line, items in `text`, disabled ones in
/// `text_dim`, and the focused one as a bar of `panel_bg` text on
/// `panel_border_focus`.
pub fn paint(palette: &Palette, view: &MenuView, buf: &mut GlyphBuffer, x: i32, y: i32) {
    let color = |u| palette.get(u);
    let bg = color(UiColor::PanelBg);
    let (w, h) = size(view);
    let rect = Rect::new(x, y, w, h);
    buf.fill_rect(rect, Cell::new(' ', color(UiColor::Text), bg));
    buf.draw_box(rect, BoxStyle::Single, color(UiColor::PanelBorder), bg);
    for (row, item) in (y + 1..).zip(&view.items) {
        let (fg, row_bg) = match (item.focused, item.enabled) {
            (_, false) => (color(UiColor::TextDim), bg),
            (true, true) => (bg, color(UiColor::PanelBorderFocus)),
            (false, true) => (color(UiColor::Text), bg),
        };
        buf.fill_rect(Rect::new(x + 1, row, w - 2, 1), Cell::new(' ', fg, row_bg));
        buf.print(x + 2, row, &item.label, fg, row_bg);
        if let Some((text, suffix_color)) = &item.suffix {
            let at = x + 3 + i32::try_from(item.label.chars().count()).unwrap_or(0);
            // On the focus bar the colour would vanish: keep the bar's.
            let fg = if item.focused && item.enabled {
                fg
            } else {
                color(*suffix_color)
            };
            buf.print(at, row, text, fg, row_bg);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::tests::game_palette;

    fn item(label: &str, enabled: bool, focused: bool) -> MenuItemView {
        MenuItemView {
            label: label.to_owned(),
            suffix: None,
            enabled,
            focused,
        }
    }

    fn painted(view: &MenuView) -> (GlyphBuffer, Palette) {
        let p = game_palette();
        let (w, h) = size(view);
        let blank = Cell::new(' ', p.get(UiColor::Text), p.get(UiColor::Black));
        let mut buf = GlyphBuffer::new(
            u16::try_from(w + 2).unwrap(),
            u16::try_from(h + 2).unwrap(),
            blank,
        );
        paint(&p, view, &mut buf, 1, 1);
        (buf, p)
    }

    fn row(buf: &GlyphBuffer, y: i32) -> String {
        (0..i32::from(buf.width()))
            .map(|x| buf.get(x, y).map_or(' ', |c| c.glyph))
            .collect()
    }

    #[test]
    fn the_box_is_the_widest_item_plus_a_space_and_border_each_side() {
        assert_eq!(size(&MenuView { items: vec![] }), (4, 2));
        let view = MenuView {
            items: vec![item("a", true, true), item("bcd", false, false)],
        };
        assert_eq!(size(&view), (7, 4));
        // A suffix counts, with the space before it.
        let mut with = item("ab", true, true);
        with.suffix = Some(("(x)".to_owned(), UiColor::HpLow));
        assert_eq!(size(&MenuView { items: vec![with] }), (10, 3));
    }

    #[test]
    fn items_are_painted_one_to_a_row_inside_the_border() {
        let view = MenuView {
            items: vec![item("Move", true, false), item("Wait", true, true)],
        };
        let (buf, _) = painted(&view);
        assert_eq!(row(&buf, 1).trim(), "┌──────┐");
        assert_eq!(row(&buf, 2).trim(), "│ Move │");
        assert_eq!(row(&buf, 3).trim(), "│ Wait │");
        assert_eq!(row(&buf, 4).trim(), "└──────┘");
    }

    #[test]
    fn the_focused_item_is_a_bar_and_a_disabled_one_is_dim() {
        let view = MenuView {
            items: vec![
                item("Move", true, false),
                item("Attack", false, false),
                item("Wait", true, true),
            ],
        };
        let (buf, p) = painted(&view);
        let bar = p.get(UiColor::PanelBorderFocus);
        let at = |x, y| *buf.get(x, y).unwrap();
        // The bar fills the row inside the border, in the panel's colour.
        assert_eq!(at(2, 4).bg, bar);
        assert_eq!(at(2, 4).fg, p.get(UiColor::PanelBg));
        assert_eq!(at(8, 4).bg, bar);
        assert_ne!(at(2, 2).bg, bar);
        assert_eq!(at(3, 2).fg, p.get(UiColor::Text));
        assert_eq!(at(3, 3).fg, p.get(UiColor::TextDim));
        assert_ne!(at(3, 3).bg, bar);
    }

    #[test]
    fn a_suffix_keeps_its_colour_except_on_the_focus_bar() {
        let mut broken = item("Sword", false, false);
        broken.suffix = Some(("(broken)".to_owned(), UiColor::HpLow));
        let mut focused = item("Axe", true, true);
        focused.suffix = Some(("(worn)".to_owned(), UiColor::HpLow));
        let view = MenuView {
            items: vec![broken, focused],
        };
        let (buf, p) = painted(&view);
        let at = |x, y| *buf.get(x, y).unwrap();
        // After the label and a space: column 3 + 5 on the first row.
        assert_eq!(row(&buf, 2).trim(), "│ Sword (broken) │");
        assert_eq!(at(9, 2).glyph, '(');
        assert_eq!(at(9, 2).fg, p.get(UiColor::HpLow));
        assert_eq!(at(7, 3).glyph, '(');
        assert_eq!(at(7, 3).fg, p.get(UiColor::PanelBg));
    }
}
