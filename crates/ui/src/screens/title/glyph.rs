//! The glyph look of the title screen: paints a [`TitleView`] as centred
//! text with the menu box under it. Everything about the look lives here
//! (the rows, the colours); nothing here decides what the screen shows.
//! Another look is another `paint` of the same view (ADR-0054).

use super::view::TitleView;
use crate::color::UiColor;
use crate::glyph_buffer::{Cell, GlyphBuffer};
use crate::screen::Ctx;
use crate::screens::{centre_x, paint_debug_hint, print_centred};
use crate::widgets::menu::glyph as menu_glyph;

/// Row of the title text.
const TITLE_ROW: i32 = 9;
/// Row of the subtitle.
const SUBTITLE_ROW: i32 = 11;
/// Top row of the menu box (and of the prompt).
const MENU_ROW: i32 = 14;

/// Paints `view` over the whole of `buf`.
pub fn paint(ctx: &Ctx, view: &TitleView, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let black = c(UiColor::Black);
    buf.fill_rect(buf.bounds(), Cell::new(' ', c(UiColor::Text), black));
    print_centred(
        buf,
        TITLE_ROW,
        &view.title,
        c(UiColor::TextHighlight),
        black,
    );
    print_centred(
        buf,
        SUBTITLE_ROW,
        &view.subtitle,
        c(UiColor::TextDim),
        black,
    );
    let bottom = i32::from(buf.height()) - 1;
    if let Some(prompt) = &view.prompt {
        print_centred(buf, MENU_ROW, prompt, c(UiColor::TextDim), black);
    }
    if let Some(menu) = &view.menu {
        let (w, h) = menu_glyph::size(menu);
        let x = centre_x(buf, usize::try_from(w).unwrap_or(0));
        menu_glyph::paint(&ctx.palette, menu, buf, x, MENU_ROW);
        if let Some(notice) = &view.notice {
            print_centred(buf, MENU_ROW + h + 1, notice, c(UiColor::HpLow), black);
        }
    }
    if let Some(help) = &view.help {
        print_centred(buf, bottom, help, c(UiColor::TextDim), black);
    }
    if let Some(hint) = &view.debug_hint {
        paint_debug_hint(&ctx.palette, hint, buf, bottom);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::{CONSOLE_H, CONSOLE_W};
    use crate::screen::tests::ctx;
    use crate::widgets::{MenuItemView, MenuView};

    /// Row `y` of `buf` as text.
    fn text(buf: &GlyphBuffer, y: i32) -> String {
        (0..i32::from(buf.width()))
            .map(|x| buf.get(x, y).map_or(' ', |c| c.glyph))
            .collect()
    }

    fn item(label: &str, focused: bool) -> MenuItemView {
        MenuItemView {
            label: label.to_owned(),
            suffix: None,
            enabled: true,
            focused,
        }
    }

    fn view() -> TitleView {
        TitleView {
            title: "THE GAME".to_owned(),
            subtitle: "a subtitle".to_owned(),
            prompt: None,
            menu: Some(MenuView {
                items: vec![item("New Game", true), item("Quit", false)],
            }),
            notice: None,
            help: Some("help".to_owned()),
            debug_hint: None,
        }
    }

    fn painted(c: &Ctx, view: &TitleView) -> GlyphBuffer {
        let stale = Cell::new(
            'x',
            c.palette.get(UiColor::Enemy),
            c.palette.get(UiColor::Enemy),
        );
        let mut buf = GlyphBuffer::new(CONSOLE_W, CONSOLE_H, stale);
        paint(c, view, &mut buf);
        buf
    }

    /// The text is centred on its row.
    fn centred(buf: &GlyphBuffer, y: i32) -> String {
        text(buf, y).trim().to_owned()
    }

    #[test]
    fn the_title_and_subtitle_have_their_rows_and_the_menu_starts_under_them() {
        let c = ctx();
        let buf = painted(&c, &view());
        assert_eq!(centred(&buf, TITLE_ROW), "THE GAME");
        assert_eq!(centred(&buf, SUBTITLE_ROW), "a subtitle");
        assert!(text(&buf, MENU_ROW).contains('┌'));
        assert!(text(&buf, MENU_ROW + 1).contains("│ New Game │"));
        assert!(text(&buf, MENU_ROW + 2).contains("│ Quit     │"));
        assert_eq!(centred(&buf, i32::from(CONSOLE_H) - 1), "help");
        // The title is centred on the console.
        let t = text(&buf, TITLE_ROW);
        let left = t.chars().take_while(|&ch| ch == ' ').count();
        assert_eq!(left, (usize::from(CONSOLE_W) - 8) / 2);
        // The menu's box is too.
        let row = text(&buf, MENU_ROW);
        assert_eq!(row.find('┌'), Some((usize::from(CONSOLE_W) - 12) / 2));
    }

    #[test]
    fn colours() {
        let c = ctx();
        let buf = painted(&c, &view());
        let fg_at = |y, ch: char| {
            let x = text(&buf, y).chars().position(|g| g == ch).unwrap();
            buf.get(i32::try_from(x).unwrap(), y).unwrap().fg
        };
        assert_eq!(fg_at(TITLE_ROW, 'T'), c.palette.get(UiColor::TextHighlight));
        assert_eq!(fg_at(SUBTITLE_ROW, 's'), c.palette.get(UiColor::TextDim));
        assert_eq!(
            fg_at(i32::from(CONSOLE_H) - 1, 'h'),
            c.palette.get(UiColor::TextDim)
        );
    }

    #[test]
    fn the_prompt_takes_the_menus_place() {
        let c = ctx();
        let mut v = view();
        v.menu = None;
        v.help = None;
        v.prompt = Some("Press any key".to_owned());
        let buf = painted(&c, &v);
        assert_eq!(centred(&buf, MENU_ROW), "Press any key");
        assert!(!(0..i32::from(CONSOLE_H)).any(|y| text(&buf, y).contains('┌')));
        assert_eq!(centred(&buf, i32::from(CONSOLE_H) - 1), "");
        let x = text(&buf, MENU_ROW).find("Press").unwrap();
        assert_eq!(
            buf.get(i32::try_from(x).unwrap(), MENU_ROW).unwrap().fg,
            c.palette.get(UiColor::TextDim)
        );
    }

    /// The notice sits one blank row under the menu's box, in the warning
    /// colour.
    #[test]
    fn the_notice_is_under_the_menu() {
        let c = ctx();
        let mut v = view();
        v.notice = Some("This save can't be read".to_owned());
        let buf = painted(&c, &v);
        let below = MENU_ROW + 4 + 1;
        assert!(text(&buf, below - 1).trim().is_empty());
        assert_eq!(centred(&buf, below), "This save can't be read");
        let x = text(&buf, below).find("This").unwrap();
        assert_eq!(
            buf.get(i32::try_from(x).unwrap(), below).unwrap().fg,
            c.palette.get(UiColor::HpLow)
        );
        // None with the prompt: there is no menu to be under.
        v.menu = None;
        let buf = painted(&c, &v);
        assert!(!(0..i32::from(CONSOLE_H)).any(|y| text(&buf, y).contains("This save")));
    }

    #[test]
    fn the_debug_hint_is_on_the_bottom_row_at_the_right() {
        let c = ctx();
        let mut v = view();
        v.debug_hint = Some("F2 debug".to_owned());
        let buf = painted(&c, &v);
        let bottom = text(&buf, i32::from(CONSOLE_H) - 1);
        assert!(bottom.ends_with("F2 debug "), "{bottom:?}");
        assert!(bottom.contains("help"));
        // With the prompt too.
        v.menu = None;
        v.help = None;
        v.prompt = Some("Press any key".to_owned());
        let buf = painted(&c, &v);
        assert!(text(&buf, i32::from(CONSOLE_H) - 1).ends_with("F2 debug "));
    }

    /// Opaque screens must paint every cell, not rely on `Game` clearing.
    #[test]
    fn it_covers_the_whole_buffer() {
        let c = ctx();
        let stale = Cell::new(
            'x',
            c.palette.get(UiColor::Enemy),
            c.palette.get(UiColor::Enemy),
        );
        let buf = painted(&c, &view());
        let left = (0..i32::from(CONSOLE_H))
            .flat_map(|y| (0..i32::from(CONSOLE_W)).map(move |x| (x, y)))
            .filter(|&(x, y)| buf.get(x, y) == Some(&stale))
            .count();
        assert_eq!(left, 0);
    }
}
