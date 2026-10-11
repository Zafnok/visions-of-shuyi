//! The glyph look of the Game Over and "To be continued" screens: a
//! centred heading, Game Over's menu box under it and the help line on the
//! bottom row. Everything about the look lives here; nothing here decides
//! what the screens show. Another look is another `paint` of the same view
//! (ADR-0054).

use super::view::{GameOverView, ToBeContinuedView};
use crate::color::UiColor;
use crate::glyph_buffer::{Cell, GlyphBuffer};
use crate::screen::Ctx;
use crate::screens::{centre_x, print_centred};
use crate::widgets::menu::glyph as menu_glyph;

/// Row of the headings.
const HEADING_ROW: i32 = 10;
/// Top row of Game Over's menu.
const MENU_ROW: i32 = 13;

/// Fills `buf` with blank cells and prints `heading` and `help` on it.
fn paint_text(ctx: &Ctx, heading: &str, help: &str, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let black = c(UiColor::Black);
    buf.fill_rect(buf.bounds(), Cell::new(' ', c(UiColor::Text), black));
    print_centred(buf, HEADING_ROW, heading, c(UiColor::TextHighlight), black);
    let bottom = i32::from(buf.height()) - 1;
    print_centred(buf, bottom, help, c(UiColor::TextDim), black);
}

/// Paints Game Over over the whole of `buf`.
pub fn paint(ctx: &Ctx, view: &GameOverView, buf: &mut GlyphBuffer) {
    paint_text(ctx, &view.heading, &view.help, buf);
    let (w, _) = menu_glyph::size(&view.menu);
    let x = centre_x(buf, usize::try_from(w).unwrap_or(0));
    menu_glyph::paint(&ctx.palette, &view.menu, buf, x, MENU_ROW);
}

/// Paints "To be continued" over the whole of `buf`.
pub fn paint_to_be_continued(ctx: &Ctx, view: &ToBeContinuedView, buf: &mut GlyphBuffer) {
    paint_text(ctx, &view.heading, &view.help, buf);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::{CONSOLE_H, CONSOLE_W};
    use crate::screen::tests::ctx;
    use crate::widgets::{MenuItemView, MenuView};

    fn row(buf: &GlyphBuffer, y: i32) -> String {
        (0..i32::from(buf.width()))
            .map(|x| buf.get(x, y).map_or(' ', |c| c.glyph))
            .collect::<String>()
            .trim()
            .to_owned()
    }

    fn blank(c: &Ctx) -> GlyphBuffer {
        let stale = Cell::new(
            'x',
            c.palette.get(UiColor::Enemy),
            c.palette.get(UiColor::Enemy),
        );
        GlyphBuffer::new(CONSOLE_W, CONSOLE_H, stale)
    }

    #[test]
    fn game_over_is_a_heading_a_menu_and_a_help_line() {
        let c = ctx();
        let item = |label: &str, focused| MenuItemView {
            label: label.to_owned(),
            suffix: None,
            enabled: true,
            focused,
        };
        let view = GameOverView {
            heading: "OVER".to_owned(),
            menu: MenuView {
                items: vec![item("Retry", true), item("Title", false)],
            },
            help: "help".to_owned(),
        };
        let mut buf = blank(&c);
        paint(&c, &view, &mut buf);
        assert_eq!(row(&buf, HEADING_ROW), "OVER");
        assert_eq!(row(&buf, MENU_ROW + 1), "│ Retry │");
        assert_eq!(row(&buf, MENU_ROW + 2), "│ Title │");
        assert_eq!(row(&buf, i32::from(CONSOLE_H) - 1), "help");
        // The heading is centred.
        let x = (0..i32::from(CONSOLE_W))
            .find(|&x| buf.get(x, HEADING_ROW).is_some_and(|c| c.glyph == 'O'))
            .unwrap();
        assert_eq!(x, (i32::from(CONSOLE_W) - 4) / 2);
        // Nothing of the buffer's old contents is left.
        assert!(buf.get(0, 0).is_some_and(|c| c.glyph == ' '));
    }

    #[test]
    fn to_be_continued_is_a_heading_and_a_help_line() {
        let c = ctx();
        let view = ToBeContinuedView {
            heading: "TBC".to_owned(),
            help: "help".to_owned(),
        };
        let mut buf = blank(&c);
        paint_to_be_continued(&c, &view, &mut buf);
        assert_eq!(row(&buf, HEADING_ROW), "TBC");
        assert_eq!(row(&buf, i32::from(CONSOLE_H) - 1), "help");
        assert_eq!(row(&buf, MENU_ROW + 1), "");
    }
}
