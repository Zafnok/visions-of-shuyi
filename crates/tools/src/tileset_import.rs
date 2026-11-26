//! `cargo xtask tileset-import [--list]`: turns the bought Tiled tilesets
//! into the game's tileset, `assets-private/game/tilesets/tiny_tales.png`
//! and `.ron` (ticket 0437, ADR-0052).
//!
//! The bought sets are Tiled files: a 16×16-tile image and a `.tsx` that
//! names, for a "terrain" of the set, the tile for each mix of its four
//! corners. Which bought terrain stands for which of ours is the mapping
//! file [`MAPPING_PATH`]: ours, with names and numbers and no art. The
//! importer reads both, copies each tile the mapping uses once into one
//! packed image (a recoloured copy where the mapping asks for a tint) and
//! writes the tileset file: the terrain looks, then the unit pictures of
//! `map-sprite-import`'s table.
//!
//! A tile the looks use that the `.tsx` animates (Tiled's `<animation>`)
//! moves in the game too (ticket 1315): its frames are packed and written
//! as the tileset's `frames`, each still on a lesser graphics tier. A loop
//! whose frames are all one picture doesn't move and is left out: Tiny
//! Tales' coasts, hills and woods are such loops (0096).
//!
//! A pack with no Tiled files (ELV Games' *Rogue Adventure*, ticket 1318)
//! is read as plain sheets: a tile by its place, a `Patch` layer from the
//! usual three-by-three layout, and a sheet's moving tiles by `moving`
//! (each frame a step along the row).
//!
//! A `Quarters` layer (ticket 1359) builds each corner picture from
//! quarters of a kit's cells, RPG Maker style, so inside corners are round
//! too; `recolour` changes a sheet's colours as it is read (Lonesome
//! Forest's stones in a sandy brown): the recoloured art is never kept.
//!
//! It is the only reader of Tiled files: the game reads our RON and PNG.
//! The XML it needs is three kinds of tag and their attributes, read by
//! [`tags`]; no XML crate.
//!
//! The output is bought art (ADR-0040): it goes to the private assets
//! checkout, never into this repository.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use serde::Deserialize;
use trpg_content::TerrainDef;
use trpg_content::map::{DEFAULT_TILES, TILE_LOOKS};
use trpg_content::tileset::parse_mix;

use crate::map_sprite_import::{self, GAME_DIR, MapSprite, TILESET_ID};
use crate::tileset_ron::{self, At, FramesText, LayerText, LookText};
use shuyi_cli::codec::{decode_png, encode_png};

/// Shown for `--help` and after a bad argument.
pub const USAGE: &str = "usage: cargo xtask tileset-import [--list]\n\n\
Packs the bought tiles named by assets-src/tilesets/tiny_tales.ron, from\n\
assets-private/library/ (Tiny Tales' Tiled sets, other packs' plain sheets), into\n\
assets-private/game/tilesets/tiny_tales.png, and writes tiny_tales.ron beside it:\n\
the terrain looks, and the unit pictures of `map-sprite-import`'s table.\n\
--list  print which bought tiles each terrain and layer uses, and write nothing\n\n\
Run `cargo shuyi private-assets --library` first. Afterwards commit and push in\n\
assets-private/, then run `cargo shuyi private-assets --pin`.";

/// The sorted bought packs, relative to the repo root: a plain sheet
/// (`….png`) of the mapping is named by its path under it.
pub const LIBRARIES_DIR: &str = "assets-private/library";
/// The Tiny Tales tilesets, under [`LIBRARIES_DIR`]: a Tiled sheet of the
/// mapping is named by its path under it, without `.tsx`.
pub const TILED_DIR: &str = "tiny-tales/tilesets";
/// The sorted bought tilesets, relative to the repo root.
pub const LIBRARY_DIR: &str = "assets-private/library/tiny-tales/tilesets";
/// The mapping file, relative to the repo root.
pub const MAPPING_PATH: &str = "assets-src/tilesets/tiny_tales.ron";
/// A tile's side in the packed image, in pixels: the bought tiles'.
pub const TILE: u32 = 16;
/// Tiles per row of the packed image.
pub const COLUMNS: u32 = 16;
/// What a moving tile still needs for the `Minimal` tier (a GBA one day:
/// `TODO(gba)`), listed by the content's stills check (ADR-0131).
pub const MINIMAL_TODO: &str = "a one-bit tile";

/// A tile of a bought sheet: the sheet and the tile's id in it, counting
/// its 16×16 tiles row by row from 0. A sheet is a Tiled file (its `.tsx`
/// under [`TILED_DIR`], without the extension) or a plain sheet (a `.png`
/// under [`LIBRARIES_DIR`], with it): a pack with no Tiled files.
pub type TileRef = (String, u32);

/// The mapping file.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mapping {
    /// Each of our terrains' own tile, unless a look has its own.
    pub terrain: BTreeMap<String, TileRef>,
    /// The layers, by a name the looks list them by.
    pub layers: BTreeMap<String, LayerMap>,
    /// The looks, by the name a map file gives.
    pub looks: BTreeMap<String, LookMap>,
    /// The plain sheets whose tiles move, by sheet: every packed tile of
    /// one loops through its frames ([`Strip`]).
    #[serde(default)]
    pub moving: BTreeMap<String, Strip>,
    /// Colours changed in a sheet as it is read, by sheet: each pixel of
    /// a pair's first colour takes its second, its alpha kept (ticket
    /// 1359: Lonesome Forest's stones in a sandy brown).
    #[serde(default)]
    pub recolour: BTreeMap<String, Vec<(Rgb, Rgb)>>,
}

/// A colour: red, green and blue.
pub type Rgb = (u8, u8, u8);

/// How a plain sheet draws its moving tiles: frame `k` of a tile is the
/// tile `k × step` places to its right, each shown for `ms`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Strip {
    /// How many frames, the tile itself the first.
    pub frames: u32,
    /// Tiles from one frame to the next, along the row.
    pub step: u32,
    /// How long each frame shows, in ms.
    pub ms: u32,
}

