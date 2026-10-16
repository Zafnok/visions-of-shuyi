//! The one-bit renderer (ADR-0074): draws a [`GlyphBuffer`] made with
//! [`ConsoleSpec::PLAYDATE`](crate::ConsoleSpec::PLAYDATE) into a 400 × 240
//! one-bit [`Frame`] laid out as the Playdate's own frame buffer, so its
//! front end copies it to the screen as it is.
//!
//! Colours become [`Ink`]s: a colour that is an ink's reserved
//! [code](Ink::code) (the Playdate's palette is made of them) is drawn in
//! that ink, any other one white or black by its brightness. Patterns are
//! fixed to the screen's pixels, so neighbouring tiles line up.
//!
//! The order: the white ground; a backdrop's scene in its window
//! (ADR-0048), drawn the same way; then the cell backgrounds, `Under`
//! items, the white edges of [`Ink::BlackEdged`] glyphs, the glyphs,
//! `Over` items, and last the [`Ink::Invert`] rectangles, which flip what
//! is under them. Pictures are one-bit [`Image`]s (`docs/design/playdate.md`,
//! *One-bit pictures*). Nothing here allocates.

use trpg_content::font::FALLBACK_GLYPH;
use trpg_content::{FontAtlasDef, ImageId, Ink};

use crate::color::Rgb;
use crate::glyph_buffer::{GlyphBuffer, Item, Layer, Paint, PxRect, Sprite};

/// The screen's width in pixels.
pub const WIDTH: i32 = 400;
/// The screen's height in pixels.
pub const HEIGHT: i32 = 240;
/// Bytes per row of a [`Frame`]: the Playdate's stride (50 bytes of
/// pixels, 2 unused).
pub const ROW_BYTES: usize = 52;
/// Bytes in a [`Frame`].
pub const FRAME_BYTES: usize = ROW_BYTES * HEIGHT as usize;

/// A one-bit picture of the screen in the Playdate's layout: row after
/// row of [`ROW_BYTES`] bytes, a byte's highest bit its left pixel, a set
/// bit white.
pub type Frame = [u8; FRAME_BYTES];

/// Whether pixel `(x, y)` of `frame` is white; off the screen, `false`.
pub fn is_white(frame: &Frame, x: i32, y: i32) -> bool {
    if !(0..WIDTH).contains(&x) || !(0..HEIGHT).contains(&y) {
        return false;
    }
    let byte = frame[byte_index(i64::from(x), i64::from(y))];
    byte >> (7 - (x & 7)) & 1 == 1
}

/// The ink a colour is drawn in: the ink it is the [code](Ink::code) of,
/// or else white if it is at least half bright (in the eye's weighting of
/// red, green and blue), black if not.
pub fn ink_of(rgb: Rgb) -> Ink {
    Ink::from_code([rgb.r, rgb.g, rgb.b]).unwrap_or_else(|| {
        let luma = 299 * u32::from(rgb.r) + 587 * u32::from(rgb.g) + 114 * u32::from(rgb.b);
        if luma >= 128 * 1000 {
            Ink::White
        } else {
            Ink::Black
        }
    })
}

/// Whether `ink` makes screen pixel `(x, y)` white. [`Ink::BlackEdged`]
/// and [`Ink::Invert`] are black here: the edge is drawn around glyphs,
/// and only rectangles flip.
pub const fn ink_is_white(ink: Ink, x: i32, y: i32) -> bool {
    match ink {
        Ink::White => true,
        Ink::Black | Ink::BlackEdged | Ink::Invert => false,
        // Black where x + y is even.
        Ink::Grey => x.rem_euclid(2) != y.rem_euclid(2),
        // Black every fourth pixel of every other row, the rows staggered.
        Ink::Dots => !(y.rem_euclid(2) == 0 && x.rem_euclid(4) == y.rem_euclid(4)),
        // Black where x − y is a multiple of 4: lines down to the right.
        Ink::Diagonals => (x - y).rem_euclid(4) != 0,
        // A checker of 2 × 2 squares: a 1-pixel line of it is dashed.
        Ink::Dashes => x.div_euclid(2).rem_euclid(2) != y.div_euclid(2).rem_euclid(2),
    }
}

