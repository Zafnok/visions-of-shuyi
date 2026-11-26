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
use crate::console::{ConsoleSizes, ConsoleSpec};
use crate::glyph_buffer::{Cell, GlyphBuffer, Layer, Paint, PxRect, Sprite};
use crate::screen::Ctx;
use crate::screens::title::glyph::{LOGO_SCALE, logo_sprite};
use crate::screens::{centre_x, print_centred};
use crate::widgets::menu::glyph as menu_glyph;
use shuyi_ui::GameData;
#[cfg(not(feature = "std"))]
use shuyi_ui::float::Float;
use shuyi_ui::screens::print_centred_styled;
use shuyi_ui::text::StyledText;

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
    let console_px = ConsoleSpec::PC.px().0;
    let dest = PxRect::new(
        (console_px - w * LOGO_SCALE) / 2,
        WORDS_ROW * i32::from(ConsoleSpec::PC.cell_h),
        w * LOGO_SCALE,
        h * LOGO_SCALE,
    );
    Sprite::new(image, PxRect::new(0, 0, w, h), dest, Layer::Over)
}

/// Game Over's words picture in the language text is shown in, with its
/// size, or `None` for the heading as text (ticket 0831): the language's
/// own copy of the shipped picture `lang/<tag>/files/<path>` along its
/// chain ([`Ctx::localized`](shuyi_ui::Ctx::localized)), asked each
/// frame so a change of language shows at once; the shipped picture in
/// the source language; text in a language with no copy, never the
/// English picture; text too when `title.ron` names no picture.
pub fn words_picture(ctx: &Ctx) -> Option<(trpg_content::ImageId, (u32, u32))> {
    let shipped = ctx.game.content.title.game_over?;
    let images = &ctx.game.content.images;
    let path = ctx.localized(trpg_content::lang::LANG_DIR, shipped.0.path(), |p| {
        images.id(p).is_some()
    });
    let source = &ctx.game.lang().source_lang().code;
    words_at(shipped, &path, ctx.effective_lang() == source, images)
}

