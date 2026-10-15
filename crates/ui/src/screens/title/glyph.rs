//! The look of the title screen (`docs/design/look-and-feel.md`, *Title
//! screen*): paints a [`TitleView`] as the logo picture over this launch's
//! backdrop, with sparks drifting up, and the menu box under the logo.
//! Everything about the look lives here (the rows, the colours, where a
//! spark is); nothing here decides what the screen shows. Another look is
//! another `paint` of the same view (ADR-0054).

use std::f32::consts::TAU;

use trpg_content::ImageId;

use super::view::{FULL, TitleView};
use crate::audio::splitmix64;
use crate::color::UiColor;
use crate::console::ConsoleSpec;
use crate::glyph_buffer::{Cell, GlyphBuffer, Layer, PxRect, Rect, Sprite};
use crate::screen::Ctx;
use crate::screens::{centre_x, paint_debug_hint, print_centred};
use crate::widgets::menu::glyph as menu_glyph;

/// Console pixels per logo pixel: a cell's width, the size of the blocks
/// the lettering is made of.
pub const LOGO_SCALE: i32 = 8;
/// Console pixels above the logo.
pub const LOGO_TOP_PX: i32 = 16;
/// Console pixels per backdrop pixel.
pub const BACKDROP_ZOOM: i32 = 2;
/// The strip of the backdrop that is drawn again, mirrored, beside each of
/// its edges to fill the console's width, in backdrop pixels.
pub const EDGE_PX: i32 = 32;
/// Top row of the menu box.
const MENU_ROW: i32 = 18;
/// Row of the "press any key or button" line.
const PROMPT_ROW: i32 = 20;

/// The pictures beside the "press any key or button" line, as rows of
/// glyphs: a keyboard on its left and a controller on its right, so it is
/// obvious either works (`docs/design/title-screen.md`; Nick's pick,
/// ticket 0226). The middle row is the line's own.
const KEYBOARD_PICTURE: [&str; 3] = ["┌─┬─┬─┬─┬─┐", "├─┴┬┴─┴┬┴─┤", "└──┴───┴──┘"];
const PAD_PICTURE: [&str; 3] = ["╭─────────╮", "│ ┼ ╭─╮ ◯ │", "╰───╯ ╰───╯"];
/// Blank cells between the line and each picture.
const PICTURE_GAP: i32 = 5;
/// Cells of the prompt's band past each picture.
const BAND_MARGIN: i32 = 2;

/// How many sparks drift up the screen (*tunable*).
pub const SPARKS: u64 = 46;
/// The rows sparks cross: every row above the help line.
pub const SPARK_ROWS: i32 = 31;
/// How long a spark takes to sway to one side, the other and back, in
/// milliseconds (*tunable*).
pub const SWAY_MS: u32 = 3520;
/// What a spark looks like: its glyph and colour, by kind.
const SPARK_KINDS: [(char, UiColor); 7] = [
    ('•', UiColor::Enemy),
    ('∙', UiColor::Cursor),
    ('∙', UiColor::TextHighlight),
    ('°', UiColor::Enemy),
    ('•', UiColor::TextHighlight),
    ('*', UiColor::Enemy),
    ('∙', UiColor::PanelBorderFocus),
];

/// A spark on the screen: its cell, glyph and colour.
pub type Spark = (i32, i32, char, UiColor);

/// Where spark `i` is `millis` after the title appeared. Each has its own
/// column, starting row, speed (9 or 18 rows a second, upward, coming
/// back in at the bottom) and look, fixed by `i` alone; it sways one or
/// two cells to each side of its column every [`SWAY_MS`].
pub fn spark(i: u64, millis: u32) -> Spark {
    let bits = splitmix64(i);
    let part = |shift: u32, n: u64| i32::try_from((bits >> shift) % n).unwrap_or(0);
    let (x0, y0) = (part(0, ConsoleSpec::PC.cols.into()), part(16, 31));
    // Two sparks in three are slow.
    let speed = if part(24, 3) == 0 { 18 } else { 9 };
    let rose = i64::from(millis) * speed / 1000;
    let y = (i64::from(y0) - rose).rem_euclid(i64::from(SPARK_ROWS));
    #[allow(clippy::cast_precision_loss)] // small numbers
    let turn = (part(32, 1000) as f32 / 1000.0) + (millis % SWAY_MS) as f32 / SWAY_MS as f32;
    #[allow(clippy::cast_possible_truncation)] // -2 to 2
    let sway = (1.5 * (turn * TAU).sin()).round() as i32;
    let x = (x0 + sway).rem_euclid(ConsoleSpec::PC.cols.into());
    let (glyph, color) = SPARK_KINDS[usize::try_from(part(40, 7)).unwrap_or(0)];
    (x, i32::try_from(y).unwrap_or(0), glyph, color)
}

