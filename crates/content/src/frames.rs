//! The windows' picture frames (`assets/ui/frames.ron`, ticket 1380;
//! `docs/design/look-and-feel.md`, *Windows (the picture look)*): each
//! frame the skins ask for by name, with its picture in the bought art's
//! `ui/` folder, where its corners end, the colour its inside is filled
//! with and how far it reaches past its window; and the plates a title
//! sits on. Format: `assets/ui/README.md`.
//!
//! The pictures are bought art (ADR-0040): a build without them has no
//! frames here, and its windows keep their glyph lines.
use crate::prelude::*;

use alloc::collections::BTreeMap;

use serde::Deserialize;
use shuyi_assets::image::{ImageRect, Picture};
pub use shuyi_assets::nine_slice::{Borders, NineSlice};

use crate::bundle;
use crate::error::ContentError;
use crate::image::ImageTable;
use crate::palette::parse_hex;
use crate::ron_loader::parse_ron;

/// The frames file inside the asset bundle.
pub const FRAMES_PATH: &str = "ui/frames.ron";
/// The folder of the frames' pictures inside the asset bundle.
pub const UI_DIR: &str = "ui";
/// Every frame a skin asks for, by name: the side panel and the battle's
/// boxes, the unit menu, the map menu.
pub const FRAME_NAMES: [&str; 3] = ["panel", "menu", "map_menu"];
/// Every title plate a skin asks for, by name.
pub const PLATE_NAMES: [&str; 1] = ["title"];

/// A window's frame: its picture in nine slices, how far it reaches past
/// the window in console pixels, and the colour of the window's inside.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Frame {
    /// The picture and where its corners end.
    pub slice: NineSlice,
    /// Console pixels past the window on each side.
    pub outset: u16,
    /// The inside's colour.
    pub fill: [u8; 3],
}

/// A title's plate: its picture (two end caps and a body), the body's
/// console pixels each side of the title, and the picture pixels from its
/// top to the title's row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Plate {
    /// The picture: its left and right borders are the caps.
    pub slice: NineSlice,
    /// Body each side of the title, in console pixels.
    pub pad: u16,
    /// Picture pixels from the top to the title's row.
    pub text_top: u16,
}

/// The frames and plates whose pictures this build has, by name.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UiFrames {
    /// The frames, by name.
    pub frames: BTreeMap<String, Frame>,
    /// The plates, by name.
    pub plates: BTreeMap<String, Plate>,
}

impl UiFrames {
    /// The frame called `name`, if its picture is in this build.
    pub fn frame(&self, name: &str) -> Option<Frame> {
        self.frames.get(name).copied()
    }

    /// The plate called `name`, if its picture is in this build.
    pub fn plate(&self, name: &str) -> Option<Plate> {
        self.plates.get(name).copied()
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FramesFile {
    frames: BTreeMap<String, FrameEntry>,
    plates: BTreeMap<String, PlateEntry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BordersEntry {
    left: u32,
    top: u32,
    right: u32,
    bottom: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FrameEntry {
    picture: String,
    borders: BordersEntry,
    fill: String,
    #[serde(default)]
    outset: u16,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CapsEntry {
    left: u32,
    right: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PlateEntry {
    picture: String,
    caps: CapsEntry,
    #[serde(default)]
    pad: u16,
    #[serde(default)]
    text_top: u16,
}

/// Loads the embedded frames file, checking it against `images`.
pub fn load(images: &ImageTable) -> Result<UiFrames, Vec<ContentError>> {
    let file = bundle::display_path(FRAMES_PATH);
    let source = bundle::file(FRAMES_PATH)
        .ok_or_else(|| vec![ContentError::new(&file, "file not found in asset bundle")])?;
    from_source(&file, &source, images)
}

/// The picture at `ui/<relative>` in `images`, whole, or `None` if this
/// build lacks it.
fn picture(relative: &str, images: &ImageTable) -> Option<(Picture, (u32, u32))> {
    let image = images.id(&format!("{UI_DIR}/{relative}"))?;
    let info = images.info(image)?;
    let (w, h) = (info.width, info.height);
    let rect = ImageRect { x: 0, y: 0, w, h };
    Some((Picture { image, rect }, (w, h)))
}

/// Parses frames `source` (errors attributed to `file`), checking it
/// against `images`: every name in [`FRAME_NAMES`] and [`PLATE_NAMES`] is
/// there and no other, each fill is a colour, and each picture the build
/// has leaves room between its corners (its borders and its caps fit).
/// A picture the build lacks leaves its frame out. Reports every problem.
pub fn from_source(
    file: &str,
    source: &str,
    images: &ImageTable,
) -> Result<UiFrames, Vec<ContentError>> {
    let raw: FramesFile = parse_ron(file, source).map_err(|e| vec![e])?;
    let mut errors = Vec::new();
    let mut error = |what: String| errors.push(ContentError::new(file, what));
    let names =
        |kind: &str, wanted: &[&str], given: Vec<&String>, error: &mut dyn FnMut(String)| {
            for name in wanted {
                if !given.iter().any(|g| g == name) {
                    error(format!("no {kind} \"{name}\": the windows ask for it"));
                }
            }
            for name in given {
                if !wanted.contains(&name.as_str()) {
                    error(format!("{kind} \"{name}\": no window asks for it"));
                }
            }
        };
    names(
        "frame",
        &FRAME_NAMES,
        raw.frames.keys().collect(),
        &mut error,
    );
    names(
        "plate",
        &PLATE_NAMES,
        raw.plates.keys().collect(),
        &mut error,
    );
    let mut out = UiFrames::default();
    for (name, f) in &raw.frames {
        let inside = parse_hex(&f.fill).map_err(|e| error(format!("frame \"{name}\": fill {e}")));
        let b = Borders {
            left: f.borders.left,
            top: f.borders.top,
            right: f.borders.right,
            bottom: f.borders.bottom,
        };
        let Some((pic, (w, h))) = picture(&f.picture, images) else {
            continue;
        };
        if b.left + b.right >= w || b.top + b.bottom >= h {
            error(format!(
                "frame \"{name}\": borders {}, {}, {}, {} leave no middle in {} ({w}×{h} px)",
                b.left, b.top, b.right, b.bottom, f.picture
            ));
            continue;
        }
        if let Ok(fill) = inside {
            let slice = NineSlice::new(pic, b);
            let frame = Frame {
                slice,
                outset: f.outset,
                fill,
            };
            out.frames.insert(name.clone(), frame);
        }
    }
    for (name, p) in &raw.plates {
        let Some((pic, (w, _))) = picture(&p.picture, images) else {
            continue;
        };
        if p.caps.left + p.caps.right >= w {
            error(format!(
                "plate \"{name}\": caps {} and {} leave no body in {} ({w} px wide)",
                p.caps.left, p.caps.right, p.picture
            ));
            continue;
        }
        let caps = Borders {
            left: p.caps.left,
            top: 0,
            right: p.caps.right,
            bottom: 0,
        };
        let slice = NineSlice::new(pic, caps);
        let plate = Plate {
            slice,
            pad: p.pad,
            text_top: p.text_top,
        };
        out.plates.insert(name.clone(), plate);
    }
    if errors.is_empty() {
        Ok(out)
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests;