/// A layer of the mapping.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum LayerMap {
    /// Pictures between tiles: a Tiled terrain's tile for each mix of
    /// corners.
    Corners {
        /// Our terrains in the layer.
        of: Vec<String>,
        /// The bought sheet.
        sheet: String,
        /// The Tiled terrain's name in it.
        terrain: String,
        /// Where the sheet has several tiles for a mix: which (0 = the
        /// first; the last if there are fewer).
        #[serde(default)]
        pick: usize,
        /// Whether to take the tile of all four corners in too. Not
        /// needed when the layer's terrains have it as their own tile.
        #[serde(default = "yes")]
        full: bool,
        /// A palette colour to recolour the tiles in, each pixel as
        /// bright as it was; empty for none.
        #[serde(default)]
        tint: String,
    },
    /// A picture on each tile.
    Tiles {
        /// Our terrains that get it.
        of: Vec<String>,
        /// The picture.
        tile: TileRef,
        /// Our terrains a neighbour is looked at for.
        #[serde(default)]
        beside: Vec<String>,
        /// The picture for a mix of sides (up, right, down, left being
        /// one of `beside`), written as four of `#` and `.`.
        #[serde(default)]
        sides: BTreeMap<String, TileRef>,
    },
    /// A picture on each tile, joined from a block of a sheet: nine tiles,
    /// three by three, of a patch of the terrain with its edge on every
    /// side (ticket 0452). Each quarter of a tile is the same quarter of
    /// the block's tile for whether the tile's neighbours that way are in
    /// the layer: a tile with no neighbour of the layer above has the top
    /// edge, one alone has the edge all round. So the patch's edge, inside
    /// its tiles, is drawn on the tiles where the terrain ends.
    Block {
        /// Our terrains that get it.
        of: Vec<String>,
        /// The bought sheet.
        sheet: String,
        /// The block's top-left tile.
        block: u32,
        /// A palette colour to recolour it in, as for `Corners`.
        #[serde(default)]
        tint: String,
    },
    /// Pictures between tiles, as `Corners`, from a sheet laid out the
    /// usual way rather than a Tiled terrain: a patch of nine tiles,
    /// three by three, whose eight outer tiles are its edge and outside
    /// corners and its middle the full tile, and nine more whose four
    /// corner tiles are the inside corners. A mix of two diagonal corners
    /// is two of the patch's outside corners, one over the other.
    Patch {
        /// Our terrains in the layer.
        of: Vec<String>,
        /// The sheet.
        sheet: String,
        /// The patch's top-left tile.
        patch: u32,
        /// The top-left tile of the nine with the inside corners.
        inside: u32,
        /// Whether to take the full tile in too, as for `Corners`.
        #[serde(default = "yes")]
        full: bool,
    },
    /// Pictures between tiles, as `Corners`, each joined from quarters of
    /// a kit's cells the way RPG Maker joins them (ticket 1359). Each
    /// quarter of a tile in the layer is the same quarter of one cell,
    /// chosen by the tile's two neighbours that way and the diagonal
    /// between them: neither in the layer, an outside corner; only the
    /// one across, an edge along the top or the bottom; only the one up
    /// or down, an edge down a side; both but not the diagonal, an inside
    /// corner; all three, nothing (the tile's own picture shows). So
    /// every corner, outside and inside, is the kit's round one.
    Quarters {
        /// Our terrains in the layer.
        of: Vec<String>,
        /// The kit's sheet.
        sheet: String,
        /// Which of its cells each kind of quarter comes from.
        kit: Kit,
    },
}

/// The cells of a [`LayerMap::Quarters`]: which cell each kind of quarter
/// is taken from, counted as a sheet's tiles are.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Kit {
    /// The outside corners, by quarter: up-left, up-right, down-left,
    /// down-right.
    pub outer: [u32; 4],
    /// The edges along the top and along the bottom.
    pub across: [u32; 2],
    /// The edges down the left and down the right.
    pub down: [u32; 2],
    /// The inside corners, by quarter.
    pub inside: [u32; 4],
}

impl Kit {
    /// The four quarters of the picture of a point whose tiles (up-left,
    /// up-right, down-left, down-right) are `places` in the layer, in
    /// that order: the cell each tile's quarter nearest the point comes
    /// from, or `None` where nothing is drawn.
    pub fn quarters(&self, places: [bool; 4]) -> [Option<u32>; 4] {
        // A tile's neighbour across is the place beside it (index ^ 1),
        // the one up or down the place above or below it (^ 2), the
        // diagonal the opposite place (^ 3); its quarter nearest the point
        // is the opposite quarter (^ 3), counted the same way.
        let mut out = [None; 4];
        for (place, cell) in out.iter_mut().enumerate() {
            let quarter = place ^ 3;
            let (across, upright, diagonal) =
                (places[place ^ 1], places[place ^ 2], places[place ^ 3]);
            *cell = match (places[place], across, upright) {
                (false, ..) => None,
                (true, false, false) => Some(self.outer[quarter]),
                (true, true, false) => Some(self.across[quarter >> 1]),
                (true, false, true) => Some(self.down[quarter & 1]),
                (true, true, true) => (!diagonal).then_some(self.inside[quarter]),
            };
        }
        out
    }
}

/// A mix of corners (up-left, up-right, down-left, down-right in the
/// layer), whether its picture is in the inside corners' nine of a
/// [`LayerMap::Patch`], and its tiles' (column, row) there, the second
/// drawn over the first.
type PatchPlace = ([bool; 4], bool, &'static [(u32, u32)]);

/// Where each mix of corners is in a [`LayerMap::Patch`].
const PATCH: [PatchPlace; 15] = {
    const fn m(ul: u8, ur: u8, dl: u8, dr: u8) -> [bool; 4] {
        [ul == 1, ur == 1, dl == 1, dr == 1]
    }
    [
        (m(0, 0, 0, 1), false, &[(0, 0)]),
        (m(0, 0, 1, 1), false, &[(1, 0)]),
        (m(0, 0, 1, 0), false, &[(2, 0)]),
        (m(0, 1, 0, 1), false, &[(0, 1)]),
        (m(1, 1, 1, 1), false, &[(1, 1)]),
        (m(1, 0, 1, 0), false, &[(2, 1)]),
        (m(0, 1, 0, 0), false, &[(0, 2)]),
        (m(1, 1, 0, 0), false, &[(1, 2)]),
        (m(1, 0, 0, 0), false, &[(2, 2)]),
        (m(1, 1, 1, 0), true, &[(0, 0)]),
        (m(1, 1, 0, 1), true, &[(2, 0)]),
        (m(1, 0, 1, 1), true, &[(0, 2)]),
        (m(0, 1, 1, 1), true, &[(2, 2)]),
        (m(1, 0, 0, 1), false, &[(2, 2), (0, 0)]),
        (m(0, 1, 1, 0), false, &[(0, 2), (2, 0)]),
    ]
};

const fn yes() -> bool {
    true
}

/// A look of the mapping.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LookMap {
    /// Own tiles that differ from the mapping's.
    #[serde(default)]
    pub terrain: BTreeMap<String, TileRef>,
    /// Its layers by name, the first undermost.
    pub layers: Vec<String>,
}

