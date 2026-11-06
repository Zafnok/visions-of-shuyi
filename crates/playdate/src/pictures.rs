//! The pictures the one-bit renderer draws (ticket 1040, ADR-0085). The
//! Playdate's bundle holds a one-bit picture in place of each PNG the
//! Playdate draws one-bit (`trpg_content::playdate_images`), at the PNG's
//! own path, so a sprite's image id finds it. Each is read once, at start.
//! An image whose file is still a PNG (the title's logo, the map's
//! pictures until their tickets) has none: the renderer draws its sprites
//! as grey boxes.

use alloc::collections::BTreeMap;

use trpg_content::bitmap::Bitmap;
use trpg_content::{ImageId, ImageTable};
use trpg_ui::onebit::{Image, Images};

/// Every image's one-bit picture, by id, and the style they are drawn in.
#[derive(Debug, Default)]
pub struct Pictures {
    bitmaps: BTreeMap<ImageId, Bitmap<'static>>,
    style: usize,
}

impl Pictures {
    /// The one-bit pictures of `table`'s images in the game's bundle.
    pub fn new(table: &ImageTable) -> Self {
        Self::from_files(table, trpg_content::bundle::bytes)
    }

    /// The one-bit pictures of `table`'s images, each file's bytes from
    /// `bytes` (by path): those that are one-bit pictures.
    pub fn from_files(table: &ImageTable, bytes: impl Fn(&str) -> Option<&'static [u8]>) -> Self {
        let bitmaps = table
            .ids()
            .filter_map(|id| Some((id, Bitmap::parse(bytes(id.path())?).ok()?)))
            .collect();
        Self { bitmaps, style: 0 }
    }

    /// How many images have a one-bit picture.
    pub fn len(&self) -> usize {
        self.bitmaps.len()
    }

    /// Whether no image has one.
    pub fn is_empty(&self) -> bool {
        self.bitmaps.is_empty()
    }

    /// Draws every picture in style `style` (0 is the first; the
    /// portraits' are A, A+ lines, F+ lines and G, the order of Options'
    /// portrait style, ticket 1042); a picture with fewer styles is drawn
    /// in its first. Whether that changed the style.
    pub fn set_style(&mut self, style: usize) -> bool {
        let changed = self.style != style;
        self.style = style;
        changed
    }
}

impl Images for Pictures {
    fn image(&self, id: ImageId) -> Option<Image<'_>> {
        let bitmap = self.bitmaps.get(&id)?;
        Image::from_bitmap(bitmap, self.style).or_else(|| Image::from_bitmap(bitmap, 0))
    }
}

#[cfg(test)]
mod tests {
    use trpg_content::bundle::{self, Bundle};
    use trpg_ui::glyph_buffer::{Cell, GlyphBuffer, Layer, Rect, Sprite};
    use trpg_ui::onebit::{FRAME_BYTES, Font, is_white, render};
    use trpg_ui::{ConsoleSizes, ConsoleSpec, Rgb};

    use super::*;

    /// `f` with this thread reading the Playdate's bundle.
    fn on_the_playdate<T>(f: impl FnOnce() -> T) -> T {
        let playdate: &'static Bundle = Box::leak(Box::new(Bundle::new(bundle::playdate_layers())));
        bundle::reading(playdate, f)
    }

