//! Tilesets (ADR-0038, ADR-0049, ADR-0052): what a sprite map skin paints
//! the battle map with. `assets/tilesets/<id>.ron` names a tile for every
//! terrain in one image and the layers painted over them, for each look a
//! map may ask for (or no terrain at all: the glyph skin then paints it),
//! and a picture for units by character and by class, each from the
//! tileset's image or from an image file of its own (format in
//! `assets/tilesets/README.md`).
//!
//! Only looks hang off the game's ids here: nothing in a tileset changes a
//! rule.

mod look;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use trpg_core::lead::{PORTRAIT_FEMALE, PORTRAIT_MALE};
use trpg_core::{CharacterId, ClassId, ClassTable, TerrainId};

use crate::bundle;
use crate::character::CharacterTable;
use crate::error::ContentError;
use crate::image::{ImageId, ImageInfo, ImageTable};
use crate::ron_loader::parse_ron;
use crate::terrain::TerrainDisplayTable;

pub use look::{CornerLayer, Layer, Look, MIXES, TileLayer, mix, mix_name, parse_mix};

/// The directory of tileset files inside the bundle.
pub const TILESETS_DIR: &str = "tilesets";

/// The extension of a tileset file.
const TILESET_EXTENSION: &str = ".ron";

/// The smallest and largest side of a map tile or a unit picture, in
/// pixels.
pub const SIDE_PX: std::ops::RangeInclusive<u32> = 8..=64;

/// The ids the lead's picture may go by in a tileset's `characters`, by
/// gender: the lead's portrait ids (`trpg_core::lead`).
pub const LEAD_PICTURES: [&str; 2] = [PORTRAIT_MALE, PORTRAIT_FEMALE];

/// The frames across (walking: one foot forward, standing, the other foot
/// forward) and down (facing down, left, right, up) a **walking sheet**:
/// an image of a unit's own with a frame for each, as the bought map
/// sprites have.
pub const WALK_FRAMES: (u32, u32) = (3, 4);

/// A rectangle of an image, in image pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageRect {
    /// Left edge.
    pub x: u32,
    /// Top edge.
    pub y: u32,
    /// Width.
    pub w: u32,
    /// Height.
    pub h: u32,
}

/// A picture: a rectangle of an image. A unit's picture need not come from
/// its tileset's image, nor be as wide as it is tall.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Picture {
    /// The image.
    pub image: ImageId,
    /// Where in it.
    pub rect: ImageRect,
}

/// A tileset's terrain tiles.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerrainTiles {
    /// The image the tiles are in.
    pub image: ImageId,
    /// A map tile's size, across × down, in pixels: in the image, and on
    /// screen.
    pub tile_px: (u32, u32),
    /// The tileset's own look: what a map is painted as unless
    /// [`looks`](Self::looks) has the look it names.
    pub look: Look,
    /// Its other looks, by the name a map file gives
    /// ([`crate::map::TILE_LOOKS`]).
    pub looks: BTreeMap<String, Look>,
}

/// A validated tileset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tileset {
    /// Its id: the file's stem.
    pub id: String,
    /// Its terrain tiles; `None` = it has only unit pictures, and the
    /// terrain is painted as glyphs (ADR-0049).
    pub terrain: Option<TerrainTiles>,
    /// Pictures of named characters, by character id; the lead's by
    /// [`LEAD_PICTURES`] too.
    pub characters: BTreeMap<CharacterId, Picture>,
    /// Pictures of classes.
    pub classes: BTreeMap<ClassId, Picture>,
    /// The picture of any unit with neither.
    pub fallback: Picture,
    /// The images that are walking sheets ([`WALK_FRAMES`] frames of the
    /// picture's size from the image's top-left): a unit whose picture is
    /// in one turns and moves its legs. Any other picture only glides.
    pub walking: BTreeSet<ImageId>,
}

impl Tileset {
    /// The tile of terrain `id` in the tileset's own look, if it has one.
    pub fn tile(&self, id: TerrainId) -> Option<Picture> {
        let terrain = self.terrain.as_ref()?;
        let rect = terrain.look.tiles.get(&id).copied()?;
        Some(Picture {
            image: terrain.image,
            rect,
        })
    }

    /// The look a map naming `name` is painted as: that look if the
    /// tileset has it, else its own. `None` without terrain tiles.
    pub fn look(&self, name: &str) -> Option<&Look> {
        let terrain = self.terrain.as_ref()?;
        Some(terrain.looks.get(name).unwrap_or(&terrain.look))
    }

    /// The picture of a unit of `class`, the named `character` if any: the
    /// first of `character`'s ids that has one (a character may go by
    /// several: the lead by gender, then by its own id), else the class's,
    /// else the fallback.
    pub fn unit_picture<'a>(
        &self,
        character: impl IntoIterator<Item = &'a CharacterId>,
        class: &ClassId,
    ) -> Picture {
        character
            .into_iter()
            .find_map(|c| self.characters.get(c))
            .or_else(|| self.classes.get(class))
            .copied()
            .unwrap_or(self.fallback)
    }

    /// The frame of `picture`'s walking sheet in walking column `column`
    /// and facing row `row`, or `None` if its image isn't a walking sheet
    /// (or has no such frame).
    pub fn walk_frame(&self, picture: Picture, column: u32, row: u32) -> Option<Picture> {
        let known = column < WALK_FRAMES.0 && row < WALK_FRAMES.1;
        (known && self.walking.contains(&picture.image)).then(|| Picture {
            image: picture.image,
            rect: grid_rect((column, row), (picture.rect.w, picture.rect.h), (0, 0)),
        })
    }
}

/// A tileset file as written.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct TilesetFile {
    id: String,
    #[serde(default, deserialize_with = "written")]
    image: Option<String>,
    #[serde(default, deserialize_with = "written")]
    tile_px: Option<(u32, u32)>,
    #[serde(default, deserialize_with = "written")]
    terrain: Option<BTreeMap<String, (u32, u32)>>,
    #[serde(default)]
    layers: Vec<look::LayerFile>,
    #[serde(default)]
    looks: BTreeMap<String, look::LookFile>,
    unit_px: (u32, u32),
    units: UnitsFile,
    #[serde(default)]
    units_origin_px: (u32, u32),
}

