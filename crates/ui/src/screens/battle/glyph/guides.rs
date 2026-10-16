//! The glyph look of the Guides list (ticket 0458): a double box over the
//! map and the side panel, its title over the top border; the topics down
//! the left, the highlighted one inverted and scrolled into view; a line
//! between; the highlighted topic's title (lit) and text on the right.

use crate::color::UiColor;
use crate::glyph_buffer::{BoxStyle, Cell, GlyphBuffer, Rect};
use crate::prelude::*;
use crate::screen::Ctx;
use crate::screens::battle::layout::{MAP_VIEW, SIDE_PANEL};
use crate::screens::battle::view::GuidesView;

/// The box: the map view and the side panel together.
pub const AREA: Rect = Rect::new(
    MAP_VIEW.x,
    MAP_VIEW.y,
    MAP_VIEW.w + SIDE_PANEL.w,
    MAP_VIEW.h,
);

/// First column of the topics.
pub const LIST_X: i32 = AREA.x + 2;

/// Widest topic title, in cells: a tip's or lesson's longest title.
pub const LIST_W: i32 = 30;

/// Column of the line between the list and the text.
pub const DIVIDER_X: i32 = LIST_X + LIST_W + 2;

/// First column of the text.
pub const TEXT_X: i32 = DIVIDER_X + 3;

/// Row of the first topic, and of the highlighted topic's title.
pub const TOP_ROW: i32 = AREA.y + 2;

/// Rows the list has, above the bottom border and a blank row.
pub const LIST_ROWS: i32 = AREA.h - 4;

/// Paints `view` over the map and the side panel.
pub fn paint(ctx: &Ctx, view: &GuidesView, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let bg = c(UiColor::PanelBg);
    let border = c(UiColor::PanelBorder);
    buf.fill_rect(AREA, Cell::new(' ', c(UiColor::Text), bg));
    buf.draw_box(AREA, BoxStyle::Double, border, bg);
    buf.print(
        LIST_X,
        AREA.y,
        &format!(" {} ", view.title),
        c(UiColor::TextHighlight),
        bg,
    );
    for y in AREA.y + 1..AREA.y + AREA.h - 1 {
        buf.set(DIVIDER_X, y, Cell::new('│', border, bg));
    }
    let first = first_shown(view.focus, LIST_ROWS);
    let shown = view.topics.iter().enumerate().skip(first);
    for ((i, title), y) in shown.zip(TOP_ROW..TOP_ROW + LIST_ROWS) {
        let (fg, row_bg) = if i == view.focus {
            (bg, c(UiColor::PanelBorderFocus))
        } else {
            (c(UiColor::Text), bg)
        };
        buf.fill_rect(
            Rect::new(LIST_X - 1, y, LIST_W + 2, 1),
            Cell::new(' ', fg, row_bg),
        );
        buf.print(LIST_X, y, title, fg, row_bg);
    }
    buf.print(
        TEXT_X,
        TOP_ROW,
        &view.heading,
        c(UiColor::TextHighlight),
        bg,
    );
    // The text starts after a blank row.
    let mut y = TOP_ROW + 1;
    for line in &view.lines {
        y += 1;
        buf.print(TEXT_X, y, line, c(UiColor::Text), bg);
    }
}

/// The first topic shown so that `focus` is within `rows` rows.
fn first_shown(focus: usize, rows: i32) -> usize {
    let rows = usize::try_from(rows).unwrap_or(1).max(1);
    focus.saturating_sub(rows - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_box_covers_the_map_and_the_panel_and_the_text_fits() {
        assert_eq!(AREA, Rect::new(0, 0, 100, 30));
        // A text line of 60 cells ends inside the right border.
        let line = i32::try_from(trpg_content::tip::MAX_LINE_CHARS).unwrap();
        assert!(TEXT_X + line < AREA.x + AREA.w - 1);
        assert_eq!(
            LIST_W,
            i32::try_from(trpg_content::tip::MAX_TITLE_CHARS).unwrap()
        );
    }

    /// The text of row `y` of `buf`.
    fn row(buf: &GlyphBuffer, y: i32) -> String {
        (0..AREA.w)
            .map(|x| buf.get(x, y).map_or(' ', |c| c.glyph))
            .collect()
    }

    /// A long list stays inside the box: as many topics as it has rows,
    /// from the top or scrolled to the last; the text starts a blank row
    /// under its heading.
    #[test]
    fn a_long_list_fits_in_the_box() {
        use crate::Rgb;
        use crate::console::ConsoleSpec;
        let c = crate::screen::tests::ctx();
        let paint_at = |focus: usize| {
            let view = GuidesView {
                title: "G".into(),
                topics: (0..40).map(|i| format!("t{i}")).collect(),
                focus,
                heading: "Head".into(),
                lines: vec!["one".into(), "two".into()],
            };
            let blank = Cell::new(' ', Rgb::new(0, 0, 0), Rgb::new(0, 0, 0));
            let mut buf = GlyphBuffer::with_spec(ConsoleSpec::PC, blank);
            paint(&c, &view, &mut buf);
            buf
        };
        let last = TOP_ROW + LIST_ROWS - 1;
        assert_eq!(last, 27);
        let top = paint_at(0);
        assert!(row(&top, TOP_ROW).contains("t0 "));
        assert!(row(&top, last).contains("t25 "));
        assert!(!row(&top, last + 1).contains('t'));
        assert!(row(&top, AREA.h - 1).starts_with("╚═══"));
        let end = paint_at(39);
        assert!(row(&end, TOP_ROW).contains("t14 "));
        assert!(row(&end, last).contains("t39 "));
        // The heading, a blank row, then the text.
        assert!(row(&top, TOP_ROW).contains("Head"));
        assert!(!row(&top, TOP_ROW + 1).contains("one"));
        assert!(row(&top, TOP_ROW + 2).contains("one"));
        assert!(row(&top, TOP_ROW + 3).contains("two"));
    }

    #[test]
    fn the_list_scrolls_to_keep_the_focus_in_view() {
        assert_eq!(first_shown(0, 5), 0);
        assert_eq!(first_shown(4, 5), 0);
        assert_eq!(first_shown(5, 5), 1);
        assert_eq!(first_shown(9, 5), 5);
        assert_eq!(first_shown(3, 0), 3);
    }
}
