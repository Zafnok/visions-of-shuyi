//! The fast renderer (ticket 1055) against [`reference`], 1032's plain
//! one: any buffer draws the same, and redrawing only what changed gives
//! the full render of the new buffer.

use alloc::rc::Rc;

use proptest::prelude::*;
use trpg_content::{FontAtlasDef, ImageId, Ink};

use super::*;
use crate::console::ConsoleSpec;
use crate::glyph_buffer::{Cell, Overlay, Rect};

/// A font of 8 × 16 cells: `X` a block, `?` (the fallback) two corners,
/// `[` and `]` a column at either side, `"` two dots, `#` every pixel.
fn test_font() -> (FontAtlasDef, Vec<u8>) {
    let atlas = FontAtlasDef {
        cell_w: 8,
        cell_h: 16,
        columns: 32,
        glyphs: [('X', 0), ('?', 1), ('[', 2), (']', 3), ('"', 4), ('#', 5)].into(),
    };
    let mut bits = vec![0u8; 6 * 16];
    bits[6] = 0b0001_1000;
    bits[7] = 0b0001_1000;
    bits[16] = 0b1000_0001;
    bits[32..48].fill(0b1000_0000);
    bits[48..64].fill(0b0000_0001);
    bits[72] = 0b0010_1000;
    bits[80..96].fill(u8::MAX);
    (atlas, bits)
}

/// The glyphs drawn: some of the font, a space, and two it lacks (one
/// ASCII, one not), drawn as the fallback.
const GLYPHS: [char; 9] = [' ', ' ', 'X', '?', '[', ']', '"', '#', 'Z'];
/// Every ink's code, and colours white or black by brightness, one each
/// side of the half.
fn colors() -> Vec<Rgb> {
    let mut colors: Vec<Rgb> = Ink::ALL.iter().map(|ink| Rgb::from(ink.code())).collect();
    colors.extend([
        Rgb::new(250, 250, 10),
        Rgb::new(200, 30, 30),
        Rgb::new(128, 128, 127),
        Rgb::new(128, 128, 128),
    ]);
    colors
}

/// A test picture 8 × 4, its mask see-through in its middle.
const PICTURE: [u8; 4] = [0b1010_0110, 0b0101_1001, 0b1111_0000, 0b0000_1111];
const PICTURE_MASK: [u8; 4] = [u8::MAX, 0b1110_0111, 0b1100_0011, u8::MAX];

/// The test picture's id.
fn picture_id() -> ImageId {
    let size = trpg_content::ImageInfo {
        width: 8,
        height: 4,
    };
    let table = trpg_content::ImageTable {
        images: [("p.png", size)].into(),
    };
    table.id("p.png").unwrap()
}

struct TestImages;

impl Images for TestImages {
    fn image(&self, id: ImageId) -> Option<Image<'_>> {
        (id == picture_id())
            .then(|| Image::new(8, 4, &PICTURE, Some(&PICTURE_MASK)))
            .flatten()
    }
}

/// A cell: a glyph, fg and bg (indices into [`GLYPHS`] and
/// [`colors`]) and whether it is see-through.
fn cell() -> impl Strategy<Value = Cell> {
    let colors = colors();
    let n = colors.len();
    (0..GLYPHS.len(), 0..n, 0..n, prop::bool::weighted(0.2)).prop_map(
        move |(glyph, fg, bg, see_through)| Cell {
            glyph: GLYPHS[glyph],
            fg: colors[fg],
            bg: colors[bg],
            see_through,
        },
    )
}

/// What may be put in a buffer.
#[derive(Clone, Debug)]
enum Thing {
    /// A rectangle in a colour of [`colors`].
    Rect(Rect, usize, bool),
    /// The test picture: `src` (inside the picture), `dest`, `clip`,
    /// flipped, opacity, brightness and paint (0 the image, 1 dimmed, 2
    /// in a colour of [`colors`] after it).
    Sprite(Rect, Rect, Rect, bool, u8, u8, usize, bool),
}