/// A field that may be left out: `Some` of what is written.
fn written<'de, D: Deserializer<'de>, T: Deserialize<'de>>(d: D) -> Result<Option<T>, D::Error> {
    T::deserialize(d).map(Some)
}

/// A tileset file's `units` table.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct UnitsFile {
    characters: BTreeMap<String, UnitEntry>,
    classes: BTreeMap<String, UnitEntry>,
    fallback: UnitEntry,
}

/// A unit picture as written.
#[derive(Debug, Clone, PartialEq, Eq)]
enum UnitEntry {
    /// `(column, row)` in the tileset's own image, from `units_origin_px`.
    Grid((u32, u32)),
    /// `(image: "…", frame: (column, row))`: a frame of an image file of
    /// its own, counted from its top-left; with `walk: true`, the image is
    /// a walking sheet.
    File(FrameFile),
}

/// A frame of an image file.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct FrameFile {
    image: String,
    frame: (u32, u32),
    #[serde(default)]
    walk: bool,
}

impl<'de> Deserialize<'de> for UnitEntry {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Entry;

        impl<'de> Visitor<'de> for Entry {
            type Value = UnitEntry;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("(column, row) or (image: \"…\", frame: (column, row))")
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<UnitEntry, A::Error> {
                let mut next = |i| {
                    seq.next_element::<u32>()?
                        .ok_or_else(|| de::Error::invalid_length(i, &self))
                };
                let at = (next(0)?, next(1)?);
                if seq.next_element::<de::IgnoredAny>()?.is_some() {
                    return Err(de::Error::invalid_length(3, &self));
                }
                Ok(UnitEntry::Grid(at))
            }

            fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<UnitEntry, A::Error> {
                FrameFile::deserialize(de::value::MapAccessDeserializer::new(map))
                    .map(UnitEntry::File)
            }
        }

        d.deserialize_any(Entry)
    }
}

/// What a tileset is checked against.
#[derive(Debug, Clone, Copy)]
pub struct TilesetRefs<'a> {
    /// The bundle's images.
    pub images: &'a ImageTable,
    /// Every terrain.
    pub terrain: &'a TerrainDisplayTable,
    /// Every class.
    pub classes: &'a ClassTable,
    /// Every named character.
    pub characters: &'a CharacterTable,
}

/// Loads every tileset in the bundle by id, reporting every problem in
/// every file.
pub fn load_all(refs: &TilesetRefs) -> Result<BTreeMap<String, Tileset>, Vec<ContentError>> {
    let mut tilesets = BTreeMap::new();
    let mut errors = Vec::new();
    for path in bundle::files_in(TILESETS_DIR) {
        let Some(stem) = path
            .strip_prefix(TILESETS_DIR)
            .and_then(|p| p.strip_prefix('/'))
            .and_then(|p| p.strip_suffix(TILESET_EXTENSION))
        else {
            continue;
        };
        let file = bundle::display_path(path);
        let result = bundle::file(path)
            .ok_or_else(|| vec![ContentError::new(&file, "file is not valid UTF-8")])
            .and_then(|source| parse_tileset(&file, stem, source, refs));
        match result {
            Ok(tileset) => {
                tilesets.insert(stem.to_owned(), tileset);
            }
            Err(e) => errors.extend(e),
        }
    }
    if errors.is_empty() {
        Ok(tilesets)
    } else {
        Err(errors)
    }
}

/// A tileset file being checked: its own image (if it names one that
/// exists) and the problems found so far.
struct Checker<'a> {
    def: &'a TilesetFile,
    images: &'a ImageTable,
    sheet: Option<ImageId>,
    walking: BTreeSet<ImageId>,
    problems: Vec<String>,
}

impl Checker<'_> {
    /// Reports `rect` of `image`, named `what`, if it doesn't lie inside
    /// the image.
    fn inside(&mut self, what: &str, image: ImageId, rect: ImageRect) {
        if let Some(info) = self.images.info(image)
            && !inside(rect, info)
        {
            let ImageRect { x, y, w, h } = rect;
            let (iw, ih) = (info.width, info.height);
            self.problems.push(format!(
                "{what}: its {w}×{h} px at ({x}, {y}) lies outside the {iw}×{ih} px image"
            ));
        }
    }

    /// Notes that `image` is a walking sheet, reporting it (named `what`)
    /// if it is too small for [`WALK_FRAMES`] frames of `unit_px`.
    fn walk_sheet(&mut self, what: &str, image: ImageId) {
        let (w, h) = self.def.unit_px;
        let (across, down) = WALK_FRAMES;
        let (sw, sh) = (w.saturating_mul(across), h.saturating_mul(down));
        let frames = ImageRect {
            x: 0,
            y: 0,
            w: sw,
            h: sh,
        };
        match self.images.info(image) {
            Some(info) if !inside(frames, info) => {
                let (iw, ih) = (info.width, info.height);
                self.problems.push(format!(
                    "{what}: `walk` needs {across} × {down} frames of {w}×{h} px ({sw}×{sh} px), \
                     and the image is {iw}×{ih} px"
                ));
            }
            _ => {
                self.walking.insert(image);
            }
        }
    }

    /// The picture `entry` names, checked; `what` names it in problems.
    /// `None` if its image is missing.
    fn picture(&mut self, what: &str, entry: &UnitEntry) -> Option<Picture> {
        let (image, rect) = match entry {
            UnitEntry::Grid(at) => {
                if self.def.image.is_none() {
                    self.problems.push(format!(
                        "{what}: a (column, row) picture needs the tileset's `image`, and it has none"
                    ));
                }
                // An image that is named but missing is reported once.
                let rect = grid_rect(*at, self.def.unit_px, self.def.units_origin_px);
                (self.sheet?, rect)
            }
            UnitEntry::File(FrameFile { image, frame, walk }) => {
                let Some(id) = self.images.id(image) else {
                    self.problems.push(format!(
                        "{what}: image \"{image}\" is not a PNG in the asset bundle"
                    ));
                    return None;
                };
                if *walk {
                    self.walk_sheet(what, id);
                }
                (id, grid_rect(*frame, self.def.unit_px, (0, 0)))
            }
        };
        self.inside(what, image, rect);
        Some(Picture { image, rect })
    }
}