/// The eight pixels of screen row `y` that `ink` paints in a byte of a
/// [`Frame`]: every pattern repeats within 8 pixels across, so the byte is
/// the same all along the row.
const fn ink_byte(ink: Ink, y: i32) -> u8 {
    let mut byte = 0;
    let mut x = 0;
    while x < 8 {
        byte = (byte << 1) + ink_is_white(ink, x, y) as u8;
        x += 1;
    }
    byte
}

/// The index in a [`Frame`] of the byte holding screen pixel `(x, y)`,
/// which must be on the screen.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // on the screen
const fn byte_index(x: i64, y: i64) -> usize {
    y as usize * ROW_BYTES + (x >> 3) as usize
}

/// The font as the one-bit renderer reads it: the glyph map and the
/// one-bit atlas (`trpg_content::font::ATLAS_1BIT_PATH`).
#[derive(Copy, Clone, Debug)]
pub struct Font<'a> {
    atlas: &'a FontAtlasDef,
    bits: &'a [u8],
}

impl<'a> Font<'a> {
    /// The font of glyph map `atlas` and one-bit atlas `bits`. Fails if a
    /// cell is wider than 8 pixels or `bits` lacks a glyph of the map.
    pub fn new(atlas: &'a FontAtlasDef, bits: &'a [u8]) -> Result<Self, &'static str> {
        if atlas.cell_w > 8 {
            // check-text: not player text (a build problem for the log)
            return Err("the one-bit font has cells at most 8 pixels wide");
        }
        let rows = atlas.cell_h as usize;
        let needed = atlas.glyphs.values().map(|&i| (i as usize + 1) * rows);
        if needed.max().unwrap_or(0) > bits.len() {
            // check-text: not player text (a build problem for the log)
            return Err("the one-bit atlas is shorter than its glyph map");
        }
        Ok(Self { atlas, bits })
    }

    /// The pixel rows of `glyph` (a set bit is a pixel of the glyph), or
    /// of the fallback glyph if the font lacks it.
    fn rows(&self, glyph: char) -> Option<&'a [u8]> {
        let index = self
            .atlas
            .glyphs
            .get(&glyph)
            .or_else(|| self.atlas.glyphs.get(&FALLBACK_GLYPH))?;
        let rows = self.atlas.cell_h as usize;
        let start = *index as usize * rows;
        self.bits.get(start..start + rows)
    }
}

/// A one-bit picture: `height` rows of [`stride`](Image::stride) bytes, a
/// byte's highest bit its left pixel, a set bit white; and optionally a
/// mask of the same shape, a set bit where the picture is drawn (the
/// Playdate's own bitmap layout). Without a mask every pixel is drawn.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Image<'a> {
    width: u16,
    height: u16,
    data: &'a [u8],
    mask: Option<&'a [u8]>,
}

impl<'a> Image<'a> {
    /// The picture, or `None` if `data` (or `mask`) is too short for it.
    pub fn new(width: u16, height: u16, data: &'a [u8], mask: Option<&'a [u8]>) -> Option<Self> {
        let size = Self::stride(width) * usize::from(height);
        let fits = |bytes: &[u8]| bytes.len() >= size;
        (fits(data) && mask.is_none_or(fits)).then_some(Self {
            width,
            height,
            data,
            mask,
        })
    }

    /// Bytes per row of a picture `width` pixels wide.
    pub const fn stride(width: u16) -> usize {
        (width as usize).div_ceil(8)
    }

    /// Pixel `(x, y)`: `Some(true)` white, `Some(false)` black, `None`
    /// see-through or off the picture.
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)] // checked first
    fn pixel(&self, x: i64, y: i64) -> Option<bool> {
        if !(0..i64::from(self.width)).contains(&x) || !(0..i64::from(self.height)).contains(&y) {
            return None;
        }
        let at = y as usize * Self::stride(self.width) + (x >> 3) as usize;
        let bit = |bytes: &[u8]| bytes[at] >> (7 - (x & 7)) & 1 == 1;
        match self.mask {
            Some(mask) if !bit(mask) => None,
            _ => Some(bit(self.data)),
        }
    }
}