fn px_rect(w: i32, h: i32) -> impl Strategy<Value = Rect> {
    (-12..w, -12..h, 0..w / 2, 0..h / 2).prop_map(|(x, y, w, h)| Rect::new(x, y, w, h))
}

fn thing() -> impl Strategy<Value = Thing> {
    let n = colors().len();
    let src = (0..8, 0..4).prop_flat_map(|(x, y)| {
        (1..=8 - x, 1..=4 - y).prop_map(move |(w, h)| Rect::new(x, y, w, h))
    });
    prop_oneof![
        (px_rect(420, 260), 0..n, any::<bool>()).prop_map(|(r, c, l)| Thing::Rect(r, c, l)),
        (
            src,
            px_rect(420, 260),
            px_rect(420, 260),
            any::<bool>(),
            prop_oneof![Just(u8::MAX), any::<u8>()],
            prop_oneof![Just(u8::MAX), any::<u8>()],
            0..n + 2,
            any::<bool>(),
        )
            .prop_map(|(s, d, c, f, o, b, p, l)| Thing::Sprite(s, d, c, f, o, b, p, l)),
    ]
}

impl Thing {
    fn put(&self, buf: &mut GlyphBuffer) {
        let colors = colors();
        let layer = |over: bool| if over { Layer::Over } else { Layer::Under };
        match *self {
            Thing::Rect(rect, color, over) => {
                buf.add_overlay(Overlay::new(rect, colors[color], layer(over)));
            }
            Thing::Sprite(src, dest, clip, flip_x, opacity, brightness, paint, over) => {
                let paint = match paint {
                    0 => Paint::Image,
                    1 => Paint::Dimmed,
                    i => Paint::Solid(colors[i - 2]),
                };
                buf.add_sprite(Sprite {
                    clip,
                    flip_x,
                    opacity,
                    brightness,
                    paint,
                    ..Sprite::new(picture_id(), src, dest, layer(over))
                });
            }
        }
    }
}

/// A buffer of Playdate cells up to `max_w × max_h`, sometimes past the
/// screen, with its cells and things.
fn plain_buffer(max_w: u16, max_h: u16) -> impl Strategy<Value = GlyphBuffer> {
    (1..=max_w, 1..=max_h)
        .prop_flat_map(|(w, h)| {
            let cells = prop::collection::vec(cell(), usize::from(w) * usize::from(h));
            (Just((w, h)), cells, prop::collection::vec(thing(), 0..6))
        })
        .prop_map(|((w, h), cells, things)| {
            let mut buf = GlyphBuffer::with_spec(ConsoleSpec::PLAYDATE.sized(w, h), cells[0]);
            for (i, cell) in (0i32..).zip(cells) {
                buf.set(i % i32::from(w), i / i32::from(w), cell);
            }
            for thing in &things {
                thing.put(&mut buf);
            }
            buf
        })
}

/// A backdrop: a scene, the window's cells, the origin and the zoom.
type Backdrop = (GlyphBuffer, Rect, (f32, f32), u8);

fn backdrop() -> impl Strategy<Value = Backdrop> {
    (
        plain_buffer(24, 8),
        (0..52, 0..16, 1..30, 1..10).prop_map(|(x, y, w, h)| Rect::new(x, y, w, h)),
        (-80.0f32..80.0, -40.0f32..40.0),
        1..=4u8,
    )
}

fn with_backdrop(mut buf: GlyphBuffer, backdrop: Option<Backdrop>) -> GlyphBuffer {
    if let Some((scene, clip, origin, zoom)) = backdrop {
        buf.set_backdrop(Rc::new(scene), clip, origin, zoom);
    }
    buf
}

/// Any buffer: a plain one, sometimes with a backdrop.
fn buffer() -> impl Strategy<Value = GlyphBuffer> {
    (
        plain_buffer(52, 16),
        prop::option::weighted(0.3, backdrop()),
    )
        .prop_map(|(buf, backdrop)| with_backdrop(buf, backdrop))
}