/// The backdrop `image` across the console: the whole picture in the
/// middle at [`BACKDROP_ZOOM`], and the strip at each of its edges again,
/// mirrored, outside it (as behind a fight, ADR-0056).
pub fn backdrop_sprites(image: ImageId) -> [Sprite; 3] {
    let (w, h) = trpg_content::battler::BACKDROP_PX;
    let (w, h) = (i32::try_from(w).unwrap_or(0), i32::try_from(h).unwrap_or(0));
    let zoom = BACKDROP_ZOOM;
    let edge = EDGE_PX * zoom;
    let strip = |src_x: i32, dest_x: i32| {
        let mut s = Sprite::new(
            image,
            PxRect::new(src_x, 0, EDGE_PX, h),
            PxRect::new(dest_x, 0, edge, h * zoom),
            Layer::Under,
        );
        s.flip_x = true;
        s
    };
    [
        Sprite::new(
            image,
            PxRect::new(0, 0, w, h),
            PxRect::new(edge, 0, w * zoom, h * zoom),
            Layer::Under,
        ),
        strip(0, 0),
        strip(w - EDGE_PX, edge + w * zoom),
    ]
}

/// The logo `image`, `size` pixels big: centred, [`LOGO_TOP_PX`] down,
/// over everything.
pub fn logo_sprite(image: ImageId, size: (u32, u32)) -> Sprite {
    let (w, h) = (
        i32::try_from(size.0).unwrap_or(0),
        i32::try_from(size.1).unwrap_or(0),
    );
    let console_px = ConsoleSpec::PC.px().0;
    let dest = PxRect::new(
        (console_px - w * LOGO_SCALE) / 2,
        LOGO_TOP_PX,
        w * LOGO_SCALE,
        h * LOGO_SCALE,
    );
    Sprite::new(image, PxRect::new(0, 0, w, h), dest, Layer::Over)
}

/// Paints the title's picture over the whole of `buf`, with nothing else
/// on it: `backdrop` (if any), the `sparks` (if any) as they are that
/// many milliseconds after the title appeared, and the logo. The title screen draws its prompt or
/// menu over this; the cinematic's logo shot is this alone (ticket 0817).
pub fn paint_logo(
    ctx: &Ctx,
    backdrop: Option<ImageId>,
    sparks: Option<u32>,
    buf: &mut GlyphBuffer,
) {
    let c = |u| ctx.palette.get(u);
    let black = c(UiColor::Black);
    buf.fill_rect(buf.bounds(), Cell::new(' ', c(UiColor::Text), black));
    for sprite in backdrop.into_iter().flat_map(backdrop_sprites) {
        buf.add_sprite(sprite);
    }
    if let Some(millis) = sparks {
        for i in 0..SPARKS {
            let (x, y, glyph, color) = spark(i, millis);
            buf.set(x, y, Cell::new(glyph, c(color), black));
        }
    }
    if let Some((image, size)) = ctx.content.title.logo {
        buf.add_sprite(logo_sprite(image, size));
    }
}

/// Adds the title's picture over what `buf` already shows, `opacity` (up
/// to [`FULL`]) solid, changing no cell's background: the pictures of
/// [`paint_logo`] that much solid, the sparks that much bright. Nothing at
/// 0.
pub fn paint_logo_over(
    ctx: &Ctx,
    backdrop: Option<ImageId>,
    sparks: Option<u32>,
    opacity: u8,
    buf: &mut GlyphBuffer,
) {
    if opacity == 0 {
        return;
    }
    let part = f32::from(opacity) / f32::from(FULL);
    let faded = |mut sprite: Sprite| {
        sprite.opacity = opacity;
        sprite
    };
    for sprite in backdrop.into_iter().flat_map(backdrop_sprites) {
        buf.add_sprite(faded(sprite));
    }
    if let Some(millis) = sparks {
        for i in 0..SPARKS {
            let (x, y, glyph, color) = spark(i, millis);
            let fg = ctx.palette.get(color).scale(part);
            buf.print_fg(x, y, glyph.encode_utf8(&mut [0; 4]), fg);
        }
    }
    if let Some((image, size)) = ctx.content.title.logo {
        buf.add_sprite(faded(logo_sprite(image, size)));
    }
}