/// Every tag of `xml` that opens an element, with its attributes. Enough
/// XML for a Tiled file: attribute values are in double quotes and hold no
/// `>`.
pub fn tags(xml: &str) -> Vec<(String, BTreeMap<String, String>)> {
    let mut out = Vec::new();
    for chunk in xml.split('<').skip(1) {
        let body = chunk.split('>').next().unwrap_or_default();
        // A closing tag, the declaration, a comment.
        if body.starts_with(['/', '?', '!']) {
            continue;
        }
        let body = body.trim_end_matches('/');
        let (name, mut rest) = body.split_once(char::is_whitespace).unwrap_or((body, ""));
        let mut attributes = BTreeMap::new();
        while let Some((key, after)) = rest.split_once("=\"") {
            let Some((value, tail)) = after.split_once('"') else {
                break;
            };
            attributes.insert(key.trim().to_owned(), unescape(value));
            rest = tail;
        }
        out.push((name.trim().to_owned(), attributes));
    }
    out
}

/// `value` with XML's five named characters written out.
fn unescape(value: &str) -> String {
    let named = [
        ("&lt;", "<"),
        ("&gt;", ">"),
        ("&quot;", "\""),
        ("&apos;", "'"),
    ];
    let mut out = value.to_owned();
    for (from, to) in named {
        out = out.replace(from, to);
    }
    // Last, so `&amp;lt;` stays the text `&lt;`.
    out.replace("&amp;", "&")
}

/// What the importer reads of a Tiled tileset file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tsx {
    /// Its image, relative to the file.
    pub image: String,
    /// The colour that stands for nothing (`trans`), if it names one.
    pub clear: Option<[u8; 3]>,
    /// A tile's size, across × down.
    pub tile: (u32, u32),
    /// Tiles per row of the image.
    pub columns: u32,
    /// Its terrains' names, in order: a tile names them by position.
    pub terrains: Vec<String>,
    /// Each tile with corner terrains: its id and the terrain at its
    /// top-left, top-right, bottom-left and bottom-right corner (`None`:
    /// none).
    pub corners: Vec<(u32, [Option<usize>; 4])>,
    /// Each animated tile's frames, by its id: each frame's tile and its
    /// time in ms, in order.
    pub animations: BTreeMap<u32, Vec<(u32, u32)>>,
}

/// A colour written as six hex digits.
fn hex(text: &str) -> Option<[u8; 3]> {
    let digits = text.trim_start_matches('#');
    if digits.len() != 6 || !digits.is_ascii() {
        return None;
    }
    let channel = |i: usize| u8::from_str_radix(&digits[i..i + 2], 16).ok();
    Some([channel(0)?, channel(2)?, channel(4)?])
}

/// The corner terrains of a tile, as its `terrain` attribute writes them:
/// four places split by commas, each a terrain's number or empty.
fn corner_terrains(text: &str) -> Option<[Option<usize>; 4]> {
    let mut corners = [None; 4];
    let mut parts = text.split(',');
    for corner in &mut corners {
        let part = parts.next()?.trim();
        if !part.is_empty() {
            *corner = Some(part.parse().ok()?);
        }
    }
    parts.next().is_none().then_some(corners)
}

/// Parses a Tiled tileset file.
pub fn parse_tsx(xml: &str) -> Result<Tsx, String> {
    let tags = tags(xml);
    let first = |name: &str| tags.iter().find(|(n, _)| n == name).map(|(_, a)| a);
    let tileset = first("tileset").ok_or("no <tileset>")?;
    let number = |key: &str| {
        let value = tileset.get(key).and_then(|v| v.parse::<u32>().ok());
        value
            .filter(|&n| n > 0)
            .ok_or_else(|| format!("<tileset> has no {key}"))
    };
    let image = first("image").ok_or("no <image>")?;
    let source = image.get("source").ok_or("<image> has no source")?;
    let clear = match image.get("trans") {
        Some(text) => Some(hex(text).ok_or_else(|| format!("trans \"{text}\" is not a colour"))?),
        None => None,
    };
    let named = |name: &'static str| tags.iter().filter(move |(n, _)| n == name);
    let terrains = named("terrain")
        .map(|(_, a)| a.get("name").cloned().unwrap_or_default())
        .collect();
    let mut corners = Vec::new();
    for (_, attributes) in named("tile") {
        let Some(text) = attributes.get("terrain") else {
            continue;
        };
        let id = attributes.get("id").and_then(|v| v.parse::<u32>().ok());
        let tile = id.zip(corner_terrains(text));
        corners.push(tile.ok_or_else(|| format!("a <tile> with terrain \"{text}\" is not read"))?);
    }
    // A <frame> belongs to the <tile> before it: frames are only in a
    // tile's <animation>.
    let mut animations: BTreeMap<u32, Vec<(u32, u32)>> = BTreeMap::new();
    let mut tile = None;
    for (name, attributes) in &tags {
        let number = |key: &str| attributes.get(key).and_then(|v| v.parse::<u32>().ok());
        match name.as_str() {
            "tile" => tile = number("id"),
            "frame" => {
                let at = tile.ok_or("a <frame> outside a <tile>")?;
                let frame = number("tileid").zip(number("duration"));
                let frame = frame.ok_or_else(|| format!("tile {at}: a <frame> is not read"))?;
                animations.entry(at).or_default().push(frame);
            }
            _ => {}
        }
    }
    Ok(Tsx {
        image: source.clone(),
        clear,
        tile: (number("tilewidth")?, number("tileheight")?),
        columns: number("columns")?,
        terrains,
        corners,
        animations,
    })
}

/// A bought sheet: its Tiled file and its image's pixels.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sheet {
    /// The Tiled file.
    pub tsx: Tsx,
    /// The image's width and height.
    pub size: (u32, u32),
    /// Its RGBA8 pixels, row by row.
    pub rgba: Vec<u8>,
}

impl Sheet {
    /// Reads sheet `name` of the packs under `libraries`: a plain sheet
    /// (`….png`) there, or a Tiled one (its `.tsx` without the extension)
    /// under [`TILED_DIR`].
    pub fn read(libraries: &Path, name: &str) -> Result<Self, String> {
        if is_plain(name) {
            return Self::plain(&libraries.join(name));
        }
        let path = libraries.join(TILED_DIR).join(format!("{name}.tsx"));
        let xml =
            fs::read_to_string(&path).map_err(|e| format!("reading {}: {e}", path.display()))?;
        let tsx = parse_tsx(&xml).map_err(|e| format!("{}: {e}", path.display()))?;
        if tsx.tile != (TILE, TILE) {
            let (w, h) = tsx.tile;
            return Err(format!(
                "{}: its tiles are {w}×{h} px, not {TILE}×{TILE}",
                path.display()
            ));
        }
        let image = path.with_file_name(&tsx.image);
        let bytes = fs::read(&image).map_err(|e| format!("reading {}: {e}", image.display()))?;
        let (width, height, rgba) =
            decode_png(&bytes).map_err(|e| format!("{}: {e}", image.display()))?;
        Ok(Self {
            tsx,
            size: (width, height),
            rgba,
        })
    }

