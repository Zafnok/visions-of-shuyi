//! The look of the title screen (`docs/design/look-and-feel.md`, *Title
//! screen*): paints a [`TitleView`] as the logo picture over this launch's
//! backdrop, with a campfire in its corner and embers rising from it, and
//! the menu box under the logo. Everything about the look lives here (the
//! rows, the colours, where an ember is); nothing here decides what the
//! screen shows. Another look is
//! another `paint` of the same view (ADR-0054).

use core::f32::consts::TAU;
#[cfg(not(feature = "std"))]
use shuyi_ui::float::Float;

use trpg_content::ImageId;
use trpg_content::title::Campfire;

use super::view::{FULL, NoteView, TitleView};
use crate::audio::splitmix64;
use crate::color::Rgb;
use crate::color::UiColor;
use crate::console::{ConsoleSizes, ConsoleSpec};
use crate::glyph_buffer::{Cell, GlyphBuffer, Layer, PxRect, Rect, Sprite};
use crate::prelude::*;
use crate::screen::Ctx;
use crate::screens::{centre_x, paint_debug_hint};
use crate::tips::glyph::{MARGIN, paint_frame_sized, paint_text};
use crate::widgets::menu::glyph as menu_glyph;
use crate::widgets::word_wrap;
use shuyi_ui::screens::print_centred_styled;

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
pub const MENU_ROW: i32 = 18;
/// Row of the "press any key or button" line.
const PROMPT_ROW: i32 = 20;
/// Cells between the menu box and the guide's note right of it (0842).
const NOTE_GAP: i32 = 2;
/// The guide's face in the note, in cells: a 64×64 bust at 1×, half the
/// tip box's (a small note, not a lesson).
pub const NOTE_FACE_CELLS: (i32, i32) = (8, 4);

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

/// The campfire's corner (ticket 0922, Nick's pick): its left edge, in
/// console pixels.
pub const FIRE_LEFT_PX: i32 = 40;
/// Console pixels per pixel of the campfire's picture (Nick: "Corner, 4×").
pub const FIRE_SCALE: i32 = 4;
/// How long each of the campfire's frames shows, in milliseconds
/// (*tunable*).
pub const FIRE_FRAME_MS: u32 = 140;
/// How far down the campfire's picture its flames are, as a part of its
/// height (the rest above is smoke): where the embers start.
const FIRE_FLAMES: f32 = 0.62;

/// How many embers rise from the fire (*tunable*).
pub const EMBERS: u64 = 26;
/// The rows embers cross: every row above the help line.
pub const EMBER_ROWS: i32 = 31;
/// How fast an ember rises, in rows a second, slowest to fastest
/// (*tunable*).
pub const EMBER_SPEED: (f32, f32) = (6.0, 11.0);
/// How many rows an ember rises before it is gone, fewest to most
/// (*tunable*).
pub const EMBER_LIFE: (f32, f32) = (8.0, 15.0);
/// How far from the fire's middle an ember starts, in cells to a side.
const EMBER_SPREAD: f32 = 2.0;
/// How far an ember drifts to one side by the end of its rise, in cells.
const EMBER_DRIFT: f32 = 3.0;
/// The share of its rise an ember is at its brightest before it fades.
const EMBER_GLOW: f32 = 0.5;
/// An ember dimmer than this is gone (dark dots read as soot).
pub const EMBER_GONE: f32 = 0.3;
/// What an ember looks like: its glyph and colour, by kind.
const EMBER_KINDS: [(char, UiColor); 5] = [
    ('•', UiColor::Enemy),
    ('∙', UiColor::Cursor),
    ('∙', UiColor::TextHighlight),
    ('•', UiColor::TextHighlight),
    ('*', UiColor::Enemy),
];
/// The size embers rise from with no fire picture: the game's own fire's.
pub const DEFAULT_FIRE: (u32, u32) = (26, 52);

/// An ember on the screen: its cell, glyph and colour, and how bright it
/// is.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ember {
    /// Its column.
    pub x: i32,
    /// Its row.
    pub y: i32,
    /// Its glyph.
    pub glyph: char,
    /// Its colour at its brightest.
    pub color: UiColor,
    /// How bright it is: 1 as it leaves the fire, less as it fades.
    pub brightness: f32,
}

/// Where the campfire's picture goes for frames `frame` big: in the
/// bottom-left corner, standing on the row over the help line, at
/// [`FIRE_SCALE`].
pub fn fire_dest(frame: (u32, u32)) -> PxRect {
    let (w, h) = (
        i32::try_from(frame.0).unwrap_or(0) * FIRE_SCALE,
        i32::try_from(frame.1).unwrap_or(0) * FIRE_SCALE,
    );
    let ground = EMBER_ROWS * i32::from(ConsoleSpec::PC.cell_h);
    PxRect::new(FIRE_LEFT_PX, ground - h, w, h)
}