/// Where the renderer finds a sprite's one-bit picture.
pub trait Images {
    /// The picture of `id`, or `None` if there is none (the sprite is
    /// drawn as a grey rectangle).
    fn image(&self, id: ImageId) -> Option<Image<'_>>;
}

/// No pictures at all.
impl Images for () {
    fn image(&self, _: ImageId) -> Option<Image<'_>> {
        None
    }
}

/// Draws `buf` into `out`, from its top-left corner; whatever lies past
/// 400 × 240 pixels is cut off. See the [module](self) for the order.
pub fn render(buf: &GlyphBuffer, font: &Font<'_>, images: &impl Images, out: &mut Frame) {
    out.fill(u8::MAX);
    let mut target = Target {
        frame: out,
        clip: Clip::SCREEN,
    };
    let place = Place {
        x: 0,
        y: 0,
        scale: 1,
    };
    if let Some(backdrop) = buf.backdrop() {
        let window = place.screen(backdrop.clip_px());
        let (dx, dy) = backdrop.scene_offset(1);
        let mut scene = Target {
            frame: &mut *target.frame,
            clip: Clip::SCREEN.within(window),
        };
        let at = Place {
            x: window.x0 + dx,
            y: window.y0 + dy,
            scale: i64::from(backdrop.zoom()),
        };
        draw_layers(&mut scene, backdrop.scene(), at, font, images);
    }
    draw_layers(&mut target, buf, place, font, images);
}

/// A rectangle of screen pixels, `x0..x1` × `y0..y1`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct Clip {
    x0: i64,
    y0: i64,
    x1: i64,
    y1: i64,
}

impl Clip {
    /// The whole screen.
    const SCREEN: Self = Self {
        x0: 0,
        y0: 0,
        x1: WIDTH as i64,
        y1: HEIGHT as i64,
    };

    /// The part of this inside `other` (empty if none is).
    fn within(self, other: Self) -> Self {
        Self {
            x0: self.x0.max(other.x0),
            y0: self.y0.max(other.y0),
            x1: self.x1.min(other.x1),
            y1: self.y1.min(other.y1),
        }
    }
}

/// Where a buffer is drawn: its top-left pixel at screen `(x, y)`, each of
/// its pixels `scale × scale` screen pixels.
#[derive(Copy, Clone, Debug)]
struct Place {
    x: i64,
    y: i64,
    scale: i64,
}

impl Place {
    /// The screen pixels of buffer pixels `r`.
    fn screen(self, r: PxRect) -> Clip {
        let x0 = self.x + i64::from(r.x) * self.scale;
        let y0 = self.y + i64::from(r.y) * self.scale;
        Clip {
            x0,
            y0,
            x1: x0 + i64::from(r.w) * self.scale,
            y1: y0 + i64::from(r.h) * self.scale,
        }
    }
}

/// A frame and the part of it that may be drawn on.
struct Target<'f> {
    frame: &'f mut Frame,
    clip: Clip,
}

/// What a run of pixels does to the frame.
#[derive(Copy, Clone)]
enum Pen {
    /// Paints them in the ink.
    Ink(Ink),
    /// Flips them.
    Flip,
}