    /// Reads the plain sheet at `path`: an image of 16×16 tiles and
    /// nothing else (no clear colour, no terrains, nothing moving).
    fn plain(path: &Path) -> Result<Self, String> {
        let bytes = fs::read(path).map_err(|e| format!("reading {}: {e}", path.display()))?;
        let (width, height, rgba) =
            decode_png(&bytes).map_err(|e| format!("{}: {e}", path.display()))?;
        if width < TILE || height < TILE {
            return Err(format!(
                "{}: a {width}×{height} px image holds no {TILE}×{TILE} tile",
                path.display()
            ));
        }
        let image = path.file_name().unwrap_or_default().to_string_lossy();
        let tsx = Tsx {
            image: image.into_owned(),
            clear: None,
            tile: (TILE, TILE),
            columns: width / TILE,
            terrains: Vec::new(),
            corners: Vec::new(),
            animations: BTreeMap::new(),
        };
        Ok(Self {
            tsx,
            size: (width, height),
            rgba,
        })
    }

    /// The RGBA8 pixels of tile `id`, row by row, the Tiled file's clear
    /// colour as nothing.
    pub fn tile(&self, id: u32) -> Result<Vec<u8>, String> {
        let (left, top) = (
            (id % self.tsx.columns) * TILE,
            (id / self.tsx.columns) * TILE,
        );
        let (width, height) = self.size;
        if left + TILE > width || top + TILE > height {
            return Err(format!(
                "tile {id} is outside the {width}×{height} px image"
            ));
        }
        let mut out = Vec::new();
        for (x, y) in (top..top + TILE).flat_map(|y| (left..left + TILE).map(move |x| (x, y))) {
            let at = usize::try_from((y * width + x) * 4).unwrap_or(usize::MAX);
            let nothing = [0; 4];
            let pixel = self.rgba.get(at..at + 4).unwrap_or(&nothing);
            let clear = self.tsx.clear.is_some_and(|c| pixel[..3] == c) || pixel[3] == 0;
            out.extend_from_slice(if clear { &nothing } else { pixel });
        }
        Ok(out)
    }

    /// The tile of each mix of corners of the Tiled terrain `terrain`:
    /// of the tiles whose corners are that terrain or none, the `pick`th
    /// listed for the mix (the last, if there are fewer).
    pub fn corners(&self, terrain: &str, pick: usize) -> Result<[Option<u32>; 16], String> {
        let index = self.tsx.terrains.iter().position(|name| name == terrain);
        let index = index.ok_or_else(|| {
            let known = self.tsx.terrains.join(", ");
            format!("no terrain \"{terrain}\" (it has: {known})")
        })?;
        let mut listed: [Vec<u32>; 16] = Default::default();
        for (id, corners) in &self.tsx.corners {
            if corners.iter().flatten().all(|&t| t == index) {
                let mix = trpg_content::tileset::mix(corners.map(|c| c.is_some()));
                listed[usize::from(mix)].push(*id);
            }
        }
        let picked = |ids: &Vec<u32>| ids.get(pick).or(ids.last()).copied();
        let mut tiles = [None; 16];
        // Mix 0 is no corner at all: never a picture.
        for mix in 1..16 {
            tiles[mix] = picked(&listed[mix]);
        }
        Ok(tiles)
    }
}

/// Whether sheet `name` is a plain sheet (`….png`), not a Tiled one.
fn is_plain(name: &str) -> bool {
    Path::new(name)
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("png"))
}

/// How bright a colour is, 0 to 255 000: the usual weights of red, green
/// and blue.
fn brightness(rgb: [u8; 3]) -> u32 {
    299 * u32::from(rgb[0]) + 587 * u32::from(rgb[1]) + 114 * u32::from(rgb[2])
}

/// Recolours the RGBA8 `pixels` in `color`: each pixel that isn't clear
/// takes the colour, as bright as the pixel was (so shading stays).
pub fn tint(pixels: &mut [u8], color: [u8; 3]) {
    let full = brightness(color).max(1);
    let (pixels, _) = pixels.as_chunks_mut::<4>();
    for pixel in pixels.iter_mut().filter(|p| p[3] != 0) {
        let own = brightness([pixel[0], pixel[1], pixel[2]]);
        for (channel, c) in pixel.iter_mut().zip(color) {
            let scaled = u32::from(c) * own / full;
            *channel = u8::try_from(scaled).unwrap_or(u8::MAX);
        }
    }
}

/// Which picture of a sheet a cell of the packed image is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cut {
    /// One tile, by its id.
    Tile(u32),
    /// A tile joined from a block's quarters ([`LayerMap::Block`]): the
    /// block's top-left tile, and the mix of sides (up, right, down,
    /// left) where the terrain ends.
    Joined { block: u32, ends: u8 },
    /// Tile `over` drawn over tile `under` ([`LayerMap::Patch`]'s
    /// diagonal mixes).
    Pair { under: u32, over: u32 },
    /// A picture joined from four quarters ([`LayerMap::Quarters`]): for
    /// its up-left, up-right, down-left and down-right quarter, the cell
    /// whose opposite quarter goes there, or nothing.
    Quarters([Option<u32>; 4]),
}

impl Cut {
    /// The column of its tile furthest right, in a sheet `columns` tiles
    /// across: a block's is two past its top-left tile's.
    fn right_column(self, columns: u32) -> u32 {
        match self {
            Self::Tile(tile) => tile % columns,
            Self::Joined { block, .. } => block % columns + 2,
            Self::Pair { under, over } => (under % columns).max(over % columns),
            Self::Quarters(cells) => cells
                .iter()
                .flatten()
                .map(|c| c % columns)
                .max()
                .unwrap_or(0),
        }
    }

    /// The same picture `places` tiles to the right in its sheet: its
    /// frame of a [`Strip`].
    const fn along(self, places: u32) -> Self {
        match self {
            Self::Tile(tile) => Self::Tile(tile + places),
            Self::Joined { block, ends } => Self::Joined {
                block: block + places,
                ends,
            },
            Self::Pair { under, over } => Self::Pair {
                under: under + places,
                over: over + places,
            },
            Self::Quarters(cells) => {
                let mut moved = cells;
                let mut i = 0;
                while i < moved.len() {
                    if let Some(cell) = moved[i] {
                        moved[i] = Some(cell + places);
                    }
                    i += 1;
                }
                Self::Quarters(moved)
            }
        }
    }
}