/// The campfire `fire` as it is `millis` after the title appeared: a frame
/// every [`FIRE_FRAME_MS`], a little brighter and darker as it flickers.
/// At `None` (idle motion is off) its first frame, still.
pub fn fire_sprite(fire: &Campfire, millis: Option<u32>) -> Sprite {
    let (w, h) = (
        i32::try_from(fire.frame.0).unwrap_or(0),
        i32::try_from(fire.frame.1).unwrap_or(0),
    );
    let frame = millis.map_or(0, |ms| (ms / FIRE_FRAME_MS) % fire.frames.max(1));
    let src_x = i32::try_from(frame).unwrap_or(0) * w;
    let mut sprite = Sprite::new(
        fire.image,
        PxRect::new(src_x, 0, w, h),
        fire_dest(fire.frame),
        Layer::Under,
    );
    if let Some(ms) = millis {
        // Two quick waves, never in step: 0.8 to 1.0.
        #[allow(clippy::cast_precision_loss)] // under a minute
        let t = (ms % 60_000) as f32 / 1000.0;
        let flicker = 0.9 + 0.06 * (t * 7.3).sin() + 0.04 * (t * 13.1).sin();
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // 0.8 to 1
        let brightness = (flicker.clamp(0.0, 1.0) * f32::from(FULL)).round() as u8;
        sprite.brightness = brightness;
    }
    sprite
}

/// Where embers start: the middle of the flames of a fire `frame` big, as
/// a column and a row (in cells, not whole).
fn ember_source(frame: (u32, u32)) -> (f32, f32) {
    let dest = fire_dest(frame);
    let (cell_w, cell_h) = (
        f32::from(ConsoleSpec::PC.cell_w),
        f32::from(ConsoleSpec::PC.cell_h),
    );
    #[allow(clippy::cast_precision_loss)] // console pixels
    let (mid, top, h) = ((dest.x + dest.w / 2) as f32, dest.y as f32, dest.h as f32);
    (mid / cell_w, (top + h * FIRE_FLAMES) / cell_h)
}

/// A number from 0 to 1 from the 16 of `bits` from `shift` up.
fn unit(bits: u64, shift: u32) -> f32 {
    f32::from(u16::try_from((bits >> shift) & 0xffff).unwrap_or(0)) / f32::from(u16::MAX)
}

/// Where ember `i` is `millis` after the title appeared, over a fire
/// `frame` big ([`fire_dest`]), if it shows. Each rises from the flames at
/// its own speed ([`EMBER_SPEED`]) for its own rows ([`EMBER_LIFE`]), both
/// fixed by `i`, then starts again from the fire; each rise starts at
/// another place near the fire's middle, drifts its own way and sways. It
/// is brightest for the first half of its rise, then fades; dimmer than
/// [`EMBER_GONE`], or off the rows above the help line, it is gone.
pub fn ember(i: u64, millis: u32, frame: (u32, u32)) -> Option<Ember> {
    let bits = splitmix64(i);
    let speed = EMBER_SPEED.0 + (EMBER_SPEED.1 - EMBER_SPEED.0) * unit(bits, 0);
    let life = EMBER_LIFE.0 + (EMBER_LIFE.1 - EMBER_LIFE.0) * unit(bits, 16);
    // Whole milliseconds, so that a rise is the same however late it is.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // 0.7 to 2.5 s
    let period = ((life / speed * 1000.0).round() as u64).max(1);
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    // under a period
    let offset = (unit(bits, 32) * period as f32) as u64;
    let t = u64::from(millis) + offset;
    #[allow(clippy::cast_precision_loss)] // under a period
    let age = (t % period) as f32 / period as f32;
    // This rise's own start, drift and look.
    let rise = splitmix64(bits ^ (t / period));
    let (sx, sy) = ember_source(frame);
    let start = sx + (unit(rise, 0) * 2.0 - 1.0) * EMBER_SPREAD;
    let drift = (unit(rise, 16) * 2.0 - 1.0) * EMBER_DRIFT * age;
    let sway = 0.8 * ((age * 2.0 + unit(rise, 32)) * TAU).sin() * age;
    let brightness = if age < EMBER_GLOW {
        1.0
    } else {
        1.0 - (age - EMBER_GLOW) * 1.8
    };
    #[allow(clippy::cast_possible_truncation)] // a cell on the console
    let (x, y) = (
        (start + drift + sway).round() as i32,
        (sy - age * life).round() as i32,
    );
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // 0 to 4
    let kind = (unit(rise, 48) * 4.99) as usize;
    let (glyph, color) = EMBER_KINDS[kind.min(EMBER_KINDS.len() - 1)];
    let cols = i32::from(ConsoleSpec::PC.cols);
    let shows = (0..cols).contains(&x) && (0..EMBER_ROWS).contains(&y);
    (brightness > EMBER_GONE && shows).then_some(Ember {
        x,
        y,
        glyph,
        color,
        brightness,
    })
}