/// Parses tileset `source` (errors attributed to `file`, whose stem is
/// `stem`) and checks it against `refs`, reporting every problem.
pub fn parse_tileset(
    file: &str,
    stem: &str,
    source: &str,
    refs: &TilesetRefs,
) -> Result<Tileset, Vec<ContentError>> {
    let def: TilesetFile = parse_ron(file, source).map_err(|e| vec![e])?;
    let mut check = Checker {
        def: &def,
        images: refs.images,
        sheet: def.image.as_deref().and_then(|path| refs.images.id(path)),
        walking: BTreeSet::new(),
        problems: own_problems(&def, stem),
    };
    if let (Some(path), None) = (&def.image, check.sheet) {
        let problem = format!("image \"{path}\" is not a PNG in the asset bundle");
        check.problems.push(problem);
    }
    let mut unknown = Vec::new();
    let tile_px = def.tile_px.unwrap_or_default();
    let mut looking = |name, terrain, layers| {
        let mut looking = look::Looking {
            check: &mut check,
            unknown: &mut unknown,
            refs,
            tile_px,
            name,
        };
        looking.look(terrain, layers)
    };
    let own = def.terrain.as_ref().map(|t| looking(None, t, &def.layers));
    let mut looks = BTreeMap::new();
    for (name, file) in &def.looks {
        let look = looking(Some(name.as_str()), &file.terrain, &file.layers);
        looks.insert(name.clone(), look);
    }
    let mut classes = BTreeMap::new();
    for (name, entry) in &def.units.classes {
        let id = ClassId(name.clone());
        if refs.classes.get(&id).is_none() {
            unknown.push(format!("class \"{name}\" is not in classes.ron"));
        }
        if let Some(picture) = check.picture(&format!("class \"{name}\""), entry) {
            classes.insert(id, picture);
        }
    }
    let mut characters = BTreeMap::new();
    for (name, entry) in &def.units.characters {
        let id = CharacterId(name.clone());
        let known = refs.characters.characters.contains_key(&id);
        if !known && !LEAD_PICTURES.contains(&name.as_str()) {
            unknown.push(format!("character \"{name}\" is not in characters.ron"));
        }
        if let Some(picture) = check.picture(&format!("character \"{name}\""), entry) {
            characters.insert(id, picture);
        }
    }
    let fallback = check.picture("fallback", &def.units.fallback);
    let sheet = check.sheet;
    let walking = check.walking;
    let mut problems = check.problems;
    problems.extend(unknown);
    if let Some(terrain) = &def.terrain {
        problems.extend(look::missing(None, terrain, refs));
    }
    for (name, file) in &def.looks {
        problems.extend(look::missing(Some(name), &file.terrain, refs));
    }
    let terrain = own.zip(sheet).map(|(look, image)| TerrainTiles {
        image,
        tile_px,
        look,
        looks,
    });
    match fallback {
        Some(fallback) if problems.is_empty() => Ok(Tileset {
            id: def.id,
            terrain,
            characters,
            classes,
            fallback,
            walking,
        }),
        _ => Err(problems
            .into_iter()
            .map(|m| ContentError::new(file, m))
            .collect()),
    }
}

/// The problems of a tileset file `stem` on its own: its id, its sizes,
/// and that terrain tiles come with their size and their image.
fn own_problems(def: &TilesetFile, stem: &str) -> Vec<String> {
    let mut problems = Vec::new();
    if def.id != stem {
        problems.push(format!(
            "id \"{}\" must be the file's name, \"{stem}\"",
            def.id
        ));
    }
    let sizes = [("tile_px", def.tile_px), ("unit_px", Some(def.unit_px))];
    for (name, size) in sizes {
        if let Some((w, h)) = size
            && (!SIDE_PX.contains(&w) || !SIDE_PX.contains(&h))
        {
            let (lo, hi) = (SIDE_PX.start(), SIDE_PX.end());
            problems.push(format!(
                "{name} is ({w}, {h}); each side must be {lo} to {hi} px"
            ));
        }
    }
    match (&def.terrain, def.tile_px) {
        (Some(_), None) => problems.push("terrain needs `tile_px`, the size of a tile".to_owned()),
        (None, Some(_)) => problems.push(
            "tile_px without `terrain`: with no terrain tiles the map keeps the glyph skin's \
             tiles; leave `tile_px` out"
                .to_owned(),
        ),
        _ => {}
    }
    if def.terrain.is_some() && def.image.is_none() {
        problems.push("terrain needs `image`, the image its tiles are in".to_owned());
    }
    if def.terrain.is_none() {
        if !def.layers.is_empty() {
            problems.push("layers need `terrain`: they are painted over its tiles".to_owned());
        }
        if !def.looks.is_empty() {
            problems.push("looks need `terrain`, the tileset's own look".to_owned());
        }
    }
    problems.extend(def.looks.keys().filter_map(|name| look::name_problem(name)));
    problems
}

/// The rectangle of cell `(column, row)` of a grid of `size` cells that
/// starts at pixel `origin`. Saturates rather than overflow (the result
/// then lies outside any image).
fn grid_rect((column, row): (u32, u32), (w, h): (u32, u32), (ox, oy): (u32, u32)) -> ImageRect {
    ImageRect {
        x: ox.saturating_add(column.saturating_mul(w)),
        y: oy.saturating_add(row.saturating_mul(h)),
        w,
        h,
    }
}