/// Tile `over` of `sheet` drawn over tile `under`: RGBA8, row by row.
fn paired(sheet: &Sheet, under: u32, over: u32) -> Result<Vec<u8>, String> {
    let mut out = sheet.tile(under)?;
    let top = sheet.tile(over)?;
    let (out_pixels, _) = out.as_chunks_mut::<4>();
    let (top_pixels, _) = top.as_chunks::<4>();
    for (pixel, drawn) in out_pixels.iter_mut().zip(top_pixels) {
        if drawn[3] == u8::MAX || pixel[3] == 0 {
            *pixel = *drawn;
        } else if drawn[3] != 0 {
            // A part see-through pixel over a solid one: mixed by its
            // alpha (the bought art has none, but a sheet might).
            let a = u32::from(drawn[3]);
            for (channel, top) in pixel.iter_mut().zip(drawn).take(3) {
                let mixed = (u32::from(*top) * a + u32::from(*channel) * (255 - a)) / 255;
                *channel = u8::try_from(mixed).unwrap_or(u8::MAX);
            }
            pixel[3] = pixel[3].max(drawn[3]);
        }
    }
    Ok(out)
}

/// A picture joined from quarters of `sheet`'s cells: for each of its
/// quarters (up-left, up-right, down-left, down-right), the opposite
/// quarter of the cell `cells` names there, or nothing. RGBA8, row by row.
fn quartered(sheet: &Sheet, cells: [Option<u32>; 4]) -> Result<Vec<u8>, String> {
    let half = usize::try_from(TILE / 2).unwrap_or(0);
    let line = usize::try_from(TILE * 4).unwrap_or(0);
    let mut out = vec![0; line * half * 2];
    for (to, cell) in cells.into_iter().enumerate() {
        let Some(cell) = cell else {
            continue;
        };
        let tile = sheet.tile(cell)?;
        let from = to ^ 3;
        let corner = |q: usize| (q >> 1) * half * line + (q & 1) * half * 4;
        for y in 0..half {
            let (src, dst) = (corner(from) + y * line, corner(to) + y * line);
            out[dst..dst + half * 4].copy_from_slice(&tile[src..src + half * 4]);
        }
    }
    Ok(out)
}

/// Changes each pixel of RGBA8 `pixels` whose colour is the first of one
/// of `pairs` to the second, its alpha kept.
pub fn recolour(pixels: &mut [u8], pairs: &[(Rgb, Rgb)]) {
    let (pixels, _) = pixels.as_chunks_mut::<4>();
    for pixel in pixels.iter_mut().filter(|p| p[3] != 0) {
        let own = (pixel[0], pixel[1], pixel[2]);
        if let Some(&(_, (r, g, b))) = pairs.iter().find(|(from, _)| *from == own) {
            pixel[..3].copy_from_slice(&[r, g, b]);
        }
    }
}

/// A tile joined from the quarters of the block of `sheet` whose top-left
/// tile is `block`, for a tile whose terrain ends on the mix of sides
/// `ends` (up, right, down, left): RGBA8, row by row.
fn joined(sheet: &Sheet, block: u32, ends: u8) -> Result<Vec<u8>, String> {
    if block % sheet.tsx.columns + 3 > sheet.tsx.columns {
        return Err(format!(
            "the block from tile {block} runs past the image's right edge"
        ));
    }
    let [up, right, down, left] = [8, 4, 2, 1].map(|bit| ends & bit != 0);
    // The block's row for the tile's top and bottom half, and its column
    // for the left and right half.
    let rows = [u32::from(!up), if down { 2 } else { 1 }];
    let columns = [u32::from(!left), if right { 2 } else { 1 }];
    let half = usize::try_from(TILE / 2).unwrap_or(0);
    let line = usize::try_from(TILE * 4).unwrap_or(0);
    let mut out = vec![0; line * half * 2];
    for (qy, row) in rows.into_iter().enumerate() {
        for (qx, column) in columns.into_iter().enumerate() {
            let tile = sheet.tile(block + row * sheet.tsx.columns + column)?;
            for y in qy * half..(qy + 1) * half {
                let at = y * line + qx * half * 4;
                out[at..at + half * 4].copy_from_slice(&tile[at..at + half * 4]);
            }
        }
    }
    Ok(out)
}

/// The packed image being made: each bought tile the mapping uses, once.
struct Packer<'a> {
    /// The packs, [`LIBRARIES_DIR`].
    dir: &'a Path,
    colors: &'a dyn Fn(&str) -> Option<[u8; 3]>,
    /// The plain sheets whose tiles move.
    strips: &'a BTreeMap<String, Strip>,
    /// The colours changed in each sheet as it is read.
    recolours: &'a BTreeMap<String, Vec<(Rgb, Rgb)>>,
    sheets: BTreeMap<String, Sheet>,
    /// The sheet, picture and tint of each cell, in the image's order.
    cells: Vec<(String, Cut, String)>,
    /// The cells' pixels, in the same order.
    pixels: Vec<Vec<u8>>,
}

