//! The frame name → look helper (ticket 1380): the picture look, the
//! glyph look, and a build without the picture.

use shuyi_assets::image::{ImageInfo, ImageTable};
use trpg_content::frames::{Borders, NineSlice, UiFrames};

use super::*;
use crate::glyph_buffer::BoxStyle;
use crate::screen::tests::ctx;

/// The bought frames' stand-ins: the embedded `frames.ron` read over
/// images of the bought pictures' sizes, so a test sees what a build with
/// the art sees (never the art itself).
pub(crate) fn bought_frames() -> UiFrames {
    let panel = ImageInfo::new(96, 96);
    let images = ImageTable::new(
        [
            ("ui/panel.png", panel),
            ("ui/menu.png", panel),
            ("ui/map_menu.png", panel),
            ("ui/plate.png", ImageInfo::new(33, 32)),
        ]
        .into(),
    );
    trpg_content::frames::load(&images).unwrap()
}

/// [`ctx`] with the bought frames' stand-ins.
pub(crate) fn framed_ctx() -> Ctx {
    let mut c = ctx();
    c.game.content.frames = bought_frames();
    c
}

/// Asserts that the window on the cells `window` wears the frame `image`
/// (its picture's path, `ui/panel.png`): the picture's sprites lie over
/// the window widened by the frame's outset (5 console pixels) and cover
/// all of it, and none lies over its inside cells.
pub(crate) fn assert_framed(buf: &GlyphBuffer, image: &str, window: Rect) {
    let (cw, ch) = buf.spec().cell_px();
    let outer = Rect::new(
        window.x * cw - 5,
        window.y * ch - 5,
        window.w * cw + 10,
        window.h * ch + 10,
    );
    let inside = Rect::new(
        (window.x + 1) * cw,
        (window.y + 1) * ch,
        (window.w - 2) * cw,
        (window.h - 2) * ch,
    );
    let parts: Vec<_> = buf
        .sprites()
        .into_iter()
        .filter(|s| s.image.path() == image)
        .collect();
    assert_ne!(parts.len(), 0, "no {image} sprite for {window:?}");
    let (mut x0, mut y0, mut x1, mut y1) = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
    for s in &parts {
        x0 = x0.min(s.dest.x);
        y0 = y0.min(s.dest.y);
        x1 = x1.max(s.dest.x + s.dest.w);
        y1 = y1.max(s.dest.y + s.dest.h);
        assert_eq!(
            s.clip.intersect(&inside),
            None,
            "{image} over the inside: {s:?}"
        );
    }
    let bounds = buf.pixel_bounds();
    let expected = outer.intersect(&bounds).unwrap();
    let found = Rect::new(x0, y0, x1 - x0, y1 - y0)
        .intersect(&bounds)
        .unwrap();
    assert_eq!(found, expected, "{image} around {window:?}");
}

/// Asserts that a title plate (`ui/plate.png`) lies behind the text on the
/// cells `text`.
pub(crate) fn assert_plated(buf: &GlyphBuffer, text: Rect) {
    let (cw, ch) = buf.spec().cell_px();
    let behind = Rect::new(text.x * cw, text.y * ch, text.w * cw, ch);
    let plate: Vec<_> = buf
        .sprites()
        .into_iter()
        .filter(|s| s.image.path() == "ui/plate.png")
        .collect();
    let body = plate.iter().find(|s| s.src.w == 1);
    let body = body.unwrap_or_else(|| panic!("no plate behind {text:?}: {plate:?}"));
    assert_eq!(
        body.dest.y,
        behind.y - 8,
        "the title's row 8 px down the plate"
    );
    assert_eq!(body.dest.x, behind.x - 6, "6 px of body before the title");
    assert_eq!(body.dest.w, behind.w + 12, "and after it");
}

fn plain() -> PanelLook {
    PanelLook::new(BoxStyle::Double, Rgb::new(1, 2, 3), Rgb::new(4, 5, 6))
}

#[test]
fn the_picture_look_frames_a_window_with_its_fill_and_the_plate() {
    let c = framed_ctx();
    let look = framed(&c, PANEL, plain());
    let frame = look.frame.expect("a frame");
    assert_eq!(frame.outset, 5);
    assert_eq!(
        frame.slice.borders,
        Borders {
            left: 22,
            top: 33,
            right: 22,
            bottom: 32
        }
    );
    assert_eq!(look.bg, Rgb::new(0x3a, 0x2c, 0x36));
    assert_eq!(look.text, plain().text);
    let plate = look.plate.expect("a plate");
    assert_eq!((plate.pad, plate.text_top), (6, 8));
    assert_eq!(bg(&c, MAP_MENU), Rgb::new(0x26, 0x38, 0x57));
    assert_eq!(framed(&c, MENU, plain()).bg, Rgb::new(0x38, 0x2b, 0x33));
}

#[test]
fn the_glyph_look_keeps_the_lines() {
    let mut c = framed_ctx();
    c.change_settings(|s| s.game.window_look = WindowLook::Glyphs)
        .unwrap();
    assert_eq!(framed(&c, PANEL, plain()), plain());
    assert_eq!(bg(&c, PANEL), c.palette.get(UiColor::PanelBg));
}

#[test]
fn without_the_picture_the_lines_stay_whatever_the_look() {
    let c = ctx();
    assert_eq!(c.settings().game.window_look, WindowLook::Pictures);
    assert_eq!(framed(&c, PANEL, plain()), plain());
    // One frame's picture missing: that window keeps its lines.
    let mut c = framed_ctx();
    c.game.content.frames.frames.remove(MENU);
    assert_eq!(framed(&c, MENU, plain()), plain());
    assert!(framed(&c, PANEL, plain()).frame.is_some());
}

#[test]
fn the_playdate_keeps_its_one_bit_windows() {
    let mut c = framed_ctx();
    c.game.skin_set = SkinSet::Playdate;
    assert_eq!(framed(&c, PANEL, plain()), plain());
}

#[test]
fn a_plate_goes_behind_a_skins_own_title() {
    let c = framed_ctx();
    let mut buf = GlyphBuffer::new(
        20,
        4,
        crate::glyph_buffer::Cell::new(' ', Rgb::new(0, 0, 0), Rgb::new(0, 0, 0)),
    );
    plate_behind(&c, &mut buf, Rect::new(5, 0, 8, 1));
    let sprites = buf.sprites();
    assert_eq!(sprites.len(), 3, "{sprites:?}");
    let slice: NineSlice = bought_frames().plate(TITLE).unwrap().slice;
    assert!(sprites.iter().all(|s| s.image == slice.picture.image));
    let mut glyphs = ctx();
    glyphs.game.content.frames = bought_frames();
    glyphs
        .change_settings(|s| s.game.window_look = WindowLook::Glyphs)
        .unwrap();
    let mut buf = GlyphBuffer::new(
        20,
        4,
        crate::glyph_buffer::Cell::new(' ', Rgb::new(0, 0, 0), Rgb::new(0, 0, 0)),
    );
    plate_behind(&glyphs, &mut buf, Rect::new(5, 0, 8, 1));
    assert_eq!(buf.sprites().len(), 0);
}
