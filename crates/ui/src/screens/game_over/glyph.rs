//! The glyph look of the Game Over and "To be continued" screens.
//! Everything about the look lives here; nothing here decides what the
//! screens show. Another look is another `paint` of the same view
//! (ADR-0054).
//!
//! Game Over (`docs/design/look-and-feel.md`, *Game Over screen*): what
//! the buffer already shows (the lost battle) fades to black; the logo
//! fades in where the title has it, drained of its colour; the words
//! `GAME OVER` fade in under it, as wide as it; then the menu and the help
//! line are there. "To be continued" is a centred heading and the help
//! line.

use super::view::{FULL, GameOverView, ToBeContinuedView};
use crate::color::UiColor;
use crate::console::{CELL_H_PX, CELL_W_PX, CONSOLE_W};
use crate::glyph_buffer::{Cell, GlyphBuffer, Layer, Paint, PxRect, Sprite};
use crate::screen::Ctx;
use crate::screens::title::glyph::{LOGO_SCALE, logo_sprite};
use crate::screens::{centre_x, print_centred};
use crate::widgets::menu::glyph as menu_glyph;

/// Row of "To be continued"'s heading.
const HEADING_ROW: i32 = 10;
/// Top row of Game Over's words ([`words_sprite`] says it in words).
const WORDS_ROW: i32 = 19;
/// Top row of Game Over's menu.
const MENU_ROW: i32 = 24;

/// The part of the animation in which the lost battle fades out, as
/// shares of it: from, to.
const BATTLE_OUT: (f32, f32) = (0.0, 0.35);
/// The part in which the logo fades in.
const LOGO_IN: (f32, f32) = (0.3, 0.7);
/// The part in which the words fade in.
const WORDS_IN: (f32, f32) = (0.55, 0.95);

/// How far through the part `(from, to)` an animation at `progress` is,
/// 0 to 1.
fn ramp(progress: u8, (from, to): (f32, f32)) -> f32 {
    let at = f32::from(progress) / f32::from(FULL);
    ((at - from) / (to - from)).clamp(0.0, 1.0)
}

/// `part` (0 to 1) as an opacity.
fn opacity(part: f32) -> u8 {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // 0 to 255
    let opacity = (part * f32::from(FULL)).round() as u8;
    opacity
}

/// How bright the lost battle still is behind Game Over at `progress`: 1
/// as it was, 0 gone.
pub fn battle_brightness(progress: u8) -> f32 {
    1.0 - ramp(progress, BATTLE_OUT)
}

/// The words picture `image`, `size` pixels big: centred like the logo,
/// its top on row 19, over everything.
pub fn words_sprite(image: trpg_content::ImageId, size: (u32, u32)) -> Sprite {
    let (w, h) = (
        i32::try_from(size.0).unwrap_or(0),
        i32::try_from(size.1).unwrap_or(0),
    );
    let console_px = i32::from(CONSOLE_W) * i32::from(CELL_W_PX);
    let dest = PxRect::new(
        (console_px - w * LOGO_SCALE) / 2,
        WORDS_ROW * i32::from(CELL_H_PX),
        w * LOGO_SCALE,
        h * LOGO_SCALE,
    );
    Sprite::new(image, PxRect::new(0, 0, w, h), dest, Layer::Over)
}

/// Fills `buf` with blank cells.
fn clear(ctx: &Ctx, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    buf.fill_rect(
        buf.bounds(),
        Cell::new(' ', c(UiColor::Text), c(UiColor::Black)),
    );
}

/// Prints `help` on the bottom row of `buf`.
fn paint_help(ctx: &Ctx, help: &str, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let bottom = i32::from(buf.height()) - 1;
    print_centred(buf, bottom, help, c(UiColor::TextDim), c(UiColor::Black));
}

/// Paints Game Over over the whole of `buf`. While the lost battle still
/// shows ([`battle_brightness`]) it is what `buf` holds already, darkened
/// here; after that nothing of `buf`'s old contents is left.
pub fn paint(ctx: &Ctx, view: &GameOverView, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let black = c(UiColor::Black);
    let battle = battle_brightness(view.progress);
    if battle > 0.0 {
        buf.fade(battle);
    } else {
        clear(ctx, buf);
    }
    let logo = opacity(ramp(view.progress, LOGO_IN));
    if let Some((image, size)) = ctx.content.title.logo
        && logo > 0
    {
        let mut sprite = logo_sprite(image, size).painted(Paint::Dimmed);
        sprite.opacity = logo;
        buf.add_sprite(sprite);
    }
    let words = ramp(view.progress, WORDS_IN);
    if words > 0.0 {
        if let Some((image, size)) = ctx.content.title.game_over {
            let mut sprite = words_sprite(image, size);
            sprite.opacity = opacity(words);
            buf.add_sprite(sprite);
        } else {
            // No picture of the words: the heading as text.
            let fg = c(UiColor::TextHighlight).scale(words);
            print_centred(buf, WORDS_ROW + 1, &view.heading, fg, black);
        }
    }
    if let Some(menu) = &view.menu {
        let (w, _) = menu_glyph::size(menu);
        let x = centre_x(buf, usize::try_from(w).unwrap_or(0));
        menu_glyph::paint(&ctx.palette, menu, buf, x, MENU_ROW);
    }
    if let Some(help) = &view.help {
        paint_help(ctx, help, buf);
    }
}