impl Packer<'_> {
    /// Sheet `name`, read once.
    fn sheet(&mut self, name: &str) -> Result<&Sheet, String> {
        if !self.sheets.contains_key(name) {
            let mut sheet = Sheet::read(self.dir, name)?;
            if let Some(pairs) = self.recolours.get(name) {
                recolour(&mut sheet.rgba, pairs);
            }
            self.sheets.insert(name.to_owned(), sheet);
        }
        self.sheets.get(name).ok_or_else(|| name.to_owned())
    }

    /// Where tile `tile` of sheet `sheet`, recoloured in the palette's
    /// `tint_name` (empty: as it is), is in the packed image; packed now
    /// if it wasn't.
    fn cell(&mut self, sheet: &str, tile: u32, tint_name: &str) -> Result<At, String> {
        self.packed(sheet, Cut::Tile(tile), tint_name)
    }

    /// Where picture `cut` of sheet `sheet`, recoloured in the palette's
    /// `tint_name` (empty: as it is), is in the packed image; packed now
    /// if it wasn't.
    fn packed(&mut self, sheet: &str, cut: Cut, tint_name: &str) -> Result<At, String> {
        let key = (sheet.to_owned(), cut, tint_name.to_owned());
        let packed = self.cells.iter().position(|cell| *cell == key);
        let index = if let Some(index) = packed {
            index
        } else {
            let source = self.sheet(sheet)?;
            let pixels = match cut {
                Cut::Tile(tile) => source.tile(tile),
                Cut::Joined { block, ends } => joined(source, block, ends),
                Cut::Pair { under, over } => paired(source, under, over),
                Cut::Quarters(cells) => quartered(source, cells),
            };
            let mut pixels = pixels.map_err(|e| format!("{sheet}: {e}"))?;
            if !tint_name.is_empty() {
                let color = (self.colors)(tint_name)
                    .ok_or_else(|| format!("no palette colour \"{tint_name}\""))?;
                tint(&mut pixels, color);
            }
            self.cells.push(key);
            self.pixels.push(pixels);
            self.cells.len() - 1
        };
        let index = u32::try_from(index).unwrap_or(u32::MAX);
        Ok((index % COLUMNS, index / COLUMNS))
    }

    /// The frames of each packed tile its sheet animates, packed now: a
    /// cell for every frame, in the same tint, the loop turned to start on
    /// the tile itself (what stands still). A loop of one picture, or one
    /// that never shows the tile itself, is left out.
    fn moving(&mut self) -> Result<Vec<FramesText>, String> {
        let mut out = Vec::new();
        for index in 0..self.cells.len() {
            let (sheet, cut, tint_name) = self.cells[index].clone();
            if let Some(&strip) = self.strips.get(&sheet) {
                out.extend(self.stripped(index, &sheet, cut, &tint_name, strip)?);
                continue;
            }
            let Cut::Tile(tile) = cut else {
                continue;
            };
            let source = self.sheet(&sheet)?;
            let Some(mut frames) = source.tsx.animations.get(&tile).cloned() else {
                continue;
            };
            let Some(first) = frames.iter().position(|&(id, _)| id == tile) else {
                continue;
            };
            frames.rotate_left(first);
            let mut pictures = Vec::new();
            for &(id, _) in &frames {
                pictures.push(source.tile(id).map_err(|e| format!("{sheet}: {e}"))?);
            }
            if pictures.iter().all(|p| *p == pictures[0]) {
                continue;
            }
            let mut packed = Vec::new();
            for (id, ms) in frames {
                packed.push((self.packed(&sheet, Cut::Tile(id), &tint_name)?, ms.max(1)));
            }
            let index = u32::try_from(index).unwrap_or(u32::MAX);
            out.push(FramesText {
                at: (index % COLUMNS, index / COLUMNS),
                frames: packed,
                minimal_todo: Some(MINIMAL_TODO.to_owned()),
            });
        }
        Ok(out)
    }

    /// The frames of packed cell `index`, picture `cut` of plain sheet
    /// `sheet` moving by `strip`, packed now; `None` if every frame is
    /// the same picture.
    fn stripped(
        &mut self,
        index: usize,
        sheet: &str,
        cut: Cut,
        tint_name: &str,
        strip: Strip,
    ) -> Result<Option<FramesText>, String> {
        let columns = self.sheet(sheet)?.tsx.columns;
        let last = (strip.frames.max(1) - 1) * strip.step;
        if cut.right_column(columns) + last >= columns {
            return Err(format!(
                "{sheet}: the frames of {cut:?} run past the image's right edge"
            ));
        }
        let mut packed = Vec::new();
        for k in 0..strip.frames {
            let frame = cut.along(k * strip.step);
            let at = self.packed(sheet, frame, tint_name);
            let at = at.map_err(|e| format!("{e} (frame {k} of a moving tile)"))?;
            packed.push((at, strip.ms.max(1)));
        }
        let pixels = |at: At| {
            let i = usize::try_from(at.1 * COLUMNS + at.0).unwrap_or(usize::MAX);
            self.pixels.get(i)
        };
        if packed
            .iter()
            .all(|&(at, _)| pixels(at) == pixels(packed[0].0))
        {
            return Ok(None);
        }
        let index = u32::try_from(index).unwrap_or(u32::MAX);
        Ok(Some(FramesText {
            at: (index % COLUMNS, index / COLUMNS),
            frames: packed,
            minimal_todo: Some(MINIMAL_TODO.to_owned()),
        }))
    }

    /// The packed image: width, height and RGBA8 pixels.
    fn image(&self) -> (u32, u32, Vec<u8>) {
        let count = u32::try_from(self.cells.len()).unwrap_or(u32::MAX);
        let (width, height) = (COLUMNS * TILE, count.div_ceil(COLUMNS).max(1) * TILE);
        let row = usize::try_from(width * 4).unwrap_or(0);
        let mut rgba = vec![0; row * usize::try_from(height).unwrap_or(0)];
        let tile = usize::try_from(TILE).unwrap_or(0);
        let columns = usize::try_from(COLUMNS).unwrap_or(1);
        for (index, pixels) in self.pixels.iter().enumerate() {
            let (left, top) = (index % columns * tile * 4, index / columns * tile);
            for (y, line) in pixels.chunks_exact(tile * 4).enumerate() {
                let at = (top + y) * row + left;
                rgba[at..at + line.len()].copy_from_slice(line);
            }
        }
        (width, height, rgba)
    }
}

/// What an import makes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Imported {
    /// The packed image: width, height and RGBA8 pixels.
    pub image: (u32, u32, Vec<u8>),
    /// The look of a map that names none: the tileset's own.
    pub own: LookText,
    /// The other looks, by name.
    pub others: Vec<(String, LookText)>,
    /// The packed tiles that move, and their frames.
    pub moving: Vec<FramesText>,
}

/// One layer of the mapping as the tileset file has it, its tiles packed.
/// `terrains` are ours, by id.
fn layer(packer: &mut Packer, layer: &LayerMap, terrains: &[String]) -> Result<LayerText, String> {
    match layer {
        LayerMap::Corners {
            of,
            sheet,
            terrain,
            pick,
            full,
            tint,
        } => {
            let table = packer
                .sheet(sheet)?
                .corners(terrain, *pick)
                .map_err(|e| format!("{sheet}: {e}"))?;
            let mut tiles = Vec::new();
            let last = if *full { 15 } else { 14 };
            for mix in 1..=last {
                if let Some(tile) = table[usize::from(mix)] {
                    tiles.push((mix, packer.cell(sheet, tile, tint)?));
                }
            }
            if tiles.is_empty() {
                return Err(format!("{sheet}: no tile has corners of \"{terrain}\""));
            }
            Ok(LayerText::Corners {
                of: of.clone(),
                tiles,
            })
        }
        LayerMap::Tiles {
            of,
            tile: (sheet, tile),
            beside,
            sides,
        } => {
            let at = packer.cell(sheet, *tile, "")?;
            let mut pictures = Vec::new();
            for (text, (sheet, tile)) in sides {
                let mix = parse_mix(text)
                    .ok_or_else(|| format!("sides \"{text}\" must be four of `#` and `.`"))?;
                pictures.push((mix, packer.cell(sheet, *tile, "")?));
            }
            pictures.sort_unstable();
            Ok(LayerText::Tiles {
                of: of.clone(),
                at,
                beside: beside.clone(),
                sides: pictures,
            })
        }
        LayerMap::Patch {
            of,
            sheet,
            patch,
            inside,
            full,
        } => patch_layer(packer, of, sheet, (*patch, *inside), *full),
        LayerMap::Quarters { of, sheet, kit } => quarters_layer(packer, of, sheet, kit),
        LayerMap::Block {
            of,
            sheet,
            block,
            tint,
        } => {
            let mut join = |ends| {
                packer.packed(
                    sheet,
                    Cut::Joined {
                        block: *block,
                        ends,
                    },
                    tint,
                )
            };
            // Where the terrain ends on no side: the block's middle.
            let at = join(0)?;
            let mut sides = Vec::new();
            for ends in 1..16 {
                sides.push((ends, join(ends)?));
            }
            // The terrain ends on a side whose neighbour is any other.
            let beside = terrains.iter().filter(|id| !of.contains(id)).cloned();
            Ok(LayerText::Tiles {
                of: of.clone(),
                at,
                beside: beside.collect(),
                sides,
            })
        }
    }
}