/// A change to a buffer.
#[derive(Clone, Debug)]
enum Edit {
    Set(i32, i32, Cell),
    Put(Thing),
    /// Fills cells, which cuts the items over them.
    Fill(Rect, Cell),
    Backdrop(Option<Backdrop>),
}

fn edit() -> impl Strategy<Value = Edit> {
    prop_oneof![
        6 => (0..52, 0..16, cell()).prop_map(|(x, y, c)| Edit::Set(x, y, c)),
        2 => thing().prop_map(Edit::Put),
        1 => (0..52, 0..16, 1..12, 1..6, cell())
            .prop_map(|(x, y, w, h, c)| Edit::Fill(Rect::new(x, y, w, h), c)),
        1 => prop::option::of(backdrop()).prop_map(Edit::Backdrop),
    ]
}

impl Edit {
    fn apply(&self, buf: GlyphBuffer) -> GlyphBuffer {
        let mut buf = buf;
        match self {
            Edit::Set(x, y, cell) => buf.set(*x, *y, *cell),
            Edit::Put(thing) => thing.put(&mut buf),
            Edit::Fill(rect, cell) => buf.fill_rect(*rect, *cell),
            Edit::Backdrop(None) => buf.clear_backdrop(),
            Edit::Backdrop(Some(b)) => return with_backdrop(buf, Some(b.clone())),
        }
        buf
    }
}

fn fast(buf: &GlyphBuffer) -> Box<Frame> {
    let (atlas, bits) = test_font();
    let font = Font::new(&atlas, &bits).unwrap();
    let mut frame = Box::new([0u8; FRAME_BYTES]);
    render(buf, &font, &TestImages, &mut frame);
    frame
}

fn slow(buf: &GlyphBuffer) -> Box<Frame> {
    let (atlas, bits) = test_font();
    let font = Font::new(&atlas, &bits).unwrap();
    let mut frame = Box::new([0u8; FRAME_BYTES]);
    reference::render(buf, &font, &TestImages, &mut frame);
    frame
}

/// Where two frames first differ, for a failure's message.
fn first_difference(a: &Frame, b: &Frame) -> Option<(usize, usize)> {
    let i = a.iter().zip(b.iter()).position(|(a, b)| a != b)?;
    Some((i % ROW_BYTES * 8, i / ROW_BYTES))
}

proptest! {
    #[test]
    fn the_fast_renderer_draws_as_the_reference_does(buf in buffer()) {
        let (fast, slow) = (fast(&buf), slow(&buf));
        prop_assert!(fast == slow, "first differs at {:?}", first_difference(&fast, &slow));
    }

    #[test]
    fn redrawing_what_changed_gives_the_full_render(
        old in buffer(),
        edits in prop::collection::vec(edit(), 0..8),
    ) {
        let new = edits.iter().fold(old.clone(), |buf, edit| edit.apply(buf));
        let (atlas, bits) = test_font();
        let font = Font::new(&atlas, &bits).unwrap();
        let mut frame = fast(&old);
        render_changed(&old, &new, &font, &TestImages, &mut frame);
        let full = fast(&new);
        prop_assert!(*frame == *full, "first differs at {:?}", first_difference(&frame, &full));
    }
}

/// Every ink's looked-up byte is the pattern's pixels, on every row and
/// all along it.
#[test]
fn pattern_bytes_are_the_inks_pixels() {
    for ink in Ink::ALL {
        for y in 0..HEIGHT {
            let byte = pattern(ink, y);
            for x in 0..WIDTH {
                let white = byte >> (7 - (x & 7)) & 1 == 1;
                assert_eq!(white, ink_is_white(ink, x, y), "{ink:?} at ({x}, {y})");
            }
        }
    }
}