impl Target<'_> {
    /// Applies `pen` to the pixels of screen row `y` from `x` on that
    /// `bits` selects (its highest bit is pixel `x`), inside the clip.
    #[allow(clippy::cast_possible_truncation)] // a byte's bits, and screen rows
    fn run(&mut self, y: i64, x: i64, bits: u64, pen: Pen) {
        let clip = self.clip;
        if bits == 0 || y < clip.y0 || y >= clip.y1 {
            return;
        }
        let (lo, hi) = (x.max(clip.x0), (x + 64).min(clip.x1));
        let mut start = lo & !7;
        while start < hi {
            // The bits of the 8 pixels from `start`, which is 7 pixels
            // left of `x` at most: with `bits` 8 places up a `u128`, pixel
            // `start` is the highest bit after this shift.
            let wide = u128::from(bits) << 56 << (start - x + 8);
            let mut mask = (wide >> 120) as u8;
            // Only those inside `lo..hi`.
            mask &= u8::MAX >> (lo - start).max(0);
            mask &= u8::MAX << (start + 8 - hi).max(0);
            let byte = &mut self.frame[byte_index(start, y)];
            *byte = match pen {
                // The ink's bits where `mask` is set, the frame's elsewhere.
                Pen::Ink(ink) => *byte ^ ((*byte ^ ink_byte(ink, y as i32)) & mask),
                Pen::Flip => *byte ^ mask,
            };
            start += 8;
        }
    }

    /// Applies `pen` to every pixel of `rect` inside the clip.
    fn fill(&mut self, rect: Clip, pen: Pen) {
        let rect = rect.within(self.clip);
        for y in rect.y0..rect.y1 {
            let mut x = rect.x0;
            while x < rect.x1 {
                let len = (rect.x1 - x).min(64);
                self.run(y, x, u64::MAX << (64 - len), pen);
                x += 64;
            }
        }
    }

    /// Sets pixel `(x, y)`, which must be inside the clip.
    fn set(&mut self, x: i64, y: i64, white: bool) {
        let byte = &mut self.frame[byte_index(x, y)];
        let bit = 0x80 >> (x & 7);
        if white {
            *byte |= bit;
        } else {
            *byte &= !bit;
        }
    }
}

/// Draws `buf`'s cells and items at `at` into `target`: cell backgrounds
/// (never a see-through cell's), `Under` items, the white edges of
/// edged glyphs, the glyphs, `Over` items, the invert rectangles.
fn draw_layers(
    target: &mut Target<'_>,
    buf: &GlyphBuffer,
    at: Place,
    font: &Font<'_>,
    images: &impl Images,
) {
    let spec = buf.spec();
    let (cell_w, cell_h) = (i32::from(spec.cell_w), i32::from(spec.cell_h));
    let cells = || {
        (0..i32::from(buf.height())).flat_map(move |y| {
            (0..i32::from(buf.width())).filter_map(move |x| {
                let rect = PxRect::new(x * cell_w, y * cell_h, cell_w, cell_h);
                buf.get(x, y).map(|cell| (at.screen(rect), cell))
            })
        })
    };
    for (rect, cell) in cells() {
        if !cell.see_through {
            target.fill(rect, Pen::Ink(ink_of(cell.bg)));
        }
    }
    draw_items(target, buf, Layer::Under, at, images);
    for (rect, cell) in cells() {
        if ink_of(cell.fg) == Ink::BlackEdged {
            draw_glyph(target, font, cell.glyph, rect, at.scale, Glyph::Edge);
        }
    }
    for (rect, cell) in cells() {
        draw_glyph(
            target,
            font,
            cell.glyph,
            rect,
            at.scale,
            Glyph::Ink(ink_of(cell.fg)),
        );
    }
    draw_items(target, buf, Layer::Over, at, images);
    for item in buf.items() {
        if let Item::Rect(o) = item
            && ink_of(o.color) == Ink::Invert
        {
            target.fill(at.screen(o.rect), Pen::Flip);
        }
    }
}

/// Draws `buf`'s items of `layer` in order, but for the invert
/// rectangles, which are drawn last.
fn draw_items(
    target: &mut Target<'_>,
    buf: &GlyphBuffer,
    layer: Layer,
    at: Place,
    images: &impl Images,
) {
    for item in buf.items().iter().filter(|item| item.layer() == layer) {
        match item {
            Item::Rect(o) => {
                let ink = ink_of(o.color);
                if ink != Ink::Invert {
                    target.fill(at.screen(o.rect), Pen::Ink(ink));
                }
            }
            Item::Sprite(s) => draw_sprite(target, s, at, images),
        }
    }
}

/// What part of a glyph is drawn.
#[derive(Copy, Clone)]
enum Glyph {
    /// The glyph's pixels in this ink.
    Ink(Ink),
    /// White around it, one screen pixel wide all round (corners too).
    Edge,
}