/// A [`LayerMap::Quarters`] as the tileset file has it, its pictures
/// packed: one for each mix of corners that draws anything.
fn quarters_layer(
    packer: &mut Packer,
    of: &[String],
    sheet: &str,
    kit: &Kit,
) -> Result<LayerText, String> {
    let mut tiles = Vec::new();
    for mix in 1..16u8 {
        let cells = kit.quarters([8, 4, 2, 1].map(|bit| mix & bit != 0));
        if cells.iter().any(Option::is_some) {
            tiles.push((mix, packer.packed(sheet, Cut::Quarters(cells), "")?));
        }
    }
    Ok(LayerText::Corners {
        of: of.to_vec(),
        tiles,
    })
}

/// A [`LayerMap::Patch`] as the tileset file has it, its pictures packed:
/// the patch and the inside corners from their top-left tiles.
fn patch_layer(
    packer: &mut Packer,
    of: &[String],
    sheet: &str,
    (patch, inside): (u32, u32),
    full: bool,
) -> Result<LayerText, String> {
    let columns = packer.sheet(sheet)?.tsx.columns;
    for (what, block) in [("patch", patch), ("inside corners", inside)] {
        if block % columns + 3 > columns {
            return Err(format!(
                "{sheet}: the {what} from tile {block} runs past the image's right edge"
            ));
        }
    }
    let at = |block: u32, (column, row): (u32, u32)| block + row * columns + column;
    let mut tiles = Vec::new();
    for (corners, in_inside, places) in PATCH {
        let mix = trpg_content::tileset::mix(corners);
        if mix == 15 && !full {
            continue;
        }
        let block = if in_inside { inside } else { patch };
        let cut = match places {
            [one] => Cut::Tile(at(block, *one)),
            [under, over] => Cut::Pair {
                under: at(block, *under),
                over: at(block, *over),
            },
            _ => continue,
        };
        tiles.push((mix, packer.packed(sheet, cut, "")?));
    }
    tiles.sort_unstable();
    Ok(LayerText::Corners {
        of: of.to_vec(),
        tiles,
    })
}

/// An error naming (as `what`) the first of `names` that isn't one of
/// `terrains`.
fn known<'a>(
    what: &str,
    names: impl IntoIterator<Item = &'a String>,
    terrains: &[String],
) -> Result<(), String> {
    let unknown = names.into_iter().find(|name| !terrains.contains(name));
    unknown.map_or(Ok(()), |name| {
        Err(format!("{what}: terrain \"{name}\" is not in terrain.ron"))
    })
}

/// What is wrong with `mapping` that shows without the bought files, the
/// first problem found: a terrain that isn't one of `terrains` (ours, by
/// id), a mix of sides that isn't one, a look a map can't name, no look
/// for a map that names none, a look without a tile for one of our
/// terrains, or a look listing a layer the mapping lacks.
pub fn check(mapping: &Mapping, terrains: &[String]) -> Result<(), String> {
    known("terrain", mapping.terrain.keys(), terrains)?;
    for (name, layer) in &mapping.layers {
        let what = format!("layer \"{name}\"");
        match layer {
            LayerMap::Corners { of, .. }
            | LayerMap::Block { of, .. }
            | LayerMap::Patch { of, .. }
            | LayerMap::Quarters { of, .. } => {
                known(&what, of, terrains)?;
            }
            LayerMap::Tiles {
                of, beside, sides, ..
            } => {
                known(&what, of.iter().chain(beside), terrains)?;
                if let Some(text) = sides.keys().find(|text| parse_mix(text).is_none()) {
                    return Err(format!(
                        "{what}: sides \"{text}\" must be four of `#` and `.`"
                    ));
                }
            }
        }
    }
    for (sheet, strip) in &mapping.moving {
        if !is_plain(sheet) {
            return Err(format!(
                "moving \"{sheet}\": only a plain sheet (….png) moves this way"
            ));
        }
        if strip.frames < 2 || strip.step == 0 || strip.ms == 0 {
            return Err(format!(
                "moving \"{sheet}\": it needs 2 frames or more, a step and a time"
            ));
        }
    }
    if !mapping.looks.contains_key(DEFAULT_TILES) {
        return Err(format!("the mapping has no look \"{DEFAULT_TILES}\""));
    }
    for (name, look) in &mapping.looks {
        let what = format!("look \"{name}\"");
        if !TILE_LOOKS.contains(&name.as_str()) {
            return Err(format!("{what} is not a look a map can name"));
        }
        known(&what, look.terrain.keys(), terrains)?;
        let has =
            |id: &&String| look.terrain.contains_key(*id) || mapping.terrain.contains_key(*id);
        if let Some(id) = terrains.iter().find(|id| !has(id)) {
            return Err(format!("{what}: terrain \"{id}\" has no tile"));
        }
        let lacking = look
            .layers
            .iter()
            .find(|l| !mapping.layers.contains_key(*l));
        if let Some(layer) = lacking {
            return Err(format!("{what}: no layer \"{layer}\""));
        }
    }
    Ok(())
}