/// Whether `rect` lies inside an image of size `info`.
fn inside(rect: ImageRect, info: ImageInfo) -> bool {
    let right = u64::from(rect.x) + u64::from(rect.w);
    let bottom = u64::from(rect.y) + u64::from(rect.h);
    right <= u64::from(info.width) && bottom <= u64::from(info.height)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::fmt::Write as _;

    use super::*;
    use crate::image::png_header;

    /// The game's terrain, classes and characters, one 64 × 64 image,
    /// `tilesets/t.png`, and a 48 × 80 sheet of one unit, `units/a.png`.
    struct Fixture {
        content: crate::Content,
        images: ImageTable,
    }

    impl Fixture {
        fn new() -> Self {
            let (header, sheet) = (png_header(64, 64), png_header(48, 80));
            let files = [
                ("tilesets/t.png", header.as_slice()),
                ("units/a.png", sheet.as_slice()),
            ];
            Self {
                content: crate::load_embedded().unwrap(),
                images: ImageTable::from_files(files).unwrap(),
            }
        }

        fn refs(&self) -> TilesetRefs<'_> {
            TilesetRefs {
                images: &self.images,
                terrain: &self.content.terrain.display,
                classes: &self.content.classes,
                characters: &self.content.characters,
            }
        }

        /// Every terrain on tile (0, 0) of a 16 px grid, then `extra`.
        fn terrain(&self, extra: &str) -> String {
            let mut out = String::new();
            for t in &self.content.terrain.display.terrains {
                let _ = write!(out, "\"{}\": (0, 0), ", t.id);
            }
            out.push_str(extra);
            out
        }

        fn parse(&self, source: &str) -> Result<Tileset, Vec<ContentError>> {
            parse_tileset("assets/tilesets/t.ron", "t", source, &self.refs())
        }

        /// The messages of the errors `source` gives.
        fn errors(&self, source: &str) -> Vec<String> {
            let errors = self.parse(source).unwrap_err();
            for e in &errors {
                assert_eq!(e.file, "assets/tilesets/t.ron");
            }
            errors.into_iter().map(|e| e.message).collect()
        }
    }

    /// A tileset file with `fields` in place of the defaults' (a field
    /// written in `fields` is left out of the defaults; one given as `""`
    /// is left out of the file).
    fn file(f: &Fixture, fields: &[(&str, &str)]) -> String {
        let terrain = format!("{{ {} }}", f.terrain(""));
        let defaults = [
            ("id", "\"t\"".to_owned()),
            ("image", "\"tilesets/t.png\"".to_owned()),
            ("tile_px", "(16, 16)".to_owned()),
            ("terrain", terrain),
            ("layers", String::new()),
            ("looks", String::new()),
            ("unit_px", "(16, 24)".to_owned()),
            (
                "units",
                "(characters: { \"test_lord\": (1, 0) }, classes: { \"brigand\": (0, 0) }, \
                 fallback: (2, 0))"
                    .to_owned(),
            ),
            ("units_origin_px", "(0, 16)".to_owned()),
        ];
        let body: Vec<String> = defaults
            .iter()
            .map(|(name, value)| {
                let value = fields
                    .iter()
                    .find(|(n, _)| n == name)
                    .map_or(value.as_str(), |(_, v)| v);
                (name, value)
            })
            .filter(|(_, value)| !value.is_empty())
            .map(|(name, value)| format!("{name}: {value}"))
            .collect();
        format!("({})", body.join(",\n"))
    }

    #[test]
    fn a_tileset_resolves_its_ids_and_rectangles() {
        let f = Fixture::new();
        let t = f.parse(&file(&f, &[])).unwrap();
        let image = f.images.id("tilesets/t.png").unwrap();
        let display = &f.content.terrain.display;
        let terrain = t.terrain.as_ref().unwrap();
        assert_eq!(t.id, "t");
        assert_eq!((terrain.image, terrain.tile_px), (image, (16, 16)));
        assert_eq!(terrain.look.tiles.len(), display.terrains.len());
        assert!(terrain.look.layers.is_empty() && terrain.looks.is_empty());
        let plain = display.id_of("plain").unwrap();
        let tile = ImageRect {
            x: 0,
            y: 0,
            w: 16,
            h: 16,
        };
        assert_eq!(t.tile(plain), Some(Picture { image, rect: tile }));
        assert_eq!(t.tile(TerrainId(999)), None);
        // Unit pictures: 16 × 24 cells from (0, 16).
        let at = |x| Picture {
            image,
            rect: ImageRect {
                x,
                y: 16,
                w: 16,
                h: 24,
            },
        };
        let lord = CharacterId("test_lord".into());
        let brigand = ClassId("brigand".into());
        let mage = ClassId("mage".into());
        assert_eq!(t.unit_picture(Some(&lord), &brigand), at(16));
        assert_eq!(t.unit_picture(None, &brigand), at(0));
        let nobody = CharacterId("nobody".into());
        assert_eq!(t.unit_picture(Some(&nobody), &brigand), at(0));
        assert_eq!(t.unit_picture(None, &mage), at(32));
        assert_eq!(t.fallback, at(32));
    }

    #[test]
    fn grids_start_at_their_origin_and_saturate() {
        let r = grid_rect((2, 3), (24, 20), (5, 96));
        assert_eq!(
            r,
            ImageRect {
                x: 53,
                y: 156,
                w: 24,
                h: 20
            }
        );
        let huge = grid_rect((u32::MAX, 1), (24, 24), (1, 0));
        assert_eq!((huge.x, huge.y), (u32::MAX, 24));
        let info = |width, height| ImageInfo { width, height };
        let rect = |x, y, w, h| ImageRect { x, y, w, h };
        assert!(inside(rect(24, 24, 24, 24), info(48, 48)));
        assert!(!inside(rect(25, 24, 24, 24), info(48, 48)));
        assert!(!inside(rect(24, 25, 24, 24), info(48, 48)));
        assert!(!inside(huge, info(48, 48)));
    }

    #[test]
    fn a_bad_file_is_one_error_with_its_position() {
        let f = Fixture::new();
        let errors = f.parse("(id: \"t\", bogus: 1)").unwrap_err();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].file, "assets/tilesets/t.ron");
        assert!(errors[0].line.is_some());
    }

    #[test]
    fn the_id_is_the_file_name() {
        let f = Fixture::new();
        assert_eq!(
            f.errors(&file(&f, &[("id", "\"other\"")])),
            ["id \"other\" must be the file's name, \"t\""]
        );
    }

    #[test]
    fn tile_and_unit_sides_are_8_to_64_px() {
        let f = Fixture::new();
        for ok in ["(8, 64)", "(64, 8)"] {
            let source = file(&f, &[("tile_px", ok), ("units_origin_px", "(0, 0)")]);
            assert!(f.parse(&source).is_ok(), "{ok}");
        }
        assert_eq!(
            f.errors(&file(&f, &[("tile_px", "(7, 16)")])),
            ["tile_px is (7, 16); each side must be 8 to 64 px"]
        );
        assert_eq!(
            f.errors(&file(&f, &[("unit_px", "(16, 7)")])),
            ["unit_px is (16, 7); each side must be 8 to 64 px"]
        );
    }

    #[test]
    fn the_image_must_be_in_the_bundle() {
        let f = Fixture::new();
        // Its rectangles can't be checked, so they aren't.
        assert_eq!(
            f.errors(&file(&f, &[("image", "\"tilesets/none.png\"")])),
            ["image \"tilesets/none.png\" is not a PNG in the asset bundle"]
        );
    }

    #[test]
    fn every_rectangle_lies_inside_the_image() {
        let f = Fixture::new();
        let terrain = f
            .terrain("")
            .replace("\"plain\": (0, 0)", "\"plain\": (4, 0)");
        let terrain = format!("{{ {terrain} }}");
        let units = "(characters: { \"test_lord\": (0, 2) }, classes: { \"brigand\": (4, 0) }, \
                     fallback: (3, 2))";
        assert_eq!(
            f.errors(&file(&f, &[("terrain", &terrain), ("units", units)])),
            [
                "terrain \"plain\": its 16×16 px at (64, 0) lies outside the 64×64 px image",
                "class \"brigand\": its 16×24 px at (64, 16) lies outside the 64×64 px image",
                "character \"test_lord\": its 16×24 px at (0, 64) lies outside the 64×64 px image",
                "fallback: its 16×24 px at (48, 64) lies outside the 64×64 px image",
            ]
        );
    }

    #[test]
    fn every_terrain_has_a_tile_and_every_name_exists() {
        let f = Fixture::new();
        let terrain = format!(
            "{{ {} }}",
            f.terrain("\"lava\": (0, 0)")
                .replace("\"forest\": (0, 0), ", "")
        );
        let units = "(characters: { \"nobody\": (0, 0) }, classes: { \"wizard\": (0, 0) }, \
                     fallback: (0, 0))";
        assert_eq!(
            f.errors(&file(&f, &[("terrain", &terrain), ("units", units)])),
            [
                "terrain \"lava\" is not in terrain.ron",
                "class \"wizard\" is not in classes.ron",
                "character \"nobody\" is not in characters.ron",
                "terrain \"forest\" has no tile",
            ]
        );
    }

    #[test]
    fn the_embedded_test_tileset_loads() {
        let f = Fixture::new();
        let tilesets = &f.content.tilesets;
        let ids: Vec<_> = tilesets.keys().collect();
        assert_eq!(ids, ["test", "test_auto", "test_units"]);
        let test = &tilesets["test"];
        let terrain = test.terrain.as_ref().unwrap();
        assert_eq!(terrain.tile_px, (24, 24));
        assert_eq!(terrain.image.path(), "tilesets/test.png");
        // Every class has a picture of its own.
        assert_eq!(test.classes.len(), f.content.classes.classes.len());
        assert!(test.characters.is_empty());
        // The fixture of unit sheets: no terrain, a 48 × 80 sheet per class
        // of the Quick Battle, each unit its standing frame.
        let units = &tilesets["test_units"];
        assert_eq!(units.terrain, None);
        let guard = units.classes[&ClassId("guard".into())];
        assert_eq!(guard.image.path(), "tilesets/test_units/guard.png");
        let standing = ImageRect {
            x: 16,
            y: 0,
            w: 16,
            h: 20,
        };
        assert_eq!(guard.rect, standing);
        assert_eq!(units.fallback.rect, standing);
        // Every sheet is a walking sheet; the test tileset's one image of
        // single pictures isn't.
        assert_eq!(units.walking.len(), units.classes.len() + 1);
        assert!(units.walking.contains(&guard.image));
        assert!(test.walking.is_empty());
        assert_eq!(
            f.content.images.info(guard.image),
            Some(ImageInfo {
                width: 48,
                height: 80
            })
        );
        // The fixture with layers and looks: 16 × 16 tiles, a shore and
        // the woods' edge between tiles, a fort and a bridge on theirs,
        // an indoor look with the floor's edge, and the unit sheets.
        let auto = &tilesets["test_auto"];
        let terrain = auto.terrain.as_ref().unwrap();
        assert_eq!(terrain.tile_px, (16, 16));
        assert_eq!(terrain.image.path(), "tilesets/test_auto.png");
        let kinds = |look: &Look| -> Vec<&str> {
            let kind = |layer: &Layer| match layer {
                Layer::Corners(_) => "corners",
                Layer::Tiles(_) => "tiles",
            };
            look.layers.iter().map(kind).collect()
        };
        assert_eq!(
            kinds(&terrain.look),
            ["corners", "corners", "tiles", "tiles"]
        );
        let Layer::Corners(shore) = &terrain.look.layers[0] else {
            panic!("{:?}", terrain.look.layers[0]);
        };
        // Every mix but none and all.
        let pictures = shore.tiles.iter().filter(|t| t.is_some()).count();
        assert_eq!(
            (pictures, shore.tiles[0], shore.tiles[15]),
            (14, None, None)
        );
        assert_eq!(terrain.looks.keys().collect::<Vec<_>>(), ["indoor"]);
        assert_eq!(kinds(&terrain.looks["indoor"]), ["corners"]);
        assert_eq!(auto.look("indoor"), terrain.looks.get("indoor"));
        assert_eq!(auto.classes[&ClassId("guard".into())], guard);
    }

    /// A tileset of unit sheets only: no image, no terrain, no tile size.
    fn sheets(units: &str) -> Vec<(&'static str, String)> {
        vec![
            ("image", String::new()),
            ("tile_px", String::new()),
            ("terrain", String::new()),
            ("units_origin_px", String::new()),
            ("unit_px", "(16, 20)".to_owned()),
            ("units", units.to_owned()),
        ]
    }

    fn sheets_file(f: &Fixture, units: &str) -> String {
        let fields = sheets(units);
        let fields: Vec<(&str, &str)> = fields.iter().map(|(n, v)| (*n, v.as_str())).collect();
        file(f, &fields)
    }

    #[test]
    fn a_unit_picture_may_be_a_frame_of_its_own_image_file() {
        let f = Fixture::new();
        let units = "(characters: { \"lead_f\": (image: \"units/a.png\", frame: (0, 3)) }, \
                     classes: { \"brigand\": (image: \"units/a.png\", frame: (1, 0)) }, \
                     fallback: (image: \"units/a.png\", frame: (2, 1)))";
        let t = f.parse(&sheets_file(&f, units)).unwrap();
        assert_eq!(t.terrain, None);
        assert_eq!(t.tile(TerrainId(0)), None);
        let image = f.images.id("units/a.png").unwrap();
        let at = |x, y| Picture {
            image,
            rect: ImageRect { x, y, w: 16, h: 20 },
        };
        // The standing, front-facing frame of a 48 × 80 sheet: (1, 0).
        let brigand = ClassId("brigand".into());
        assert_eq!(t.unit_picture(None, &brigand), at(16, 0));
        assert_eq!(t.fallback, at(32, 20));
        // The lead goes by gender, then by its own id.
        let ids = [CharacterId("lead_f".into()), CharacterId("lead".into())];
        assert_eq!(t.unit_picture(&ids, &brigand), at(0, 60));
        let ids = [CharacterId("lead_m".into()), CharacterId("lead".into())];
        assert_eq!(t.unit_picture(&ids, &brigand), at(16, 0));
        // Both kinds in one file: the grid's from `units_origin_px`, a
        // file's from its own top-left.
        let units = "(characters: {}, classes: { \"brigand\": (1, 1) }, \
                     fallback: (image: \"units/a.png\", frame: (1, 0)))";
        let t = f.parse(&file(&f, &[("units", units)])).unwrap();
        let sheet = f.images.id("tilesets/t.png").unwrap();
        let rect = ImageRect {
            x: 16,
            y: 40,
            w: 16,
            h: 24,
        };
        assert_eq!(t.classes[&brigand], Picture { image: sheet, rect });
        assert_eq!(t.fallback.rect.h, 24);
    }

    #[test]
    fn a_walking_sheet_has_a_frame_for_each_facing_and_step() {
        let f = Fixture::new();
        let units = "(characters: {}, \
                     classes: { \"brigand\": (image: \"units/a.png\", frame: (1, 0), walk: true) }, \
                     fallback: (image: \"tilesets/t.png\", frame: (0, 0)))";
        let t = f.parse(&sheets_file(&f, units)).unwrap();
        let sheet = f.images.id("units/a.png").unwrap();
        assert_eq!(t.walking.iter().collect::<Vec<_>>(), [&sheet]);
        let brigand = t.classes[&ClassId("brigand".into())];
        let at = |x, y| {
            Some(Picture {
                image: sheet,
                rect: ImageRect { x, y, w: 16, h: 20 },
            })
        };
        // Columns: the walking frames; rows: facing down, left, right, up.
        assert_eq!(t.walk_frame(brigand, 0, 0), at(0, 0));
        assert_eq!(t.walk_frame(brigand, 1, 0), at(16, 0));
        assert_eq!(t.walk_frame(brigand, 2, 1), at(32, 20));
        assert_eq!(t.walk_frame(brigand, 0, 2), at(0, 40));
        assert_eq!(t.walk_frame(brigand, 2, 3), at(32, 60));
        // No such frame, or a picture that isn't in a walking sheet.
        assert_eq!(t.walk_frame(brigand, 3, 0), None);
        assert_eq!(t.walk_frame(brigand, 0, 4), None);
        assert_eq!(t.walk_frame(t.fallback, 1, 0), None);
        assert_eq!(WALK_FRAMES, (3, 4));
        // Left out, or false: a single picture.
        let units = "(characters: {}, \
                     classes: { \"brigand\": (image: \"units/a.png\", frame: (1, 0), walk: false) }, \
                     fallback: (image: \"units/a.png\", frame: (1, 0)))";
        assert!(f.parse(&sheets_file(&f, units)).unwrap().walking.is_empty());
        // An image too small for the frames: 64 × 64 px, 80 px needed down.
        let units = "(characters: {}, classes: {}, \
                     fallback: (image: \"tilesets/t.png\", frame: (0, 0), walk: true))";
        assert_eq!(
            f.errors(&sheets_file(&f, units)),
            [
                "fallback: `walk` needs 3 × 4 frames of 16×20 px (48×80 px), and the image is \
                 64×64 px"
            ]
        );
    }

    #[test]
    fn a_unit_image_must_exist_and_hold_its_frame() {
        let f = Fixture::new();
        let units = "(characters: { \"test_lord\": (image: \"units/a.png\", frame: (3, 0)) }, \
                     classes: { \"brigand\": (image: \"units/none.png\", frame: (1, 0)), \
                     \"wizard\": (image: \"units/a.png\", frame: (0, 4)) }, \
                     fallback: (image: \"units/missing.png\", frame: (0, 0)))";
        assert_eq!(
            f.errors(&sheets_file(&f, units)),
            [
                "class \"brigand\": image \"units/none.png\" is not a PNG in the asset bundle",
                "class \"wizard\": its 16×20 px at (0, 80) lies outside the 48×80 px image",
                "character \"test_lord\": its 16×20 px at (48, 0) lies outside the 48×80 px image",
                "fallback: image \"units/missing.png\" is not a PNG in the asset bundle",
                "class \"wizard\" is not in classes.ron",
            ]
        );
        // A character that is neither in characters.ron nor the lead's.
        let units = "(characters: { \"lead_x\": (image: \"units/a.png\", frame: (0, 0)) }, \
                     classes: {}, fallback: (image: \"units/a.png\", frame: (0, 0)))";
        assert_eq!(
            f.errors(&sheets_file(&f, units)),
            ["character \"lead_x\" is not in characters.ron"]
        );
    }

    #[test]
    fn terrain_its_size_and_the_image_come_together() {
        let f = Fixture::new();
        let frame = "(image: \"units/a.png\", frame: (1, 0))";
        let all = format!("(characters: {{}}, classes: {{}}, fallback: {frame})");
        // A grid picture with no image to be in.
        let grid = "(characters: {}, classes: { \"brigand\": (0, 0) }, fallback: (1, 0))";
        assert_eq!(
            f.errors(&sheets_file(&f, grid)),
            [
                "class \"brigand\": a (column, row) picture needs the tileset's `image`, and it has none",
                "fallback: a (column, row) picture needs the tileset's `image`, and it has none",
            ]
        );
        // The image alone is fine: grid pictures, glyph terrain.
        let source = file(&f, &[("tile_px", ""), ("terrain", "")]);
        assert_eq!(f.parse(&source).unwrap().terrain, None);
        assert_eq!(
            f.errors(&file(&f, &[("terrain", ""), ("units", &all)])),
            [
                "tile_px without `terrain`: with no terrain tiles the map keeps the glyph skin's \
                 tiles; leave `tile_px` out"
            ]
        );
        assert_eq!(
            f.errors(&file(&f, &[("tile_px", ""), ("units", &all)])),
            ["terrain needs `tile_px`, the size of a tile"]
        );
        assert_eq!(
            f.errors(&file(&f, &[("image", ""), ("units", &all)])),
            ["terrain needs `image`, the image its tiles are in"]
        );
    }

    /// Layers of every kind: a shore between tiles, and a bridge that
    /// turns to cross the water.
    const LAYERS: &str = r#####"[
        Corners(of: ["water", "sea"], tiles: [
            (corners: "#...", at: (1, 0)),
            (corners: ".###", at: (2, 0)),
            (corners: "####", at: (3, 0)),
        ]),
        Tiles(of: ["fort"], at: (0, 1)),
        Tiles(of: ["bridge"], at: (1, 1), beside: ["water", "sea"],
              sides: [(sides: ".#.#", at: (2, 1)), (sides: "####", at: (3, 1))]),
    ]"#####;

    #[test]
    fn a_tileset_reads_its_layers_and_its_other_looks() {
        let f = Fixture::new();
        let indoor = format!(
            r#"{{ "indoor": (terrain: {{ {} }}, layers: [Tiles(of: ["door"], at: (3, 3))]) }}"#,
            f.terrain("").replace("(0, 0)", "(1, 2)")
        );
        let source = file(&f, &[("layers", LAYERS), ("looks", &indoor)]);
        let t = f.parse(&source).unwrap();
        let id = |name| f.content.terrain.display.id_of(name).unwrap();
        let at = |column: u32, row: u32| ImageRect {
            x: 16 * column,
            y: 16 * row,
            w: 16,
            h: 16,
        };
        let terrain = t.terrain.as_ref().unwrap();
        let own = &terrain.look;
        assert_eq!(own.tiles[&id("plain")], at(0, 0));
        assert_eq!(own.layers.len(), 3);
        let Layer::Corners(shore) = &own.layers[0] else {
            panic!("{:?}", own.layers[0]);
        };
        assert_eq!(shore.of, BTreeSet::from([id("water"), id("sea")]));
        let mut tiles = [None; MIXES];
        tiles[8] = Some(at(1, 0));
        tiles[7] = Some(at(2, 0));
        tiles[15] = Some(at(3, 0));
        assert_eq!(shore.tiles, tiles);
        let Layer::Tiles(fort) = &own.layers[1] else {
            panic!("{:?}", own.layers[1]);
        };
        assert_eq!(fort.of, BTreeSet::from([id("fort")]));
        assert_eq!(fort.tile, at(0, 1));
        assert!(fort.beside.is_empty());
        assert_eq!(fort.sides, [None; MIXES]);
        let Layer::Tiles(bridge) = &own.layers[2] else {
            panic!("{:?}", own.layers[2]);
        };
        assert_eq!(bridge.beside, shore.of);
        assert_eq!(bridge.picture(5), at(2, 1));
        assert_eq!(bridge.picture(15), at(3, 1));
        assert_eq!(bridge.picture(10), at(1, 1));
        // The other look: its own tiles and layers.
        assert_eq!(terrain.looks.keys().collect::<Vec<_>>(), ["indoor"]);
        let indoor = &terrain.looks["indoor"];
        assert_eq!(indoor.tiles[&id("plain")], at(1, 2));
        assert_eq!(indoor.tiles.len(), own.tiles.len());
        assert_eq!(indoor.layers.len(), 1);
        // A map's look is the one it names, else the tileset's own.
        assert_eq!(t.look("indoor"), Some(indoor));
        assert_eq!(t.look("outdoor"), Some(own));
        assert_eq!(t.look("cave"), Some(own));
        assert_eq!(t.tile(id("plain")).map(|p| p.rect), Some(at(0, 0)));
        // No terrain tiles: no look at all.
        let bare = file(&f, &[("tile_px", ""), ("terrain", "")]);
        assert_eq!(f.parse(&bare).unwrap().look("outdoor"), None);
    }

    #[test]
    fn a_layer_names_real_terrain_and_real_mixes_inside_the_image() {
        let f = Fixture::new();
        let layers = r###"[
            Corners(of: [], tiles: [(corners: "##..", at: (0, 0))]),
            Corners(of: ["lava"], tiles: []),
            Corners(of: ["water"], tiles: [
                (corners: "##.", at: (0, 0)),
                (corners: "....", at: (0, 0)),
                (corners: "#..#", at: (4, 0)),
                (corners: "##..", at: (0, 0)),
                (corners: "##..", at: (1, 0)),
            ]),
            Tiles(of: ["fort"], at: (0, 4), beside: ["moat"],
                  sides: [(sides: "#x#.", at: (0, 0)), (sides: "....", at: (4, 4)),
                          (sides: "....", at: (0, 0))]),
        ]"###;
        assert_eq!(
            f.errors(&file(&f, &[("layers", layers)])),
            [
                "layer 1: `of` names no terrain",
                "layer 2: it has no pictures",
                "layer 3: corners \"##.\" must be four of `#` (in) and `.` (out)",
                "layer 3, corners \"#..#\": its 16×16 px at (64, 0) lies outside the 64×64 px image",
                "layer 3: corners \"##..\" is listed twice",
                "layer 3: corners \"....\" has no corner in the layer, so there is nothing to draw",
                "layer 4: its 16×16 px at (0, 64) lies outside the 64×64 px image",
                "layer 4: sides \"#x#.\" must be four of `#` (in) and `.` (out)",
                "layer 4, sides \"....\": its 16×16 px at (64, 64) lies outside the 64×64 px image",
                "layer 4: sides \"....\" is listed twice",
                "layer 2: terrain \"lava\" is not in terrain.ron",
                "layer 4: terrain \"moat\" is not in terrain.ron",
            ]
        );
        // A layer is one of the two kinds, with only its own fields.
        for bad in [
            r#"[Sides(of: ["fort"], at: (0, 0))]"#,
            r#"[Tiles(of: ["fort"], at: (0, 0), tiles: [])]"#,
            r###"[Corners(of: ["fort"], tiles: [(corners: "##..", at: (0, 0), flip: true)])]"###,
            r###"[Tiles(of: ["fort"], at: (0, 0), sides: [(corners: "##..", at: (0, 0))])]"###,
        ] {
            let errors = f.parse(&file(&f, &[("layers", bad)])).unwrap_err();
            assert_eq!(errors.len(), 1, "{bad}");
            assert!(errors[0].line.is_some(), "{bad}: {errors:?}");
        }
    }

    #[test]
    fn every_look_has_a_tile_for_every_terrain_and_a_name_a_map_can_give() {
        let f = Fixture::new();
        let terrain = f
            .terrain("\"lava\": (0, 0)")
            .replace("\"forest\": (0, 0), ", "")
            .replace("\"plain\": (0, 0)", "\"plain\": (0, 4)");
        let looks = format!(
            r#"{{ "indoor": (terrain: {{ {terrain} }}, layers: [Tiles(of: ["moat"], at: (4, 0))]) }}"#
        );
        assert_eq!(
            f.errors(&file(&f, &[("looks", &looks)])),
            [
                "look \"indoor\": terrain \"plain\": its 16×16 px at (0, 64) lies outside the 64×64 px image",
                "look \"indoor\": layer 1: its 16×16 px at (64, 0) lies outside the 64×64 px image",
                "look \"indoor\": terrain \"lava\" is not in terrain.ron",
                "look \"indoor\": layer 1: terrain \"moat\" is not in terrain.ron",
                "look \"indoor\": terrain \"forest\" has no tile",
            ]
        );
        let all = format!("(terrain: {{ {} }})", f.terrain(""));
        let looks = format!(r#"{{ "outdoor": {all}, "cave": {all} }}"#);
        assert_eq!(
            f.errors(&file(&f, &[("looks", &looks)])),
            [
                "look \"cave\" is not a look a map can name (\"outdoor\", \"indoor\")",
                "look \"outdoor\" is the tileset's own `terrain` and `layers`: write it there",
            ]
        );
        // Layers and looks are painted with the terrain tiles: none
        // without them.
        let frame = "(image: \"units/a.png\", frame: (1, 0))";
        let units = format!("(characters: {{}}, classes: {{}}, fallback: {frame})");
        let looks = format!(r#"{{ "indoor": {all} }}"#);
        let fields = [
            ("tile_px", ""),
            ("terrain", ""),
            ("units", units.as_str()),
            ("layers", r#"[Tiles(of: ["fort"], at: (0, 0))]"#),
            ("looks", looks.as_str()),
        ];
        assert_eq!(
            f.errors(&file(&f, &fields)),
            [
                "layers need `terrain`: they are painted over its tiles",
                "looks need `terrain`, the tileset's own look",
            ]
        );
    }

    #[test]
    fn a_unit_entry_is_two_numbers_or_an_image_and_a_frame() {
        let f = Fixture::new();
        for bad in [
            "(1)",
            "(1, 2, 3)",
            "(image: \"units/a.png\")",
            "(image: \"units/a.png\", frame: (1, 0), flip: true)",
            "\"units/a.png\"",
        ] {
            let units = format!("(characters: {{}}, classes: {{}}, fallback: {bad})");
            let errors = f.parse(&sheets_file(&f, &units)).unwrap_err();
            assert_eq!(errors.len(), 1, "{bad}");
            assert!(errors[0].line.is_some(), "{bad}: {errors:?}");
        }
        // What isn't either kind is told what was expected.
        let units = "(characters: {}, classes: {}, fallback: \"units/a.png\")";
        let errors = f.parse(&sheets_file(&f, units)).unwrap_err();
        let expected = "(column, row) or (image: \"…\", frame: (column, row))";
        assert!(errors[0].message.contains(expected), "{errors:?}");
    }
}