/// The ASCII table finds the rows the glyph map does, the fallback's for
/// glyphs the font lacks.
#[test]
fn ascii_glyphs_are_looked_up_as_the_glyph_map_has_them() {
    let (atlas, bits) = test_font();
    let font = Font::new(&atlas, &bits).unwrap();
    let from_map = |glyph: char| {
        let start = rows_start(&atlas, glyph)?;
        bits.get(start..start + 16)
    };
    for glyph in (0..=0x2fff).filter_map(char::from_u32) {
        assert_eq!(font.rows(glyph), from_map(glyph), "{glyph:?}");
    }
    assert_eq!(font.rows('Z'), font.rows('?'));
    assert_eq!(font.rows('é'), font.rows('?'));
    // Without a fallback, a glyph it lacks has no rows.
    let mut bare = atlas.clone();
    bare.glyphs.remove(&'?');
    let font = Font::new(&bare, &bits).unwrap();
    assert_eq!(font.rows('Z'), None);
    assert_eq!(font.rows('é'), None);
    assert_eq!(font.rows('X'), Some(&bits[..16]));
}

/// A run's one-byte path: a whole cell's row, inside the clip, at its
/// edges and cut by them.
#[test]
fn a_byte_at_a_whole_cell_is_written_whole_only_inside_the_clip() {
    let mut frame = Box::new([u8::MAX; FRAME_BYTES]);
    let mut target = Target {
        frame: &mut frame,
        clip: Clip {
            x0: 8,
            y0: 0,
            x1: 20,
            y1: 1,
        },
    };
    let bits = u64::from(0b1000_0001u8) << 56;
    target.run(0, 0, bits, Pen::Ink(Ink::Black));
    target.run(0, 8, bits, Pen::Ink(Ink::Black));
    target.run(0, 16, bits, Pen::Ink(Ink::Black));
    target.run(0, 16, bits >> 1, Pen::Flip);
    let row: String = (0..24)
        .map(|x| if is_white(&frame, x, 0) { '.' } else { '#' })
        .collect();
    assert_eq!(row, "........#......###......");
}

/// Changed things mark the tiles they touch, which come back as runs.
#[test]
fn tiles_gather_into_runs_along_their_rows() {
    let mut tiles = Tiles::default();
    tiles.add(Clip {
        x0: 7,
        y0: 15,
        x1: 17,
        y1: 17,
    });
    tiles.add(Clip {
        x0: 392,
        y0: 230,
        x1: 900,
        y1: 900,
    });
    tiles.add(Clip {
        x0: -20,
        y0: -20,
        x1: 0,
        y1: 3,
    });
    let regions: Vec<Clip> = tiles.regions().collect();
    let clip = |x0, y0, x1, y1| Clip { x0, y0, x1, y1 };
    assert_eq!(
        regions,
        [
            clip(0, 0, 24, 16),
            clip(0, 16, 24, 32),
            clip(392, 224, 400, 240)
        ]
    );
    assert_eq!(Tiles::default().regions().count(), 0);
}

/// Every phase is marked once, in order.
#[test]
fn a_render_marks_each_phase_in_order() {
    let (atlas, bits) = test_font();
    let font = Font::new(&atlas, &bits).unwrap();
    let blank = Cell::new(' ', Rgb::new(0, 0, 0), Rgb::new(255, 255, 255));
    let buf = GlyphBuffer::with_spec(ConsoleSpec::PLAYDATE, blank);
    let mut frame = Box::new([0u8; FRAME_BYTES]);
    for slow in [false, true] {
        let mut marked = Vec::new();
        let mut mark = |phase| marked.push(phase);
        if slow {
            reference::render_marked(&buf, &font, &(), &mut frame, &mut mark);
        } else {
            render_marked(&buf, &font, &(), &mut frame, &mut mark);
        }
        assert_eq!(marked, Phase::ALL);
    }
    let names: Vec<&str> = Phase::ALL.iter().map(|p| p.name()).collect();
    assert_eq!(
        names,
        [
            "ground",
            "backdrop",
            "backgrounds",
            "under items",
            "edges",
            "glyphs",
            "over items",
            "invert"
        ]
    );
}