    /// The picture of the image at `path`.
    fn image<'p>(table: &ImageTable, pictures: &'p Pictures, path: &str) -> Option<Image<'p>> {
        let id = table.id(path).unwrap_or_else(|| panic!("no {path}"));
        pictures.image(id)
    }

    #[test]
    fn the_playdates_converted_pictures_are_found_by_their_pngs_ids() {
        let (table, pictures) = on_the_playdate(|| {
            let table = trpg_content::image::load().unwrap_or_default();
            let pictures = Pictures::new(&table);
            (table, pictures)
        });
        assert!(pictures.len() >= 20, "{}", pictures.len());
        assert!(!pictures.is_empty());
        // A portrait: 64 × 64 for a 32 × 32 image, in four styles.
        let face = "portraits/lead_f/happy.png";
        let a = image(&table, &pictures, face).unwrap();
        let mut lined = Pictures::new(&table);
        assert!(lined.is_empty(), "the PC's bundle has no one-bit pictures");
        lined = on_the_playdate(|| Pictures::new(&table));
        assert!(lined.set_style(3), "a new style redraws the frame");
        assert!(!lined.set_style(3), "the same style doesn't");
        let g = image(&table, &lined, face).unwrap();
        assert_ne!(a, g);
        // A fighter has only style A, so it is drawn in it.
        let fighter = "battlers/default.png";
        assert_eq!(
            image(&table, &lined, fighter),
            image(&table, &pictures, fighter)
        );
        assert!(image(&table, &pictures, fighter).is_some());
        // The logo is a lettering (ticket 1044), found by its PNG's id too.
        assert!(image(&table, &pictures, "title/logo.png").is_some());
        assert!(Pictures::default().is_empty());
    }

    /// Ticket 1042: every face a portrait sidecar names is a one-bit picture
    /// in the Playdate's bundle, twice its PNG's size, in every one of
    /// Options' portrait styles (with `private-assets`, the bought faces
    /// too: `cargo test -p trpg-playdate --features private-assets`).
    #[test]
    fn every_portrait_has_a_one_bit_face_in_each_portrait_style() {
        use trpg_ui::PortraitStyle;

        let (content, faces) = on_the_playdate(|| {
            let content = trpg_content::load_embedded().unwrap_or_else(|e| panic!("{e}"));
            let faces: Vec<(String, Option<&'static [u8]>)> = content
                .portraits
                .values()
                .flat_map(|p| &p.expressions)
                .map(|e| e.image.path().to_owned())
                .map(|path| {
                    let bytes = bundle::bytes(&path);
                    (path, bytes)
                })
                .collect();
            (content, faces)
        });
        assert!(faces.len() >= 10, "{}", faces.len());
        let sizes = content
            .portraits
            .values()
            .flat_map(|p| &p.expressions)
            .map(|e| (e.size.width, e.size.height));
        for ((path, bytes), (w, h)) in faces.iter().zip(sizes) {
            let bytes = bytes.unwrap_or_else(|| panic!("{path} isn't in the bundle"));
            let bitmap = Bitmap::parse(bytes).unwrap_or_else(|e| panic!("{path}: {e}"));
            let size = (u32::from(bitmap.width()), u32::from(bitmap.height()));
            assert_eq!(size, (2 * w, 2 * h), "{path}");
            for style in PortraitStyle::ALL {
                let image = Image::from_bitmap(&bitmap, style.index());
                assert!(image.is_some(), "{path} has no {style:?}");
            }
        }
    }

    /// A converted portrait drawn at twice its image's size covers 64 × 64
    /// screen pixels, one picture pixel each.
    #[test]
    fn a_portrait_draws_one_screen_pixel_a_picture_pixel() {
        let face = "portraits/lead_m/neutral.png";
        let (table, pictures, bits) = on_the_playdate(|| {
            let table = trpg_content::image::load().unwrap_or_default();
            let pictures = Pictures::new(&table);
            (table, pictures, bundle::bytes(face).unwrap_or_default())
        });
        let bitmap = Bitmap::parse(bits).unwrap();
        let id = table.id(face).unwrap();
        let white = Rgb::new(255, 255, 255);
        let mut buf = GlyphBuffer::with_spec(ConsoleSpec::PLAYDATE, Cell::new(' ', white, white));
        buf.add_sprite(Sprite::new(
            id,
            Rect::new(0, 0, 32, 32),
            Rect::new(8, 16, 64, 64),
            Layer::Over,
        ));
        let atlas = trpg_content::font::load().unwrap();
        let font_bits = bundle::bytes(trpg_content::font::ATLAS_1BIT_PATH).unwrap();
        let font = Font::new(&atlas, font_bits).unwrap();
        let mut frame = Box::new([0u8; FRAME_BYTES]);
        render(&buf, &font, &pictures, &mut frame);
        for y in 0..64u16 {
            for x in 0..64u16 {
                let shown = is_white(&frame, 8 + i32::from(x), 16 + i32::from(y));
                // See-through pixels show the white ground.
                let expected = bitmap.pixel(0, x, y).unwrap_or(true);
                assert_eq!(shown, expected, "({x}, {y})");
            }
        }
    }
}