/// Imports the looks of `mapping` from the bought packs under `dir`
/// ([`LIBRARIES_DIR`]):
/// packs the tiles they use and writes them out as tileset looks, each
/// with a tile for every one of `terrains` (ours, in `terrain.ron`'s
/// order). `colors` gives a palette colour by name, for tints.
pub fn import(
    dir: &Path,
    mapping: &Mapping,
    terrains: &[String],
    colors: &dyn Fn(&str) -> Option<[u8; 3]>,
) -> Result<Imported, String> {
    check(mapping, terrains)?;
    let mut packer = Packer {
        dir,
        colors,
        strips: &mapping.moving,
        recolours: &mapping.recolour,
        sheets: BTreeMap::new(),
        cells: Vec::new(),
        pixels: Vec::new(),
    };
    let mut looks = Vec::new();
    // The look of a map that names none first: its tiles lead the image.
    let own_first = |(name, _): &(&String, &LookMap)| *name != DEFAULT_TILES;
    let mut ordered: Vec<(&String, &LookMap)> = mapping.looks.iter().collect();
    ordered.sort_by_key(own_first);
    for (name, look) in ordered {
        let what = format!("look \"{name}\"");
        let mut text = LookText::default();
        for id in terrains {
            let tile = look.terrain.get(id).or_else(|| mapping.terrain.get(id));
            let (sheet, tile) =
                tile.ok_or_else(|| format!("{what}: terrain \"{id}\" has no tile"))?;
            let at = packer.cell(sheet, *tile, "");
            text.terrain
                .push((id.clone(), at.map_err(|e| format!("{what}: {e}"))?));
        }
        for layer_name in &look.layers {
            let map = mapping.layers.get(layer_name);
            let map = map.ok_or_else(|| format!("{what}: no layer \"{layer_name}\""))?;
            let made = layer(&mut packer, map, terrains);
            text.layers
                .push(made.map_err(|e| format!("layer \"{layer_name}\": {e}"))?);
        }
        looks.push((name.clone(), text));
    }
    let mut looks = looks.into_iter();
    let own = looks.next().filter(|(name, _)| name == DEFAULT_TILES);
    let (_, own) = own.ok_or_else(|| format!("the mapping has no look \"{DEFAULT_TILES}\""))?;
    let moving = packer.moving()?;
    Ok(Imported {
        image: packer.image(),
        own,
        others: looks.collect(),
        moving,
    })
}

/// The bundle path of the packed image.
fn image_path() -> String {
    format!("tilesets/{TILESET_ID}.png")
}

/// The tileset file: `imported`'s looks, then `sprites`' unit pictures.
pub fn tileset(imported: &Imported, sprites: &[MapSprite]) -> String {
    let mut out = format!(
        "// Written by `cargo xtask tileset-import` (ticket 0437). Do not edit: change\n\
         // {MAPPING_PATH} (the terrain) or the table in\n\
         // crates/xtask/src/map_sprite_import.rs (the units) and run it again.\n\
         (\n    id: \"{TILESET_ID}\",\n    image: \"{}\",\n    tile_px: ({TILE}, {TILE}),\n",
        image_path()
    );
    out.push_str(&tileset_ron::look(&imported.own, "    "));
    out.push_str(&tileset_ron::looks(&imported.others));
    out.push_str(&tileset_ron::frames(&imported.moving));
    out.push_str(&map_sprite_import::units(sprites));
    out.push_str(")\n");
    out
}

/// Which bought tiles each look's terrains and layers use, one per line.
pub fn list(mapping: &Mapping) -> String {
    let tile = |(sheet, tile): &TileRef| format!("{sheet} tile {tile}");
    let mut out = String::new();
    for (name, look) in &mapping.looks {
        // Writing to a `String` can't fail.
        let _ = writeln!(out, "look {name}");
        let mut own = mapping.terrain.clone();
        own.extend(look.terrain.clone());
        for (id, at) in &own {
            let _ = writeln!(out, "  {id:<16} {}", tile(at));
        }
        for layer_name in &look.layers {
            let described = match mapping.layers.get(layer_name) {
                Some(LayerMap::Corners {
                    of,
                    sheet,
                    terrain,
                    tint,
                    ..
                }) => {
                    let tinted = if tint.is_empty() {
                        String::new()
                    } else {
                        format!(", recoloured {tint}")
                    };
                    format!(
                        "between tiles of {}: {sheet} \"{terrain}\"{tinted}",
                        of.join(", ")
                    )
                }
                Some(LayerMap::Tiles { of, tile: at, .. }) => {
                    format!("on {}: {}", of.join(", "), tile(at))
                }
                Some(LayerMap::Block {
                    of, sheet, block, ..
                }) => format!("on {}: {sheet} block from tile {block}", of.join(", ")),
                Some(LayerMap::Patch {
                    of, sheet, patch, ..
                }) => format!(
                    "between tiles of {}: {sheet} patch from tile {patch}",
                    of.join(", ")
                ),
                Some(LayerMap::Quarters { of, sheet, .. }) => {
                    let recoloured = if mapping.recolour.contains_key(sheet) {
                        ", recoloured"
                    } else {
                        ""
                    };
                    format!(
                        "between tiles of {}: {sheet} quarters{recoloured}",
                        of.join(", ")
                    )
                }
                None => "(no such layer)".to_owned(),
            };
            let _ = writeln!(out, "  layer {layer_name:<10} {described}");
        }
    }
    out
}

/// Reads the mapping file under repo root `root`.
pub fn read_mapping(root: &Path) -> Result<Mapping, String> {
    let path = root.join(MAPPING_PATH);
    let text = fs::read_to_string(&path).map_err(|e| format!("reading {}: {e}", path.display()))?;
    ron::from_str(&text).map_err(|e| format!("{MAPPING_PATH}: {e}"))
}

/// Runs the command under repo root `root`: imports the mapping's looks
/// and writes the packed image and the tileset file naming them and
/// `sprites`. Nothing is written unless everything was read.
pub fn run(root: &Path, sprites: &[MapSprite]) -> Result<String, String> {
    let library = root.join(LIBRARY_DIR);
    let game = root.join(GAME_DIR);
    if !library.is_dir() || !game.is_dir() {
        return Err(format!(
            "{LIBRARY_DIR}/ or {GAME_DIR}/ is missing: run `cargo shuyi private-assets --library` first"
        ));
    }
    let mapping = read_mapping(root)?;
    let joined = |errors: Vec<trpg_content::ContentError>| {
        let lines: Vec<String> = errors.iter().map(ToString::to_string).collect();
        lines.join("\n")
    };
    let palette = trpg_content::palette::load().map_err(joined)?;
    let terrain = TerrainDef::load(Some(&palette)).map_err(joined)?;
    let terrains: Vec<String> = terrain
        .display
        .terrains
        .iter()
        .map(|t| t.id.clone())
        .collect();
    let packs = root.join(LIBRARIES_DIR);
    let imported = import(&packs, &mapping, &terrains, &|name| palette.get(name))?;
    let (width, height, rgba) = &imported.image;
    let png = encode_png(*width, *height, rgba)?;
    let ron_path = format!("tilesets/{TILESET_ID}.ron");
    let files = [
        (image_path(), png),
        (ron_path.clone(), tileset(&imported, sprites).into_bytes()),
    ];
    for (bundle_path, bytes) in files {
        let path = game.join(&bundle_path);
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).map_err(|e| format!("creating {}: {e}", dir.display()))?;
        }
        fs::write(&path, bytes).map_err(|e| format!("writing {}: {e}", path.display()))?;
    }
    Ok(format!(
        "tileset-import: wrote {GAME_DIR}/{} ({width}×{height}) and {GAME_DIR}/{ron_path} \
         ({} moving tiles)",
        image_path(),
        imported.moving.len()
    ))
}

#[cfg(test)]
mod tests;
