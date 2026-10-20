//! The glyph look of the game mode screen: paints a [`ModeSelectView`] as
//! a centred heading, the menu box and the explaining lines under it, the
//! focused mode's line brighter. Everything about the look lives here;
//! nothing here decides what the screen shows. Another look is another
//! `paint` of the same view (ADR-0054). The familiarity screen (0835) has
//! the same shape and is painted by it too.

use super::view::ModeSelectView;
use crate::color::UiColor;
use crate::glyph_buffer::{Cell, GlyphBuffer};
use crate::screen::Ctx;
use crate::screens::{centre_x, print_centred};
use crate::widgets::menu::glyph as menu_glyph;

/// Row of the heading.
const HEADING_ROW: i32 = 8;
/// Top row of the menu box.
const MENU_ROW: i32 = 11;
/// Row of the first explanation line (one blank row between lines).
const TEXT_ROW: i32 = 17;

/// Paints `view` over the whole of `buf`.
pub fn paint(ctx: &Ctx, view: &ModeSelectView, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let black = c(UiColor::Black);
    buf.fill_rect(buf.bounds(), Cell::new(' ', c(UiColor::Text), black));
    print_centred(
        buf,
        HEADING_ROW,
        &view.heading,
        c(UiColor::TextHighlight),
        black,
    );
    let (w, _) = menu_glyph::size(ctx, &view.menu);
    let x = centre_x(buf, usize::try_from(w).unwrap_or(0));
    menu_glyph::paint(ctx, &view.menu, buf, x, MENU_ROW);
    for (y, line) in (TEXT_ROW..).step_by(2).zip(&view.lines) {
        let fg = if line.focused {
            UiColor::Text
        } else {
            UiColor::TextDim
        };
        print_centred(buf, y, &line.text, c(fg), black);
    }
    let bottom = i32::from(buf.height()) - 1;
    print_centred(buf, bottom, &view.help, c(UiColor::TextDim), black);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::{ConsoleSizes, ConsoleSpec};
    use crate::screen::tests::ctx;
    use crate::screens::mode_select::view::ModeLineView;
    use crate::widgets::{MenuItemView, MenuView};

    fn row(buf: &GlyphBuffer, y: i32) -> String {
        (0..i32::from(buf.width()))
            .map(|x| buf.get(x, y).map_or(' ', |c| c.glyph))
            .collect::<String>()
            .trim()
            .to_owned()
    }

    fn view() -> ModeSelectView {
        let item = |label: &str, focused| MenuItemView {
            label: label.into(),
            suffix: None,
            note: None,
            enabled: true,
            focused,
        };
        let line = |text: &str, focused| ModeLineView {
            text: text.to_owned(),
            focused,
        };
        ModeSelectView {
            heading: "PICK".to_owned(),
            menu: MenuView {
                items: vec![item("Classic", false), item("Casual", true)],
                first_shown: 0,
                shown: 2,
            },
            lines: vec![line("first line", false), line("second line", true)],
            help: "help".to_owned(),
        }
    }

    #[test]
    fn the_rows_are_the_heading_the_menu_the_lines_and_the_help() {
        let c = ctx();
        let stale = Cell::new(
            'x',
            c.palette.get(UiColor::Enemy),
            c.palette.get(UiColor::Enemy),
        );
        let mut buf = GlyphBuffer::with_spec(ConsoleSpec::PC, stale);
        paint(&c, &view(), &mut buf);
        assert_eq!(row(&buf, HEADING_ROW), "PICK");
        assert_eq!(row(&buf, MENU_ROW + 1), "│ Classic │");
        assert_eq!(row(&buf, MENU_ROW + 2), "│ Casual  │");
        assert_eq!(row(&buf, TEXT_ROW), "first line");
        assert_eq!(row(&buf, TEXT_ROW + 1), "");
        assert_eq!(row(&buf, TEXT_ROW + 2), "second line");
        assert_eq!(row(&buf, i32::from(ConsoleSpec::PC.rows) - 1), "help");
        // Nothing of the buffer's old contents is left.
        assert!(
            (0..i32::from(ConsoleSpec::PC.rows))
                .flat_map(|y| (0..i32::from(ConsoleSpec::PC.cols)).map(move |x| (x, y)))
                .all(|(x, y)| buf.get(x, y) != Some(&stale))
        );
    }

    #[test]
    fn the_focused_modes_line_is_brighter() {
        let c = ctx();
        let mut buf = GlyphBuffer::with_spec(
            ConsoleSpec::PC,
            Cell::new(
                ' ',
                c.palette.get(UiColor::Text),
                c.palette.get(UiColor::Black),
            ),
        );
        paint(&c, &view(), &mut buf);
        let fg = |y| {
            let x = (0..i32::from(buf.width()))
                .find(|&x| buf.get(x, y).is_some_and(|c| c.glyph != ' '))
                .unwrap();
            buf.get(x, y).unwrap().fg
        };
        assert_eq!(fg(TEXT_ROW), c.palette.get(UiColor::TextDim));
        assert_eq!(fg(TEXT_ROW + 2), c.palette.get(UiColor::Text));
    }
}