/// The embers that show `millis` after the title appeared, over the
/// title's fire (or where it would be, at [`DEFAULT_FIRE`]'s size).
pub fn embers(ctx: &Ctx, millis: u32) -> impl Iterator<Item = Ember> {
    let frame = ctx
        .game
        .content
        .title
        .campfire
        .map_or(DEFAULT_FIRE, |f| f.frame);
    (0..EMBERS).filter_map(move |i| ember(i, millis, frame))
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
/// on it: `backdrop` (if any), the `embers` (if any) as they are that
/// many milliseconds after the title appeared, and the logo. The title screen draws its prompt or
/// menu over this; the cinematic's logo shot is this alone (ticket 0817).
pub fn paint_logo(
    ctx: &Ctx,
    backdrop: Option<ImageId>,
    embers: Option<u32>,
    buf: &mut GlyphBuffer,
) {
    let c = |u| ctx.palette.get(u);
    let black = c(UiColor::Black);
    paint_black(ctx, buf);
    for sprite in backdrop.into_iter().flat_map(backdrop_sprites) {
        buf.add_sprite(sprite);
    }
    if let Some(fire) = &ctx.game.content.title.campfire {
        buf.add_sprite(fire_sprite(fire, embers));
    }
    if let Some(millis) = embers {
        for e in self::embers(ctx, millis) {
            let fg = c(e.color).scale(e.brightness);
            buf.set(e.x, e.y, Cell::new(e.glyph, fg, black));
        }
    }
    if let Some((image, size)) = ctx.game.content.title.logo {
        buf.add_sprite(logo_sprite(image, size));
    }
}

/// Paints the whole of `buf` black.
fn paint_black(ctx: &Ctx, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    buf.fill_rect(
        buf.bounds(),
        Cell::new(' ', c(UiColor::Text), c(UiColor::Black)),
    );
}

/// Adds the title's picture over what `buf` already shows, `opacity` (up
/// to [`FULL`]) solid, changing no cell's background: the pictures of
/// [`paint_logo`] that much solid, the fire and the embers that much bright. Nothing at
/// 0.
pub fn paint_logo_over(
    ctx: &Ctx,
    backdrop: Option<ImageId>,
    embers: Option<u32>,
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
    if let Some(fire) = &ctx.game.content.title.campfire {
        buf.add_sprite(faded(fire_sprite(fire, embers)));
    }
    if let Some(millis) = embers {
        for e in self::embers(ctx, millis) {
            let fg = ctx.palette.get(e.color).scale(e.brightness * part);
            buf.print_fg(e.x, e.y, e.glyph.encode_utf8(&mut [0; 4]), fg);
        }
    }
    if let Some((image, size)) = ctx.game.content.title.logo {
        buf.add_sprite(faded(logo_sprite(image, size)));
    }
}

/// Paints `view` over the whole of `buf`: the cinematic as bright as it
/// is, or black, with the title's picture over it as solid as the view's
/// `picture`, then the prompt or the menu, as bright as the view's
/// `menu_opacity`.
pub fn paint(ctx: &Ctx, view: &TitleView, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let (dim, panel) = (c(UiColor::TextDim), c(UiColor::PanelBg));
    match &view.cinematic {
        Some(shot) => {
            #[allow(clippy::cast_precision_loss)] // far below 2^24 ms
            let t = shot.t_ms as f32 / 1000.0;
            let brightness = f32::from(shot.brightness) / f32::from(FULL);
            shot.player.draw(ctx, t, brightness, buf);
        }
        None if view.picture == FULL => paint_logo(ctx, view.backdrop, view.embers, buf),
        None => paint_black(ctx, buf),
    }
    if view.cinematic.is_some() || view.picture < FULL {
        paint_logo_over(ctx, view.backdrop, view.embers, view.picture, buf);
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
        let (w, h) = menu_glyph::size(ctx, menu);
        let x = centre_x(buf, usize::try_from(w).unwrap_or(0));
        menu_glyph::paint(ctx, menu, buf, x, MENU_ROW);
        buf.dim(Rect::new(x, MENU_ROW, w, h), solid);
        if let Some(note) = &view.note {
            let menu_box = Rect::new(x, MENU_ROW, w, h);
            paint_note(ctx, note, menu_box, solid, buf);
        }
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
        // Faded with the menu, its key caps too.
        let fade = |(fg, bg): (Rgb, Option<Rgb>)| (fg.scale(solid), bg.map(|b| b.scale(solid)));
        if view.cinematic.is_some() {
            // Over the cinematic: its letters alone, no strip behind them.
            let colors = |role| fade(ctx.palette.text_colors(role, dim));
            print_centred_styled(buf, bottom, help, colors);
        } else {
            let on = ctx.palette.text_colors_on(dim, c(UiColor::Black));
            print_centred_styled(buf, bottom, help, |role| fade(on(role)));
        }
    }
    if let Some(hint) = &view.debug_hint {
        paint_debug_hint(&ctx.palette, hint, buf, bottom);
    }
}

/// Where the guide's note goes right of the menu box `menu` in `buf`, its
/// top level with the menu's and its right edge a cell short of the
/// screen's at most, and its rows: the heading's, then the text's, each
/// wrapped to the room beside the face.
pub fn note_layout(note: &NoteView, menu: Rect, buf: &GlyphBuffer) -> (Rect, Vec<String>) {
    let left = menu.x + menu.w + NOTE_GAP;
    let face_w = note.face.as_ref().map_or(0, |_| NOTE_FACE_CELLS.0 + MARGIN);
    let most = i32::from(buf.width()) - 1 - left;
    let room = usize::try_from(most - MARGIN - face_w - MARGIN).unwrap_or(1);
    let paragraphs = core::iter::once(&note.heading).chain(&note.lines);
    let rows: Vec<String> = paragraphs
        .flat_map(|p| word_wrap(p, room, &shuyi_assets::font::CellWidths::NARROW))
        .collect();
    let len = |s: &String| i32::try_from(s.chars().count()).unwrap_or(i32::MAX);
    let text_w = rows.iter().map(len).max().unwrap_or(0);
    let title_w = len(&note.title) + 2;
    let w = MARGIN + face_w + text_w.max(title_w - face_w) + MARGIN;
    let face_h = note.face.as_ref().map_or(0, |_| NOTE_FACE_CELLS.1);
    let text_h = i32::try_from(rows.len()).unwrap_or(0);
    (Rect::new(left, menu.y, w, face_h.max(text_h) + 2), rows)
}

/// Paints the guide's note right of the menu box `menu` ([`note_layout`]),
/// as the tip box with a smaller face, as solid as the menu (`solid`).
fn paint_note(ctx: &Ctx, note: &NoteView, menu: Rect, solid: f32, buf: &mut GlyphBuffer) {
    let (rect, rows) = note_layout(note, menu, buf);
    let face = note
        .face
        .as_ref()
        .map(|f| (f, NOTE_FACE_CELLS, 1.0 - solid));
    let x = paint_frame_sized(ctx, rect, &note.title, face, buf);
    if let Some((heading, lines)) = rows.split_first() {
        let lines: Vec<_> = lines.iter().map(|l| l.as_str().into()).collect();
        paint_text(ctx, (x, rect.y + 1), heading, &lines, buf);
    }
    buf.dim(rect, solid);
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::cinematic::CinematicPlayer;
    use crate::console::{ConsoleSizes, ConsoleSpec};
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
            label: label.into(),
            suffix: None,
            note: None,
            enabled: true,
            focused,
        }
    }

    fn view() -> TitleView {
        TitleView {
            cinematic: None,
            picture: FULL,
            menu_opacity: FULL,
            backdrop: None,
            embers: None,
            prompt: None,
            menu: Some(MenuView::new(vec![
                item("New Game", true),
                item("Quit", false),
            ])),
            notice: None,
            note: None,
            help: Some("help".into()),
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

    fn logo_painted(c: &Ctx, backdrop: Option<ImageId>, embers: Option<u32>) -> GlyphBuffer {
        let mut buf = GlyphBuffer::with_spec(ConsoleSpec::PC, stale(c));
        paint_logo(c, backdrop, embers, &mut buf);
        buf
    }

    fn px(x: i32, y: i32, w: i32, h: i32) -> PxRect {
        PxRect::new(x, y, w, h)
    }

    #[test]
    fn the_logo_is_a_centred_picture_drawn_in_blocks_over_everything() {
        let c = ctx();
        let (image, size) = c.game.content.title.logo.unwrap();
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
        // With no backdrop, only the campfire is under it, standing still.
        let fire = c.game.content.title.campfire.unwrap();
        assert_eq!(
            logo_painted(&c, None, None).sprites(),
            [fire_sprite(&fire, None), logo]
        );
        // The menu sits under it, clear of it.
        let menu_top = MENU_ROW * i32::from(crate::console::ConsoleSpec::PC.cell_h);
        assert!(logo.dest.y + logo.dest.h <= menu_top);
    }

    #[test]
    fn the_backdrop_fills_the_width_with_its_edges_mirrored_under_the_logo() {
        let c = ctx();
        let image = c.game.content.title.backdrops[0];
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
        let (logo, size) = c.game.content.title.logo.unwrap();
        let fire = fire_sprite(&c.game.content.title.campfire.unwrap(), Some(0));
        assert_eq!(
            logo_painted(&c, Some(image), Some(0)).sprites(),
            [middle, left, right, fire, logo_sprite(logo, size)]
        );
        // The view's backdrop is the one painted.
        let mut v = view();
        v.menu = None;
        v.backdrop = Some(image);
        assert_eq!(painted(&c, &v).sprites()[..3], [middle, left, right]);
    }

    /// The game's fire's frames.
    const FIRE: (u32, u32) = (26, 52);

    #[test]
    fn the_campfire_stands_in_the_corner_and_flickers_through_its_frames() {
        let c = ctx();
        let fire = c.game.content.title.campfire.unwrap();
        // 4× in the bottom-left corner, its foot on the help line's top.
        assert_eq!(fire_dest(fire.frame), px(40, 288, 104, 208));
        let still = fire_sprite(&fire, None);
        assert_eq!(
            (still.src, still.dest),
            (px(0, 0, 26, 52), fire_dest(fire.frame))
        );
        assert_eq!(
            (still.layer, still.opacity, still.brightness),
            (Layer::Under, FULL, FULL)
        );
        // A frame every 140 ms, round and round.
        let frames: Vec<i32> = [0, 139, 140, 280, 420, 560, 700]
            .map(|ms| fire_sprite(&fire, Some(ms)).src.x)
            .to_vec();
        assert_eq!(frames, [0, 0, 26, 52, 78, 0, 26]);
        // It flickers: a little darker now and then, never brighter than
        // the picture, never far darker.
        let lights: Vec<u8> = (0..200)
            .map(|step| fire_sprite(&fire, Some(step * 37)).brightness)
            .collect();
        assert!(
            lights.iter().all(|&b| (200..=FULL).contains(&b)),
            "{lights:?}"
        );
        assert!(lights.iter().any(|&b| b < 220) && lights.iter().any(|&b| b > 245));
        // However long the title is left open.
        assert!((200..=FULL).contains(&fire_sprite(&fire, Some(u32::MAX)).brightness));
    }

    /// Pinned by hand from the first run: a change to where embers are is
    /// a change of the look.
    #[test]
    fn embers_are_where_they_were() {
        let now = [
            ember(0, 0, FIRE),
            ember(1, 1284, FIRE),
            ember(7, 60_150, FIRE),
            ember(25, 3_600_000, FIRE),
        ];
        assert_eq!(now, EMBER_PINS);
    }

    #[test]
    fn embers_rise_from_the_fire_and_fade() {
        let (sx, sy) = ember_source(FIRE);
        let reach = EMBER_SPREAD + EMBER_DRIFT + 1.5;
        let mut kinds = std::collections::BTreeSet::new();
        for i in 0..EMBERS {
            let mut rows = std::collections::BTreeSet::new();
            let (mut faded, mut whole) = (false, false);
            for t in (0..10_000).step_by(10) {
                let Some(e) = ember(i, t, FIRE) else { continue };
                kinds.insert((e.glyph, e.color.name()));
                // Near the fire and above it, never higher than its life.
                #[allow(clippy::cast_precision_loss)] // a cell
                let (x, y) = (e.x as f32, e.y as f32);
                assert!((x - sx).abs() <= reach, "ember {i} x {} at {t}", e.x);
                assert!(
                    y <= sy + 0.5 && y >= sy - EMBER_LIFE.1 - 0.5,
                    "ember {i} y {} at {t}",
                    e.y
                );
                assert!(e.brightness > EMBER_GONE && e.brightness <= 1.0);
                // Bright as it leaves the fire.
                if y >= sy - 1.5 {
                    assert!(
                        (e.brightness - 1.0).abs() < f32::EPSILON,
                        "ember {i} at {t}"
                    );
                }
                faded |= e.brightness < 0.9;
                whole |= (e.brightness - 1.0).abs() < f32::EPSILON;
                rows.insert(e.y);
            }
            // It rises several rows, bright and then fading.
            assert!(rows.len() >= 5, "ember {i}: {rows:?}");
            assert!(faded && whole, "ember {i}");
        }
        assert!(kinds.len() >= 4, "{kinds:?}");
        // However long the title is left open, they are where they would be.
        for i in 0..EMBERS {
            for late in [u32::MAX, u32::MAX - 1234, 4_000_000_000] {
                if let Some(e) = ember(i, late, FIRE) {
                    assert!((0..EMBER_ROWS).contains(&e.y), "ember {i} at {late}");
                }
            }
        }
        // Gone above the top row: a fire high up loses its highest embers.
        let high = (26, 400);
        assert!((0..3000).all(|t| ember(3, t, high).is_none_or(|e| e.y >= 0)));
    }

    #[test]
    fn every_ember_is_painted_above_the_help_line() {
        let c = ctx();
        let black = c.palette.get(UiColor::Black);
        for millis in [0, 777, 123_456] {
            let buf = logo_painted(&c, None, Some(millis));
            let embers: Vec<Ember> = embers(&c, millis).collect();
            assert!(embers.len() as u64 >= EMBERS / 2, "{}", embers.len());
            for e in &embers {
                // Where two share a cell, the later one shows.
                let last = embers
                    .iter()
                    .rev()
                    .find(|o| (o.x, o.y) == (e.x, e.y))
                    .unwrap();
                let fg = c.palette.get(last.color).scale(last.brightness);
                let want = Cell::new(last.glyph, fg, black);
                assert_eq!(
                    buf.get(e.x, e.y),
                    Some(&want),
                    "({}, {}) at {millis}",
                    e.x,
                    e.y
                );
            }
            // Nothing else is on the screen: every other cell is blank.
            let empty = Cell::new(' ', c.palette.get(UiColor::Text), black);
            for y in 0..i32::from(ConsoleSpec::PC.rows) {
                for x in 0..i32::from(ConsoleSpec::PC.cols) {
                    if !embers.iter().any(|e| (e.x, e.y) == (x, y)) {
                        assert_eq!(buf.get(x, y), Some(&empty), "({x}, {y})");
                    }
                }
            }
            assert_eq!(text(&buf, i32::from(ConsoleSpec::PC.rows) - 1).trim(), "");
        }
        // The view's embers are the ones painted, and none is none.
        let mut v = view();
        v.menu = None;
        v.help = None;
        let none = painted(&c, &v);
        assert_eq!(none, logo_painted(&c, None, None));
        assert!((0..i32::from(ConsoleSpec::PC.rows)).all(|y| text(&none, y).trim().is_empty()));
        v.embers = Some(0);
        let at_0 = painted(&c, &v);
        v.embers = Some(1000);
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

    /// Ticket 0842: the guide's note sits right of the menu, level with
    /// its top, clear of the menu, the logo and the help line and inside
    /// the screen, with the widest menu (the debug one in the test
    /// language) and the face or none; its heading and text are inside it.
    #[test]
    fn the_guides_note_is_right_of_the_menu_clear_of_it_and_the_logo() {
        use crate::screens::TitleScreen;
        use crate::tips::view::FaceView;
        let mut c = ctx();
        let (image, size) = c.game.content.title.logo.unwrap();
        let logo = logo_sprite(image, size);
        let cell_h = i32::from(ConsoleSpec::PC.cell_h);
        for lang in ["en", trpg_content::lang::TEST] {
            c.lang = trpg_content::LangCode::new(lang).unwrap();
            let mut view = TitleScreen::with_quick_battle(&c).view(&c);
            let menu = view.menu.clone().unwrap();
            let (w, h) = menu_glyph::size(&c, &menu);
            let x = (i32::from(ConsoleSpec::PC.cols) - w) / 2;
            let menu_box = Rect::new(x, MENU_ROW, w, h);
            for face in [
                None,
                Some(FaceView {
                    portrait: "guide".into(),
                    expression: "neutral".into(),
                }),
            ] {
                let note = view.note.as_mut().unwrap();
                note.face = face;
                let buf = painted(&c, &view);
                let note = view.note.as_ref().unwrap();
                let (rect, rows) = note_layout(note, menu_box, &buf);
                assert_eq!(rect.y, MENU_ROW, "{lang}");
                assert!(rect.x > menu_box.x + menu_box.w, "{lang}: {rect:?}");
                assert!(logo.dest.y + logo.dest.h <= rect.y * cell_h, "{lang}");
                assert!(rect.x + rect.w < i32::from(buf.width()), "{lang}: {rect:?}");
                assert!(
                    rect.y + rect.h < i32::from(buf.height()) - 1,
                    "{lang}: {rect:?}"
                );
                assert!(rows.len() >= 3, "{lang}: {rows:?}");
                let x =
                    rect.x + MARGIN + note.face.as_ref().map_or(0, |_| NOTE_FACE_CELLS.0 + MARGIN);
                for (y, row) in (rect.y + 1..).zip(&rows) {
                    assert_eq!(column(&buf, y, row), x, "{lang}: {row}");
                    assert!(y < rect.y + rect.h - 1);
                }
                assert!(text(&buf, rect.y).contains(&format!(" {} ", note.title)));
            }
        }
    }

    fn note(face: bool, title: &str, lines: &[&str]) -> NoteView {
        use crate::tips::view::FaceView;
        NoteView {
            title: title.to_owned(),
            face: face.then(|| FaceView {
                portrait: "guide".into(),
                expression: "neutral".into(),
            }),
            heading: "H".to_owned(),
            lines: lines.iter().map(|l| (*l).to_owned()).collect(),
        }
    }

    /// The note is as wide as its text or its title (with a cell each side
    /// of it) beside the face, plus its margins, and as tall as the face or
    /// the text between its borders; text that won't fit is wrapped to the
    /// room left before the screen's last column.
    #[test]
    fn the_note_is_as_big_as_its_text_or_title_beside_the_face() {
        let c = ctx();
        let buf = GlyphBuffer::with_spec(ConsoleSpec::PC, stale(&c));
        let menu = Rect::new(40, MENU_ROW, 11, 4);
        let left = 53;
        let size = |n: &NoteView| {
            let (rect, rows) = note_layout(n, menu, &buf);
            assert_eq!((rect.x, rect.y), (left, MENU_ROW));
            (rect.w, rect.h, rows)
        };
        // A title of 5 cells: wider than the text.
        let rows = vec!["H".to_owned(), "a".to_owned()];
        assert_eq!(
            size(&note(false, "Vix", &["a"])),
            (2 + 5 + 2, 2 + 2, rows.clone())
        );
        // The face's 8 cells and a margin, beside text one cell wide: the
        // title runs over the face's place too.
        assert_eq!(
            size(&note(true, "Vix", &["a"])),
            (2 + 8 + 2 + 1 + 2, 4 + 2, rows)
        );
        // 46 cells to the screen's last column: 42 for the text, or 32
        // beside the face; one-letter words fill all but one of them.
        let long = ["a"; 30].join(" ");
        for (face, room) in [(false, 42), (true, 32)] {
            let (w, h, rows) = size(&note(face, "Vix", &[&long]));
            assert_eq!(rows[1].chars().count(), room - 1, "{face}");
            let face_w = if face { 10 } else { 0 };
            assert_eq!(w, 2 + face_w + i32::try_from(room).unwrap() - 1 + 2);
            assert!(left + w < i32::from(buf.width()));
            assert_eq!(
                h,
                i32::try_from(rows.len())
                    .unwrap()
                    .max(if face { 4 } else { 0 })
                    + 2
            );
        }
    }

    /// The face fades with the menu, as the note's text does.
    #[test]
    fn the_notes_face_fades_with_the_menu() {
        let mut c = ctx();
        let mut guide = c.game.content.portraits["test_lord"].clone();
        let first = guide.expressions[0].clone();
        guide.expressions.push(trpg_content::portrait::Expression {
            name: "neutral".into(),
            ..first
        });
        c.game.content.portraits.insert("guide".into(), guide);
        let (image, size) = c.game.content.title.logo.unwrap();
        let logo = logo_sprite(image, size);
        let fire = fire_sprite(&c.game.content.title.campfire.unwrap(), None);
        let face_opacity = |opacity: u8| {
            let mut v = view();
            v.note = Some(note(true, "Vix", &["a"]));
            v.menu_opacity = opacity;
            let buf = painted(&c, &v);
            let faces: Vec<u8> = buf
                .sprites()
                .iter()
                .filter(|s| s.dest != logo.dest && s.dest != fire.dest)
                .map(|s| s.opacity)
                .collect();
            assert_eq!(faces.len(), 1, "{faces:?}");
            faces[0]
        };
        assert_eq!(face_opacity(FULL), 255);
        let half = face_opacity(128);
        assert!((120..=135).contains(&half), "{half}");
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
        v.picture = 0;
        v.backdrop = Some(c.game.content.title.backdrops[0]);
        v.embers = Some(500);
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

    /// The cinematic is drawn as bright as the view says, and the title's
    /// pictures over it as solid as the view's `picture`.
    #[test]
    fn the_titles_picture_fades_over_the_cinematic() {
        let c = ctx();
        let image = c.game.content.title.backdrops[0];
        let (logo, size) = c.game.content.title.logo.unwrap();
        let mut v = view();
        v.menu = None;
        v.help = None;
        v.backdrop = Some(image);
        v.embers = Some(0);
        let playing = shot(&c, 3000, 64);
        v.cinematic = Some(playing.clone());
        v.menu_opacity = 191;
        v.picture = 191;
        let buf = painted(&c, &v);
        // The cinematic, a quarter bright.
        let under = drawn(&c, &playing, 3.0, 64.0 / 255.0);
        assert_eq!(buf.backdrop(), under.backdrop());
        assert_ne!(buf.backdrop(), drawn(&c, &playing, 3.0, 1.0).backdrop());
        // The title's five pictures, three quarters solid, as they are
        // placed on the title itself.
        let fire = c.game.content.title.campfire.unwrap();
        let mut want = backdrop_sprites(image).to_vec();
        want.push(fire_sprite(&fire, Some(0)));
        want.push(logo_sprite(logo, size));
        for sprite in &mut want {
            sprite.opacity = 191;
        }
        assert_eq!(buf.sprites(), want);
        // The embers: their glyphs over the map, no cell made solid.
        let embers: Vec<Ember> = embers(&c, 0).collect();
        let part = 191.0 / f32::from(FULL);
        for y in 0..i32::from(ConsoleSpec::PC.rows) {
            for x in 0..i32::from(ConsoleSpec::PC.cols) {
                let cell = buf.get(x, y).unwrap();
                assert!(cell.see_through, "({x}, {y})");
                match embers.iter().rev().find(|e| (e.x, e.y) == (x, y)) {
                    Some(e) => {
                        assert_eq!(cell.glyph, e.glyph, "({x}, {y})");
                        let fg = c.palette.get(e.color).scale(e.brightness * part);
                        assert_eq!(cell.fg, fg, "({x}, {y})");
                    }
                    None => assert_eq!(Some(cell), under.get(x, y), "({x}, {y})"),
                }
            }
        }
        // No embers, no backdrop: only the fire, still, and the logo.
        v.embers = None;
        v.backdrop = None;
        let buf = painted(&c, &v);
        let mut still = fire_sprite(&fire, None);
        still.opacity = 191;
        assert_eq!(buf.sprites(), [still, want[4]]);
        assert_eq!(buf.get(35, 15), under.get(35, 15));
        // Fully bright, nothing of the title is.
        v.embers = Some(0);
        v.backdrop = Some(image);
        v.picture = 0;
        v.cinematic = Some(shot(&c, 3000, FULL));
        assert_eq!(painted(&c, &v), drawn(&c, &playing, 3.0, 1.0));
        let mut buf = drawn(&c, &playing, 3.0, 1.0);
        paint_logo_over(&c, Some(image), Some(0), 0, &mut buf);
        assert_eq!(buf, drawn(&c, &playing, 3.0, 1.0));
        // Fully solid, the pictures are as on the title.
        paint_logo_over(&c, Some(image), None, FULL, &mut buf);
        assert_eq!(buf.sprites(), logo_painted(&c, Some(image), None).sprites());
    }

    /// Going to black before the cinematic (ticket 0843): with no
    /// cinematic the title's pictures are as solid as the view's `picture`
    /// over black, and at 0 the screen is black alone.
    #[test]
    fn the_titles_picture_goes_to_black() {
        let c = ctx();
        let image = c.game.content.title.backdrops[0];
        let mut v = view();
        v.menu = None;
        v.help = None;
        v.backdrop = Some(image);
        v.embers = None;
        v.picture = 128;
        let buf = painted(&c, &v);
        assert_ne!(buf.sprites().len(), 0);
        assert!(buf.sprites().iter().all(|s| s.opacity == 128));
        v.picture = 0;
        let buf = painted(&c, &v);
        assert_eq!(buf.sprites().len(), 0);
        let black = c.palette.get(UiColor::Black);
        for y in 0..i32::from(ConsoleSpec::PC.rows) {
            for x in 0..i32::from(ConsoleSpec::PC.cols) {
                let cell = buf.get(x, y).unwrap();
                assert_eq!((cell.glyph, cell.bg), (' ', black), "({x}, {y})");
            }
        }
    }

    /// The menu, the notice and the help line are as bright as the view's
    /// `menu_opacity`; the logo behind them is not touched.
    #[test]
    fn the_menu_its_notice_and_the_help_fade_together() {
        let c = ctx();
        let mut v = view();
        v.notice = Some("Notice".to_owned());
        v.backdrop = Some(c.game.content.title.backdrops[0]);
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
        v.picture = 0;
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

    const EMBER_PINS: [Option<Ember>; 4] = [
        Some(Ember {
            x: 11,
            y: 19,
            glyph: '•',
            color: UiColor::TextHighlight,
            brightness: 0.718_502_16,
        }),
        Some(Ember {
            x: 13,
            y: 26,
            glyph: '*',
            color: UiColor::Enemy,
            brightness: 1.0,
        }),
        Some(Ember {
            x: 11,
            y: 26,
            glyph: '*',
            color: UiColor::Enemy,
            brightness: 1.0,
        }),
        Some(Ember {
            x: 14,
            y: 19,
            glyph: '•',
            color: UiColor::TextHighlight,
            brightness: 0.981_680_9,
        }),
    ];
}