/// The picture at `path` with its size, where `path` is where the words
/// `shipped` are read in a language ([`words_picture`]): `shipped` itself
/// in the `source` language, `None` in another with no copy of it.
fn words_at(
    shipped: (trpg_content::ImageId, (u32, u32)),
    path: &str,
    source: bool,
    images: &trpg_content::ImageTable,
) -> Option<(trpg_content::ImageId, (u32, u32))> {
    if path == shipped.0.path() {
        return source.then_some(shipped);
    }
    let id = images.id(path)?;
    let info = images.info(id)?;
    Some((id, (info.width, info.height)))
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
fn paint_help(ctx: &Ctx, help: &StyledText, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let bottom = i32::from(buf.height()) - 1;
    let colors = ctx
        .palette
        .text_colors_on(c(UiColor::TextDim), c(UiColor::Black));
    print_centred_styled(buf, bottom, help, colors);
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
    if let Some((image, size)) = ctx.game.content.title.logo
        && logo > 0
    {
        let mut sprite = logo_sprite(image, size).painted(Paint::Dimmed);
        sprite.opacity = logo;
        buf.add_sprite(sprite);
    }
    let words = ramp(view.progress, WORDS_IN);
    if words > 0.0 {
        if let Some((image, size)) = words_picture(ctx) {
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
        let (w, _) = menu_glyph::size(ctx, menu);
        let x = centre_x(buf, usize::try_from(w).unwrap_or(0));
        menu_glyph::paint(ctx, menu, buf, x, MENU_ROW);
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
    use crate::console::{ConsoleSizes, ConsoleSpec};
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
        GlyphBuffer::with_spec(ConsoleSpec::PC, stale(c))
    }

    /// Game Over at `progress`, with its menu and help once it is over.
    fn view(progress: u8) -> GameOverView {
        let item = |label: &str, focused| MenuItemView {
            label: label.into(),
            suffix: None,
            note: None,
            enabled: true,
            focused,
        };
        let over = progress == FULL;
        GameOverView {
            heading: "OVER".to_owned(),
            progress,
            menu: over.then(|| MenuView::new(vec![item("Retry", true), item("Title", false)])),
            help: over.then(|| "help".into()),
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
        let (logo, _) = c.game.content.title.logo.unwrap();
        let (words, _) = c.game.content.title.game_over.unwrap();
        let of = |image| sprites.iter().find(|s| s.image == image).map(|s| s.opacity);
        assert!(sprites.iter().all(|s| s.image == logo || s.image == words));
        (of(logo), of(words))
    }

    #[test]
    fn the_finished_screen_is_the_logo_the_words_a_menu_and_a_help_line() {
        let c = ctx();
        let buf = painted(&c, FULL);
        let (logo, logo_size) = c.game.content.title.logo.unwrap();
        let (words, words_size) = c.game.content.title.game_over.unwrap();
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
        assert!(gold.dest.y + gold.dest.h <= MENU_ROW * i32::from(ConsoleSpec::PC.cell_h));
        assert_eq!(row(&buf, MENU_ROW), "┌───────┐");
        assert_eq!(row(&buf, MENU_ROW + 1), "│ Retry │");
        assert_eq!(row(&buf, MENU_ROW + 2), "│ Title │");
        let x = (0..i32::from(ConsoleSpec::PC.cols))
            .find(|&x| buf.get(x, MENU_ROW).is_some_and(|c| c.glyph == '┌'))
            .unwrap();
        assert_eq!(x, (i32::from(ConsoleSpec::PC.cols) - 9) / 2);
        assert_eq!(row(&buf, i32::from(ConsoleSpec::PC.rows) - 1), "help");
        // The words are a picture: the heading isn't printed as well.
        assert!((0..i32::from(ConsoleSpec::PC.rows)).all(|y| !row(&buf, y).contains("OVER")));
        // Nothing of the buffer's old contents is left.
        let old = stale(&c);
        assert!(
            (0..i32::from(ConsoleSpec::PC.rows))
                .all(|y| (0..100).all(|x| buf.get(x, y) != Some(&old)))
        );
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
        assert!(
            (0..i32::from(ConsoleSpec::PC.rows))
                .all(|y| (0..100).all(|x| buf.get(x, y) == Some(&empty)))
        );
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
        assert!((0..i32::from(ConsoleSpec::PC.rows)).all(|y| row(&buf, y).is_empty()));
    }

    /// With no picture of the words, the heading is printed in their
    /// place, as bright as the words would be solid.
    #[test]
    fn without_the_picture_the_heading_is_text() {
        let mut c = ctx();
        c.game.content.title.game_over = None;
        let gold = c.palette.get(UiColor::TextHighlight);
        let at = (i32::from(ConsoleSpec::PC.cols) - 4) / 2;
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

    /// The test pack's own copy of the words (0831).
    const TEST_COPY: &str = "lang/test/files/title/game_over.png";

    fn in_test_language(c: &mut Ctx) {
        c.lang = trpg_content::LangCode::new(trpg_content::lang::TEST).unwrap();
    }

    /// English shows the shipped words; a language with its own copy
    /// shows it, from the frame it is chosen; one with no copy shows its
    /// heading as text, never the English picture.
    #[test]
    fn each_language_gets_its_own_words_or_text() {
        let mut c = ctx();
        let shipped = c.game.content.title.game_over;
        assert_eq!(words_picture(&c), shipped);
        in_test_language(&mut c);
        let copy = c.game.content.images.id(TEST_COPY).unwrap();
        assert_eq!(words_picture(&c), Some((copy, (23, 7))));
        let buf = painted(&c, FULL);
        let words = buf.sprites()[1];
        assert_eq!((words.image, words.src), (copy, PxRect::new(0, 0, 23, 7)));
        assert_eq!(words.dest, PxRect::new(308, 304, 184, 56));
        assert!((0..i32::from(ConsoleSpec::PC.rows)).all(|y| !row(&buf, y).contains("OVER")));
        // Without debug tools the test pack isn't shown: English again.
        c.debug_tools = false;
        assert_eq!(words_picture(&c), shipped);
        c.debug_tools = true;
        // A language with no copy: the heading as text.
        c.game.content.images.images.remove(TEST_COPY);
        assert_eq!(words_picture(&c), None);
        let buf = painted(&c, FULL);
        assert_eq!(row(&buf, WORDS_ROW + 1), "OVER");
        assert_eq!(buf.sprites().len(), 1, "the logo alone");
        // No picture named in `title.ron`: text in every language.
        c.game.content.title.game_over = None;
        assert_eq!(words_picture(&c), None);
    }

    /// A copy along the language's chain is found as its own is: a
    /// `pt-BR` with none reads `pt`'s.
    #[test]
    fn a_copy_along_the_chain_is_the_languages_words() {
        let c = ctx();
        let shipped = c.game.content.title.game_over.unwrap();
        let mut images = c.game.content.images.clone();
        let pt = "lang/pt/files/title/game_over.png";
        images
            .images
            .insert(pt, trpg_content::ImageInfo::new(30, 7));
        let tag = |t| trpg_content::LangCode::new(t).unwrap();
        let chain = [tag("pt-BR"), tag("pt"), tag("en")];
        let path = shuyi_assets::lang::localized_path("lang", &chain, shipped.0.path(), |p| {
            images.id(p).is_some()
        });
        assert_eq!(path, pt);
        let found = words_at(shipped, &path, false, &images);
        assert_eq!(found, Some((images.id(pt).unwrap(), (30, 7))));
        // The plain path: the shipped words in the source, else none.
        assert_eq!(
            words_at(shipped, shipped.0.path(), true, &images),
            Some(shipped)
        );
        assert_eq!(words_at(shipped, shipped.0.path(), false, &images), None);
    }

    #[test]
    fn to_be_continued_is_a_heading_and_a_help_line() {
        let c = ctx();
        let view = ToBeContinuedView {
            heading: "TBC".to_owned(),
            help: "help".into(),
        };
        let mut buf = blank(&c);
        paint_to_be_continued(&c, &view, &mut buf);
        assert_eq!(row(&buf, HEADING_ROW), "TBC");
        let x = (0..i32::from(ConsoleSpec::PC.cols))
            .find(|&x| buf.get(x, HEADING_ROW).is_some_and(|c| c.glyph == 'T'))
            .unwrap();
        assert_eq!(x, (i32::from(ConsoleSpec::PC.cols) - 3) / 2);
        assert_eq!(row(&buf, i32::from(ConsoleSpec::PC.rows) - 1), "help");
        assert_eq!(buf.sprites(), []);
        let old = stale(&c);
        assert!(
            (0..i32::from(ConsoleSpec::PC.rows))
                .all(|y| (0..100).all(|x| buf.get(x, y) != Some(&old)))
        );
    }
}