/// Draws `glyph` (a space is nothing) of `font` in `part` with its cell's
/// top-left at `cell`, each of its pixels `scale` screen pixels big.
fn draw_glyph(
    target: &mut Target<'_>,
    font: &Font<'_>,
    glyph: char,
    cell: Clip,
    scale: i64,
    part: Glyph,
) {
    if glyph == ' ' {
        return;
    }
    let Some(rows) = font.rows(glyph) else {
        return;
    };
    let height = i64::try_from(rows.len()).unwrap_or(0) * scale;
    // The glyph's pixels on screen row `top + dy`, from `cell.x0`.
    let row = |dy: i64| {
        let at = (0..height).contains(&dy).then(|| dy / scale);
        let byte = at.and_then(|at| rows.get(usize::try_from(at).ok()?));
        byte.map_or(0, |&byte| widen(byte, scale))
    };
    match part {
        Glyph::Ink(ink) => {
            for dy in 0..height {
                target.run(cell.y0 + dy, cell.x0, row(dy), Pen::Ink(ink));
            }
        }
        Glyph::Edge => {
            for dy in -1..=height {
                // From one pixel left of the cell.
                let near = (row(dy - 1) | row(dy) | row(dy + 1)) >> 1;
                let edge = near | near << 1 | near >> 1;
                target.run(cell.y0 + dy, cell.x0 - 1, edge, Pen::Ink(Ink::White));
            }
        }
    }
}

/// `byte`'s eight pixels each `scale` (1 to 8) pixels wide, from the
/// highest bit down.
fn widen(byte: u8, scale: i64) -> u64 {
    let mut out = 0u64;
    for i in (0..8).rev() {
        for _ in 0..scale {
            out = (out << 1) + u64::from(byte >> i & 1);
        }
    }
    out << (64 - 8 * scale)
}

/// The Bayer 4 × 4 order a sprite's pixels fade out in, as its opacity
/// falls (`docs/design/playdate.md` dithers in the same pattern).
const BAYER: [[u32; 4]; 4] = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];

/// Whether a pixel of a sprite of `opacity` is drawn at screen `(x, y)`:
/// all of them at 255, none at 0, an even dither between.
#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)] // 0..4
fn shows(opacity: u8, x: i64, y: i64) -> bool {
    let rank = BAYER[(y & 3) as usize][(x & 3) as usize];
    u32::from(opacity) * 16 > rank * 255 + 127
}

/// Draws the visible part of `sprite` at `at`: each screen pixel takes
/// the image pixel under its centre, as nearest sampling does, coloured
/// as its paint says. Without an image it is a grey rectangle.
fn draw_sprite(target: &mut Target<'_>, sprite: &Sprite, at: Place, images: &impl Images) {
    let shown = at.screen(sprite.clip).within(target.clip);
    let Some(image) = images.image(sprite.image) else {
        target.fill(shown, Pen::Ink(Ink::Grey));
        return;
    };
    let (src, dest) = (sprite.src, sprite.dest);
    let (src_w, src_h) = (i64::from(src.w), i64::from(src.h));
    let (dest_w, dest_h) = (i64::from(dest.w), i64::from(dest.h));
    let solid = match sprite.paint {
        Paint::Solid(color) => Some(ink_of(color)),
        Paint::Image | Paint::Dimmed | Paint::Shaded(_) => None,
    };
    for y in shown.y0..shown.y1 {
        let dy = (y - at.y).div_euclid(at.scale) - i64::from(dest.y);
        let v = (2 * dy + 1) * src_h / (2 * dest_h);
        for x in shown.x0..shown.x1 {
            let mut dx = (x - at.x).div_euclid(at.scale) - i64::from(dest.x);
            if sprite.flip_x {
                dx = dest_w - 1 - dx;
            }
            let u = (2 * dx + 1) * src_w / (2 * dest_w);
            let Some(white) = image.pixel(i64::from(src.x) + u, i64::from(src.y) + v) else {
                continue;
            };
            if !shows(sprite.opacity, x, y) {
                continue;
            }
            #[allow(clippy::cast_possible_truncation)] // screen pixels
            let white = match (solid, sprite.paint) {
                (Some(ink), _) => ink_is_white(ink, x as i32, y as i32),
                // One-bit has no dimming: every other pixel goes white
                // (`docs/design/playdate.md`, the faded listener).
                (None, Paint::Dimmed) => white || (x + y) & 1 == 1,
                (None, _) => white,
            };
            target.set(x, y, white);
        }
    }
}

#[cfg(test)]
mod tests;