/// Paints `view` over the whole of `buf`: the cinematic, if it shows one,
/// with the title's picture over it as solid as the cinematic is dark
/// (else the title's picture alone), then the prompt or the menu, as
/// bright as the view's `menu_opacity`.
pub fn paint(ctx: &Ctx, view: &TitleView, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let (dim, panel) = (c(UiColor::TextDim), c(UiColor::PanelBg));
    if let Some(shot) = &view.cinematic {
        #[allow(clippy::cast_precision_loss)] // far below 2^24 ms
        let t = shot.t_ms as f32 / 1000.0;
        let brightness = f32::from(shot.brightness) / f32::from(FULL);
        shot.player.draw(ctx, t, brightness, buf);
        let rest = FULL - shot.brightness;
        paint_logo_over(ctx, view.backdrop, view.sparks, rest, buf);
    } else {
        paint_logo(ctx, view.backdrop, view.sparks, buf);
    }
    // What is painted from here on fades with the menu.
    let solid = f32::from(view.menu_opacity) / f32::from(FULL);
    let bottom = i32::from(buf.height()) - 1;
    if let Some(prompt) = &view.prompt {
        let len = i32::try_from(prompt.chars().count()).unwrap_or(0);
        let left = centre_x(buf, prompt.chars().count());
        let right = left + len + PICTURE_GAP;
        let w = i32::try_from(KEYBOARD_PICTURE[0].chars().count()).unwrap_or(0);
        // A band behind the line and its pictures: readable on any backdrop.
        let band_x = left - PICTURE_GAP - w - BAND_MARGIN;
        let band = Rect::new(band_x, PROMPT_ROW - 1, 2 * (left - band_x) + len, 3);
        buf.fill_rect(band, Cell::new(' ', dim, panel));
        buf.print(left, PROMPT_ROW, prompt, dim, panel);
        for (y, (keys, pad)) in (PROMPT_ROW - 1..).zip(KEYBOARD_PICTURE.iter().zip(PAD_PICTURE)) {
            buf.print(left - PICTURE_GAP - w, y, keys, dim, panel);
            buf.print(right, y, pad, dim, panel);
        }
        buf.dim(band, solid);
    }
    if let Some(menu) = &view.menu {
        let (w, h) = menu_glyph::size(menu);
        let x = centre_x(buf, usize::try_from(w).unwrap_or(0));
        menu_glyph::paint(&ctx.palette, menu, buf, x, MENU_ROW);
        buf.dim(Rect::new(x, MENU_ROW, w, h), solid);
        if let Some(notice) = &view.notice {
            // On a band of its own, a blank cell wider on each side.
            let len = notice.chars().count();
            let at = centre_x(buf, len + 2);
            let band = Rect::new(at, MENU_ROW + h + 1, i32::try_from(len).unwrap_or(0) + 2, 1);
            buf.fill_rect(band, Cell::new(' ', dim, panel));
            buf.print(at + 1, MENU_ROW + h + 1, notice, c(UiColor::HpLow), panel);
            buf.dim(band, solid);
        }
    }
    if let Some(help) = &view.help {
        if view.cinematic.is_some() {
            // Over the cinematic: its letters alone, no strip behind them.
            let x = centre_x(buf, help.chars().count());
            buf.print_fg(x, bottom, help, dim.scale(solid));
        } else {
            print_centred(buf, bottom, help, dim.scale(solid), c(UiColor::Black));
        }
    }
    if let Some(hint) = &view.debug_hint {
        paint_debug_hint(&ctx.palette, hint, buf, bottom);
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::cinematic::CinematicPlayer;
    use crate::console::ConsoleSpec;
    use crate::screen::tests::ctx;
    use crate::screens::title::view::CinematicView;
    use crate::widgets::{MenuItemView, MenuView};

    /// Row `y` of `buf` as text.
    fn text(buf: &GlyphBuffer, y: i32) -> String {
        (0..i32::from(buf.width()))
            .map(|x| buf.get(x, y).map_or(' ', |c| c.glyph))
            .collect()
    }

    /// The column where `needle` starts on row `y`.
    fn column(buf: &GlyphBuffer, y: i32, needle: &str) -> i32 {
        let row = text(buf, y);
        let at = row
            .find(needle)
            .unwrap_or_else(|| panic!("{needle:?} not in {row:?}"));
        i32::try_from(row[..at].chars().count()).unwrap()
    }

    fn item(label: &str, focused: bool) -> MenuItemView {
        MenuItemView {
            label: label.to_owned(),
            suffix: None,
            note: None,
            enabled: true,
            focused,
        }
    }

    fn view() -> TitleView {
        TitleView {
            cinematic: None,
            menu_opacity: FULL,
            backdrop: None,
            sparks: None,
            prompt: None,
            menu: Some(MenuView {
                items: vec![item("New Game", true), item("Quit", false)],
            }),
            notice: None,
            help: Some("help".to_owned()),
            debug_hint: None,
        }
    }

    fn stale(c: &Ctx) -> Cell {
        Cell::new(
            'x',
            c.palette.get(UiColor::Enemy),
            c.palette.get(UiColor::Enemy),
        )
    }

    fn painted(c: &Ctx, view: &TitleView) -> GlyphBuffer {
        let mut buf = GlyphBuffer::with_spec(ConsoleSpec::PC, stale(c));
        paint(c, view, &mut buf);
        buf
    }

    fn logo_painted(c: &Ctx, backdrop: Option<ImageId>, sparks: Option<u32>) -> GlyphBuffer {
        let mut buf = GlyphBuffer::with_spec(ConsoleSpec::PC, stale(c));
        paint_logo(c, backdrop, sparks, &mut buf);
        buf
    }

    fn px(x: i32, y: i32, w: i32, h: i32) -> PxRect {
        PxRect::new(x, y, w, h)
    }

    #[test]
    fn the_logo_is_a_centred_picture_drawn_in_blocks_over_everything() {
        let c = ctx();
        let (image, size) = c.content.title.logo.unwrap();
        assert_eq!(size, (52, 31), "the shipped logo");
        let logo = logo_sprite(image, size);
        assert_eq!(logo.src, px(0, 0, 52, 31));
        assert_eq!(logo.dest, px(192, 16, 416, 248));
        assert_eq!(
            (logo.layer, logo.flip_x, logo.opacity),
            (Layer::Over, false, 255)
        );
        // Another size is centred too, and as far down.
        assert_eq!(logo_sprite(image, (10, 4)).dest, px(360, 16, 80, 32));
        // With no backdrop it is the only picture.
        assert_eq!(logo_painted(&c, None, None).sprites(), [logo]);
        // The menu sits under it, clear of it.
        let menu_top = MENU_ROW * i32::from(crate::console::ConsoleSpec::PC.cell_h);
        assert!(logo.dest.y + logo.dest.h <= menu_top);
    }

    #[test]
    fn the_backdrop_fills_the_width_with_its_edges_mirrored_under_the_logo() {
        let c = ctx();
        let image = c.content.title.backdrops[0];
        let [middle, left, right] = backdrop_sprites(image);
        assert_eq!(
            (middle.src, middle.dest),
            (px(0, 0, 336, 248), px(64, 0, 672, 496))
        );
        assert_eq!(
            (left.src, left.dest),
            (px(0, 0, 32, 248), px(0, 0, 64, 496))
        );
        assert_eq!(
            (right.src, right.dest),
            (px(304, 0, 32, 248), px(736, 0, 64, 496))
        );
        assert_eq!(
            [middle.flip_x, left.flip_x, right.flip_x],
            [false, true, true]
        );
        for s in [middle, left, right] {
            assert_eq!((s.image, s.layer, s.opacity), (image, Layer::Under, 255));
        }
        let (logo, size) = c.content.title.logo.unwrap();
        assert_eq!(
            logo_painted(&c, Some(image), Some(0)).sprites(),
            [middle, left, right, logo_sprite(logo, size)]
        );
        // The view's backdrop is the one painted.
        let mut v = view();
        v.menu = None;
        v.backdrop = Some(image);
        assert_eq!(painted(&c, &v).sprites()[..3], [middle, left, right]);
    }

    /// Pinned by hand from the first run: a change to where sparks are is
    /// a change of the look.
    #[test]
    fn sparks_are_where_they_were() {
        let now = [
            spark(0, 0),
            spark(1, 1234),
            spark(7, 60_000),
            spark(45, 3_600_000),
        ];
        assert_eq!(now, SPARK_PINS);
    }

    #[test]
    fn sparks_rise_sway_and_keep_their_looks() {
        let rows = i64::from(SPARK_ROWS);
        let mut speeds = std::collections::BTreeSet::new();
        let mut kinds = std::collections::BTreeSet::new();
        for i in 0..SPARKS {
            let (_, y0, glyph, color) = spark(i, 0);
            assert!(SPARK_KINDS.contains(&(glyph, color)));
            kinds.insert((glyph, color.name()));
            // A second later it is 9 or 18 rows up, coming back in below.
            let (_, y1, ..) = spark(i, 1000);
            let up = (i64::from(y0) - i64::from(y1)).rem_euclid(rows);
            assert!(up == 9 || up == 18, "spark {i} rose {up}");
            speeds.insert(up);
            // Just short of a row's time, a slow one hasn't moved.
            let (_, y_short, ..) = spark(i, 55);
            assert_eq!(y_short, y0, "spark {i}");
            let mut columns = std::collections::BTreeSet::new();
            for step in 0..64 {
                let t = step * SWAY_MS / 64;
                let (x, y, g, col) = spark(i, t);
                assert!(
                    (0..i32::from(ConsoleSpec::PC.cols)).contains(&x),
                    "spark {i} x {x}"
                );
                assert!((0..SPARK_ROWS).contains(&y), "spark {i} y {y}");
                assert_eq!((g, col), (glyph, color));
                // A sway later it is back in the same column.
                assert_eq!(spark(i, t + SWAY_MS).0, x);
                columns.insert(x);
            }
            // It sways: more than one column, never more than two to a side.
            assert!((2..=5).contains(&columns.len()), "spark {i}: {columns:?}");
        }
        // However long the title is left open, a spark sways as it did in
        // its first sway: the time doesn't run out or lose its precision.
        for i in 0..SPARKS {
            for late in [u32::MAX, u32::MAX - 1234, 4_000_000_000] {
                let (x, y, ..) = spark(i, late);
                assert_eq!(x, spark(i, late % SWAY_MS).0, "spark {i} at {late}");
                assert!((0..SPARK_ROWS).contains(&y), "spark {i} at {late}");
            }
        }
        assert_eq!(speeds.len(), 2, "slow and fast sparks");
        assert!(kinds.len() >= 5, "{kinds:?}");
    }

    #[test]
    fn every_spark_is_painted_above_the_help_line() {
        let c = ctx();
        let black = c.palette.get(UiColor::Black);
        for millis in [0, 777, 123_456] {
            let buf = logo_painted(&c, None, Some(millis));
            let sparks: Vec<Spark> = (0..SPARKS).map(|i| spark(i, millis)).collect();
            for &(x, y, ..) in &sparks {
                // Where two share a cell, the later one shows.
                let &(_, _, glyph, color) =
                    sparks.iter().rev().find(|s| (s.0, s.1) == (x, y)).unwrap();
                let want = Cell::new(glyph, c.palette.get(color), black);
                assert_eq!(buf.get(x, y), Some(&want), "({x}, {y}) at {millis}");
            }
            // Nothing else is on the screen: every other cell is blank.
            let empty = Cell::new(' ', c.palette.get(UiColor::Text), black);
            for y in 0..i32::from(ConsoleSpec::PC.rows) {
                for x in 0..i32::from(ConsoleSpec::PC.cols) {
                    if !sparks.iter().any(|s| (s.0, s.1) == (x, y)) {
                        assert_eq!(buf.get(x, y), Some(&empty), "({x}, {y})");
                    }
                }
            }
            assert_eq!(text(&buf, i32::from(ConsoleSpec::PC.rows) - 1).trim(), "");
        }
        // The view's sparks are the ones painted, and none is none.
        let mut v = view();
        v.menu = None;
        v.help = None;
        let none = painted(&c, &v);
        assert_eq!(none, logo_painted(&c, None, None));
        assert!((0..i32::from(ConsoleSpec::PC.rows)).all(|y| text(&none, y).trim().is_empty()));
        v.sparks = Some(0);
        let at_0 = painted(&c, &v);
        v.sparks = Some(1000);
        let later = painted(&c, &v);
        assert_eq!(at_0, logo_painted(&c, None, Some(0)));
        assert_eq!(later, logo_painted(&c, None, Some(1000)));
        assert_ne!(at_0, later);
        assert_ne!(at_0, none);
    }

    #[test]
    fn the_menu_is_centred_under_the_logo_and_the_help_is_at_the_bottom() {
        let c = ctx();
        let buf = painted(&c, &view());
        let middle = (i32::from(ConsoleSpec::PC.cols) - 12) / 2;
        assert_eq!(column(&buf, MENU_ROW, "┌──────────┐"), middle);
        assert_eq!(column(&buf, MENU_ROW + 1, "│ New Game │"), middle);
        assert_eq!(column(&buf, MENU_ROW + 2, "│ Quit     │"), middle);
        assert_eq!(column(&buf, MENU_ROW + 3, "└──────────┘"), middle);
        let bottom = i32::from(ConsoleSpec::PC.rows) - 1;
        assert_eq!(text(&buf, bottom).trim(), "help");
        assert_eq!(
            column(&buf, bottom, "help"),
            (i32::from(ConsoleSpec::PC.cols) - 4) / 2
        );
        let help = buf.get(column(&buf, bottom, "help"), bottom).unwrap();
        assert_eq!(help.fg, c.palette.get(UiColor::TextDim));
        assert_eq!(help.bg, c.palette.get(UiColor::Black));
    }

    /// A keyboard on the line's left and a controller on its right, each
    /// the same gap from it whatever the line's length, on a band that
    /// reaches as far past each.
    #[test]
    fn the_prompt_takes_the_menus_place_between_a_keyboard_and_a_controller() {
        let c = ctx();
        let (dim, panel) = (
            c.palette.get(UiColor::TextDim),
            c.palette.get(UiColor::PanelBg),
        );
        for prompt in ["Press any key or button", "PRESS ANY KEY OR BUTTON NOW"] {
            let mut v = view();
            v.menu = None;
            v.help = None;
            v.prompt = Some(prompt.to_owned());
            let buf = painted(&c, &v);
            let len = i32::try_from(prompt.chars().count()).unwrap();
            let left = (i32::from(ConsoleSpec::PC.cols) - len) / 2;
            let rows = [
                "┌─┬─┬─┬─┬─┐".to_owned() + &" ".repeat(prompt.len() + 10) + "╭─────────╮",
                format!("├─┴┬┴─┴┬┴─┤     {prompt}     │ ┼ ╭─╮ ◯ │"),
                "└──┴───┴──┘".to_owned() + &" ".repeat(prompt.len() + 10) + "╰───╯ ╰───╯",
            ];
            for (y, row) in (PROMPT_ROW - 1..).zip(&rows) {
                assert_eq!(column(&buf, y, row), left - 16, "{row}");
            }
            // The band: two cells past each picture, three rows, no more.
            let (first, last) = (left - 18, left + len + 17);
            for y in PROMPT_ROW - 2..=PROMPT_ROW + 2 {
                for x in first - 1..=last + 1 {
                    let cell = buf.get(x, y).unwrap();
                    let inside = (first..=last).contains(&x)
                        && (PROMPT_ROW - 1..=PROMPT_ROW + 1).contains(&y);
                    assert_eq!(cell.bg == panel, inside, "({x}, {y})");
                    if inside {
                        assert_eq!(cell.fg, dim, "({x}, {y})");
                    }
                }
            }
            assert!(
                !(0..i32::from(ConsoleSpec::PC.rows)).any(|y| text(&buf, y).contains("New Game"))
            );
            assert_eq!(text(&buf, i32::from(ConsoleSpec::PC.rows) - 1).trim(), "");
        }
    }

    /// The notice sits one blank row under the menu's box, in the warning
    /// colour, on a band one cell wider than it on each side.
    #[test]
    fn the_notice_is_under_the_menu() {
        let c = ctx();
        let mut v = view();
        v.notice = Some("This save can't be read".to_owned());
        let buf = painted(&c, &v);
        let below = MENU_ROW + 4 + 1;
        let at = column(&buf, below, "This save can't be read");
        assert_eq!(at, (i32::from(ConsoleSpec::PC.cols) - 23) / 2);
        let panel = c.palette.get(UiColor::PanelBg);
        for x in at - 2..=at + 24 {
            let cell = buf.get(x, below).unwrap();
            assert_eq!(cell.bg == panel, (at - 1..=at + 23).contains(&x), "{x}");
            assert_ne!(buf.get(x, below - 1).unwrap().bg, panel, "{x}");
        }
        assert_eq!(
            buf.get(at, below).unwrap().fg,
            c.palette.get(UiColor::HpLow)
        );
        // None with the prompt: there is no menu to be under.
        v.menu = None;
        let buf = painted(&c, &v);
        assert!(!(0..i32::from(ConsoleSpec::PC.rows)).any(|y| text(&buf, y).contains("This save")));
    }

    #[test]
    fn the_debug_hint_is_on_the_bottom_row_at_the_right() {
        let c = ctx();
        let mut v = view();
        v.debug_hint = Some("F2 debug".to_owned());
        let buf = painted(&c, &v);
        let bottom = text(&buf, i32::from(ConsoleSpec::PC.rows) - 1);
        assert!(bottom.ends_with("F2 debug "), "{bottom:?}");
        assert!(bottom.contains("help"));
        // With the prompt too.
        v.menu = None;
        v.help = None;
        v.prompt = Some("Press any key".to_owned());
        let buf = painted(&c, &v);
        assert!(text(&buf, i32::from(ConsoleSpec::PC.rows) - 1).ends_with("F2 debug "));
    }

    /// Opaque screens must paint every cell, not rely on `Game` clearing.
    #[test]
    fn it_covers_the_whole_buffer() {
        let c = ctx();
        for buf in [painted(&c, &view()), logo_painted(&c, None, Some(0))] {
            let left = (0..i32::from(ConsoleSpec::PC.rows))
                .flat_map(|y| (0..i32::from(ConsoleSpec::PC.cols)).map(move |x| (x, y)))
                .filter(|&(x, y)| buf.get(x, y) == Some(&stale(&c)))
                .count();
            assert_eq!(left, 0);
        }
    }

    /// The test cinematic (a pan for six seconds; the logo from 16) at
    /// `t_ms`, this bright.
    fn shot(c: &Ctx, t_ms: u32, brightness: u8) -> CinematicView {
        let player = CinematicPlayer::new(c, "test", None).unwrap();
        CinematicView {
            player: Rc::new(player),
            t_ms,
            brightness,
        }
    }

    /// The cinematic's own frame, as its player draws it.
    fn drawn(c: &Ctx, shot: &CinematicView, t: f32, brightness: f32) -> GlyphBuffer {
        let mut buf = GlyphBuffer::with_spec(ConsoleSpec::PC, stale(c));
        shot.player.draw(c, t, brightness, &mut buf);
        buf
    }

    /// `color` as it shows `opacity` solid.
    fn at_opacity(color: crate::color::Rgb, opacity: u8) -> crate::color::Rgb {
        color.scale(f32::from(opacity) / 255.0)
    }

    /// The cinematic alone is its player's frame at the view's time, and
    /// nothing of the title over it.
    #[test]
    fn the_cinematic_alone_is_its_own_frame() {
        let c = ctx();
        let mut v = view();
        v.menu = None;
        v.help = None;
        v.menu_opacity = 0;
        v.backdrop = Some(c.content.title.backdrops[0]);
        v.sparks = Some(500);
        // The pan, half way; then the logo shot.
        for (t_ms, t) in [(3000, 3.0), (1500, 1.5), (17_000, 17.0)] {
            let playing = shot(&c, t_ms, FULL);
            v.cinematic = Some(playing.clone());
            assert_eq!(painted(&c, &v), drawn(&c, &playing, t, 1.0), "{t_ms}");
        }
        assert_ne!(
            drawn(&c, &shot(&c, 0, FULL), 3.0, 1.0),
            drawn(&c, &shot(&c, 0, FULL), 1.5, 1.0)
        );
        assert!(painted(&c, &v).backdrop().is_none(), "the logo shot");
    }

    /// While the menu fades out the cinematic is drawn as bright as the
    /// fade is far, and the title's pictures over it as solid as the rest.
    #[test]
    fn the_titles_picture_fades_over_the_cinematic() {
        let c = ctx();
        let image = c.content.title.backdrops[0];
        let (logo, size) = c.content.title.logo.unwrap();
        let mut v = view();
        v.menu = None;
        v.help = None;
        v.backdrop = Some(image);
        v.sparks = Some(0);
        let playing = shot(&c, 3000, 64);
        v.cinematic = Some(playing.clone());
        v.menu_opacity = 191;
        let buf = painted(&c, &v);
        // The cinematic, a quarter bright.
        let under = drawn(&c, &playing, 3.0, 64.0 / 255.0);
        assert_eq!(buf.backdrop(), under.backdrop());
        assert_ne!(buf.backdrop(), drawn(&c, &playing, 3.0, 1.0).backdrop());
        // The title's four pictures, three quarters solid, as they are
        // placed on the title itself.
        let mut want = backdrop_sprites(image).to_vec();
        want.push(logo_sprite(logo, size));
        for sprite in &mut want {
            sprite.opacity = 191;
        }
        assert_eq!(buf.sprites(), want);
        // The sparks: their glyphs over the map, no cell made solid.
        let sparks: Vec<Spark> = (0..SPARKS).map(|i| spark(i, 0)).collect();
        for y in 0..i32::from(ConsoleSpec::PC.rows) {
            for x in 0..i32::from(ConsoleSpec::PC.cols) {
                let cell = buf.get(x, y).unwrap();
                assert!(cell.see_through, "({x}, {y})");
                match sparks.iter().rev().find(|s| (s.0, s.1) == (x, y)) {
                    Some(&(_, _, glyph, color)) => {
                        assert_eq!(cell.glyph, glyph, "({x}, {y})");
                        assert_eq!(cell.fg, at_opacity(c.palette.get(color), 191));
                    }
                    None => assert_eq!(Some(cell), under.get(x, y), "({x}, {y})"),
                }
            }
        }
        // No sparks, no backdrop: only the logo is added.
        v.sparks = None;
        v.backdrop = None;
        let buf = painted(&c, &v);
        assert_eq!(buf.sprites(), [want[3]]);
        assert_eq!(buf.get(35, 15), under.get(35, 15));
        // Fully bright, nothing of the title is.
        v.sparks = Some(0);
        v.backdrop = Some(image);
        v.cinematic = Some(shot(&c, 3000, FULL));
        assert_eq!(painted(&c, &v), drawn(&c, &playing, 3.0, 1.0));
        let mut buf = drawn(&c, &playing, 3.0, 1.0);
        paint_logo_over(&c, Some(image), Some(0), 0, &mut buf);
        assert_eq!(buf, drawn(&c, &playing, 3.0, 1.0));
        // Fully solid, the pictures are as on the title.
        paint_logo_over(&c, Some(image), None, FULL, &mut buf);
        assert_eq!(buf.sprites(), logo_painted(&c, Some(image), None).sprites());
    }

    /// The menu, the notice and the help line are as bright as the view's
    /// `menu_opacity`; the logo behind them is not touched.
    #[test]
    fn the_menu_its_notice_and_the_help_fade_together() {
        let c = ctx();
        let mut v = view();
        v.notice = Some("Notice".to_owned());
        v.backdrop = Some(c.content.title.backdrops[0]);
        let full = painted(&c, &v);
        let bottom = i32::from(ConsoleSpec::PC.rows) - 1;
        let rows = [MENU_ROW, MENU_ROW + 1, MENU_ROW + 3, MENU_ROW + 5, bottom];
        for opacity in [128, 0] {
            v.menu_opacity = opacity;
            let buf = painted(&c, &v);
            let mut faded = 0;
            for y in 0..i32::from(ConsoleSpec::PC.rows) {
                for x in 0..i32::from(ConsoleSpec::PC.cols) {
                    let (was, is) = (full.get(x, y).unwrap(), buf.get(x, y).unwrap());
                    assert_eq!(is.glyph, was.glyph, "({x}, {y})");
                    let lit = was.glyph != ' ' || was.bg != c.palette.get(UiColor::Black);
                    if rows.contains(&y) && lit {
                        assert_eq!(is.fg, at_opacity(was.fg, opacity), "({x}, {y})");
                        assert_eq!(is.bg, at_opacity(was.bg, opacity), "({x}, {y})");
                        faded += 1;
                    } else if !(MENU_ROW..=MENU_ROW + 5).contains(&y) && y != bottom {
                        assert_eq!(is, was, "({x}, {y})");
                    }
                }
            }
            // The box's 12 cells on each row, the notice's band of 8, the
            // help's four letters.
            assert_eq!(faded, 3 * 12 + 8 + 4, "{opacity}");
            assert_eq!(buf.sprites(), full.sprites());
        }
    }

    /// Over the cinematic the help line is its letters alone, and the
    /// menu's box hides what is under it.
    #[test]
    fn over_the_cinematic_the_help_has_no_strip_and_the_menu_is_a_box() {
        let c = ctx();
        let mut v = view();
        let playing = shot(&c, 3000, 128);
        v.cinematic = Some(playing.clone());
        v.menu_opacity = 127;
        let buf = painted(&c, &v);
        let under = drawn(&c, &playing, 3.0, 128.0 / 255.0);
        let bottom = i32::from(ConsoleSpec::PC.rows) - 1;
        let at = column(&buf, bottom, "help");
        assert_eq!(at, (i32::from(ConsoleSpec::PC.cols) - 4) / 2);
        let dim = at_opacity(c.palette.get(UiColor::TextDim), 127);
        for x in 0..i32::from(ConsoleSpec::PC.cols) {
            let cell = buf.get(x, bottom).unwrap();
            assert!(cell.see_through, "{x}");
            if (at..at + 4).contains(&x) {
                assert_eq!(cell.fg, dim, "{x}");
            } else {
                assert_eq!(Some(cell), under.get(x, bottom), "{x}");
            }
        }
        // The menu: solid cells, as bright as its opacity.
        let middle = (i32::from(ConsoleSpec::PC.cols) - 12) / 2;
        let mut plain = view();
        plain.menu_opacity = 127;
        let alone = painted(&c, &plain);
        for y in MENU_ROW..MENU_ROW + 4 {
            for x in middle..middle + 12 {
                assert_eq!(buf.get(x, y), alone.get(x, y), "({x}, {y})");
                assert!(!buf.get(x, y).unwrap().see_through);
            }
            assert!(buf.get(middle - 1, y).unwrap().see_through);
            assert!(buf.get(middle + 12, y).unwrap().see_through);
        }
    }

    /// The prompt's band fades with the same number (the title never
    /// fades it today: the prompt is over before anything does).
    #[test]
    fn the_prompts_band_fades_too() {
        let c = ctx();
        let mut v = view();
        v.menu = None;
        v.help = None;
        v.prompt = Some("Press".to_owned());
        let full = painted(&c, &v);
        v.menu_opacity = 64;
        let buf = painted(&c, &v);
        let panel = c.palette.get(UiColor::PanelBg);
        let mut band = 0;
        for y in 0..i32::from(ConsoleSpec::PC.rows) {
            for x in 0..i32::from(ConsoleSpec::PC.cols) {
                let (was, is) = (full.get(x, y).unwrap(), buf.get(x, y).unwrap());
                if was.bg == panel {
                    assert_eq!(is.fg, at_opacity(was.fg, 64), "({x}, {y})");
                    assert_eq!(is.bg, at_opacity(was.bg, 64), "({x}, {y})");
                    band += 1;
                } else {
                    assert_eq!(is, was, "({x}, {y})");
                }
            }
        }
        assert_eq!(band, 3 * (5 + 2 * (5 + 11 + 2)));
    }

    const SPARK_PINS: [Spark; 4] = [
        (35, 15, '∙', UiColor::PanelBorderFocus),
        (64, 4, '•', UiColor::TextHighlight),
        (88, 12, '°', UiColor::Enemy),
        (27, 19, '°', UiColor::Enemy),
    ];
}