/// Paints "To be continued" over the whole of `buf`.
pub fn paint_to_be_continued(ctx: &Ctx, view: &ToBeContinuedView, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    clear(ctx, buf);
    let fg = c(UiColor::TextHighlight);
    print_centred(buf, HEADING_ROW, &view.heading, fg, c(UiColor::Black));
    paint_help(ctx, &view.help, buf);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::CONSOLE_H;
    use crate::screen::tests::ctx;
    use crate::widgets::{MenuItemView, MenuView};

    fn row(buf: &GlyphBuffer, y: i32) -> String {
        (0..i32::from(buf.width()))
            .map(|x| buf.get(x, y).map_or(' ', |c| c.glyph))
            .collect::<String>()
            .trim()
            .to_owned()
    }

    fn stale(c: &Ctx) -> Cell {
        Cell::new(
            'x',
            c.palette.get(UiColor::Enemy),
            c.palette.get(UiColor::Enemy),
        )
    }

    fn blank(c: &Ctx) -> GlyphBuffer {
        GlyphBuffer::new(CONSOLE_W, CONSOLE_H, stale(c))
    }

    /// Game Over at `progress`, with its menu and help once it is over.
    fn view(progress: u8) -> GameOverView {
        let item = |label: &str, focused| MenuItemView {
            label: label.to_owned(),
            suffix: None,
            enabled: true,
            focused,
        };
        let over = progress == FULL;
        GameOverView {
            heading: "OVER".to_owned(),
            progress,
            menu: over.then(|| MenuView {
                items: vec![item("Retry", true), item("Title", false)],
            }),
            help: over.then(|| "help".to_owned()),
        }
    }

    fn painted(c: &Ctx, progress: u8) -> GlyphBuffer {
        let mut buf = blank(c);
        paint(c, &view(progress), &mut buf);
        buf
    }

    /// The logo and the words as they are drawn at `progress`: each one's
    /// opacity, or `None` while it isn't drawn.
    fn pictures(c: &Ctx, progress: u8) -> (Option<u8>, Option<u8>) {
        let sprites = painted(c, progress).sprites();
        let (logo, _) = c.content.title.logo.unwrap();
        let (words, _) = c.content.title.game_over.unwrap();
        let of = |image| sprites.iter().find(|s| s.image == image).map(|s| s.opacity);
        assert!(sprites.iter().all(|s| s.image == logo || s.image == words));
        (of(logo), of(words))
    }

    #[test]
    fn the_finished_screen_is_the_logo_the_words_a_menu_and_a_help_line() {
        let c = ctx();
        let buf = painted(&c, FULL);
        let (logo, logo_size) = c.content.title.logo.unwrap();
        let (words, words_size) = c.content.title.game_over.unwrap();
        assert_eq!(words_size, (52, 7), "the shipped words");
        // The logo where the title has it, grey; the words under it, as
        // wide and in their own colour.
        let grey = logo_sprite(logo, logo_size).painted(Paint::Dimmed);
        let gold = words_sprite(words, words_size);
        assert_eq!(buf.sprites(), [grey, gold]);
        assert_eq!(gold.src, PxRect::new(0, 0, 52, 7));
        assert_eq!(gold.dest, PxRect::new(192, 304, 416, 56));
        assert_eq!(
            (gold.layer, gold.flip_x, gold.opacity, gold.paint),
            (Layer::Over, false, 255, Paint::Image)
        );
        assert_eq!((gold.dest.x, gold.dest.w), (grey.dest.x, grey.dest.w));
        assert!(grey.dest.y + grey.dest.h <= gold.dest.y);
        // Another size is centred too, on the same row.
        assert_eq!(
            words_sprite(words, (10, 4)).dest,
            PxRect::new(360, 304, 80, 32)
        );
        // The menu under the words, clear of them; the help at the bottom.
        assert!(gold.dest.y + gold.dest.h <= MENU_ROW * i32::from(CELL_H_PX));
        assert_eq!(row(&buf, MENU_ROW), "┌───────┐");
        assert_eq!(row(&buf, MENU_ROW + 1), "│ Retry │");
        assert_eq!(row(&buf, MENU_ROW + 2), "│ Title │");
        let x = (0..i32::from(CONSOLE_W))
            .find(|&x| buf.get(x, MENU_ROW).is_some_and(|c| c.glyph == '┌'))
            .unwrap();
        assert_eq!(x, (i32::from(CONSOLE_W) - 9) / 2);
        assert_eq!(row(&buf, i32::from(CONSOLE_H) - 1), "help");
        // The words are a picture: the heading isn't printed as well.
        assert!((0..i32::from(CONSOLE_H)).all(|y| !row(&buf, y).contains("OVER")));
        // Nothing of the buffer's old contents is left.
        let old = stale(&c);
        assert!((0..i32::from(CONSOLE_H)).all(|y| (0..100).all(|x| buf.get(x, y) != Some(&old))));
    }

    /// What is under the screen fades to black over the first 35% of the
    /// animation, and is gone after it.
    #[test]
    fn the_lost_battle_fades_out_first() {
        let c = ctx();
        assert!((battle_brightness(0) - 1.0).abs() < 1e-6);
        assert!((battle_brightness(51) - (1.0 - 0.2 / 0.35)).abs() < 1e-6);
        assert!(battle_brightness(89) > 0.0);
        assert!(battle_brightness(90).abs() < 1e-6);
        assert!(battle_brightness(FULL).abs() < 1e-6);
        // At the start what was there is untouched.
        assert_eq!(painted(&c, 0), blank(&c));
        // On the way it is as dark as the brightness says, nothing else.
        let mut faded = blank(&c);
        faded.fade(battle_brightness(51));
        assert_eq!(painted(&c, 51), faded);
        assert_ne!(faded, blank(&c));
        // After it the screen is black cells.
        let empty = Cell::new(
            ' ',
            c.palette.get(UiColor::Text),
            c.palette.get(UiColor::Black),
        );
        let buf = painted(&c, 90);
        assert!((0..i32::from(CONSOLE_H)).all(|y| (0..100).all(|x| buf.get(x, y) == Some(&empty))));
    }

    /// The logo fades in from 30% to 70%, the words from 55% to 95%; the
    /// menu and the help line are not the skin's to time.
    #[test]
    fn the_logo_fades_in_then_the_words() {
        let c = ctx();
        assert_eq!(pictures(&c, 0), (None, None));
        assert_eq!(pictures(&c, 76), (None, None));
        // 0.4 of the way: a quarter into the logo's part.
        assert_eq!(pictures(&c, 102), (Some(64), None));
        assert_eq!(pictures(&c, 140), (Some(159), None));
        // 0.6: three quarters of the logo, an eighth of the words.
        assert_eq!(pictures(&c, 153), (Some(191), Some(32)));
        assert_eq!(pictures(&c, 179), (Some(255), Some(97)));
        assert_eq!(pictures(&c, 242), (Some(255), Some(254)));
        assert_eq!(pictures(&c, FULL), (Some(255), Some(255)));
        // Before the end there is no menu and no help line.
        let buf = painted(&c, 242);
        assert!((0..i32::from(CONSOLE_H)).all(|y| row(&buf, y).is_empty()));
    }

    /// With no picture of the words, the heading is printed in their
    /// place, as bright as the words would be solid.
    #[test]
    fn without_the_picture_the_heading_is_text() {
        let mut c = ctx();
        c.content.title.game_over = None;
        let gold = c.palette.get(UiColor::TextHighlight);
        let at = (i32::from(CONSOLE_W) - 4) / 2;
        let buf = painted(&c, FULL);
        assert_eq!(row(&buf, WORDS_ROW + 1), "OVER");
        assert_eq!(buf.get(at, WORDS_ROW + 1).unwrap().fg, gold);
        assert_eq!(buf.sprites().len(), 1, "the logo alone");
        let buf = painted(&c, 153);
        assert_eq!(row(&buf, WORDS_ROW + 1), "OVER");
        let part = ramp(153, WORDS_IN);
        assert!((part - 0.125).abs() < 1e-3);
        assert_eq!(buf.get(at, WORDS_ROW + 1).unwrap().fg, gold.scale(part));
        assert_eq!(row(&painted(&c, 140), WORDS_ROW + 1), "");
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
        let x = (0..i32::from(CONSOLE_W))
            .find(|&x| buf.get(x, HEADING_ROW).is_some_and(|c| c.glyph == 'T'))
            .unwrap();
        assert_eq!(x, (i32::from(CONSOLE_W) - 3) / 2);
        assert_eq!(row(&buf, i32::from(CONSOLE_H) - 1), "help");
        assert_eq!(buf.sprites(), []);
        let old = stale(&c);
        assert!((0..i32::from(CONSOLE_H)).all(|y| (0..100).all(|x| buf.get(x, y) != Some(&old))));
    }
}
