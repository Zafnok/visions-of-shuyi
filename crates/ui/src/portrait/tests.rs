use trpg_content::portrait::{Expression, FRAME_PX};
use trpg_content::{ImageInfo, ImageTable};

use super::*;
use crate::color::Rgb;
use crate::console::{ConsoleSizes, ConsoleSpec};
use crate::glyph_buffer::{Cell, Layer, Rect, Sprite};

const BLANK: Cell = Cell::new(' ', Rgb::new(200, 200, 200), Rgb::new(0, 0, 0));

fn size(width: u32, height: u32) -> ImageInfo {
    ImageInfo { width, height }
}

/// A portrait whose one expression, `neutral`, is a `width × height` image.
fn portrait(width: u32, height: u32) -> Portrait {
    let table = ImageTable {
        images: [("portraits/t/neutral.png", size(width, height))].into(),
    };
    let image = table.id("portraits/t/neutral.png").unwrap();
    Portrait {
        character: "t".to_owned(),
        expressions: vec![Expression {
            name: "neutral".to_owned(),
            image,
            size: size(width, height),
        }],
    }
}

/// The one sprite of a `width × height` portrait drawn at cell `at` of a
/// `cells`-sized buffer, if any of it shows.
fn drawn(
    (width, height): (u32, u32),
    cells: (u16, u16),
    at: (i32, i32),
    dim: f32,
    mirror: bool,
) -> Option<Sprite> {
    let mut buf = GlyphBuffer::new(cells.0, cells.1, BLANK);
    let before = buf.clone();
    let art = portrait(width, height);
    assert!(draw_portrait(&mut buf, at, &art, "neutral", dim, mirror));
    // Only a sprite is added: no cell changes.
    for y in 0..i32::from(cells.1) {
        for x in 0..i32::from(cells.0) {
            assert_eq!(buf.get(x, y), before.get(x, y));
        }
    }
    assert_eq!(buf.items().len(), buf.sprites().len());
    assert!(buf.sprites().len() <= 1);
    buf.sprites().first().copied()
}

#[test]
fn the_frame_is_what_content_checks_images_against() {
    let px = |cells: i32, cell: u16| u32::try_from(cells * i32::from(cell)).unwrap();
    assert_eq!(
        (
            px(FRAME_CELLS.0, ConsoleSpec::PC.cell_w),
            px(FRAME_CELLS.1, ConsoleSpec::PC.cell_h)
        ),
        FRAME_PX
    );
}

#[test]
fn a_cut_bust_fills_the_frame_at_4x() {
    let s = drawn((64, 64), (40, 20), (3, 2), 0.0, false).unwrap();
    assert_eq!(s.dest, Rect::new(24, 32, 256, 256));
    assert_eq!(s.clip, s.dest);
    assert_eq!(s.src, Rect::new(0, 0, 64, 64));
    assert_eq!(s.image.path(), "portraits/t/neutral.png");
    assert_eq!(s.layer, Layer::Under);
    assert!(!s.flip_x);
    assert_eq!(s.opacity, 255);
}

#[test]
fn a_face_is_drawn_at_5x_centred() {
    let s = drawn((48, 48), (40, 20), (3, 2), 0.0, false).unwrap();
    assert_eq!(s.dest, Rect::new(24 + 8, 32 + 8, 240, 240));
    assert_eq!(s.src, Rect::new(0, 0, 48, 48));
}

#[test]
fn a_picture_too_big_is_not_drawn_but_the_expression_was_there() {
    // The expression exists, so the call still says it drew.
    assert_eq!(drawn((257, 8), (40, 20), (0, 0), 0.0, false), None);
}

#[test]
fn unknown_expression_draws_nothing() {
    let mut buf = GlyphBuffer::new(40, 20, BLANK);
    let before = buf.clone();
    let art = portrait(64, 64);
    assert!(!draw_portrait(&mut buf, (0, 0), &art, "bored", 0.0, false));
    assert_eq!(buf, before);
}

#[test]
fn the_placeholders_fill_the_frame() {
    let content = trpg_content::load_embedded().unwrap();
    let mut buf = GlyphBuffer::new(34, 18, BLANK);
    let lord = &content.portraits["test_lord"];
    assert!(draw_portrait(&mut buf, (1, 1), lord, "happy", 0.0, false));
    let sprites = buf.sprites();
    assert_eq!(sprites.len(), 1);
    assert_eq!(sprites[0].image.path(), "portraits/test_lord/happy.png");
    assert_eq!(sprites[0].src, Rect::new(0, 0, 32, 32));
    assert_eq!(sprites[0].dest, Rect::new(8, 16, 256, 256));
}
